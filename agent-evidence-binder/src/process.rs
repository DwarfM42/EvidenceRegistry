//! Bounded top-level process observations, not authorship, sandboxing or Store authority.
//! Caller validates the exact Store Request immediately before `run_managed`.
//!
//! Bounds cover this dispatch's owned pipe/spool handles and bytes, not a child
//! process resource sandbox. Only descriptor budget 64 is currently supported;
//! smaller declarations fail before spawn. `output_files`/`output_bytes` are
//! enforced separately by Store capture, NOT by monitoring arbitrary child files.
//! Null stdin; prompts may be explicit bounded literal command arguments only.
//! Every platform requires an absolute executable; Windows additionally requires
//! an `.exe` (no implicit PATH lookup or batch-file shell).
//!
//! There are no reader threads, wait-with-output buffers or blocking child joins.
//! Windows peeks owned pipes before reading available bytes; Unix uses nonblocking
//! descriptors. Polling and kill/reap have deadlines; synchronous OS filesystem
//! calls/spawn itself are not a hard-real-time guarantee on a hung filesystem.
//! Failures preserve spools, never unlink a competing file or recreate a permit.
//!
//! File/root holds and identity checks are incremental defenses. Initial ancestor
//! acquisition races, lock-ignoring same-principal writes, post-close mutation and
//! process-tree containment are not covered. Store still owns capture-time reads;
//! this witness is not agent authorship or a predispatch Store root capability.
use crate::{
    ledger::{DispatchIntent, FaultKind, LedgerError, LedgerWriter, Observation, Pipe},
    RequestBinding,
};
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};
#[cfg(windows)]
#[path = "process/windows.rs"]
mod platform;
#[cfg(unix)]
#[path = "process/unix.rs"]
mod platform;

fn check_components(path: &Path) -> io::Result<()> {
    for p in path.ancestors() {
        if platform::reparse(&std::fs::symlink_metadata(p)?) {
            return Err(io::Error::other("reparse_component"));
        }
    }
    Ok(())
}
fn check_identity(file: &File, path: &Path, directory: bool) -> io::Result<()> {
    check_components(path)?;
    let m = file.metadata()?;
    if platform::reparse(&m) || (directory && !m.is_dir()) || (!directory && !m.is_file()) {
        return Err(io::Error::other("spool_type"));
    }
    let other = platform::open(path, directory, false)?;
    let (id, links) = platform::identity(file)?;
    let (other_id, other_links) = platform::identity(&other)?;
    if id != other_id || (!directory && (links != 1 || other_links != 1)) {
        return Err(io::Error::other("spool_substitution"));
    }
    Ok(())
}
#[cfg(test)]
#[path = "process/tests.rs"]
mod fault_tests;

pub const STDOUT_NAME: &str = "binder-stdout.bin";
pub const STDERR_NAME: &str = "binder-stderr.bin";
/// Only this module can mint this non-Clone, non-serializable witness.
/// The Store bridge must consume it, not accept a path or historical ledger frame.
/// On Windows the root is read-only from acquisition; completed spools retain
/// same-filesystem-object read-only handles with no write/delete sharing. The
/// write-to-read transition is not an atomic snapshot; Store owns capture bytes.
///
/// ```compile_fail
/// use ai_agent_evidence_binder::process::ManagedCompletion;
/// fn duplicate(value: ManagedCompletion) { let _second = value.clone(); }
/// ```
/// ```compile_fail
/// use ai_agent_evidence_binder::process::ManagedCompletion;
/// fn rebind(mut value: ManagedCompletion) { value.attempt = "old-log-id".into(); }
/// ```
#[must_use]
#[derive(Debug)]
pub struct ManagedCompletion {
    attempt: String,
    request: RequestBinding,
    intent: DispatchIntent,
    stdout: PathBuf,
    stderr: PathBuf,
    bytes: [u64; 2],
    _spools: [File; 2],
    _root: File,
}
impl ManagedCompletion {
    pub fn attempt_id(&self) -> &str {
        &self.attempt
    }
    pub fn request(&self) -> &RequestBinding {
        &self.request
    }
    pub fn intent(&self) -> &DispatchIntent {
        &self.intent
    }
    pub fn stdout_path(&self) -> &Path {
        &self.stdout
    }
    pub fn stderr_path(&self) -> &Path {
        &self.stderr
    }
    pub fn stdout_bytes(&self) -> u64 {
        self.bytes[0]
    }
    pub fn stderr_bytes(&self) -> u64 {
        self.bytes[1]
    }
}
#[derive(Debug)]
pub enum ProcessError {
    Ledger(LedgerError),
    Io(io::Error),
    SpoolValidation(io::Error),
    CompletionHold(io::Error),
    StreamIo {
        operation: &'static str,
        pipe: Pipe,
        source: io::Error,
    },
    Nonzero,
    Fault(FaultKind),
    Cleanup {
        primary: Box<ProcessError>,
        detail: String,
    },
}
impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ProcessError {}
impl From<io::Error> for ProcessError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<LedgerError> for ProcessError {
    fn from(e: LedgerError) -> Self {
        Self::Ledger(e)
    }
}

