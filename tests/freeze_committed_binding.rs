use evidence_registry::{
    derive_freeze_root, validate_freeze_committed_binding,
    validate_resolved_freeze_committed_binding, AuthoritativeRegistryStore, EventRecordId,
    EventTypeId, ExactRecordByteResolver, FreezeAttemptId, FreezeAttemptStartRecord,
    FreezeAttemptStartRecordInput, FreezeCommittedBindingInput, FreezeCommittedBindingOutcome,
    FreezeReceiptRecord, GenesisJournalEntry, GenesisRecord, GenesisRecordInput, IntendedRootId,
    JournalEntryHash, JournalEntryIndex, JournalReference, RecordId, RegistryId,
    ResolvedFreezeCommittedBindingError, ResolvedFreezeCommittedBindingOutcome, RetainedJournal,
    StrictRecordFrame,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const GENESIS_HASH_HEX: &str = "ae1919549c80a0c3708cce0485af881a5223fbcf80c6b27a2bbe602418dabd51";
const START_RECORD_ID_HEX: &str =
    "949f6d7f389641c8c615904f8b1473083b8b029f9866873de4bec17d4e1a685c";
const START_HASH_HEX: &str = "0d678a8c08b32c670e27eb5e815f8077e9be62f5c8578d19ce72835bd88ba168";
const RECEIPT_ID_HEX: &str = "e44f96285b374af8a593d34ad0059f3c0d957eeca2db2b1e5ab20bc84a7a413d";
const COMMITTED_HASH_HEX: &str = "9681583be58ff6e25b13173d329587dffcbfa79d651106ec3f3877d243e685a0";
const MANIFEST_RECORD_HEX: &str = concat!(
    "84781a45766964656e636552656769737472792e5265636f72642e76310201a700010102105820",
    "e0e1e2e3e4e5e6e7e8e9eaebecedeeeff0f1f2f3f4f5f6f7f8f9fafbfcfdfeff110112011301148185",
    "0181416100015820000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
);

const START_RECORD_HEX: &str = concat!(
    "84781a45766964656e636552656769737472792e5265636f72642e76310301a600010103105820",
    "a0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebf115820",
    "7fa3d1bcfc31bc7b43176aec28a6d3d944acf931d877c80c6b61f788abe57db7125820",
    "e0e1e2e3e4e5e6e7e8e9eaebecedeeeff0f1f2f3f4f5f6f7f8f9fafbfcfdfeff135820",
    "404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f"
);
const START_ENTRY_HEX: &str = concat!(
    "82782045766964656e636552656769737472792e4a6f75726e616c456e7472792e7631ae0001015820",
    "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f0201035820",
    "ae1919549c80a0c3708cce0485af881a5223fbcf80c6b27a2bbe602418dabd51041864055820",
    "949f6d7f389641c8c615904f8b1473083b8b029f9866873de4bec17d4e1a685c068007800802095820",
    "a0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebf0a5820",
    "404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f0b5820",
    "606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f105820",
    "a0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebf115820",
    "7fa3d1bcfc31bc7b43176aec28a6d3d944acf931d877c80c6b61f788abe57db7"
);
const RECEIPT_HEX: &str = concat!(
    "84781a45766964656e636552656769737472792e5265636f72642e76310401b100010104105820",
    "a0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebf115820",
    "b0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0c1c2c3c4c5c6c7c8c9cacbcccdcecf12855820",
    "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f015820",
    "0d678a8c08b32c670e27eb5e815f8077e9be62f5c8578d19ce72835bd88ba16818645820",
    "949f6d7f389641c8c615904f8b1473083b8b029f9866873de4bec17d4e1a685c135820",
    "e0e1e2e3e4e5e6e7e8e9eaebecedeeeff0f1f2f3f4f5f6f7f8f9fafbfcfdfeff145820",
    "b4f57d90cf94e711e7ca69bd8cc79120a0b792ff0bf3d10675f41ff72fb4fcb51501165820",
    "d0d1d2d3d4d5d6d7d8d9dadbdcdddedfe0e1e2e3e4e5e6e7e8e9eaebecedeeef170118185820",
    "f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff000102030405060708090a0b0c0d0e0f18195820",
    "404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f181a01181b01",
    "181c01181df5181f6474657374"
);
const COMMITTED_ENTRY_HEX: &str = concat!(
    "82782045766964656e636552656769737472792e4a6f75726e616c456e7472792e7631ae0001015820",
    "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f0202035820",
    "0d678a8c08b32c670e27eb5e815f8077e9be62f5c8578d19ce72835bd88ba168041865055820",
    "e44f96285b374af8a593d34ad0059f3c0d957eeca2db2b1e5ab20bc84a7a413d06800781855820",
    "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f015820",
    "0d678a8c08b32c670e27eb5e815f8077e9be62f5c8578d19ce72835bd88ba16818645820",
    "949f6d7f389641c8c615904f8b1473083b8b029f9866873de4bec17d4e1a685c0802095820",
    "a0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebf0a5820",
    "404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f0b5820",
    "606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f105820",
    "a0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebf12855820",
    "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f015820",
    "0d678a8c08b32c670e27eb5e815f8077e9be62f5c8578d19ce72835bd88ba16818645820",
    "949f6d7f389641c8c615904f8b1473083b8b029f9866873de4bec17d4e1a685c"
);

fn hex_bytes(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).unwrap())
        .collect()
}

