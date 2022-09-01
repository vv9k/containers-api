use futures_util::io::{AsyncRead, AsyncWrite};

use pin_project::pin_project;
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

#[pin_project]
pub struct Compat<S> {
    #[pin]
    tokio_multiplexer: S,
}

impl<S> Compat<S> {
    pub fn new(tokio_multiplexer: S) -> Self {
        Self { tokio_multiplexer }
    }
}

impl<S> AsyncRead for Compat<S>
where
    S: tokio::io::AsyncRead,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let mut readbuf = tokio::io::ReadBuf::new(buf);
        match self.project().tokio_multiplexer.poll_read(cx, &mut readbuf) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(())) => Poll::Ready(Ok(readbuf.filled().len())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
        }
    }
}

impl<S> AsyncWrite for Compat<S>
where
    S: tokio::io::AsyncWrite,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.project().tokio_multiplexer.poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().tokio_multiplexer.poll_flush(cx)
    }
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().tokio_multiplexer.poll_shutdown(cx)
    }
}
