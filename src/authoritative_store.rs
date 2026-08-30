use super::*;
use std::fs;
use std::path::{Path, PathBuf};

/// A fail-closed error while opening the authoritative on-disk Registry namespaces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthoritativeRegistryStoreOpenError {
    Io,
    RootNamespaceInvalid,
    RegistryNamespaceInvalid,
    JournalNamespaceInvalid,
    JournalSlotNameInvalid,
    JournalSlotSequenceInvalid,
    RecordNamespaceInvalid,
    RecordFilenameInvalid,
    RecordDecode,
    RecordIdentityMismatch,
    GenesisRecordDecode,
    GenesisRecordIdentityMismatch,
    GenesisRegistryMismatch,
    GenesisProfileMismatch,
    GenesisCapabilityMismatch,
    GenesisEnvironmentMismatch,
    RetainedJournal(RetainedJournalError),
}

/// An authoritative Registry store opened from its exact retained Journal and Record namespaces.
///
/// The root path is only a locator. Registry identity, current head, Journal history, and Record
/// identities are derived from strict retained bytes. No caller-selected Journal head, Record body,
/// or prevalidated authority token participates in resolution.
#[derive(Debug)]
pub struct AuthoritativeRegistryStore {
    root: PathBuf,
    retained_journal: RetainedJournal,
    records: Vec<(RecordId, Vec<u8>)>,
}

/// A positive Freeze-authority witness derived only from one opened authoritative Registry store.
///
/// This type has no public constructor. Its exact START, Receipt, Manifest, and terminal event
/// identities have already passed retained-Journal, strict Record, identity, dependency, and
/// continuity validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthoritativeFreezeCommittedBinding {
    registry_id: RegistryId,
    committed_event_reference: JournalReference,
    start_event_reference: JournalReference,
    receipt_record_id: RecordId,
    manifest_record_id: RecordId,
    start_record_id: RecordId,
    freeze_attempt_id: FreezeAttemptId,
    freeze_id: [u8; ID_LENGTH],
    subject_id: [u8; ID_LENGTH],
    policy_record_id: RecordId,
}

impl AuthoritativeFreezeCommittedBinding {
    pub fn registry_id(&self) -> RegistryId {
        self.registry_id
    }

    pub fn committed_event_reference(&self) -> &JournalReference {
        &self.committed_event_reference
    }

    pub fn start_event_reference(&self) -> &JournalReference {
        &self.start_event_reference
    }

    pub fn receipt_record_id(&self) -> RecordId {
        self.receipt_record_id
    }

    pub fn manifest_record_id(&self) -> RecordId {
        self.manifest_record_id
    }

    pub fn start_record_id(&self) -> RecordId {
        self.start_record_id
    }

    pub fn freeze_attempt_id(&self) -> FreezeAttemptId {
        self.freeze_attempt_id
    }

    pub fn freeze_id(&self) -> [u8; ID_LENGTH] {
        self.freeze_id
    }

    pub fn subject_id(&self) -> [u8; ID_LENGTH] {
        self.subject_id
    }

    pub fn policy_record_id(&self) -> RecordId {
        self.policy_record_id
    }
}

/// A fail-closed error before positive Freeze authority can be established.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthoritativeFreezeCommittedBindingError {
    Structural(ResolvedFreezeCommittedBindingError),
}

