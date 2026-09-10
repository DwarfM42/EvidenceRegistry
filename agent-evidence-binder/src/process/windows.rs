use super::*;
use std::ffi::c_void;
use std::os::windows::io::{AsRawHandle, FromRawHandle};
#[link(name = "kernel32")]
unsafe extern "system" {
    fn PeekNamedPipe(
        handle: *mut c_void,
        buffer: *mut c_void,
        size: u32,
        read: *mut u32,
        available: *mut u32,
        remaining: *mut u32,
    ) -> i32;
}
pub fn prepare<T: AsRawHandle>(_: &T) -> io::Result<()> {
    Ok(())
}
/// Only this thread owns/reads the anonymous read handle. Peek's available bytes
/// cannot be consumed by another reader; read never asks for more than available.
pub fn read<T: Read + AsRawHandle>(pipe: &mut T, buffer: &mut [u8]) -> io::Result<Option<usize>> {
    let mut available = 0;
    let ok = unsafe {
        PeekNamedPipe(
            pipe.as_raw_handle(),
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            &mut available,
            std::ptr::null_mut(),
        )
    };
    if ok == 0 {
        let error = io::Error::last_os_error();
        return if error.raw_os_error() == Some(109) {
            Ok(Some(0))
        } else {
            Err(error)
        };
    }
    if available == 0 {
        return Ok(None);
    }
    let len = buffer.len().min(available as usize);
    pipe.read(&mut buffer[..len]).map(Some)
}

use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
pub fn open(path: &Path, directory: bool, create: bool) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        // The root is only held/validated, never written or flushed here. Granting
        // it GENERIC_WRITE needlessly blocks Store's FILE_SHARE_READ root opener.
        .write(create)
        .create_new(create)
        .share_mode(if create { 1 } else { 3 })
        .custom_flags(0x0020_0000 | if directory { 0x0200_0000 } else { 0 })
        .open(path)
}
pub fn reparse(m: &std::fs::Metadata) -> bool {
    m.file_attributes() & 0x400 != 0
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn ReOpenFile(
        original: *mut c_void,
        desired_access: u32,
        share_mode: u32,
        flags: u32,
    ) -> *mut c_void;
}

fn reopen_read(file: &File, share_mode: u32) -> io::Result<File> {
    // Win32 ReOpenFile reopens the specified filesystem object, not an ambient
    // pathname. GENERIC_READ, OPEN_REPARSE_POINT; no inheritance/delete flags.
    // https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-reopenfile
    let handle = unsafe { ReOpenFile(file.as_raw_handle(), 0x8000_0000, share_mode, 0x0020_0000) };
    if handle == (-1isize) as *mut c_void {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful ReOpenFile returns a fresh owned handle.
    Ok(unsafe { File::from_raw_handle(handle) })
}

/// Remove write access without ever releasing the source object's last hold.
/// DuplicateHandle with reduced access is insufficient: it retains the original
/// file object's write-sharing accounting, even after the writable handle closes.
pub fn finish_spools(spools: &mut [File; 2], paths: [&Path; 2], bytes: [u64; 2]) -> io::Result<()> {
    let validate = |new: &[File; 2], old: &[File; 2]| -> io::Result<()> {
        for i in 0..2 {
            if identity(&new[i])? != identity(&old[i])? {
                return Err(io::Error::other("completion_hold_identity"));
            }
            check_identity(&new[i], paths[i], false)?;
            if new[i].metadata()?.len() != bytes[i] {
                return Err(io::Error::other("completion_hold_size"));
            }
        }
        Ok(())
    };
    // Both writable originals remain alive until BOTH same-object read handles
    // and identity/length checks succeed. Their EOF sync_all already succeeded.
    // READ|WRITE sharing is required while originals still have write access;
    // neither transition stage shares DELETE, so replacement remains blocked.
    let reduced = [reopen_read(&spools[0], 3)?, reopen_read(&spools[1], 3)?];
    validate(&reduced, spools)?;
    *spools = reduced; // drop both writable file objects, retaining both read holds

    // Seal sharing again before minting completion. A writer racing the brief
    // READ|WRITE transition makes this fail closed if still open. This is not an
    // atomic snapshot or a claim of equality with bytes at EOF; Store captures
    // its own bytes. No unheld pathname reopen or Binder copy is used here.
    let sealed = [reopen_read(&spools[0], 1)?, reopen_read(&spools[1], 1)?];
    validate(&sealed, spools)?;
    *spools = sealed;
    Ok(())
}
// Win32 BY_HANDLE_FILE_INFORMATION: DWORD fields and three FILETIMEs.
#[repr(C)]
#[derive(Default)]
struct Info {
    attrs: u32,
    creation: [u32; 2],
    access: [u32; 2],
    write: [u32; 2],
    volume: u32,
    size_hi: u32,
    size_lo: u32,
    links: u32,
    index_hi: u32,
    index_lo: u32,
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetFileInformationByHandle(handle: *mut c_void, info: *mut Info) -> i32;
}
pub fn identity(file: &File) -> io::Result<((u64, u64), u64)> {
    let mut info = Info::default();
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok((
        (
            info.volume as u64,
            ((info.index_hi as u64) << 32) | info.index_lo as u64,
        ),
        info.links as u64,
    ))
}
