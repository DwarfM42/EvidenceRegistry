//! Store-private, handle-relative source intake. The initial locator binding is not
//! trusted-ancestor authorization, predispatch identity, or an atomic snapshot.
use super::*;
use std::fs::File;
type Error = SelectedEmbeddedFreezePreparationError;
#[cfg(test)]
#[path = "source_capture_native_tests.rs"]
mod tests;
const INVALID: Error = Error::SourceInvalid;
const LIMIT: Error = Error::SourceResourceLimit;

pub(super) fn bind(root: &Path) -> Result<File, Error> {
    #[cfg(any(windows, target_os = "linux", target_os = "macos"))]
    {
        open_real_directory_hold(root).map_err(|_| INVALID)
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        let _ = root;
        Err(INVALID)
    }
}

fn component(name: &str) -> Result<(), Error> {
    if name.is_empty()
        || matches!(name, "." | "..")
        || name.contains(['/', '\\', ':', '\0'])
        || name.ends_with([' ', '.'])
        || name.chars().any(char::is_control)
    {
        return Err(INVALID);
    }
    Ok(())
}

fn append_name(
    names: &mut Vec<String>,
    name: String,
    budget: &mut SelectedEmbeddedTraversalBudget,
    depth: usize,
) -> Result<(), Error> {
    if matches!(name.as_str(), "." | "..") {
        return Ok(());
    }
    budget.reserve_child(&name, depth)?;
    component(&name)?;
    names.try_reserve(1).map_err(|_| LIMIT)?;
    names.push(name);
    Ok(())
}

pub(super) fn names(
    directory: &File,
    budget: &mut SelectedEmbeddedTraversalBudget,
    depth: usize,
) -> Result<Vec<String>, Error> {
    let metadata = directory.metadata().map_err(|_| INVALID)?;
    if !metadata.is_dir() || metadata_is_reparse(&metadata) {
        return Err(INVALID);
    }
    let mut names = native_names(directory, budget, depth)?;
    names.sort_unstable_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    if names.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(INVALID);
    }
    Ok(names)
}

pub(super) fn open_at(parent: &File, name: &str) -> Result<File, Error> {
    component(name)?;
    native_open(parent, name)
}

pub(super) fn directory_at(parent: &File, name: &str) -> Result<File, Error> {
    let file = open_at(parent, name)?;
    let metadata = file.metadata().map_err(|_| INVALID)?;
    if !metadata.is_dir() || metadata_is_reparse(&metadata) {
        return Err(INVALID);
    }
    Ok(file)
}

pub(super) fn matches_at(parent: &File, name: &str, held: &File) -> Result<(), Error> {
    let compared = open_at(parent, name).map_err(|_| Error::SourceChanged)?;
    let metadata = compared.metadata().map_err(|_| Error::SourceChanged)?;
    if metadata_is_reparse(&metadata) || !handles_identify_same_object(&compared, held) {
        return Err(Error::SourceChanged);
    }
    Ok(())
}