/// Executes exactly the declared executable and literal arguments, with inherited
/// environment, null stdin and cwd=approved root. Does not validate Store authority.
pub fn run_managed(
    writer: &mut LedgerWriter,
    request: RequestBinding,
    attempt_id: &str,
    intent: DispatchIntent,
    cancellation: &AtomicBool,
) -> Result<ManagedCompletion, ProcessError> {
    run_inner(
        writer,
        request,
        attempt_id,
        intent,
        cancellation,
        #[cfg(test)]
        None,
        #[cfg(test)]
        None,
    )
}

#[cfg(test)]
type TestHook = fn(&mut std::process::ChildStdout, &mut [File; 2], &Path);
#[cfg(test)]
type CompletionHook = fn(&Path, &[File; 2]);
fn run_inner(
    writer: &mut LedgerWriter,
    request: RequestBinding,
    attempt_id: &str,
    intent: DispatchIntent,
    cancellation: &AtomicBool,
    #[cfg(test)] hook: Option<TestHook>,
    #[cfg(test)] completion_hook: Option<CompletionHook>,
) -> Result<ManagedCompletion, ProcessError> {
    let permit = writer.append_intent(request.clone(), attempt_id, intent.clone())?;
    // v0 supports only the maximal declared budget. This bounds Binder-owned
    // per-dispatch handles, not arbitrary child/descendant resource allocation.
    // Fixed live set: root + 2 spools + 2 pipes + child (+ Windows thread during
    // spawn); borrowed ledger 2; identity validation adds at most one; std spawn
    // transient pipe/null/error handles fit the conservative 64-handle budget.
    // Windows completion replaces two spools in two stages, adding at most two
    // read holds plus one identity-check handle after both owned pipes reach EOF.
    // No traversal/reader pool or descriptor growth depends on agent output.
    if intent.limits.open_descriptors != 64 {
        writer.append_observation(
            attempt_id,
            Observation::SpawnFailed {
                code: "unsupported_descriptor_budget".into(),
            },
        )?;
        return Err(
            io::Error::new(io::ErrorKind::Unsupported, "unsupported_descriptor_budget").into(),
        );
    }
    let executable = Path::new(&intent.command[0]);
    if !executable.is_absolute()
        || (cfg!(windows)
            && !executable
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("exe")))
    {
        writer.append_observation(
            attempt_id,
            Observation::SpawnFailed {
                code: "unsupported_executable".into(),
            },
        )?;
        return Err(
            io::Error::new(io::ErrorKind::Unsupported, "absolute_executable_required").into(),
        );
    }
    let root = Path::new(&intent.approved_output_root);
    let stdout = root.join(STDOUT_NAME);
    let stderr = root.join(STDERR_NAME);
    let reserved = (|| -> io::Result<(File, [File; 2])> {
        check_components(root)?;
        let held_root = platform::open(root, true, false)?;
        check_identity(&held_root, root, true)?;
        for path in [&stdout, &stderr] {
            match std::fs::symlink_metadata(path) {
                Err(e) if e.kind() == io::ErrorKind::NotFound => (),
                Err(e) => return Err(e),
                Ok(_) => return Err(io::Error::new(io::ErrorKind::AlreadyExists, "spool_exists")),
            }
        }
        let out = platform::open(&stdout, false, true)?;
        let err = platform::open(&stderr, false, true)?;
        check_identity(&held_root, root, true)?;
        check_identity(&out, &stdout, false)?;
        check_identity(&err, &stderr, false)?;
        Ok((held_root, [out, err]))
    })();
    let (held_root, [out, err]) = match reserved {
        Ok(files) => files,
        Err(e) => {
            writer.append_observation(
                attempt_id,
                Observation::SpawnFailed {
                    code: "preflight_spools".into(),
                },
            )?;
            return Err(e.into());
        }
    };
    if cancellation.load(std::sync::atomic::Ordering::Acquire) {
        writer.append_observation(
            attempt_id,
            Observation::SpawnFailed {
                code: "cancelled_before_spawn".into(),
            },
        )?;
        return Err(ProcessError::Fault(FaultKind::Cancelled));
    }
    let attempt = permit.consume();
    let child = Command::new(&intent.command[0])
        .args(&intent.command[1..])
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let mut child = match child {
        Ok(child) => child,
        Err(e) => {
            writer.append_observation(
                &attempt,
                Observation::SpawnFailed {
                    code: "os_spawn_failed".into(),
                },
            )?;
            return Err(e.into());
        }
    };
    let mut spools = [out, err];
    let mut bytes = [0; 2];
    let mut status = None;
    let started = Instant::now();
    let observed = (|| -> Result<(), ProcessError> {
        writer.append_observation(&attempt, Observation::Spawned { pid: child.id() })?;
        let mut out_pipe = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("stdout_missing"))?;
        let mut err_pipe = child
            .stderr
            .take()
            .ok_or_else(|| io::Error::other("stderr_missing"))?;
        #[cfg(test)]
        if let Some(hook) = hook {
            hook(&mut out_pipe, &mut spools, root);
        }
        platform::prepare(&out_pipe)?;
        platform::prepare(&err_pipe)?;
        let mut hashes = [Sha256::new(), Sha256::new()];
        let mut eof = [false; 2];
        let mut exited_at = None;
        loop {
            check_identity(&held_root, root, true)?;
            for (i, path) in [&stdout, &stderr].into_iter().enumerate() {
                check_identity(&spools[i], path, false)?;
                if spools[i].metadata()?.len() != bytes[i] {
                    return Err(io::Error::other("spool_size_drift").into());
                }
            }
            if cancellation.load(std::sync::atomic::Ordering::Acquire) {
                return Err(ProcessError::Fault(FaultKind::Cancelled));
            }
            if status.is_none()
                && started.elapsed() >= Duration::from_millis(intent.limits.runtime_ms)
            {
                return Err(ProcessError::Fault(FaultKind::Timeout));
            }
            if status.is_none() {
                if let Some(s) = child.try_wait()? {
                    // Save the actual observation before attempting its persistence.
                    status = Some(s);
                    exited_at = Some(Instant::now());
                    writer.append_observation(&attempt, Observation::Exited { code: s.code() })?;
                }
            }
            for (i, pipe) in [Pipe::Stdout, Pipe::Stderr].into_iter().enumerate() {
                if eof[i] {
                    continue;
                }
                let mut buffer = [0u8; 4096];
                let read = if i == 0 {
                    platform::read(&mut out_pipe, &mut buffer)
                } else {
                    platform::read(&mut err_pipe, &mut buffer)
                }
                .map_err(|source| ProcessError::StreamIo {
                    operation: "pipe_read",
                    pipe,
                    source,
                })?;
                match read {
                    Some(0) => {
                        spools[i].sync_all()?;
                        writer.append_observation(
                            &attempt,
                            Observation::PipeEof {
                                pipe,
                                bytes: bytes[i],
                                sha256: format!("{:x}", hashes[i].clone().finalize()),
                            },
                        )?;
                        eof[i] = true;
                    }
                    Some(n) => {
                        let max = if i == 0 {
                            intent.limits.stdout_bytes
                        } else {
                            intent.limits.stderr_bytes
                        };
                        let keep = (max - bytes[i]).min(n as u64) as usize;
                        spools[i].write_all(&buffer[..keep]).map_err(|source| {
                            ProcessError::StreamIo {
                                operation: "spool_write",
                                pipe,
                                source,
                            }
                        })?;
                        hashes[i].update(&buffer[..keep]);
                        bytes[i] += keep as u64;
                        if keep != n {
                            return Err(ProcessError::Fault(if i == 0 {
                                FaultKind::StdoutOverflow
                            } else {
                                FaultKind::StderrOverflow
                            }));
                        }
                    }
                    None => (),
                }
            }
            if eof != [true; 2]
                && exited_at.is_some_and(|t: Instant| {
                    t.elapsed() >= Duration::from_millis(intent.limits.pipe_drain_ms)
                })
            {
                return Err(ProcessError::Fault(FaultKind::PipeDrainTimeout));
            }
            if eof == [true; 2] && status.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        #[cfg(test)]
        if let Some(hook) = completion_hook {
            hook(root, &spools);
        }
        (|| -> io::Result<()> {
            check_identity(&held_root, root, true)?;
            for (i, path) in [&stdout, &stderr].into_iter().enumerate() {
                check_identity(&spools[i], path, false)?;
                if spools[i].metadata()?.len() != bytes[i] {
                    return Err(io::Error::other("spool_size_drift"));
                }
            }
            Ok(())
        })()
        .map_err(ProcessError::SpoolValidation)?;
        if !status.is_some_and(|s| s.success()) {
            return Err(ProcessError::Nonzero);
        }
        #[cfg(windows)]
        platform::finish_spools(&mut spools, [&stdout, &stderr], bytes)
            .map_err(ProcessError::CompletionHold)?;
        Ok(())
    })();
    if let Err(mut error) = observed {
        let fault = match &error {
            ProcessError::Fault(f) => Some(*f),
            ProcessError::Io(_)
            | ProcessError::SpoolValidation(_)
            | ProcessError::CompletionHold(_)
            | ProcessError::StreamIo { .. } => Some(FaultKind::StreamTruncated),
            ProcessError::Ledger(_) => Some(FaultKind::ObservationWriteFailed),
            _ => None,
        };
        if let Some(fault) = fault {
            if let Err(e) = writer.append_observation(
                &attempt,
                Observation::Fault {
                    fault,
                    detail: match &error {
                        ProcessError::SpoolValidation(_) => "spool_validation".into(),
                        ProcessError::CompletionHold(_) => "completion_hold".into(),
                        ProcessError::StreamIo {
                            operation, pipe, ..
                        } => format!("{operation}_{pipe:?}"),
                        _ => format!("{fault:?}"),
                    },
                },
            ) {
                error = ProcessError::Cleanup {
                    primary: Box::new(error),
                    detail: format!("observation_persistence:{e}"),
                };
            }
        }
        if status.is_none() {
            let kill_error = child.kill().err();
            let kill_started = Instant::now();
            let mut confirmed = false;
            while kill_started.elapsed() < Duration::from_millis(intent.limits.pipe_drain_ms) {
                match child.try_wait() {
                    Ok(Some(s)) => {
                        confirmed = true;
                        if let Err(e) = writer
                            .append_observation(&attempt, Observation::Exited { code: s.code() })
                        {
                            error = ProcessError::Cleanup {
                                primary: Box::new(error),
                                detail: format!("exit_persistence:{e}"),
                            };
                        }
                        break;
                    }
                    Ok(None) => std::thread::sleep(Duration::from_millis(2)),
                    Err(_) => break,
                }
            }
            if !confirmed {
                let detail = if kill_error.is_some() {
                    "kill_failed_exit_unconfirmed"
                } else {
                    "kill_exit_unconfirmed"
                };
                let _ = writer.append_observation(
                    &attempt,
                    Observation::Fault {
                        fault: FaultKind::Unavailable,
                        detail: detail.into(),
                    },
                );
                error = ProcessError::Cleanup {
                    primary: Box::new(error),
                    detail: detail.into(),
                };
            }
        }
        // Owned read handles dropped with the closure. No threads or blocking joins.
        return Err(error);
    }
    Ok(ManagedCompletion {
        attempt,
        request,
        intent,
        stdout,
        stderr,
        bytes,
        _spools: spools,
        _root: held_root,
    })
}