impl AuthoritativeRegistryStore {
    /// Opens one authoritative Registry from exact namespace bytes.
    ///
    /// Journal slots must be a contiguous sequence of exact twenty-digit names. Every retained
    /// Entry is strictly decoded and replayed. Every Record namespace object must have the exact
    /// lowercase content-addressed filename and strict self-hash identity required by its bytes.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, AuthoritativeRegistryStoreOpenError> {
        let root = root.as_ref();
        ensure_real_directory(root)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RootNamespaceInvalid)?;
        let root = fs::canonicalize(root).map_err(|_| AuthoritativeRegistryStoreOpenError::Io)?;
        let registry_dir = root.join("registry");
        let journal_dir = root.join("journal");
        let records_dir = root.join("records");
        ensure_real_directory(&registry_dir)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RegistryNamespaceInvalid)?;
        ensure_real_directory(&journal_dir)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::JournalNamespaceInvalid)?;
        ensure_real_directory(&records_dir)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RecordNamespaceInvalid)?;

        let genesis_record_bytes = read_regular_file(&registry_dir.join("genesis.cbor"))
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RegistryNamespaceInvalid)?;
        let genesis_record = GenesisRecord::decode_authoritative(&genesis_record_bytes)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::GenesisRecordDecode)?;

        let mut records = load_record_namespace(&records_dir)?;
        match records
            .iter()
            .find(|(record_id, _)| *record_id == genesis_record.record_id())
        {
            Some((_, bytes)) if bytes != &genesis_record_bytes => {
                return Err(AuthoritativeRegistryStoreOpenError::GenesisRecordIdentityMismatch)
            }
            Some(_) => {}
            None => records.push((genesis_record.record_id(), genesis_record_bytes)),
        }
        records.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));

        let journal_slots = load_journal_slots(&journal_dir)?;
        let genesis_entry =
            GenesisJournalEntry::decode_authoritative(&journal_slots[0]).map_err(|_| {
                AuthoritativeRegistryStoreOpenError::RetainedJournal(
                    RetainedJournalError::DecodeError,
                )
            })?;
        if genesis_entry.event_record_id.as_bytes() != genesis_record.record_id().as_bytes() {
            return Err(AuthoritativeRegistryStoreOpenError::GenesisRecordIdentityMismatch);
        }
        if genesis_entry.registry_id != genesis_record.input.registry_id {
            return Err(AuthoritativeRegistryStoreOpenError::GenesisRegistryMismatch);
        }
        if genesis_record.input.journal_format_version != 1
            || genesis_record.input.record_identity_profile_id != 1
        {
            return Err(AuthoritativeRegistryStoreOpenError::GenesisProfileMismatch);
        }
        if genesis_entry.storage_capability_class_id
            != genesis_record.input.storage_capability_class_id
        {
            return Err(AuthoritativeRegistryStoreOpenError::GenesisCapabilityMismatch);
        }
        if genesis_entry.environment_observation_id
            != genesis_record.input.environment_observation_id
        {
            return Err(AuthoritativeRegistryStoreOpenError::GenesisEnvironmentMismatch);
        }

        let mut retained_journal = RetainedJournal::from_genesis(genesis_entry)
            .map_err(AuthoritativeRegistryStoreOpenError::RetainedJournal)?;
        for entry_bytes in journal_slots.iter().skip(1) {
            retained_journal
                .append_strict_entry(entry_bytes)
                .map_err(AuthoritativeRegistryStoreOpenError::RetainedJournal)?;
        }

        Ok(Self {
            root,
            retained_journal,
            records,
        })
    }

    /// The canonical root locator from which this store's authority namespaces were opened.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The exact retained Journal reconstructed from the authoritative Journal namespace.
    pub fn retained_journal(&self) -> &RetainedJournal {
        &self.retained_journal
    }

    /// Establishes positive Freeze authority from this store's exact retained namespaces.
    pub fn validate_freeze_committed_authority(
        &self,
        committed_event_reference: JournalReference,
    ) -> Result<AuthoritativeFreezeCommittedBinding, AuthoritativeFreezeCommittedBindingError> {
        match validate_resolved_freeze_committed_binding(
            &self.retained_journal,
            committed_event_reference.clone(),
            self,
        )
        .map_err(AuthoritativeFreezeCommittedBindingError::Structural)?
        {
            ResolvedFreezeCommittedBindingOutcome::AuthorityEvidenceUnavailable => {}
        }

        let receipt_record_id = RecordId::try_from(
            committed_event_reference
                .event_record_id()
                .as_bytes()
                .as_slice(),
        )
        .expect("EventRecordId has RecordId width");
        let receipt_bytes = self
            .resolve(receipt_record_id)
            .expect("the structural binding resolved the exact Receipt bytes");
        let receipt = FreezeReceiptRecord::decode_authoritative(receipt_bytes)
            .expect("the structural binding strictly decoded the Receipt");
        let start_event_reference = receipt.input().attempt_start_journal_ref.clone();
        let start_record_id = RecordId::try_from(
            start_event_reference
                .event_record_id()
                .as_bytes()
                .as_slice(),
        )
        .expect("EventRecordId has RecordId width");

        Ok(AuthoritativeFreezeCommittedBinding {
            registry_id: self.retained_journal.registry_id,
            committed_event_reference,
            start_event_reference,
            receipt_record_id,
            manifest_record_id: receipt.input().manifest_id,
            start_record_id,
            freeze_attempt_id: receipt.input().freeze_attempt_id,
            freeze_id: receipt.input().freeze_id,
            subject_id: receipt.input().subject_id,
            policy_record_id: receipt.input().policy_record_id,
        })
    }
}