#[cfg(windows)]
fn native_open(parent: &File, name: &str) -> Result<File, Error> {
    use std::os::windows::io::{AsRawHandle, FromRawHandle};
    #[repr(C)]
    struct UnicodeString {
        length: u16,
        maximum_length: u16,
        buffer: *mut u16,
    }
    #[repr(C)]
    struct ObjectAttributes {
        length: u32,
        root_directory: *mut core::ffi::c_void,
        object_name: *mut UnicodeString,
        attributes: u32,
        security_descriptor: *mut core::ffi::c_void,
        security_quality_of_service: *mut core::ffi::c_void,
    }
    #[repr(C)]
    struct IoStatusBlock {
        status: isize,
        information: usize,
    }
    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn NtCreateFile(
            file_handle: *mut *mut core::ffi::c_void,
            desired_access: u32,
            object_attributes: *mut ObjectAttributes,
            io_status: *mut IoStatusBlock,
            allocation_size: *const i64,
            file_attributes: u32,
            share_access: u32,
            create_disposition: u32,
            create_options: u32,
            ea_buffer: *const core::ffi::c_void,
            ea_length: u32,
        ) -> i32;
    }
    let mut wide = Vec::new();
    wide.try_reserve_exact(name.len()).map_err(|_| LIMIT)?;
    wide.extend(name.encode_utf16());
    let length = wide
        .len()
        .checked_mul(2)
        .and_then(|n| u16::try_from(n).ok())
        .ok_or(INVALID)?;
    let mut unicode = UnicodeString {
        length,
        maximum_length: length,
        buffer: wide.as_mut_ptr(),
    };
    let mut attributes = ObjectAttributes {
        length: std::mem::size_of::<ObjectAttributes>() as u32,
        root_directory: parent.as_raw_handle(),
        object_name: &mut unicode,
        attributes: 0,
        security_descriptor: std::ptr::null_mut(),
        security_quality_of_service: std::ptr::null_mut(),
    };
    let mut io = IoStatusBlock {
        status: 0,
        information: 0,
    };
    let mut raw = std::ptr::null_mut();
    // Existing, read/list + read attributes + synchronize. No create disposition,
    // no write/delete access, read sharing only. Reparse objects are opened, not followed.
    let status = unsafe {
        NtCreateFile(
            &mut raw,
            0x0010_0081,
            &mut attributes,
            &mut io,
            std::ptr::null(),
            0,
            1,
            1,
            0x0020_0000 | 0x20,
            std::ptr::null(),
            0,
        )
    };
    if status < 0 || raw.is_null() {
        return Err(INVALID);
    }
    Ok(unsafe { File::from_raw_handle(raw) })
}

#[cfg(windows)]
fn native_names(
    directory: &File,
    budget: &mut SelectedEmbeddedTraversalBudget,
    depth: usize,
) -> Result<Vec<String>, Error> {
    use std::os::windows::io::AsRawHandle;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetFileInformationByHandleEx(
            file: *mut core::ffi::c_void,
            class: i32,
            info: *mut core::ffi::c_void,
            size: u32,
        ) -> i32;
    }
    let mut names = Vec::new();
    let mut first = true;
    loop {
        // Fixed 64 KiB, eight-byte aligned. Clear each call: API has no byte-count output.
        let mut buffer = [0u64; 8192];
        let ok = unsafe {
            GetFileInformationByHandleEx(
                directory.as_raw_handle(),
                if first { 11 } else { 10 },
                buffer.as_mut_ptr().cast(),
                65536,
            )
        };
        first = false;
        if ok == 0 {
            if std::io::Error::last_os_error().raw_os_error() == Some(18) {
                break;
            }
            return Err(INVALID);
        }
        let bytes = unsafe { std::slice::from_raw_parts(buffer.as_ptr().cast::<u8>(), 65536) };
        parse_windows_names(bytes, &mut names, budget, depth)?;
    }
    Ok(names)
}

