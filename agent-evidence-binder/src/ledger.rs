//! Independent, nonauthoritative all-attempt ledger. Keep its path OUTSIDE Store.
//!
//! Wire v1: `BNDLGR01`, then repeated LE u32 JSON length, canonical serde JSON
//! Frame, and SHA-256(`binder-ledger-frame-v1\0` || length || JSON). Frame.prev_hash
//! links the preceding digest (zero for sequence 1). No repair/truncation API.
//! Bounds include reader allocations. Unknown/duplicate keys, noncanonical JSON,
//! version/order/binding drift and damaged tails stop at the last valid prefix.
//!
//! One retained OS-locked file serializes cooperating writers. Readers use a
//! nonblocking shared lock; a live writer makes separate readers return Busy.
//! Local no-follow/link/identity checks are incremental defenses, not custody:
//! initial ancestor-open races, lock-ignoring same-principal mutation, tail or
//! whole-file deletion, remote filesystems and hardware power-cut guarantees are
//! NOT established. Completed sync calls are not historical execution evidence.
use crate::{approved_artifact_name, RequestBinding};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::{Component, Path, PathBuf},
};

pub const MAX_FRAME_BYTES: usize = 65_536;
pub const MAX_FRAMES: usize = 16_384;
pub const MAX_LEDGER_BYTES: u64 = 67_108_864;
pub const MAX_ATTEMPTS: usize = 1_024;
const MAGIC: &[u8; 8] = b"BNDLGR01";
const DOMAIN: &[u8] = b"binder-ledger-frame-v1\0";

#[derive(Debug)]
pub enum LedgerError {
    Io(io::Error),
    Busy,
    Invalid,
    Limit,
    Corrupt(Boundary),
    NotLive,
    Poisoned,
    Substitution,
}
impl std::fmt::Display for LedgerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for LedgerError {}
impl From<io::Error> for LedgerError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
pub type Result<T> = std::result::Result<T, LedgerError>;

