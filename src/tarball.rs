//! Utility functions to compression.

use flate2::{write::GzEncoder, Compression};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf, MAIN_SEPARATOR},
};
use tar::Builder;

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
    canonical: PathBuf,
    base_path: String,
}

impl<W: Write> ArchiveBuilder<W> {
    /// Initializes new archive builder that creates a tarball of the `path` directory compressed uzing gzip.
    pub fn new(archive: Builder<W>, path: &Path) -> io::Result<Self> {
        let canonical = path.canonicalize()?;
        let base_path = resolve_base_path(&canonical)?;

        Ok(Self {
            archive,
            canonical,
            base_path,
        })
    }

    /// Starts the traversal by bundling files/directories in the base path to the archive.
    pub fn start(&mut self) -> io::Result<()> {
        let canonical = self.canonical.clone();
        self.bundle(&canonical, false)
    }

    /// Finishes creating the tarball archive.
    pub fn finish(mut self) -> io::Result<()> {
        self.archive.finish()?;
        self.archive.into_inner()?.flush()
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

/// Writes a gunzip encoded tarball to `buf` from entries found in `path`.
pub fn dir<W, P>(buf: W, path: P) -> io::Result<()>
where
    W: Write,
    P: AsRef<Path>,
{
    let archive = Builder::new(GzEncoder::new(buf, Compression::best()));
    let path = path.as_ref();
    let mut builder = ArchiveBuilder::new(archive, path)?;
    builder.start()?;
    builder.finish()?;

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
    use gzp::deflate::Gzip;
    use gzp::par::compress::{ParCompress, ParCompressBuilder};
    use memfile::MemFile;
    use std::io::{Read, Seek};

    let tx = MemFile::create_default(&path.as_ref().to_string_lossy())?;
    let mut rx = tx.try_clone()?;
    let pars: ParCompress<Gzip> = ParCompressBuilder::new().from_writer(tx);
    let archive = Builder::new(pars);

    let path = path.as_ref();
    let mut builder = ArchiveBuilder::new(archive, path)?;
    builder.start()?;
    builder.finish()?;

    rx.rewind()?;
    let mut data = vec![];
    rx.read_to_end(&mut data)?;
    Ok(data)
}