fn hex_id(hex: &str) -> [u8; 32] {
    hex_bytes(hex).try_into().unwrap()
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

static NEXT_AUTHORITATIVE_STORE_FIXTURE: AtomicU64 = AtomicU64::new(1);

struct AuthoritativeStoreFixtureDir {
    path: PathBuf,
}

impl AuthoritativeStoreFixtureDir {
    fn new() -> Self {
        let sequence = NEXT_AUTHORITATIVE_STORE_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "evidence-registry-authoritative-freeze-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        fs::create_dir(path.join("registry")).unwrap();
        fs::create_dir(path.join("journal")).unwrap();
        fs::create_dir(path.join("records")).unwrap();
        Self { path }
    }
}

impl Drop for AuthoritativeStoreFixtureDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn record_filename(record_id: RecordId) -> String {
    let mut name = String::with_capacity(69);
    for byte in record_id.as_bytes() {
        use std::fmt::Write as _;
        write!(&mut name, "{byte:02x}").unwrap();
    }
    name.push_str(".cbor");
    name
}

fn write_record(root: &Path, record_id: RecordId, bytes: &[u8]) {
    fs::write(root.join("records").join(record_filename(record_id)), bytes).unwrap();
}

fn fixed_binding_fixture() -> (
    RetainedJournal,
    JournalReference,
    FreezeAttemptStartRecord,
    FreezeReceiptRecord,
) {
    let registry_id = RegistryId::try_from(
        hex_id("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f").as_slice(),
    )
    .unwrap();
    let genesis = evidence_registry::GenesisJournalEntry::new(
        registry_id,
        EventRecordId::try_from(
            hex_id("202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f").as_slice(),
        )
        .unwrap(),
        RecordId::try_from(
            hex_id("404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f").as_slice(),
        )
        .unwrap(),
        RecordId::try_from(
            hex_id("606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f").as_slice(),
        )
        .unwrap(),
    );
    assert_eq!(genesis.entry_hash().as_bytes(), &hex_id(GENESIS_HASH_HEX));
    let start_bytes = hex_bytes(START_ENTRY_HEX);
    let start_record_bytes = hex_bytes(START_RECORD_HEX);
    assert_eq!(sha256(&start_record_bytes), hex_id(START_RECORD_ID_HEX));
    let start_record = FreezeAttemptStartRecord::decode_authoritative(&start_record_bytes).unwrap();
    assert_eq!(
        start_record.record_id().as_bytes(),
        &hex_id(START_RECORD_ID_HEX)
    );
    let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
    journal.append_strict_entry(&start_bytes).unwrap();
    assert_eq!(
        journal
            .reconstruct_state()
            .unwrap()
            .journal_head_hash()
            .as_bytes(),
        &hex_id(START_HASH_HEX)
    );
    let receipt_bytes = hex_bytes(RECEIPT_HEX);
    assert_eq!(sha256(&receipt_bytes), hex_id(RECEIPT_ID_HEX));
    let receipt = FreezeReceiptRecord::decode_authoritative(&receipt_bytes).unwrap();
    assert_eq!(receipt.record_id().as_bytes(), &hex_id(RECEIPT_ID_HEX));
    journal
        .append_strict_entry(&hex_bytes(COMMITTED_ENTRY_HEX))
        .unwrap();
    assert_eq!(
        journal
            .reconstruct_state()
            .unwrap()
            .journal_head_hash()
            .as_bytes(),
        &hex_id(COMMITTED_HASH_HEX)
    );
    let committed_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(2_u64).unwrap(),
        JournalEntryHash::try_from(hex_id(COMMITTED_HASH_HEX).as_slice()).unwrap(),
        EventTypeId::try_from(101_u64).unwrap(),
        EventRecordId::try_from(hex_id(RECEIPT_ID_HEX).as_slice()).unwrap(),
    );
    (journal, committed_reference, start_record, receipt)
}

fn fixed_start_fixture() -> (
    RetainedJournal,
    RegistryId,
    FreezeAttemptStartRecord,
    JournalReference,
) {
    let registry_id = RegistryId::try_from(
        hex_id("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f").as_slice(),
    )
    .unwrap();
    let genesis = evidence_registry::GenesisJournalEntry::new(
        registry_id,
        EventRecordId::try_from(
            hex_id("202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f").as_slice(),
        )
        .unwrap(),
        RecordId::try_from(
            hex_id("404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f").as_slice(),
        )
        .unwrap(),
        RecordId::try_from(
            hex_id("606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f").as_slice(),
        )
        .unwrap(),
    );
    assert_eq!(genesis.entry_hash().as_bytes(), &hex_id(GENESIS_HASH_HEX));
    let start_record =
        FreezeAttemptStartRecord::decode_authoritative(&hex_bytes(START_RECORD_HEX)).unwrap();
    let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
    journal
        .append_strict_entry(&hex_bytes(START_ENTRY_HEX))
        .unwrap();
    let start_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(1_u64).unwrap(),
        JournalEntryHash::try_from(hex_id(START_HASH_HEX).as_slice()).unwrap(),
        EventTypeId::try_from(100_u64).unwrap(),
        EventRecordId::try_from(hex_id(START_RECORD_ID_HEX).as_slice()).unwrap(),
    );
    (journal, registry_id, start_record, start_reference)
}

fn replace_unique(bytes: &mut [u8], needle: &[u8], replacement_at_offset: usize, replacement: u8) {
    let matches = bytes
        .windows(needle.len())
        .enumerate()
        .filter_map(|(index, candidate)| (candidate == needle).then_some(index))
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "fixture marker must occur exactly once");
    bytes[matches[0] + replacement_at_offset] = replacement;
}

fn binding_result_for_receipt_bytes(
    receipt_bytes: &[u8],
) -> Result<FreezeCommittedBindingOutcome, evidence_registry::FreezeCommittedBindingError> {
    let (mut journal, registry_id, start_record, start_reference) = fixed_start_fixture();
    let receipt = FreezeReceiptRecord::decode_authoritative(receipt_bytes).unwrap();
    journal
        .append_strict_entry(&freeze_committed_bytes(
            JournalEntryHash::try_from(hex_id(START_HASH_HEX).as_slice()).unwrap(),
            &start_reference,
            receipt.record_id(),
        ))
        .unwrap();
    let committed_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(2_u64).unwrap(),
        journal.reconstruct_state().unwrap().journal_head_hash(),
        EventTypeId::try_from(101_u64).unwrap(),
        EventRecordId::try_from(receipt.record_id().as_bytes().as_slice()).unwrap(),
    );
    validate_freeze_committed_binding(FreezeCommittedBindingInput {
        retained_journal: &journal,
        committed_event_reference: committed_reference,
        freeze_attempt_start_record: &start_record,
        freeze_receipt_record: &receipt,
    })
}

fn binding_result_for_fixed_receipt_and_committed_entry(
    receipt_bytes: &[u8],
    committed_entry_bytes: &[u8],
    committed_event_record_id: [u8; 32],
) -> Result<FreezeCommittedBindingOutcome, evidence_registry::FreezeCommittedBindingError> {
    let (mut journal, registry_id, start_record, _) = fixed_start_fixture();
    let receipt = FreezeReceiptRecord::decode_authoritative(receipt_bytes).unwrap();
    journal.append_strict_entry(committed_entry_bytes).unwrap();
    let committed_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(2_u64).unwrap(),
        JournalEntryHash::try_from(sha256(committed_entry_bytes).as_slice()).unwrap(),
        EventTypeId::try_from(101_u64).unwrap(),
        EventRecordId::try_from(committed_event_record_id.as_slice()).unwrap(),
    );
    validate_freeze_committed_binding(FreezeCommittedBindingInput {
        retained_journal: &journal,
        committed_event_reference: committed_reference,
        freeze_attempt_start_record: &start_record,
        freeze_receipt_record: &receipt,
    })
}

fn id(first: u8) -> [u8; 32] {
    core::array::from_fn(|index| first.wrapping_add(index as u8))
}

fn append_bstr_32(output: &mut Vec<u8>, value: &[u8; 32]) {
    output.extend_from_slice(&[0x58, 0x20]);
    output.extend_from_slice(value);
}

fn append_reference(output: &mut Vec<u8>, reference: &JournalReference) {
    output.push(0x85);
    append_bstr_32(output, reference.registry_id().as_bytes());
    output.push(reference.entry_index().value() as u8);
    append_bstr_32(output, reference.entry_hash().as_bytes());
    output.extend_from_slice(&[0x18, reference.event_type_id().value() as u8]);
    append_bstr_32(output, reference.event_record_id().as_bytes());
}

