//! Types for working with TTY streams

use crate::conn::{Error, Result};
use futures_util::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, ReadHalf},
    stream::{Stream, TryStreamExt},
};
use pin_project::pin_project;
use std::{convert::TryInto, io};
use std::{
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
/// An enum representing a chunk of TTY text streamed from a Podman container.
///
/// For convenience, this type can deref to the contained `Vec<u8>`.
pub enum TtyChunk {
    StdIn(Vec<u8>),
    StdOut(Vec<u8>),
    StdErr(Vec<u8>),
}

impl From<TtyChunk> for Vec<u8> {
    fn from(tty_chunk: TtyChunk) -> Self {
        match tty_chunk {
            TtyChunk::StdIn(bytes) | TtyChunk::StdOut(bytes) | TtyChunk::StdErr(bytes) => bytes,
        }
    }
}

impl AsRef<Vec<u8>> for TtyChunk {
    fn as_ref(&self) -> &Vec<u8> {
        match self {
            TtyChunk::StdIn(bytes) | TtyChunk::StdOut(bytes) | TtyChunk::StdErr(bytes) => bytes,
        }
    }
}

impl std::ops::Deref for TtyChunk {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl std::ops::DerefMut for TtyChunk {
    fn deref_mut(&mut self) -> &mut Vec<u8> {
        match self {
            TtyChunk::StdIn(bytes) | TtyChunk::StdOut(bytes) | TtyChunk::StdErr(bytes) => bytes,
        }
    }
}

pub async fn decode_chunk<S>(mut stream: S) -> Option<(Result<TtyChunk>, S)>
where
    S: AsyncRead + Unpin,
{
    let mut header_bytes = [0u8; 8];

    match stream.read_exact(&mut header_bytes).await {
        Err(e) if e.kind() == futures_util::io::ErrorKind::UnexpectedEof => return None,
        Err(e) => return Some((Err(Error::IO(e)), stream)),
        _ => (),
    }

    let size_bytes = &header_bytes[4..];
    let data_length = u32::from_be_bytes(size_bytes.try_into().ok()?);

    let mut data = vec![0u8; data_length as usize];

    if stream.read_exact(&mut data).await.is_err() {
        return None;
    }

    let chunk = match header_bytes[0] {
        0 => TtyChunk::StdIn(data),
        1 => TtyChunk::StdOut(data),
        2 => TtyChunk::StdErr(data),
        n => panic!("invalid stream number from podman daemon: '{n}'"),
    };

    Some((Ok(chunk), stream))
}

/// Decodes a TTY chunk from a stream.
pub fn decode<S>(hyper_chunk_stream: S) -> impl Stream<Item = Result<TtyChunk>>
where
    S: Stream<Item = Result<hyper::body::Bytes>> + Unpin,
{
    let stream = hyper_chunk_stream
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        .into_async_read();

    futures_util::stream::unfold(stream, decode_chunk)
}

pub async fn decode_raw<S>(stream: S) -> Option<(Result<TtyChunk>, S)>
where
    S: AsyncRead + Unpin,
{
    use futures_util::io::AsyncBufReadExt;
    let mut reader = futures_util::io::BufReader::new(stream);
    match reader.fill_buf().await {
        Ok(buf) if buf.is_empty() => None,
        Ok(buf) => Some((Ok(TtyChunk::StdOut(buf.to_vec())), reader.into_inner())),
        Err(e) if e.kind() == futures_util::io::ErrorKind::UnexpectedEof => None,
        Err(e) => Some((Err(Error::IO(e)), reader.into_inner())),
    }
}

type TtyReader = Pin<Box<dyn Stream<Item = Result<TtyChunk>> + Send + 'static>>;
type TtyWriter = Pin<Box<dyn AsyncWrite + Send + 'static>>;

/// This object can emit a stream of `TtyChunk`s and also implements `AsyncWrite` for streaming bytes to Stdin.
#[pin_project]
pub struct Multiplexer {
    #[pin]
    reader: TtyReader,
    #[pin]
    writer: TtyWriter,
}

impl Multiplexer {
    pub fn new<Con, F, Fut>(tcp_connection: Con, mut read_fn: F) -> Self
    where
        Con: AsyncRead + AsyncWrite + Send + 'static,
        F: FnMut(ReadHalf<Con>) -> Fut + Send + 'static,
        Fut: futures_util::Future<Output = Option<(Result<TtyChunk>, ReadHalf<Con>)>>
            + Send
            + 'static,
    {
        let (reader, writer) = tcp_connection.split();

        Self {
            reader: Box::pin(futures_util::stream::unfold(reader, move |reader| {
                read_fn(reader)
            })),
            writer: Box::pin(writer),
        }
    }
}

impl Stream for Multiplexer {
    type Item = Result<TtyChunk>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().reader.poll_next(cx)
    }
}

impl AsyncWrite for Multiplexer {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.project().writer.poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().writer.poll_flush(cx)
    }
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().writer.poll_close(cx)
    }
}

impl Multiplexer {
    /// Split the `Multiplexer` into the component `Stream` and `AsyncWrite` parts
    pub fn split(self) -> (impl Stream<Item = Result<TtyChunk>>, impl AsyncWrite + Send) {
        (self.reader, self.writer)
    }
}