/// Hard maxima equal these defaults; a command may request smaller positive limits.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandLimits {
    pub stdout_bytes: u64,
    pub stderr_bytes: u64,
    pub output_bytes: u64,
    pub output_files: u32,
    pub runtime_ms: u64,
    pub pipe_drain_ms: u64,
    pub open_descriptors: u32,
}
impl Default for CommandLimits {
    fn default() -> Self {
        Self {
            stdout_bytes: 8_388_608,
            stderr_bytes: 8_388_608,
            output_bytes: 67_108_864,
            output_files: 256,
            runtime_ms: 300_000,
            pipe_drain_ms: 5_000,
            open_descriptors: 64,
        }
    }
}
impl CommandLimits {
    fn valid(&self) -> bool {
        let max = Self::default();
        [
            (self.stdout_bytes, max.stdout_bytes),
            (self.stderr_bytes, max.stderr_bytes),
            (self.output_bytes, max.output_bytes),
            (self.output_files as u64, max.output_files as u64),
            (self.runtime_ms, max.runtime_ms),
            (self.pipe_drain_ms, max.pipe_drain_ms),
            (self.open_descriptors as u64, max.open_descriptors as u64),
        ]
        .iter()
        .all(|(n, m)| *n > 0 && n <= m)
    }
}
/// Captures the entire approved root, not a caller-selected subset. Mandatory
/// names are sorted unique portable relative names. This is a declaration, not
/// a retained filesystem capability. Parent must validate Q before append.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchIntent {
    pub approved_output_root: String,
    pub mandatory_files: Vec<String>,
    pub command: Vec<String>,
    pub binder_identity: String,
    pub limits: CommandLimits,
    pub predecessor: Option<String>,
}
impl DispatchIntent {
    fn valid(&self) -> bool {
        text(&self.approved_output_root, 4096)
            && Path::new(&self.approved_output_root).is_absolute()
            && !Path::new(&self.approved_output_root)
                .components()
                .any(|c| matches!(c, Component::ParentDir))
            && !self.mandatory_files.is_empty()
            && self.mandatory_files.len() <= self.limits.output_files as usize
            && self
                .mandatory_files
                .iter()
                .all(|n| approved_artifact_name(n))
            && self.mandatory_files.windows(2).all(|w| w[0] < w[1])
            && !self.command.is_empty()
            && self.command.len() <= 64
            && self.command.iter().all(|s| text(s, 4096))
            && self.command.iter().map(String::len).sum::<usize>() <= 16_384
            && text(&self.binder_identity, 256)
            && self.limits.valid()
            && self.predecessor.as_ref().is_none_or(|s| id(s))
    }
}
fn text(s: &str, max: usize) -> bool {
    !s.is_empty() && s.len() <= max && !s.chars().any(char::is_control)
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn digest(length: &[u8; 4], payload: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(DOMAIN);
    h.update(length);
    h.update(payload);
    h.finalize().into()
}

/// Caller assertions of observations; ledger never manufactures or authenticates
/// process observations and never regards this data as a live execution witness.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Observation {
    Spawned {
        pid: u32,
    },
    SpawnFailed {
        code: String,
    },
    Exited {
        code: Option<i32>,
    },
    PipeEof {
        pipe: Pipe,
        bytes: u64,
        sha256: String,
    },
    Fault {
        fault: FaultKind,
        detail: String,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FaultKind {
    Timeout,
    Cancelled,
    StdoutOverflow,
    StderrOverflow,
    PipeDrainTimeout,
    StreamTruncated,
    ObservationWriteFailed,
    CaptureFailed,
    InvalidSubmission,
    Unavailable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pipe {
    Stdout,
    Stderr,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Use OutputPreparation before a separate Store prepare mutation, then
/// OutputCapture before commit. A single-call capture may omit preparation.
pub enum PublicationKind {
    OutputPreparation,
    OutputCapture,
    ReviewResult,
    Admission,
}
/// Exact Store-returned reference syntax only. Parent resolves each through Core.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetainedReference {
    pub registry_id: String,
    pub entry_index: u64,
    pub entry_hash: String,
    pub event_type_id: u64,
    pub event_record_id: String,
}
impl RetainedReference {
    fn valid(&self) -> bool {
        crate::submission::exact_hex_id(&self.registry_id)
            && crate::submission::exact_hex_id(&self.entry_hash)
            && crate::submission::exact_hex_id(&self.event_record_id)
            && self.entry_index <= evidence_registry::ER_UINT_MAX
            && self.event_type_id > 0
            && self.event_type_id <= evidence_registry::ER_UINT_MAX
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Publication {
    Intent {
        operation: PublicationKind,
    },
    Returned {
        operation: PublicationKind,
        references: Vec<RetainedReference>,
        receipt: String,
    },
    Uncertain {
        operation: PublicationKind,
        code: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PublicationStatus {
    Unresolved,
    KnownReferences,
    Uncertain(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicationProjection {
    pub operation: PublicationKind,
    pub status: PublicationStatus,
    pub references: Vec<RetainedReference>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", deny_unknown_fields)]
pub enum Event {
    Intent(DispatchIntent),
    Observation(Observation),
    Publication(Publication),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub version: u32,
    pub sequence: u64,
    pub prev_hash: String,
    pub request: RequestBinding,
    pub attempt_id: String,
    pub event: Event,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoundaryKind {
    Torn,
    Magic,
    Length,
    Digest,
    Grammar,
    Chain,
    Order,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Boundary {
    pub offset: u64,
    pub kind: BoundaryKind,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaunchProjection {
    Unresolved,
    SpawnObserved,
    SpawnFailed,
}
#[derive(Clone, Debug)]
pub struct AttemptProjection {
    pub attempt_id: String,
    pub request: RequestBinding,
    pub predecessor: Option<String>,
    pub launch: LaunchProjection,
    pub live_witness: bool,
    pub publications: Vec<PublicationProjection>,
    pub sequences: Vec<u64>,
}
#[derive(Clone, Debug)]
pub struct ReadReport {
    pub boundary: Option<Boundary>,
    pub frames: Vec<Frame>,
    pub valid_bytes: u64,
    pub last_digest: String,
}
impl ReadReport {
    pub fn attempts(&self) -> Vec<AttemptProjection> {
        let mut result: Vec<AttemptProjection> = vec![];
        for f in &self.frames {
            if let Event::Intent(intent) = &f.event {
                result.push(AttemptProjection {
                    attempt_id: f.attempt_id.clone(),
                    request: f.request.clone(),
                    predecessor: intent.predecessor.clone(),
                    launch: LaunchProjection::Unresolved,
                    live_witness: false,
                    publications: vec![],
                    sequences: vec![f.sequence],
                });
            } else if let Some(a) = result.iter_mut().find(|a| a.attempt_id == f.attempt_id) {
                a.sequences.push(f.sequence);
                if matches!(f.event, Event::Observation(Observation::Spawned { .. })) {
                    a.launch = LaunchProjection::SpawnObserved;
                }
                if matches!(f.event, Event::Observation(Observation::SpawnFailed { .. })) {
                    a.launch = LaunchProjection::SpawnFailed;
                }
                match &f.event {
                    Event::Publication(Publication::Intent { operation }) => {
                        a.publications.push(PublicationProjection {
                            operation: *operation,
                            status: PublicationStatus::Unresolved,
                            references: vec![],
                        })
                    }
                    Event::Publication(Publication::Returned {
                        operation,
                        references,
                        ..
                    }) => {
                        if let Some(p) = a
                            .publications
                            .iter_mut()
                            .find(|p| p.operation == *operation)
                        {
                            p.status = PublicationStatus::KnownReferences;
                            p.references = references.clone();
                        }
                    }
                    Event::Publication(Publication::Uncertain { operation, code }) => {
                        if let Some(p) = a
                            .publications
                            .iter_mut()
                            .find(|p| p.operation == *operation)
                        {
                            p.status = PublicationStatus::Uncertain(code.clone());
                        }
                    }
                    _ => (),
                }
            }
        }
        result
    }
}

#[derive(Clone)]
struct AttemptState {
    request: RequestBinding,
    limits: CommandLimits,
    spawned: bool,
    spawn_failed: bool,
    exit: Option<Option<i32>>,
    eof: [bool; 2],
    faulted: bool,
    publications: Vec<(PublicationKind, PublicationStatus)>,
}
#[derive(Clone, Default)]
struct State {
    attempts: BTreeMap<String, AttemptState>,
}
impl State {
    fn apply(&mut self, frame: &Frame) -> Result<()> {
        if !id(&frame.attempt_id) || !frame.request.valid() {
            return Err(LedgerError::Invalid);
        }
        match &frame.event {
            Event::Intent(intent) => {
                if !intent.valid() || self.attempts.contains_key(&frame.attempt_id) {
                    return Err(LedgerError::Invalid);
                }
                if self.attempts.len() >= MAX_ATTEMPTS {
                    return Err(LedgerError::Limit);
                }
                let previous: Vec<_> = self
                    .attempts
                    .iter()
                    .filter(|(_, a)| a.request == frame.request)
                    .collect();
                match &intent.predecessor {
                    Some(p)
                        if self
                            .attempts
                            .get(p)
                            .is_some_and(|a| a.request == frame.request) => {}
                    None if previous.is_empty() => (),
                    _ => return Err(LedgerError::Invalid),
                }
                self.attempts.insert(
                    frame.attempt_id.clone(),
                    AttemptState {
                        request: frame.request.clone(),
                        limits: intent.limits.clone(),
                        spawned: false,
                        spawn_failed: false,
                        exit: None,
                        eof: [false; 2],
                        faulted: false,
                        publications: vec![],
                    },
                );
            }
            Event::Observation(observation) => {
                let a = self
                    .attempts
                    .get_mut(&frame.attempt_id)
                    .ok_or(LedgerError::Invalid)?;
                // Capture publication can succeed before its bytes fail submission
                // validation. Retain that later failure without altering earlier
                // references or permitting any subsequent publication. Ordinary
                // process observations remain closed once publication starts.
                if a.request != frame.request
                    || (!a.publications.is_empty()
                        && !matches!(observation, Observation::Fault { .. }))
                {
                    return Err(LedgerError::Invalid);
                }
                match observation {
                    Observation::Spawned { pid } => {
                        if a.spawned || a.spawn_failed || *pid == 0 {
                            return Err(LedgerError::Invalid);
                        }
                        a.spawned = true;
                    }
                    Observation::SpawnFailed { code } => {
                        if a.spawned || a.spawn_failed || !id(code) {
                            return Err(LedgerError::Invalid);
                        }
                        a.spawn_failed = true;
                    }
                    Observation::Exited { code } => {
                        if !a.spawned || a.exit.is_some() {
                            return Err(LedgerError::Invalid);
                        }
                        a.exit = Some(*code);
                    }
                    Observation::PipeEof {
                        pipe,
                        bytes,
                        sha256,
                    } => {
                        let (i, max) = match pipe {
                            Pipe::Stdout => (0, a.limits.stdout_bytes),
                            Pipe::Stderr => (1, a.limits.stderr_bytes),
                        };
                        if !a.spawned
                            || a.eof[i]
                            || *bytes > max
                            || !crate::submission::exact_hex_id(sha256)
                        {
                            return Err(LedgerError::Invalid);
                        }
                        a.eof[i] = true;
                    }
                    Observation::Fault { detail, .. } => {
                        if !a.spawned || !text(detail, 1024) {
                            return Err(LedgerError::Invalid);
                        }
                        a.faulted = true;
                    }
                }
            }
            Event::Publication(publication) => {
                let a = self
                    .attempts
                    .get_mut(&frame.attempt_id)
                    .ok_or(LedgerError::Invalid)?;
                if a.request != frame.request
                    || a.faulted
                    || a.exit != Some(Some(0))
                    || a.eof != [true; 2]
                {
                    return Err(LedgerError::Invalid);
                }
                match publication {
                    Publication::Intent { operation } => {
                        let ordered = match a.publications.last() {
                            None => matches!(
                                operation,
                                PublicationKind::OutputPreparation | PublicationKind::OutputCapture
                            ),
                            Some((
                                PublicationKind::OutputPreparation,
                                PublicationStatus::KnownReferences,
                            )) => *operation == PublicationKind::OutputCapture,
                            Some((
                                PublicationKind::OutputCapture,
                                PublicationStatus::KnownReferences,
                            )) => *operation == PublicationKind::ReviewResult,
                            Some((
                                PublicationKind::ReviewResult,
                                PublicationStatus::KnownReferences,
                            )) => *operation == PublicationKind::Admission,
                            _ => false,
                        };
                        if !ordered {
                            return Err(LedgerError::Invalid);
                        }
                        a.publications
                            .push((*operation, PublicationStatus::Unresolved));
                    }
                    Publication::Returned {
                        operation,
                        references,
                        receipt,
                    } => {
                        if a.publications.last()
                            != Some(&(*operation, PublicationStatus::Unresolved))
                            || references.is_empty()
                            || references.len() > 8
                            || !text(receipt, 4096)
                            || references
                                .iter()
                                .any(|r| !r.valid() || r.registry_id != frame.request.registry_id)
                            || references
                                .iter()
                                .enumerate()
                                .any(|(i, r)| references[..i].contains(r))
                        {
                            return Err(LedgerError::Invalid);
                        }
                        a.publications.last_mut().ok_or(LedgerError::Invalid)?.1 =
                            PublicationStatus::KnownReferences;
                    }
                    Publication::Uncertain { operation, code } => {
                        if a.publications.last()
                            != Some(&(*operation, PublicationStatus::Unresolved))
                            || !id(code)
                        {
                            return Err(LedgerError::Invalid);
                        }
                        a.publications.last_mut().ok_or(LedgerError::Invalid)?.1 =
                            PublicationStatus::Uncertain(code.clone());
                    }
                }
            }
        }
        Ok(())
    }
}

/// Non-Clone, non-serializable durable-intent receipt. Only the successful append
/// caller receives it; consume it at most once in the parent's spawn route. It is
/// NOT proof of spawn. Reopen cannot issue one for an existing attempt.
#[must_use]
#[derive(Debug)]
pub struct SpawnPermit {
    attempt: String,
    sequence: u64,
}
impl SpawnPermit {
    pub fn attempt_id(&self) -> &str {
        &self.attempt
    }
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
    pub fn consume(self) -> String {
        self.attempt
    }
}

pub struct LedgerWriter {
    file: File,
    parent: File,
    path: PathBuf,
    parent_path: PathBuf,
    state: State,
    report: ReadReport,
    live: BTreeSet<String>,
    poisoned: bool,
}
impl LedgerWriter {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = absolute(path.as_ref())?;
        check_components(path.parent().ok_or(LedgerError::Invalid)?)?;
        let parent_path = path.parent().ok_or(LedgerError::Invalid)?.to_path_buf();
        let parent = platform::directory(&parent_path)?;
        let (mut file, created) = match platform::regular(&path, true, true) {
            Ok(f) => (f, true),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                (platform::regular(&path, true, false)?, false)
            }
            Err(e) => return Err(e.into()),
        };
        file.try_lock().map_err(|e| match e {
            fs::TryLockError::WouldBlock => LedgerError::Busy,
            fs::TryLockError::Error(e) => LedgerError::Io(e),
        })?;
        check_identity(&file, &path, false)?;
        check_identity(&parent, &parent_path, true)?;
        if created {
            // Never overwrite a competing creator's initialized bytes. A loser
            // may observe empty and refuse, but cannot initialize our new file.
            if file.metadata()?.len() != 0 {
                return Err(LedgerError::Substitution);
            }
            file.write_all(MAGIC)?;
            file.sync_all()?;
            parent.sync_all()?;
        }
        let report = parse(&mut file)?;
        if let Some(b) = &report.boundary {
            // Release the acquired advisory lock explicitly before surfacing a
            // corrupt prefix. This avoids retaining a process-local flock on
            // Unix while the caller continues adversarial boundary checks.
            file.unlock()?;
            return Err(LedgerError::Corrupt(b.clone()));
        }
        let mut state = State::default();
        for frame in &report.frames {
            state.apply(frame)?;
        }
        check_identity(&file, &path, false)?;
        check_identity(&parent, &parent_path, true)?;
        Ok(Self {
            file,
            parent,
            path,
            parent_path,
            state,
            report,
            live: BTreeSet::new(),
            poisoned: false,
        })
    }
    /// Absolute locator for namespace-overlap rejection only. This is not a
    /// retained-path capability, permission to reread, or evidence of custody.
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// This report is retained data only, even for the writer's current session.
    pub fn report(&self) -> &ReadReport {
        &self.report
    }
    pub fn append_intent(
        &mut self,
        request: RequestBinding,
        attempt: &str,
        intent: DispatchIntent,
    ) -> Result<SpawnPermit> {
        let sequence = self.append(request, attempt, Event::Intent(intent))?;
        self.live.insert(attempt.to_owned());
        Ok(SpawnPermit {
            attempt: attempt.to_owned(),
            sequence,
        })
    }
    pub fn append_observation(&mut self, attempt: &str, observation: Observation) -> Result<u64> {
        if !self.live.contains(attempt) {
            return Err(LedgerError::NotLive);
        }
        let request = self
            .state
            .attempts
            .get(attempt)
            .ok_or(LedgerError::Invalid)?
            .request
            .clone();
        self.append(request, attempt, Event::Observation(observation))
    }
    /// Record the intent BEFORE entering Store mutation; record returned refs or
    /// uncertainty separately AFTER return. No returned event may resolve an
    /// explicitly uncertain publication; cold reconciliation is read-only.
    pub fn append_publication(&mut self, attempt: &str, publication: Publication) -> Result<u64> {
        if !self.live.contains(attempt) {
            return Err(LedgerError::NotLive);
        }
        let request = self
            .state
            .attempts
            .get(attempt)
            .ok_or(LedgerError::Invalid)?
            .request
            .clone();
        self.append(request, attempt, Event::Publication(publication))
    }
    fn append(&mut self, request: RequestBinding, attempt: &str, event: Event) -> Result<u64> {
        if self.poisoned {
            return Err(LedgerError::Poisoned);
        }
        let frame = Frame {
            version: 1,
            sequence: self.report.frames.len() as u64 + 1,
            prev_hash: self.report.last_digest.clone(),
            request,
            attempt_id: attempt.to_owned(),
            event,
        };
        let mut next = self.state.clone();
        next.apply(&frame)?;
        let payload = serde_json::to_vec(&frame).map_err(|_| LedgerError::Invalid)?;
        let size = 4 + payload.len() as u64 + 32;
        if payload.len() > MAX_FRAME_BYTES
            || self.report.frames.len() >= MAX_FRAMES
            || self.report.valid_bytes + size > MAX_LEDGER_BYTES
        {
            return Err(LedgerError::Limit);
        }
        let length = (payload.len() as u32).to_le_bytes();
        let hash = digest(&length, &payload);
        // Any I/O or substitution failure poisons this session; a possibly
        // partially durable event must never authorize spawn or another write.
        self.poisoned = true;
        check_identity(&self.parent, &self.parent_path, true)?;
        check_identity(&self.file, &self.path, false)?;
        if self.file.metadata()?.len() != self.report.valid_bytes {
            return Err(LedgerError::Substitution);
        }
        self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(&length)?;
        self.file.write_all(&payload)?;
        self.file.write_all(&hash)?;
        self.file.sync_all()?;
        self.parent.sync_all()?;
        check_identity(&self.file, &self.path, false)?;
        check_identity(&self.parent, &self.parent_path, true)?;
        if self.file.metadata()?.len() != self.report.valid_bytes + size {
            return Err(LedgerError::Substitution);
        }
        self.report.valid_bytes += size;
        self.report.last_digest = hex(&hash);
        let sequence = frame.sequence;
        self.report.frames.push(frame);
        self.state = next;
        self.poisoned = false;
        Ok(sequence)
    }
}

/// Read-only snapshot; corruption returns its validated prefix and exact boundary.
/// A writer never consumes a corrupt report to resume, repair or spawn.
pub fn read_ledger(path: impl AsRef<Path>) -> Result<ReadReport> {
    let path = absolute(path.as_ref())?;
    check_components(&path)?;
    let mut file = platform::regular(&path, false, false)?;
    file.try_lock_shared().map_err(|e| match e {
        fs::TryLockError::WouldBlock => LedgerError::Busy,
        fs::TryLockError::Error(e) => LedgerError::Io(e),
    })?;
    check_identity(&file, &path, false)?;
    let report = parse(&mut file)?;
    check_identity(&file, &path, false)?;
    file.unlock()?;
    Ok(report)
}
fn parse(file: &mut File) -> Result<ReadReport> {
    file.seek(SeekFrom::Start(0))?;
    let len = file.metadata()?.len();
    let mut report = ReadReport {
        boundary: None,
        frames: vec![],
        valid_bytes: 0,
        last_digest: "0".repeat(64),
    };
    let mut magic = [0; 8];
    if len < 8 {
        report.boundary = Some(Boundary {
            offset: 0,
            kind: BoundaryKind::Torn,
        });
        return Ok(report);
    }
    file.read_exact(&mut magic)?;
    if magic != *MAGIC {
        report.boundary = Some(Boundary {
            offset: 0,
            kind: BoundaryKind::Magic,
        });
        return Ok(report);
    }
    report.valid_bytes = 8;
    let mut state = State::default();
    while report.valid_bytes < len {
        let offset = report.valid_bytes;
        let failed = |kind| Boundary { offset, kind };
        if report.frames.len() >= MAX_FRAMES || offset >= MAX_LEDGER_BYTES {
            report.boundary = Some(failed(BoundaryKind::Limit));
            break;
        }
        if len - offset < 4 {
            report.boundary = Some(failed(BoundaryKind::Torn));
            break;
        }
        let mut length = [0; 4];
        file.read_exact(&mut length)?;
        let size = u32::from_le_bytes(length) as usize;
        if size == 0 || size > MAX_FRAME_BYTES {
            report.boundary = Some(failed(BoundaryKind::Length));
            break;
        }
        let total = 4 + size as u64 + 32;
        if offset + total > MAX_LEDGER_BYTES {
            report.boundary = Some(failed(BoundaryKind::Limit));
            break;
        }
        if len - offset < total {
            report.boundary = Some(failed(BoundaryKind::Torn));
            break;
        }
        let mut payload = vec![0; size];
        file.read_exact(&mut payload)?;
        let mut hash = [0; 32];
        file.read_exact(&mut hash)?;
        if hash != digest(&length, &payload) {
            report.boundary = Some(failed(BoundaryKind::Digest));
            break;
        }
        let frame: Frame = match serde_json::from_slice(&payload) {
            Ok(f) => f,
            Err(_) => {
                report.boundary = Some(failed(BoundaryKind::Grammar));
                break;
            }
        };
        if frame.version != 1
            || serde_json::to_vec(&frame).map_err(|_| LedgerError::Invalid)? != payload
        {
            report.boundary = Some(failed(BoundaryKind::Grammar));
            break;
        }
        if frame.sequence != report.frames.len() as u64 + 1 || frame.prev_hash != report.last_digest
        {
            report.boundary = Some(failed(BoundaryKind::Chain));
            break;
        }
        if let Err(e) = state.apply(&frame) {
            report.boundary = Some(failed(if matches!(e, LedgerError::Limit) {
                BoundaryKind::Limit
            } else {
                BoundaryKind::Order
            }));
            break;
        }
        report.frames.push(frame);
        report.last_digest = hex(&hash);
        report.valid_bytes += total;
    }
    Ok(report)
}
fn absolute(path: &Path) -> Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    if path.file_name().is_none() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(LedgerError::Invalid);
    }
    Ok(path)
}
fn check_components(path: &Path) -> Result<()> {
    for p in path.ancestors() {
        let m = fs::symlink_metadata(p)?;
        if platform::reparse(&m) {
            return Err(LedgerError::Substitution);
        }
    }
    Ok(())
}
fn check_identity(file: &File, path: &Path, directory: bool) -> Result<()> {
    check_components(path)?;
    let meta = file.metadata()?;
    if platform::reparse(&meta) || (directory && !meta.is_dir()) || (!directory && !meta.is_file())
    {
        return Err(LedgerError::Substitution);
    }
    let other = if directory {
        platform::directory(path)?
    } else {
        platform::regular(path, false, false)?
    };
    let (identity, links) = platform::identity(file)?;
    let (other_identity, other_links) = platform::identity(&other)?;
    if identity != other_identity || (!directory && (links != 1 || other_links != 1)) {
        return Err(LedgerError::Substitution);
    }
    Ok(())
}

#[cfg(windows)]
mod platform {
    use super::*;
    use std::os::windows::{
        fs::{MetadataExt, OpenOptionsExt},
        io::AsRawHandle,
    };
    const NOFOLLOW: u32 = 0x0020_0000;
    const BACKUP: u32 = 0x0200_0000;
    pub fn regular(path: &Path, write: bool, create: bool) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .write(write)
            .create_new(create)
            .share_mode(3)
            .custom_flags(NOFOLLOW)
            .open(path)
    }
    pub fn directory(path: &Path) -> io::Result<File> {
        // FlushFileBuffers requires write access; a read-only backup-semantics
        // handle is NOT a directory-durability implementation. Deny delete share.
        OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(3)
            .custom_flags(BACKUP | NOFOLLOW)
            .open(path)
    }
    pub fn reparse(m: &fs::Metadata) -> bool {
        m.file_attributes() & 0x400 != 0
    }
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
        fn GetFileInformationByHandle(handle: *mut std::ffi::c_void, info: *mut Info) -> i32;
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
}
#[cfg(unix)]
mod platform {
    use super::*;
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
    #[cfg(target_os = "linux")]
    const NOFOLLOW: i32 = 0x20000;
    #[cfg(target_os = "macos")]
    const NOFOLLOW: i32 = 0x100;
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    compile_error!("ledger platform not qualified");
    #[cfg(target_os = "linux")]
    const NONBLOCK: i32 = 0x800;
    #[cfg(target_os = "macos")]
    const NONBLOCK: i32 = 0x4;
    pub fn regular(path: &Path, write: bool, create: bool) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .write(write)
            .create_new(create)
            .mode(0o600)
            .custom_flags(NOFOLLOW | NONBLOCK)
            .open(path)
    }
    pub fn directory(path: &Path) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .custom_flags(NOFOLLOW | NONBLOCK)
            .open(path)
    }
    pub fn reparse(m: &fs::Metadata) -> bool {
        m.file_type().is_symlink()
    }
    pub fn identity(file: &File) -> io::Result<((u64, u64), u64)> {
        let m = file.metadata()?;
        Ok(((m.dev(), m.ino()), m.nlink()))
    }
}

#[cfg(all(test, windows))]
mod durability_tests {
    use super::*;
    use std::os::windows::fs::OpenOptionsExt;
    #[test]
    fn actual_directory_flush_failure_returns_no_permit_and_poisoned_session() {
        let root = std::env::temp_dir().join(format!("binder-ledger-flush-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let path = root.join("ledger");
        let mut w = LedgerWriter::open(&path).unwrap();
        // Actual Windows syscall negative control: FlushFileBuffers on a
        // read-only directory handle must fail, despite completed file sync.
        w.parent = OpenOptions::new()
            .read(true)
            .share_mode(3)
            .custom_flags(0x0220_0000)
            .open(&root)
            .unwrap();
        let q = RequestBinding {
            registry_id: "1".repeat(64),
            entry_index: 1,
            entry_hash: "2".repeat(64),
            event_type_id: 300,
            event_record_id: "3".repeat(64),
        };
        let i = DispatchIntent {
            approved_output_root: root.to_string_lossy().into_owned(),
            mandatory_files: vec!["submission.json".into()],
            command: vec!["fixture".into()],
            binder_identity: "fixture".into(),
            limits: CommandLimits::default(),
            predecessor: None,
        };
        assert!(matches!(
            w.append_intent(q.clone(), "a", i.clone()),
            Err(LedgerError::Io(_))
        ));
        assert!(matches!(
            w.append_intent(q, "b", i),
            Err(LedgerError::Poisoned)
        ));
        assert_eq!(w.report().frames.len(), 0);
        drop(w);
        let r = read_ledger(&path).unwrap();
        assert_eq!(r.frames.len(), 1);
        assert_eq!(r.attempts()[0].launch, LaunchProjection::Unresolved);
        let mut w = LedgerWriter::open(&path).unwrap();
        assert!(matches!(
            w.append_observation("a", Observation::Spawned { pid: 1 }),
            Err(LedgerError::NotLive)
        ));
        drop(w);
        fs::remove_dir_all(root).unwrap();
    }
}