fn start_record(registry_id: RegistryId) -> FreezeAttemptStartRecord {
    let freeze_attempt_id = FreezeAttemptId::try_from(id(0xa0).as_slice()).unwrap();
    FreezeAttemptStartRecord::new(FreezeAttemptStartRecordInput {
        freeze_attempt_id,
        intended_root_id: derive_freeze_root(registry_id, freeze_attempt_id).intended_root_id(),
        subject_id: id(0xe0),
        policy_record_id: RecordId::try_from(id(0x40).as_slice()).unwrap(),
    })
}

fn freeze_start_bytes(
    previous_entry_hash: JournalEntryHash,
    start_record: &FreezeAttemptStartRecord,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(299);
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.push(0xae);
    bytes.extend_from_slice(&[0x00, 0x01, 0x01]);
    append_bstr_32(&mut bytes, &id(0x00));
    bytes.extend_from_slice(&[0x02, 0x01, 0x03]);
    append_bstr_32(&mut bytes, previous_entry_hash.as_bytes());
    bytes.extend_from_slice(&[0x04, 0x18, 0x64, 0x05]);
    append_bstr_32(&mut bytes, start_record.record_id().as_bytes());
    bytes.extend_from_slice(&[0x06, 0x80, 0x07, 0x80, 0x08, 0x02, 0x09]);
    append_bstr_32(&mut bytes, &id(0xa0));
    bytes.push(0x0a);
    append_bstr_32(&mut bytes, &id(0x40));
    bytes.push(0x0b);
    append_bstr_32(&mut bytes, &id(0x60));
    bytes.push(0x10);
    append_bstr_32(
        &mut bytes,
        start_record.input().freeze_attempt_id.as_bytes(),
    );
    bytes.push(0x11);
    append_bstr_32(&mut bytes, start_record.input().intended_root_id.as_bytes());
    bytes
}

fn receipt_bytes(start_reference: &JournalReference, manifest_id: RecordId) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(620);
    bytes.extend_from_slice(&[0x84, 0x78, 0x1a]);
    bytes.extend_from_slice(b"EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[0x04, 0x01, 0xb1, 0x00, 0x01, 0x01, 0x04]);
    bytes.push(0x10);
    append_bstr_32(&mut bytes, &id(0xa0));
    bytes.push(0x11);
    append_bstr_32(&mut bytes, &id(0xb0));
    bytes.push(0x12);
    append_reference(&mut bytes, start_reference);
    bytes.push(0x13);
    append_bstr_32(&mut bytes, &id(0xe0));
    bytes.push(0x14);
    append_bstr_32(&mut bytes, manifest_id.as_bytes());
    bytes.extend_from_slice(&[0x15, 0x01]);
    bytes.push(0x16);
    append_bstr_32(&mut bytes, &id(0xd0));
    bytes.extend_from_slice(&[0x17, 0x01]);
    bytes.extend_from_slice(&[0x18, 0x18]);
    append_bstr_32(&mut bytes, &id(0xf0));
    bytes.extend_from_slice(&[0x18, 0x19]);
    append_bstr_32(&mut bytes, &id(0x40));
    bytes.extend_from_slice(&[
        0x18, 0x1a, 0x01, 0x18, 0x1b, 0x01, 0x18, 0x1c, 0x01, 0x18, 0x1d, 0xf5, 0x18, 0x1f, 0x64,
    ]);
    bytes.extend_from_slice(b"test");
    bytes
}

fn freeze_committed_bytes(
    previous_entry_hash: JournalEntryHash,
    start_reference: &JournalReference,
    receipt_record_id: RecordId,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(439);
    bytes.extend_from_slice(&[0x82, 0x78, 0x20]);
    bytes.extend_from_slice(b"EvidenceRegistry.JournalEntry.v1");
    bytes.push(0xae);
    bytes.extend_from_slice(&[0x00, 0x01, 0x01]);
    append_bstr_32(&mut bytes, &id(0x00));
    bytes.extend_from_slice(&[0x02, 0x02, 0x03]);
    append_bstr_32(&mut bytes, previous_entry_hash.as_bytes());
    bytes.extend_from_slice(&[0x04, 0x18, 0x65, 0x05]);
    append_bstr_32(&mut bytes, receipt_record_id.as_bytes());
    bytes.extend_from_slice(&[0x06, 0x80, 0x07, 0x81]);
    append_reference(&mut bytes, start_reference);
    bytes.extend_from_slice(&[0x08, 0x02, 0x09]);
    append_bstr_32(&mut bytes, &id(0xa0));
    bytes.push(0x0a);
    append_bstr_32(&mut bytes, &id(0x40));
    bytes.push(0x0b);
    append_bstr_32(&mut bytes, &id(0x60));
    bytes.push(0x10);
    append_bstr_32(&mut bytes, &id(0xa0));
    bytes.push(0x12);
    append_reference(&mut bytes, start_reference);
    bytes
}

#[test]
fn freeze_committed_binding_remains_structural_without_resolved_manifest_evidence() {
    let registry_id = RegistryId::try_from(id(0x00).as_slice()).unwrap();
    let start_record = start_record(registry_id);
    let genesis = evidence_registry::GenesisJournalEntry::new(
        registry_id,
        EventRecordId::try_from(id(0x20).as_slice()).unwrap(),
        RecordId::try_from(id(0x40).as_slice()).unwrap(),
        RecordId::try_from(id(0x60).as_slice()).unwrap(),
    );
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    journal
        .append_strict_entry(&freeze_start_bytes(genesis.entry_hash(), &start_record))
        .unwrap();
    let start_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(1_u64).unwrap(),
        journal.reconstruct_state().unwrap().journal_head_hash(),
        EventTypeId::try_from(100_u64).unwrap(),
        EventRecordId::try_from(start_record.record_id().as_bytes().as_slice()).unwrap(),
    );
    let receipt_bytes = receipt_bytes(
        &start_reference,
        RecordId::try_from(id(0xc0).as_slice()).unwrap(),
    );
    assert!(StrictRecordFrame::decode_authoritative(&receipt_bytes).is_ok());
    let receipt = FreezeReceiptRecord::decode_authoritative(&receipt_bytes).unwrap();
    journal
        .append_strict_entry(&freeze_committed_bytes(
            start_reference.entry_hash(),
            &start_reference,
            receipt.record_id(),
        ))
        .unwrap();
    let committed_reference = JournalReference::new(
        RegistryId::try_from(id(0x00).as_slice()).unwrap(),
        JournalEntryIndex::try_from(2_u64).unwrap(),
        journal.reconstruct_state().unwrap().journal_head_hash(),
        EventTypeId::try_from(101_u64).unwrap(),
        EventRecordId::try_from(receipt.record_id().as_bytes().as_slice()).unwrap(),
    );

    assert!(matches!(
        validate_freeze_committed_binding(FreezeCommittedBindingInput {
            retained_journal: &journal,
            committed_event_reference: committed_reference,
            freeze_attempt_start_record: &start_record,
            freeze_receipt_record: &receipt,
        }),
        Ok(FreezeCommittedBindingOutcome::AuthorityEvidenceUnavailable)
    ));
}