impl ExactRecordByteResolver for AuthoritativeRegistryStore {
    fn resolve(&self, record_id: RecordId) -> Option<&[u8]> {
        self.records
            .binary_search_by(|(stored_id, _)| stored_id.as_bytes().cmp(record_id.as_bytes()))
            .ok()
            .map(|index| self.records[index].1.as_slice())
    }
}

fn ensure_real_directory(path: &Path) -> Result<(), ()> {
    let metadata = fs::symlink_metadata(path).map_err(|_| ())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(());
    }
    Ok(())
}

fn read_regular_file(path: &Path) -> Result<Vec<u8>, ()> {
    let metadata = fs::symlink_metadata(path).map_err(|_| ())?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(());
    }
    fs::read(path).map_err(|_| ())
}

fn load_journal_slots(
    journal_dir: &Path,
) -> Result<Vec<Vec<u8>>, AuthoritativeRegistryStoreOpenError> {
    let mut slots = Vec::new();
    for entry in fs::read_dir(journal_dir).map_err(|_| AuthoritativeRegistryStoreOpenError::Io)? {
        let entry = entry.map_err(|_| AuthoritativeRegistryStoreOpenError::Io)?;
        let file_type = entry
            .file_type()
            .map_err(|_| AuthoritativeRegistryStoreOpenError::Io)?;
        if !file_type.is_file() || file_type.is_symlink() {
            return Err(AuthoritativeRegistryStoreOpenError::JournalNamespaceInvalid);
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| AuthoritativeRegistryStoreOpenError::JournalSlotNameInvalid)?;
        let index = parse_journal_slot_name(&name)
            .ok_or(AuthoritativeRegistryStoreOpenError::JournalSlotNameInvalid)?;
        let bytes = read_regular_file(&entry.path())
            .map_err(|_| AuthoritativeRegistryStoreOpenError::Io)?;
        slots.push((index, bytes));
    }
    slots.sort_by_key(|(index, _)| *index);
    if slots.is_empty()
        || slots
            .iter()
            .enumerate()
            .any(|(expected, (actual, _))| u64::try_from(expected).ok() != Some(*actual))
    {
        return Err(AuthoritativeRegistryStoreOpenError::JournalSlotSequenceInvalid);
    }
    Ok(slots.into_iter().map(|(_, bytes)| bytes).collect())
}

fn parse_journal_slot_name(name: &str) -> Option<u64> {
    let digits = name.strip_suffix(".cbor")?;
    if digits.len() != 20 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

fn load_record_namespace(
    records_dir: &Path,
) -> Result<Vec<(RecordId, Vec<u8>)>, AuthoritativeRegistryStoreOpenError> {
    let mut records = Vec::new();
    for entry in fs::read_dir(records_dir).map_err(|_| AuthoritativeRegistryStoreOpenError::Io)? {
        let entry = entry.map_err(|_| AuthoritativeRegistryStoreOpenError::Io)?;
        let file_type = entry
            .file_type()
            .map_err(|_| AuthoritativeRegistryStoreOpenError::Io)?;
        if !file_type.is_file() || file_type.is_symlink() {
            return Err(AuthoritativeRegistryStoreOpenError::RecordNamespaceInvalid);
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RecordFilenameInvalid)?;
        let expected_id = parse_record_filename(&name)
            .ok_or(AuthoritativeRegistryStoreOpenError::RecordFilenameInvalid)?;
        let bytes = read_regular_file(&entry.path())
            .map_err(|_| AuthoritativeRegistryStoreOpenError::Io)?;
        let frame = StrictRecordFrame::decode_authoritative(&bytes)
            .map_err(|_| AuthoritativeRegistryStoreOpenError::RecordDecode)?;
        if frame.record_id() != expected_id {
            return Err(AuthoritativeRegistryStoreOpenError::RecordIdentityMismatch);
        }
        records.push((expected_id, bytes));
    }
    records.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
    if records
        .windows(2)
        .any(|pair| pair[0].0.as_bytes() == pair[1].0.as_bytes())
    {
        return Err(AuthoritativeRegistryStoreOpenError::RecordNamespaceInvalid);
    }
    Ok(records)
}

fn parse_record_filename(name: &str) -> Option<RecordId> {
    let hex = name.strip_suffix(".cbor")?;
    if hex.len() != ID_LENGTH * 2
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return None;
    }
    let mut bytes = [0_u8; ID_LENGTH];
    for (index, slot) in bytes.iter_mut().enumerate() {
        let offset = index * 2;
        *slot = u8::from_str_radix(&hex[offset..offset + 2], 16).ok()?;
    }
    RecordId::try_from(bytes.as_slice()).ok()
}
