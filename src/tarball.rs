//! Utility functions to compression.

use flate2::{write::GzEncoder, Compression};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, MAIN_SEPARATOR},
};
use tar::Builder;

#[cfg(feature = "par-compress")]
use gzp::{
    deflate::Gzip,
    par::compress::{ParCompress, ParCompressBuilder},
};

/// Writes a gunzip encoded tarball to `buf` from entries found in `path`.
pub fn dir<W, P>(buf: W, path: P) -> io::Result<()>
where
    W: Write,
    P: AsRef<Path>,
{
    let encoder = GzEncoder::new(buf, Compression::best());
    let path = path.as_ref();
    ArchiveBuilder::build(encoder, path)?;

    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd"))]
#[cfg(feature = "par-compress")]
/// Same as [`dir`](dir) but initializes the underlying buffer, returns it and utilizes compression
/// parallelization on multiple cores to speed up the work.
pub fn dir_par<P>(path: P) -> io::Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    use memfile::MemFile;
    use std::io::{Read, Seek};

    let tx = MemFile::create_default(&path.as_ref().to_string_lossy())?;
    let mut rx = tx.try_clone()?;
    let encoder: ParCompress<Gzip> = ParCompressBuilder::new().from_writer(tx);

    let path = path.as_ref();
    ArchiveBuilder::build(encoder, path)?;

    rx.rewind()?;
    let mut data = vec![];
    rx.read_to_end(&mut data)?;
    Ok(data)
}

#[cfg(not(any(target_os = "linux", target_os = "android", target_os = "freebsd")))]
#[cfg(unix)]
#[cfg(feature = "par-compress")]
/// Same as [`dir`](dir) but initializes the underlying buffer, returns it and utilizes compression
/// parallelization on multiple cores to speed up the work.
pub fn dir_par<P>(path: P) -> io::Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    use std::io::{Read, Seek};

    let tmp_dir = tempfile::tempdir()?;
    let tmp_file_path = tmp_dir.path().join("data");
    let tx = std::fs::File::create(&tmp_file_path)?;

    let encoder: ParCompress<Gzip> = ParCompressBuilder::new().from_writer(tx);

    let path = path.as_ref();
    ArchiveBuilder::build(encoder, path)?;

    let mut rx = std::fs::File::open(&tmp_file_path)?;
    rx.rewind()?;
    let mut data = vec![];
    rx.read_to_end(&mut data)?;
    Ok(data)
}

fn resolve_base_path(canonical_path: &Path) -> io::Result<String> {
    let mut base_path_str = canonical_path
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid base path"))?
        .to_owned();
    if let Some(last) = base_path_str.chars().last() {
        if last != MAIN_SEPARATOR {
            base_path_str.push(MAIN_SEPARATOR)
        }
    }
    Ok(base_path_str)
}

struct ArchiveBuilder<W: Write> {
    archive: Builder<W>,
    base_path: String,
}

impl<W: Write> ArchiveBuilder<W> {
    fn build(buf: W, path: &Path) -> io::Result<()> {
        let canonical = path.canonicalize()?;
        let mut builder = Self::new(buf, &canonical)?;
        builder.bundle(&canonical, false)?;
        builder.archive.finish()?;
        builder.archive.into_inner()?.flush()
    }

    fn new(buf: W, canonical: &Path) -> io::Result<Self> {
        let base_path = resolve_base_path(canonical)?;

        Ok(Self {
            archive: Builder::new(buf),
            base_path,
        })
    }

    /// Starts the traversal by bundling files/directories in the base path to the archive.
    fn bundle(&mut self, dir: &Path, bundle_dir: bool) -> io::Result<()> {
        if fs::metadata(dir)?.is_dir() {
            if bundle_dir {
                self.append_entry(dir)?;
            }
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                if fs::metadata(entry.path())?.is_dir() {
                    self.bundle(&entry.path(), true)?;
                } else {
                    self.append_entry(entry.path().as_path())?
                }
            }
        }
        Ok(())
    }

    fn append_entry(&mut self, path: &Path) -> io::Result<()> {
        let canonical = path.canonicalize()?;
        let relativized = canonical
            .to_str()
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "invalid canonicalized path")
            })?
            .trim_start_matches(&self.base_path[..]);
        if path.is_dir() {
            self.archive
                .append_dir(Path::new(relativized), &canonical)?
        } else {
            self.archive
                .append_file(Path::new(relativized), &mut File::open(&canonical)?)?
        }
        Ok(())
    }
}