#[test]
fn freeze_committed_binding_rejects_a_terminal_event_bound_to_a_different_receipt_identity() {
    let registry_id = RegistryId::try_from(id(0x00).as_slice()).unwrap();
    let start_record = start_record(registry_id);
    let genesis = evidence_registry::GenesisJournalEntry::new(
        registry_id,
        EventRecordId::try_from(id(0x20).as_slice()).unwrap(),
        RecordId::try_from(id(0x40).as_slice()).unwrap(),
        RecordId::try_from(id(0x60).as_slice()).unwrap(),
    );
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    journal
        .append_strict_entry(&freeze_start_bytes(genesis.entry_hash(), &start_record))
        .unwrap();
    let start_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(1_u64).unwrap(),
        journal.reconstruct_state().unwrap().journal_head_hash(),
        EventTypeId::try_from(100_u64).unwrap(),
        EventRecordId::try_from(start_record.record_id().as_bytes().as_slice()).unwrap(),
    );
    let receipt = FreezeReceiptRecord::decode_authoritative(&receipt_bytes(
        &start_reference,
        RecordId::try_from(id(0xc0).as_slice()).unwrap(),
    ))
    .unwrap();
    let different_receipt_id = RecordId::try_from(id(0x81).as_slice()).unwrap();
    journal
        .append_strict_entry(&freeze_committed_bytes(
            start_reference.entry_hash(),
            &start_reference,
            different_receipt_id,
        ))
        .unwrap();
    let committed_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(2_u64).unwrap(),
        journal.reconstruct_state().unwrap().journal_head_hash(),
        EventTypeId::try_from(101_u64).unwrap(),
        EventRecordId::try_from(different_receipt_id.as_bytes().as_slice()).unwrap(),
    );

    assert_eq!(
        validate_freeze_committed_binding(FreezeCommittedBindingInput {
            retained_journal: &journal,
            committed_event_reference: committed_reference,
            freeze_attempt_start_record: &start_record,
            freeze_receipt_record: &receipt,
        }),
        Err(evidence_registry::FreezeCommittedBindingError::ReceiptRecordMismatch)
    );
}

#[test]
fn freeze_committed_binding_accepts_only_the_independently_anchored_structural_fixture() {
    let (journal, committed_reference, start_record, receipt) = fixed_binding_fixture();

    assert_eq!(
        validate_freeze_committed_binding(FreezeCommittedBindingInput {
            retained_journal: &journal,
            committed_event_reference: committed_reference,
            freeze_attempt_start_record: &start_record,
            freeze_receipt_record: &receipt,
        }),
        Ok(FreezeCommittedBindingOutcome::AuthorityEvidenceUnavailable)
    );
}

#[test]
fn freeze_receipt_decoder_retains_exact_typed_authority_prerequisites() {
    let mut receipt_bytes = hex_bytes(RECEIPT_HEX);
    replace_unique(&mut receipt_bytes, &[0x17, 0x01, 0x18, 0x18, 0x58], 1, 0x02);
    replace_unique(&mut receipt_bytes, &[0x18, 0x1b, 0x01, 0x18, 0x1c], 2, 0x02);
    replace_unique(&mut receipt_bytes, &[0x18, 0x1c, 0x01, 0x18, 0x1d], 2, 0x03);
    let receipt = FreezeReceiptRecord::decode_authoritative(&receipt_bytes).unwrap();
    let input = receipt.input();

    assert_eq!(input.freeze_attempt_id.as_bytes(), &id(0xa0));
    assert_eq!(input.freeze_id, id(0xb0));
    assert_eq!(
        input.attempt_start_journal_ref.registry_id().as_bytes(),
        &id(0x00)
    );
    assert_eq!(input.attempt_start_journal_ref.entry_index().value(), 1);
    assert_eq!(
        input.attempt_start_journal_ref.entry_hash().as_bytes(),
        &hex_id(START_HASH_HEX)
    );
    assert_eq!(input.attempt_start_journal_ref.event_type_id().value(), 100);
    assert_eq!(
        input.attempt_start_journal_ref.event_record_id().as_bytes(),
        &hex_id(START_RECORD_ID_HEX)
    );
    assert_eq!(input.subject_id, id(0xe0));
    assert_eq!(
        input.manifest_id.as_bytes(),
        &hex_id("b4f57d90cf94e711e7ca69bd8cc79120a0b792ff0bf3d10675f41ff72fb4fcb5")
    );
    assert_eq!(input.custody_mode_id, 1);
    assert_eq!(input.creation_profile_ref.as_bytes(), &id(0xd0));
    assert_eq!(input.path_identity_profile_id, 2);
    assert_eq!(input.filesystem_profile_ref.as_bytes(), &id(0xf0));
    assert_eq!(input.policy_record_id.as_bytes(), &id(0x40));
    assert_eq!(input.file_content_flush_state, 1);
    assert_eq!(input.atomic_publish_no_replace_state, 2);
    assert_eq!(input.parent_directory_flush_state, 3);
    assert!(input.platform_strongest_available);
    assert_eq!(input.requested_commit_durability_ref, None);
    assert_eq!(input.created_by_tool_version, "test");
}

