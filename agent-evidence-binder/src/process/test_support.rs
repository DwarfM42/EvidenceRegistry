//! Native test-only synchronization; deadlines bound failed fixtures, not ordering.
use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    time::{Duration, Instant},
};

pub fn accept(listener: &TcpListener) -> TcpStream {
    listener.set_nonblocking(true).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                // Winsock may inherit the listener's nonblocking mode.
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                stream
                    .set_write_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                return stream;
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock && Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(2));
            }
            Err(e) => panic!("child barrier accept: {e}"),
        }
    }
}

pub fn send(stream: &mut TcpStream, value: u8) {
    stream.write_all(&[value]).unwrap();
}
pub fn receive(stream: &mut TcpStream, expected: u8) {
    let mut value = [0];
    stream.read_exact(&mut value).unwrap();
    assert_eq!(value, [expected]);
}

pub fn assert_disconnected(stream: &mut TcpStream) {
    match stream.read(&mut [0]) {
        Ok(0) => (),
        Err(e) if e.kind() == io::ErrorKind::ConnectionReset => (),
        other => panic!("terminated child's barrier must close, not time out: {other:?}"),
    }
}

/// Runs only in the isolated fixture process. No stdout use follows this close.
pub fn close_stdout() {
    std::io::stdout().flush().unwrap();
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
        }
        assert_ne!(unsafe { CloseHandle(std::io::stdout().as_raw_handle()) }, 0);
    }
    #[cfg(unix)]
    assert_eq!(unsafe { libc::close(libc::STDOUT_FILENO) }, 0);
}

#[cfg(windows)]
pub struct ChildProbe(std::os::windows::io::OwnedHandle);
#[cfg(windows)]
impl ChildProbe {
    pub fn open(pid: u32) -> Self {
        use std::os::windows::io::FromRawHandle;
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut std::ffi::c_void;
        }
        // SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION, no mutation rights.
        let handle = unsafe { OpenProcess(0x0010_1000, 0, pid) };
        assert!(!handle.is_null(), "{}", io::Error::last_os_error());
        Self(unsafe { std::os::windows::io::OwnedHandle::from_raw_handle(handle) })
    }
    pub fn assert_killed(&self) {
        use std::os::windows::io::AsRawHandle;
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn WaitForSingleObject(handle: *mut std::ffi::c_void, millis: u32) -> u32;
            fn GetExitCodeProcess(handle: *mut std::ffi::c_void, code: *mut u32) -> i32;
        }
        // Same retained process object acquired before release, not a reused PID.
        assert_eq!(unsafe { WaitForSingleObject(self.0.as_raw_handle(), 0) }, 0);
        let mut code = 0;
        assert_ne!(
            unsafe { GetExitCodeProcess(self.0.as_raw_handle(), &mut code) },
            0
        );
        assert_eq!(code, 1, "Rust Child::kill uses TerminateProcess(..., 1)");
    }
}

#[cfg(unix)]
pub struct ChildProbe(u32);
#[cfg(unix)]
impl ChildProbe {
    pub fn open(pid: u32) -> Self {
        assert_eq!(unsafe { libc::kill(pid as i32, 0) }, 0);
        Self(pid)
    }
    pub fn assert_killed(&self) {
        // The fixture cannot exit normally until a release byte that is not sent
        // until run_managed returns. Production must have killed and reaped it.
        assert_eq!(unsafe { libc::kill(self.0 as i32, 0) }, -1);
        assert_eq!(io::Error::last_os_error().raw_os_error(), Some(libc::ESRCH));
    }
}
