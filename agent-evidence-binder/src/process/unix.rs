use super::*;
use std::os::fd::AsRawFd;
pub fn prepare<T: AsRawFd>(pipe: &T) -> io::Result<()> {
    let flags = unsafe { libc::fcntl(pipe.as_raw_fd(), libc::F_GETFL) };
    if flags < 0
        || unsafe { libc::fcntl(pipe.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
pub fn read<T: Read + AsRawFd>(pipe: &mut T, buffer: &mut [u8]) -> io::Result<Option<usize>> {
    match pipe.read(buffer) {
        Ok(n) => Ok(Some(n)),
        Err(e)
            if matches!(
                e.kind(),
                io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
            ) =>
        {
            Ok(None)
        }
        Err(e) => Err(e),
    }
}

use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
pub fn open(path: &Path, directory: bool, create: bool) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(create)
        .create_new(create)
        .mode(0o600)
        .custom_flags(
            libc::O_NOFOLLOW | libc::O_NONBLOCK | if directory { libc::O_DIRECTORY } else { 0 },
        )
        .open(path)
}
pub fn reparse(m: &std::fs::Metadata) -> bool {
    m.file_type().is_symlink()
}
pub fn identity(file: &File) -> io::Result<((u64, u64), u64)> {
    let m = file.metadata()?;
    Ok(((m.dev(), m.ino()), m.nlink()))
}