#[test]
fn freeze_receipt_decoder_rejects_one_mutation_per_strict_field_or_canonical_boundary() {
    let baseline = hex_bytes(RECEIPT_HEX);
    assert!(StrictRecordFrame::decode_authoritative(&baseline).is_ok());
    assert!(FreezeReceiptRecord::decode_authoritative(&baseline).is_ok());

    let mut wrong_type = baseline.clone();
    replace_unique(&mut wrong_type, &[0x04, 0x01, 0xb1], 0, 0x03);
    assert!(FreezeReceiptRecord::decode_authoritative(&wrong_type).is_err());

    let mut wrong_schema = baseline.clone();
    replace_unique(&mut wrong_schema, &[0x04, 0x01, 0xb1], 1, 0x02);
    assert!(FreezeReceiptRecord::decode_authoritative(&wrong_schema).is_err());

    let mut missing_field = baseline.clone();
    replace_unique(&mut missing_field, &[0xb1, 0x00, 0x01, 0x01, 0x04], 0, 0xb0);
    assert!(FreezeReceiptRecord::decode_authoritative(&missing_field).is_err());

    let mut non_increasing_key = baseline.clone();
    replace_unique(
        &mut non_increasing_key,
        &[0x01, 0x04, 0x10, 0x58, 0x20],
        2,
        0x01,
    );
    assert!(StrictRecordFrame::decode_authoritative(&non_increasing_key).is_err());
    assert!(FreezeReceiptRecord::decode_authoritative(&non_increasing_key).is_err());

    let mut malformed_start_reference = baseline.clone();
    replace_unique(
        &mut malformed_start_reference,
        &[0x12, 0x85, 0x58, 0x20],
        1,
        0x84,
    );
    assert!(FreezeReceiptRecord::decode_authoritative(&malformed_start_reference).is_err());

    let mut invalid_custody = baseline.clone();
    replace_unique(
        &mut invalid_custody,
        &[0x15, 0x01, 0x16, 0x58, 0x20],
        1,
        0x00,
    );
    assert!(FreezeReceiptRecord::decode_authoritative(&invalid_custody).is_err());

    let mut invalid_durability = baseline.clone();
    replace_unique(
        &mut invalid_durability,
        &[0x18, 0x1a, 0x01, 0x18, 0x1b],
        2,
        0x00,
    );
    assert!(FreezeReceiptRecord::decode_authoritative(&invalid_durability).is_err());

    let mut invalid_width = baseline.clone();
    replace_unique(&mut invalid_width, &[0x18, 0x19, 0x58, 0x20], 2, 0x57);
    assert!(FreezeReceiptRecord::decode_authoritative(&invalid_width).is_err());

    let mut noncanonical_type = baseline.clone();
    let type_offset = noncanonical_type
        .windows(3)
        .position(|window| window == [0x04, 0x01, 0xb1])
        .unwrap();
    noncanonical_type.insert(type_offset, 0x18);
    assert!(StrictRecordFrame::decode_authoritative(&noncanonical_type).is_err());
    assert!(FreezeReceiptRecord::decode_authoritative(&noncanonical_type).is_err());

    let mut wrong_outer_array = baseline.clone();
    replace_unique(&mut wrong_outer_array, &[0x84, 0x78, 0x1a, b'E'], 0, 0x85);
    assert!(StrictRecordFrame::decode_authoritative(&wrong_outer_array).is_err());
    assert!(FreezeReceiptRecord::decode_authoritative(&wrong_outer_array).is_err());

    let mut wrong_outer_domain = baseline;
    replace_unique(&mut wrong_outer_domain, &[0x78, 0x1a, b'E', b'v'], 2, b'X');
    assert!(StrictRecordFrame::decode_authoritative(&wrong_outer_domain).is_err());
    assert!(FreezeReceiptRecord::decode_authoritative(&wrong_outer_domain).is_err());
}

#[test]
fn freeze_receipt_decoder_retains_only_a_well_formed_optional_commit_durability_reference() {
    let mut optional = hex_bytes(RECEIPT_HEX);
    replace_unique(&mut optional, &[0xb1, 0x00, 0x01, 0x01, 0x04], 0, 0xb2);
    let terminal_key = optional
        .windows(3)
        .position(|window| window == [0x18, 0x1f, 0x64])
        .unwrap();
    optional.splice(
        terminal_key..terminal_key,
        [0x18, 0x1e, 0x58, 0x20].into_iter().chain(hex_id(
            "101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f",
        )),
    );
    let decoded_optional = FreezeReceiptRecord::decode_authoritative(&optional).unwrap();
    assert_eq!(
        decoded_optional
            .input()
            .requested_commit_durability_ref
            .unwrap()
            .as_bytes(),
        &hex_id("101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f")
    );

    let mut malformed_optional = optional;
    replace_unique(&mut malformed_optional, &[0x18, 0x1e, 0x58, 0x20], 2, 0x57);
    assert!(FreezeReceiptRecord::decode_authoritative(&malformed_optional).is_err());
}

#[test]
fn freeze_committed_binding_rejects_each_mutated_receipt_reference_relation() {
    let baseline = hex_bytes(RECEIPT_HEX);

    let mut foreign_registry = baseline.clone();
    replace_unique(
        &mut foreign_registry,
        &[0x12, 0x85, 0x58, 0x20, 0x00],
        4,
        0xff,
    );
    assert!(matches!(
        binding_result_for_receipt_bytes(&foreign_registry),
        Err(evidence_registry::FreezeCommittedBindingError::RetainedReference(_))
    ));

    let mut wrong_hash = baseline.clone();
    replace_unique(
        &mut wrong_hash,
        &[0x01, 0x58, 0x20, 0x0d, 0x67, 0x8a],
        3,
        0xff,
    );
    assert!(matches!(
        binding_result_for_receipt_bytes(&wrong_hash),
        Err(evidence_registry::FreezeCommittedBindingError::RetainedReference(_))
    ));

    let mut wrong_event = baseline.clone();
    replace_unique(
        &mut wrong_event,
        &[0x8b, 0xa1, 0x68, 0x18, 0x64, 0x58, 0x20],
        4,
        0x65,
    );
    assert!(matches!(
        binding_result_for_receipt_bytes(&wrong_event),
        Err(evidence_registry::FreezeCommittedBindingError::RetainedReference(_))
    ));

    let mut wrong_event_record_id = baseline.clone();
    replace_unique(
        &mut wrong_event_record_id,
        &[0x18, 0x64, 0x58, 0x20, 0x94, 0x9f, 0x6d],
        4,
        0x95,
    );
    assert!(matches!(
        binding_result_for_receipt_bytes(&wrong_event_record_id),
        Err(evidence_registry::FreezeCommittedBindingError::RetainedReference(_))
    ));

    let mut non_prior = baseline;
    replace_unique(
        &mut non_prior,
        &[0x1e, 0x1f, 0x01, 0x58, 0x20, 0x0d],
        2,
        0x02,
    );
    assert_eq!(
        binding_result_for_receipt_bytes(&non_prior),
        Err(evidence_registry::FreezeCommittedBindingError::AttemptStartNotPrior)
    );
}

#[test]
fn freeze_committed_binding_rejects_an_independently_mutated_terminal_receipt_identity() {
    let receipt_bytes = hex_bytes(RECEIPT_HEX);
    let mut committed_entry_bytes = hex_bytes(COMMITTED_ENTRY_HEX);
    replace_unique(
        &mut committed_entry_bytes,
        &[0x05, 0x58, 0x20, 0xe4, 0x4f, 0x96],
        3,
        0xe5,
    );
    let mut terminal_event_record_id = hex_id(RECEIPT_ID_HEX);
    terminal_event_record_id[0] = 0xe5;
    assert_eq!(
        binding_result_for_fixed_receipt_and_committed_entry(
            &receipt_bytes,
            &committed_entry_bytes,
            terminal_event_record_id,
        ),
        Err(evidence_registry::FreezeCommittedBindingError::ReceiptRecordMismatch)
    );
}