#[cfg(windows)]
fn parse_windows_names(
    bytes: &[u8],
    names: &mut Vec<String>,
    budget: &mut SelectedEmbeddedTraversalBudget,
    depth: usize,
) -> Result<(), Error> {
    // FILE_ID_BOTH_DIR_INFO: FileNameLength at 60, FileName at 104.
    // Integer reads are byte-based, so malformed offsets never create unaligned references.
    let mut offset = 0usize;
    loop {
        let header = bytes
            .get(offset..offset.checked_add(104).ok_or(INVALID)?)
            .ok_or(INVALID)?;
        let next = u32::from_le_bytes(header[0..4].try_into().map_err(|_| INVALID)?) as usize;
        let length = u32::from_le_bytes(header[60..64].try_into().map_err(|_| INVALID)?) as usize;
        if length == 0
            || !length.is_multiple_of(2)
            || header[68] > 24
            || !header[68].is_multiple_of(2)
        {
            return Err(INVALID);
        }
        let end = offset
            .checked_add(104)
            .and_then(|v| v.checked_add(length))
            .ok_or(INVALID)?;
        let raw_name = bytes.get(offset + 104..end).ok_or(INVALID)?;
        if next != 0
            && (!next.is_multiple_of(8)
                || next < 104 + length
                || offset
                    .checked_add(next)
                    .and_then(|v| v.checked_add(104))
                    .filter(|v| *v <= bytes.len())
                    .is_none())
        {
            return Err(INVALID);
        }
        let mut wide = Vec::new();
        wide.try_reserve_exact(length / 2).map_err(|_| LIMIT)?;
        wide.extend(
            raw_name
                .chunks_exact(2)
                .map(|b| u16::from_le_bytes([b[0], b[1]])),
        );
        let name = String::from_utf16(&wide).map_err(|_| INVALID)?;
        append_name(names, name, budget, depth)?;
        if next == 0 {
            return Ok(());
        }
        offset = offset.checked_add(next).ok_or(INVALID)?;
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn native_open(parent: &File, name: &str) -> Result<File, Error> {
    use std::os::fd::{AsRawFd, FromRawFd};
    let name = std::ffi::CString::new(name).map_err(|_| INVALID)?;
    let fd = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK,
            0,
        )
    };
    if fd < 0 {
        return Err(INVALID);
    }
    Ok(unsafe { File::from_raw_fd(fd) })
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn native_names(
    directory: &File,
    budget: &mut SelectedEmbeddedTraversalBudget,
    depth: usize,
) -> Result<Vec<String>, Error> {
    use std::os::fd::AsRawFd;
    // Fresh open-file description, not dup: repeated passes need independent cursors.
    let fd = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            b".\0".as_ptr().cast(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            0,
        )
    };
    if fd < 0 {
        return Err(INVALID);
    }
    let stream = unsafe { libc::fdopendir(fd) };
    if stream.is_null() {
        unsafe {
            libc::close(fd);
        }
        return Err(INVALID);
    }
    struct Stream(*mut libc::DIR);
    impl Drop for Stream {
        fn drop(&mut self) {
            unsafe {
                libc::closedir(self.0);
            }
        }
    }
    let stream = Stream(stream);
    let mut names = Vec::new();
    loop {
        #[cfg(target_os = "linux")]
        let errno = unsafe { libc::__errno_location() };
        #[cfg(target_os = "macos")]
        let errno = unsafe { libc::__error() };
        unsafe {
            *errno = 0;
        }
        let entry = unsafe { libc::readdir(stream.0) };
        if entry.is_null() {
            if unsafe { *errno } != 0 {
                return Err(INVALID);
            }
            break;
        }
        // dirent records can be shorter than sizeof(dirent). Bound the name by
        // d_reclen, using this target's libc layout, never a Linux/Darwin guess.
        let name_offset = std::mem::offset_of!(libc::dirent, d_name);
        let record_len = unsafe { (*entry).d_reclen as usize };
        let name_len = record_len
            .checked_sub(name_offset)
            .filter(|n| *n > 0 && *n <= std::mem::size_of::<libc::dirent>() - name_offset)
            .ok_or(INVALID)?;
        let name =
            unsafe { std::slice::from_raw_parts(entry.cast::<u8>().add(name_offset), name_len) };
        let end = name.iter().position(|b| *b == 0).ok_or(INVALID)?;
        let mut raw = Vec::new();
        raw.try_reserve_exact(end).map_err(|_| LIMIT)?;
        raw.extend_from_slice(&name[..end]);
        let name = String::from_utf8(raw).map_err(|_| INVALID)?;
        append_name(&mut names, name, budget, depth)?;
    }
    Ok(names)
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
fn native_open(_parent: &File, _name: &str) -> Result<File, Error> {
    Err(INVALID)
}
#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
fn native_names(
    _directory: &File,
    _budget: &mut SelectedEmbeddedTraversalBudget,
    _depth: usize,
) -> Result<Vec<String>, Error> {
    Err(INVALID)
}