#[test]
fn freeze_committed_binding_rejects_each_mutated_record_identity_relation() {
    let baseline = hex_bytes(RECEIPT_HEX);

    let mut wrong_attempt = baseline.clone();
    replace_unique(&mut wrong_attempt, &[0x10, 0x58, 0x20, 0xa0], 3, 0xa1);
    assert_eq!(
        binding_result_for_receipt_bytes(&wrong_attempt),
        Err(evidence_registry::FreezeCommittedBindingError::FreezeAttemptMismatch)
    );

    let mut wrong_subject = baseline.clone();
    replace_unique(&mut wrong_subject, &[0x13, 0x58, 0x20, 0xe0], 3, 0xe1);
    assert_eq!(
        binding_result_for_receipt_bytes(&wrong_subject),
        Err(evidence_registry::FreezeCommittedBindingError::SubjectMismatch)
    );

    let mut wrong_policy = baseline;
    replace_unique(&mut wrong_policy, &[0x18, 0x19, 0x58, 0x20, 0x40], 4, 0x41);
    assert_eq!(
        binding_result_for_receipt_bytes(&wrong_policy),
        Err(evidence_registry::FreezeCommittedBindingError::PolicyMismatch)
    );

    let (journal, committed_reference, _, receipt) = fixed_binding_fixture();
    let mut alternate_start_bytes = hex_bytes(START_RECORD_HEX);
    replace_unique(
        &mut alternate_start_bytes,
        &[0x12, 0x58, 0x20, 0xe0],
        3,
        0xe1,
    );
    let alternate_start =
        FreezeAttemptStartRecord::decode_authoritative(&alternate_start_bytes).unwrap();
    assert_eq!(
        validate_freeze_committed_binding(FreezeCommittedBindingInput {
            retained_journal: &journal,
            committed_event_reference: committed_reference,
            freeze_attempt_start_record: &alternate_start,
            freeze_receipt_record: &receipt,
        }),
        Err(evidence_registry::FreezeCommittedBindingError::AttemptStartRecordMismatch)
    );
}

struct FixtureRecordResolver {
    records: Vec<(RecordId, Vec<u8>)>,
}

impl ExactRecordByteResolver for FixtureRecordResolver {
    fn resolve(&self, record_id: RecordId) -> Option<&[u8]> {
        self.records
            .iter()
            .find_map(|(stored_id, bytes)| (*stored_id == record_id).then_some(bytes.as_slice()))
    }
}

fn resolved_binding_result_for_manifest_bytes(
    manifest_bytes: Vec<u8>,
) -> Result<ResolvedFreezeCommittedBindingOutcome, ResolvedFreezeCommittedBindingError> {
    let registry_id = RegistryId::try_from(id(0x00).as_slice()).unwrap();
    let start_record = start_record(registry_id);
    let genesis = evidence_registry::GenesisJournalEntry::new(
        registry_id,
        EventRecordId::try_from(id(0x20).as_slice()).unwrap(),
        RecordId::try_from(id(0x40).as_slice()).unwrap(),
        RecordId::try_from(id(0x60).as_slice()).unwrap(),
    );
    let mut journal = RetainedJournal::from_genesis(genesis.clone()).unwrap();
    journal
        .append_strict_entry(&freeze_start_bytes(genesis.entry_hash(), &start_record))
        .unwrap();
    let start_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(1_u64).unwrap(),
        journal.reconstruct_state().unwrap().journal_head_hash(),
        EventTypeId::try_from(100_u64).unwrap(),
        EventRecordId::try_from(start_record.record_id().as_bytes().as_slice()).unwrap(),
    );
    let manifest_id = RecordId::try_from(sha256(&manifest_bytes).as_slice()).unwrap();
    let receipt_bytes = receipt_bytes(&start_reference, manifest_id);
    let receipt = FreezeReceiptRecord::decode_authoritative(&receipt_bytes).unwrap();
    journal
        .append_strict_entry(&freeze_committed_bytes(
            start_reference.entry_hash(),
            &start_reference,
            receipt.record_id(),
        ))
        .unwrap();
    let committed_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(2_u64).unwrap(),
        journal.reconstruct_state().unwrap().journal_head_hash(),
        EventTypeId::try_from(101_u64).unwrap(),
        EventRecordId::try_from(receipt.record_id().as_bytes().as_slice()).unwrap(),
    );
    let records = FixtureRecordResolver {
        records: vec![
            (start_record.record_id(), start_record.authoritative_cbor()),
            (receipt.record_id(), receipt_bytes),
            (manifest_id, manifest_bytes),
        ],
    };
    validate_resolved_freeze_committed_binding(&journal, committed_reference, &records)
}

#[test]
fn resolved_freeze_committed_binding_distinguishes_missing_and_invalid_record_payloads() {
    let (journal, committed_reference, start_record, receipt) = fixed_binding_fixture();
    let missing_receipt = FixtureRecordResolver {
        records: vec![(start_record.record_id(), hex_bytes(START_RECORD_HEX))],
    };
    assert_eq!(
        validate_resolved_freeze_committed_binding(
            &journal,
            committed_reference.clone(),
            &missing_receipt
        ),
        Err(ResolvedFreezeCommittedBindingError::ReceiptPayloadUnavailable)
    );

    let invalid_receipt = FixtureRecordResolver {
        records: vec![
            (start_record.record_id(), hex_bytes(START_RECORD_HEX)),
            (receipt.record_id(), vec![0]),
        ],
    };
    assert_eq!(
        validate_resolved_freeze_committed_binding(&journal, committed_reference, &invalid_receipt),
        Err(ResolvedFreezeCommittedBindingError::ReceiptDecode)
    );
}

#[test]
fn resolved_freeze_committed_binding_distinguishes_missing_invalid_and_wrong_identity_manifest_payloads(
) {
    let (journal, committed_reference, start_record, receipt) = fixed_binding_fixture();
    let missing_manifest = FixtureRecordResolver {
        records: vec![
            (start_record.record_id(), hex_bytes(START_RECORD_HEX)),
            (receipt.record_id(), hex_bytes(RECEIPT_HEX)),
        ],
    };
    assert_eq!(
        validate_resolved_freeze_committed_binding(
            &journal,
            committed_reference.clone(),
            &missing_manifest,
        ),
        Err(ResolvedFreezeCommittedBindingError::ManifestPayloadUnavailable)
    );

    let invalid_manifest = FixtureRecordResolver {
        records: vec![
            (start_record.record_id(), hex_bytes(START_RECORD_HEX)),
            (receipt.record_id(), hex_bytes(RECEIPT_HEX)),
            (receipt.input().manifest_id, vec![0]),
        ],
    };
    assert_eq!(
        validate_resolved_freeze_committed_binding(
            &journal,
            committed_reference.clone(),
            &invalid_manifest,
        ),
        Err(ResolvedFreezeCommittedBindingError::ManifestDecode)
    );

    let mut alternate_manifest = hex_bytes(MANIFEST_RECORD_HEX);
    replace_unique(&mut alternate_manifest, &[0x10, 0x58, 0x20, 0xe0], 3, 0xe1);
    let mismatched_manifest = FixtureRecordResolver {
        records: vec![
            (start_record.record_id(), hex_bytes(START_RECORD_HEX)),
            (receipt.record_id(), hex_bytes(RECEIPT_HEX)),
            (receipt.input().manifest_id, alternate_manifest),
        ],
    };
    assert_eq!(
        validate_resolved_freeze_committed_binding(
            &journal,
            committed_reference,
            &mismatched_manifest
        ),
        Err(ResolvedFreezeCommittedBindingError::ManifestIdentityMismatch)
    );
}

#[test]
fn resolved_freeze_committed_binding_reaches_exact_manifest_subject_continuity_gate() {
    let mut wrong_subject_manifest = hex_bytes(MANIFEST_RECORD_HEX);
    replace_unique(
        &mut wrong_subject_manifest,
        &[0x10, 0x58, 0x20, 0xe0],
        3,
        0xe1,
    );

    assert!(StrictRecordFrame::decode_authoritative(&wrong_subject_manifest).is_ok());
    assert_eq!(
        resolved_binding_result_for_manifest_bytes(wrong_subject_manifest),
        Err(ResolvedFreezeCommittedBindingError::ManifestSubjectMismatch)
    );
}

#[test]
fn resolved_freeze_committed_binding_reaches_exact_manifest_path_profile_continuity_gate() {
    let mut wrong_profile_manifest = hex_bytes(MANIFEST_RECORD_HEX);
    replace_unique(
        &mut wrong_profile_manifest,
        &[0x11, 0x01, 0x12, 0x01, 0x13, 0x01, 0x14],
        3,
        0x02,
    );

    assert!(StrictRecordFrame::decode_authoritative(&wrong_profile_manifest).is_ok());
    assert_eq!(
        resolved_binding_result_for_manifest_bytes(wrong_profile_manifest),
        Err(ResolvedFreezeCommittedBindingError::ManifestPathIdentityProfileMismatch)
    );
}

#[test]
fn resolved_freeze_committed_binding_rejects_a_valid_receipt_with_the_wrong_exact_identity() {
    let (journal, committed_reference, start_record, receipt) = fixed_binding_fixture();
    let mut alternate_receipt_bytes = hex_bytes(RECEIPT_HEX);
    replace_unique(
        &mut alternate_receipt_bytes,
        &[0x11, 0x58, 0x20, 0xb0],
        3,
        0xb1,
    );
    let records = FixtureRecordResolver {
        records: vec![
            (start_record.record_id(), hex_bytes(START_RECORD_HEX)),
            (receipt.record_id(), alternate_receipt_bytes),
        ],
    };

    assert_eq!(
        validate_resolved_freeze_committed_binding(&journal, committed_reference, &records),
        Err(ResolvedFreezeCommittedBindingError::ReceiptIdentityMismatch)
    );
}

#[test]
fn resolved_freeze_committed_binding_distinguishes_missing_start_payload() {
    let (journal, committed_reference, _, receipt) = fixed_binding_fixture();
    let missing_start = FixtureRecordResolver {
        records: vec![
            (receipt.record_id(), hex_bytes(RECEIPT_HEX)),
            (receipt.input().manifest_id, hex_bytes(MANIFEST_RECORD_HEX)),
        ],
    };

    assert_eq!(
        validate_resolved_freeze_committed_binding(&journal, committed_reference, &missing_start),
        Err(ResolvedFreezeCommittedBindingError::AttemptStartPayloadUnavailable)
    );
}

#[test]
fn resolved_freeze_committed_binding_rejects_wrong_type_and_invalid_or_mismatched_start_payloads() {
    let (journal, committed_reference, start_record, receipt) = fixed_binding_fixture();
    let wrong_receipt_type = FixtureRecordResolver {
        records: vec![
            (start_record.record_id(), hex_bytes(START_RECORD_HEX)),
            (receipt.record_id(), hex_bytes(START_RECORD_HEX)),
        ],
    };
    assert_eq!(
        validate_resolved_freeze_committed_binding(
            &journal,
            committed_reference.clone(),
            &wrong_receipt_type
        ),
        Err(ResolvedFreezeCommittedBindingError::ReceiptDecode)
    );

    let invalid_start = FixtureRecordResolver {
        records: vec![
            (receipt.record_id(), hex_bytes(RECEIPT_HEX)),
            (receipt.input().manifest_id, hex_bytes(MANIFEST_RECORD_HEX)),
            (start_record.record_id(), vec![0]),
        ],
    };
    assert_eq!(
        validate_resolved_freeze_committed_binding(
            &journal,
            committed_reference.clone(),
            &invalid_start
        ),
        Err(ResolvedFreezeCommittedBindingError::AttemptStartDecode)
    );

    let mut alternate_start_bytes = hex_bytes(START_RECORD_HEX);
    replace_unique(
        &mut alternate_start_bytes,
        &[0x12, 0x58, 0x20, 0xe0],
        3,
        0xe1,
    );
    let mismatched_start = FixtureRecordResolver {
        records: vec![
            (receipt.record_id(), hex_bytes(RECEIPT_HEX)),
            (receipt.input().manifest_id, hex_bytes(MANIFEST_RECORD_HEX)),
            (start_record.record_id(), alternate_start_bytes),
        ],
    };
    assert_eq!(
        validate_resolved_freeze_committed_binding(
            &journal,
            committed_reference,
            &mismatched_start
        ),
        Err(ResolvedFreezeCommittedBindingError::AttemptStartIdentityMismatch)
    );
}

#[test]
fn resolved_freeze_committed_binding_keeps_exact_structural_inputs_non_authoritative() {
    let (journal, committed_reference, start_record, receipt) = fixed_binding_fixture();
    let records = FixtureRecordResolver {
        records: vec![
            (start_record.record_id(), hex_bytes(START_RECORD_HEX)),
            (receipt.record_id(), hex_bytes(RECEIPT_HEX)),
            (receipt.input().manifest_id, hex_bytes(MANIFEST_RECORD_HEX)),
        ],
    };

    assert_eq!(
        validate_resolved_freeze_committed_binding(&journal, committed_reference, &records),
        Ok(ResolvedFreezeCommittedBindingOutcome::AuthorityEvidenceUnavailable)
    );
}

#[test]
fn freeze_committed_binding_rejects_start_record_root_that_disagrees_with_retained_start() {
    let registry_id = RegistryId::try_from(id(0x00).as_slice()).unwrap();
    let correct_start =
        FreezeAttemptStartRecord::decode_authoritative(&hex_bytes(START_RECORD_HEX)).unwrap();
    let wrong_start = FreezeAttemptStartRecord::new(FreezeAttemptStartRecordInput {
        freeze_attempt_id: correct_start.input().freeze_attempt_id,
        intended_root_id: IntendedRootId::try_from(id(0xb0).as_slice()).unwrap(),
        subject_id: correct_start.input().subject_id,
        policy_record_id: correct_start.input().policy_record_id,
    });
    let mut start_entry = hex_bytes(START_ENTRY_HEX);
    let old_event_record_id = hex_id(START_RECORD_ID_HEX);
    let event_record_offset = start_entry
        .windows(old_event_record_id.len())
        .position(|candidate| candidate == old_event_record_id)
        .expect(
            "JRN-FREEZE-ROOT-BINDING-001: fixture event Record ID must be present exactly once",
        );
    assert_eq!(
        start_entry
            .windows(old_event_record_id.len())
            .filter(|candidate| *candidate == old_event_record_id)
            .count(),
        1,
        "JRN-FREEZE-ROOT-BINDING-001"
    );
    start_entry[event_record_offset..event_record_offset + old_event_record_id.len()]
        .copy_from_slice(wrong_start.record_id().as_bytes());

    let genesis = evidence_registry::GenesisJournalEntry::new(
        registry_id,
        EventRecordId::try_from(id(0x20).as_slice()).unwrap(),
        RecordId::try_from(id(0x40).as_slice()).unwrap(),
        RecordId::try_from(id(0x60).as_slice()).unwrap(),
    );
    let mut journal = RetainedJournal::from_genesis(genesis).unwrap();
    journal.append_strict_entry(&start_entry).unwrap();
    let start_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(1_u64).unwrap(),
        JournalEntryHash::try_from(sha256(&start_entry).as_slice()).unwrap(),
        EventTypeId::try_from(100_u64).unwrap(),
        EventRecordId::try_from(wrong_start.record_id().as_bytes().as_slice()).unwrap(),
    );
    let receipt = FreezeReceiptRecord::decode_authoritative(&receipt_bytes(
        &start_reference,
        RecordId::try_from(id(0xc0).as_slice()).unwrap(),
    ))
    .unwrap();
    journal
        .append_strict_entry(&freeze_committed_bytes(
            JournalEntryHash::try_from(sha256(&start_entry).as_slice()).unwrap(),
            &start_reference,
            receipt.record_id(),
        ))
        .unwrap();
    let committed_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(2_u64).unwrap(),
        journal.reconstruct_state().unwrap().journal_head_hash(),
        EventTypeId::try_from(101_u64).unwrap(),
        EventRecordId::try_from(receipt.record_id().as_bytes().as_slice()).unwrap(),
    );

    assert_eq!(
        validate_freeze_committed_binding(FreezeCommittedBindingInput {
            retained_journal: &journal,
            committed_event_reference: committed_reference,
            freeze_attempt_start_record: &wrong_start,
            freeze_receipt_record: &receipt,
        }),
        Err(evidence_registry::FreezeCommittedBindingError::IntendedRootMismatch)
    );
}

#[test]
fn authoritative_store_derives_positive_freeze_authority_from_its_retained_namespaces() {
    let fixture = AuthoritativeStoreFixtureDir::new();
    let registry_id = RegistryId::try_from(id(0x00).as_slice()).unwrap();
    let storage_capability_class_id = RecordId::try_from(id(0x40).as_slice()).unwrap();
    let environment_observation_id = RecordId::try_from(id(0x60).as_slice()).unwrap();
    let genesis_record = GenesisRecord::new(GenesisRecordInput {
        registry_id,
        journal_format_version: 1,
        record_identity_profile_id: 1,
        storage_capability_class_id,
        environment_observation_id,
        created_by_tool_version: "authoritative-store-test".to_owned(),
    })
    .unwrap();
    let genesis_entry = GenesisJournalEntry::new(
        registry_id,
        EventRecordId::try_from(genesis_record.record_id().as_bytes().as_slice()).unwrap(),
        storage_capability_class_id,
        environment_observation_id,
    );
    let start_record = start_record(registry_id);
    let start_entry_bytes = freeze_start_bytes(genesis_entry.entry_hash(), &start_record);
    let mut structural_journal = RetainedJournal::from_genesis(genesis_entry.clone()).unwrap();
    structural_journal
        .append_strict_entry(&start_entry_bytes)
        .unwrap();
    let start_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(1_u64).unwrap(),
        structural_journal
            .reconstruct_state()
            .unwrap()
            .journal_head_hash(),
        EventTypeId::try_from(100_u64).unwrap(),
        EventRecordId::try_from(start_record.record_id().as_bytes().as_slice()).unwrap(),
    );
    let manifest_bytes = hex_bytes(MANIFEST_RECORD_HEX);
    let manifest_id = RecordId::try_from(sha256(&manifest_bytes).as_slice()).unwrap();
    let receipt_bytes = receipt_bytes(&start_reference, manifest_id);
    let receipt = FreezeReceiptRecord::decode_authoritative(&receipt_bytes).unwrap();
    let committed_entry_bytes = freeze_committed_bytes(
        start_reference.entry_hash(),
        &start_reference,
        receipt.record_id(),
    );
    structural_journal
        .append_strict_entry(&committed_entry_bytes)
        .unwrap();
    let committed_reference = JournalReference::new(
        registry_id,
        JournalEntryIndex::try_from(2_u64).unwrap(),
        structural_journal
            .reconstruct_state()
            .unwrap()
            .journal_head_hash(),
        EventTypeId::try_from(101_u64).unwrap(),
        EventRecordId::try_from(receipt.record_id().as_bytes().as_slice()).unwrap(),
    );

    fs::write(
        fixture.path.join("registry/genesis.cbor"),
        genesis_record.authoritative_cbor(),
    )
    .unwrap();
    fs::write(
        fixture.path.join("journal/00000000000000000000.cbor"),
        genesis_entry.authoritative_cbor(),
    )
    .unwrap();
    fs::write(
        fixture.path.join("journal/00000000000000000001.cbor"),
        &start_entry_bytes,
    )
    .unwrap();
    fs::write(
        fixture.path.join("journal/00000000000000000002.cbor"),
        &committed_entry_bytes,
    )
    .unwrap();
    write_record(
        &fixture.path,
        genesis_record.record_id(),
        &genesis_record.authoritative_cbor(),
    );
    write_record(
        &fixture.path,
        start_record.record_id(),
        &start_record.authoritative_cbor(),
    );
    write_record(&fixture.path, manifest_id, &manifest_bytes);
    write_record(&fixture.path, receipt.record_id(), &receipt_bytes);

    let store = AuthoritativeRegistryStore::open(&fixture.path).unwrap();
    let witness = store
        .validate_freeze_committed_authority(committed_reference.clone())
        .unwrap();

    assert_eq!(witness.registry_id(), registry_id);
    assert_eq!(witness.committed_event_reference(), &committed_reference);
    assert_eq!(witness.start_event_reference(), &start_reference);
    assert_eq!(witness.receipt_record_id(), receipt.record_id());
    assert_eq!(witness.manifest_record_id(), manifest_id);
    assert_eq!(witness.start_record_id(), start_record.record_id());
    assert_eq!(
        validate_resolved_freeze_committed_binding(
            store.retained_journal(),
            committed_reference.clone(),
            &store,
        ),
        Ok(ResolvedFreezeCommittedBindingOutcome::AuthorityEvidenceUnavailable)
    );

    drop(store);
    fs::remove_file(
        fixture
            .path
            .join("records")
            .join(record_filename(receipt.record_id())),
    )
    .unwrap();
    let missing_receipt_store = AuthoritativeRegistryStore::open(&fixture.path).unwrap();
    assert_eq!(
        missing_receipt_store.validate_freeze_committed_authority(committed_reference),
        Err(
            evidence_registry::AuthoritativeFreezeCommittedBindingError::Structural(
                ResolvedFreezeCommittedBindingError::ReceiptPayloadUnavailable,
            ),
        )
    );
}
