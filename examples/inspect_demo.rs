//! Disposable, non-authoritative onboarding example; never opens a Registry store.
use evidence_registry::{
    EventRecordId, GenesisJournalEntry, GenesisRecord, GenesisRecordInput,
    OrdinaryVerificationJournalEntry, RecordId, RegistryId, RetainedJournal, StrictRecordFrame,
};
use sha2::{Digest, Sha256};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const ORDINARY_VECTOR: &str = include_str!("../vectors/journal-ordinary-verification-v1.txt");

fn create_demo_directory(target: &Path, name: &str) -> io::Result<PathBuf> {
    if name.is_empty()
        || name.len() > 80
        || !name.as_bytes()[0].is_ascii_alphanumeric()
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "use one name: 1-80 ASCII letters/digits, hyphens, underscores; start with a letter/digit"));
    }
    let metadata = std::fs::symlink_metadata(target)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "target must be a real directory",
        ));
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "target must not be a reparse point",
            ));
        }
    }
    let output = target.join(name);
    std::fs::create_dir(&output)?;
    Ok(output)
}

fn main() -> io::Result<()> {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    let name = match arguments.as_slice() {
        [] => "readme-demo",
        [name] => name.to_str().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "output name must be ASCII")
        })?,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "usage: cargo run --example inspect_demo --locked -- [new-directory-name]",
            ))
        }
    };
    // Always use this checkout's target, not a caller-supplied Registry root.
    // Trusted local workspace only: this is not a hostile-writer-safe publication protocol.
    let output = generate_demo(&Path::new(env!("CARGO_MANIFEST_DIR")).join("target"), name)?;
    println!("Created synthetic demo: {}", output.display());
    print!(
        "{}",
        std::fs::read_to_string(output.join("SHA256SUMS.txt"))?
    );
    println!("Genesis: Record/Journal decoded; Journal-only replay completed.");
    println!("Ordinary vector: independently hashed/decoded; standalone index 24, NOT a Genesis successor.");
    println!("Capability/environment references UNRESOLVED; authority/admission UNAVAILABLE.");
    Ok(())
}

fn invalid(error: impl std::fmt::Debug) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, format!("{error:?}"))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

// Expected bytes/digest come from the tracked independent vector, not our encoder.
fn decode_ordinary_vector(vector: &str) -> io::Result<Vec<u8>> {
    let field = |key: &str| {
        vector
            .lines()
            .find_map(|line| line.strip_prefix(key))
            .ok_or_else(|| invalid(format!("missing vector field {key}")))
    };
    let encoded = field("canonical_cbor_hex=")?;
    if encoded.len() % 2 != 0 || !encoded.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(invalid("invalid hex"));
    }
    let bytes = (0..encoded.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&encoded[i..i + 2], 16).map_err(invalid))
        .collect::<io::Result<Vec<_>>>()?;
    let digest = Sha256::digest(&bytes);
    if hex(&digest) != field("entry_hash_sha256=")? {
        return Err(invalid("independent vector digest mismatch"));
    }
    let decoded =
        OrdinaryVerificationJournalEntry::decode_authoritative(&bytes).map_err(invalid)?;
    if decoded.authoritative_cbor() != bytes
        || decoded.entry_hash().as_bytes().as_slice() != digest.as_slice()
    {
        return Err(invalid("ordinary decoder/hash mismatch"));
    }
    Ok(bytes)
}

fn write_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(bytes)
}

fn generate_demo(target: &Path, name: &str) -> io::Result<PathBuf> {
    // Synthetic IDs: NO capability probe, environment observation, or referenced Records.
    let registry_id = RegistryId::try_from([0x11; 32].as_slice()).map_err(invalid)?;
    let storage = RecordId::try_from([0x22; 32].as_slice()).map_err(invalid)?;
    let environment = RecordId::try_from([0x33; 32].as_slice()).map_err(invalid)?;
    let record = GenesisRecord::new(GenesisRecordInput {
        registry_id,
        journal_format_version: 1,
        record_identity_profile_id: 1,
        storage_capability_class_id: storage,
        environment_observation_id: environment,
        created_by_tool_version: "inspect_demo/0.1.0".to_owned(),
    })
    .map_err(invalid)?;
    let record_bytes = record.authoritative_cbor();
    GenesisRecord::decode_authoritative(&record_bytes).map_err(invalid)?;
    let frame = StrictRecordFrame::decode_authoritative(&record_bytes).map_err(invalid)?;
    if frame.record_id() != record.record_id() {
        return Err(invalid("GENESIS Record identity mismatch"));
    }
    let entry = GenesisJournalEntry::new(
        registry_id,
        EventRecordId::try_from(record.record_id().as_bytes().as_slice()).map_err(invalid)?,
        storage,
        environment,
    );
    let genesis_bytes = entry.authoritative_cbor();
    GenesisJournalEntry::decode_authoritative(&genesis_bytes).map_err(invalid)?;
    RetainedJournal::from_authoritative_genesis(&genesis_bytes)
        .map_err(invalid)?
        .reconstruct_state()
        .map_err(invalid)?;
    let ordinary_bytes = decode_ordinary_vector(ORDINARY_VECTOR)?;

    // These are loose sample files, deliberately NOT a Registry-store namespace.
    let output = create_demo_directory(target, name)?;
    let mut sums = String::new();
    for (filename, bytes) in [
        ("genesis-record.cbor", record_bytes),
        ("genesis-entry.cbor", genesis_bytes),
        ("ordinary-verification-standalone.cbor", ordinary_bytes),
    ] {
        write_new(&output.join(filename), &bytes)?;
        sums.push_str(&format!("{}  {filename}\n", hex(&Sha256::digest(&bytes))));
    }
    write_new(&output.join("SHA256SUMS.txt"), sums.as_bytes())?;
    write_new(
        &output.join("DEMO.txt"),
        concat!(
        "NON-AUTHORITATIVE SYNTHETIC DEMO; not a Registry-store namespace.\n",
        "Genesis registry ID = 0x11 repeated 32 times.\n",
        "Storage capability ID = 0x22 repeated 32 times: UNRESOLVED synthetic reference.\n",
        "Environment observation ID = 0x33 repeated 32 times: UNRESOLVED synthetic reference.\n",
        "Only genesis-entry.cbor is a replay input; it references genesis-record.cbor.\n",
        "ordinary-verification-standalone.cbor is the independent index-24 vector.\n",
        "Its predecessor, dependencies, and Record payload are unresolved.\n",
        "Do NOT append it after this Genesis; it is not a successor.\n",
        "Hashes/decodes do not establish authority, Policy satisfaction, or admission.\n",
        "No live Registry was opened or mutated. No durability receipt is produced.\n",
    )
        .as_bytes(),
    )?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use evidence_registry::{
        GenesisJournalEntry, GenesisRecord, OrdinaryVerificationJournalEntry, StrictRecordFrame,
    };
    use sha2::{Digest, Sha256};
    use std::sync::atomic::{AtomicU64, Ordering};

    fn scratch() -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/tmp");
        std::fs::create_dir_all(&base).unwrap();
        loop {
            let path = base.join(format!(
                "readme-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match std::fs::create_dir(&path) {
                Ok(()) => return path,
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => panic!("{e}"),
            }
        }
    }

    #[test]
    fn existing_directory_is_refused_without_changing_its_bytes() {
        let target = scratch();
        let existing = target.join("demo");
        std::fs::create_dir(&existing).unwrap();
        std::fs::write(existing.join("sentinel"), b"keep exact bytes").unwrap();
        assert_eq!(
            create_demo_directory(&target, "demo").unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(
            std::fs::read(existing.join("sentinel")).unwrap(),
            b"keep exact bytes"
        );
        std::fs::remove_dir_all(target).unwrap();
    }

    #[test]
    fn output_name_cannot_escape_target() {
        let target = scratch();
        for name in [
            "",
            ".",
            "..",
            "../escape",
            "nested/path",
            "nested\\path",
            "/absolute",
            "C:\\absolute",
            "--help",
        ] {
            assert_eq!(
                create_demo_directory(&target, name).unwrap_err().kind(),
                io::ErrorKind::InvalidInput,
                "{name}"
            );
        }
        std::fs::remove_dir_all(target).unwrap();
    }

    #[test]
    fn individual_file_overwrite_is_refused() {
        let target = scratch();
        let file = target.join("sample");
        write_new(&file, b"original").unwrap();
        assert_eq!(
            write_new(&file, b"replacement").unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(std::fs::read(file).unwrap(), b"original");
        std::fs::remove_dir_all(target).unwrap();
    }

    #[test]
    fn independent_vector_rejects_hash_mismatch_and_malformed_cbor() {
        let bytes = decode_ordinary_vector(ORDINARY_VECTOR).unwrap();
        let digest = hex(&Sha256::digest(&bytes));
        assert!(
            decode_ordinary_vector(&ORDINARY_VECTOR.replace(&digest, &"0".repeat(64))).is_err()
        );
        // Matching digest is insufficient: strict decoding must reject truncation.
        let truncated = &bytes[..bytes.len() - 1];
        let malformed = format!(
            "canonical_cbor_hex={}\nentry_hash_sha256={}\n",
            hex(truncated),
            hex(&Sha256::digest(truncated))
        );
        assert!(decode_ordinary_vector(&malformed).is_err());
        assert!(decode_ordinary_vector("canonical_cbor_hex=0\nentry_hash_sha256=00").is_err());
        assert!(decode_ordinary_vector("canonical_cbor_hex=GG\nentry_hash_sha256=00").is_err());
        assert!(decode_ordinary_vector("canonical_cbor_hex=é\nentry_hash_sha256=00").is_err());
        assert!(decode_ordinary_vector("").is_err());
    }

    #[test]
    fn generates_typed_genesis_and_separate_independent_vector() {
        let target = scratch();
        let output = generate_demo(&target, "demo").unwrap();
        let record = std::fs::read(output.join("genesis-record.cbor")).unwrap();
        let decoded = GenesisRecord::decode_authoritative(&record).unwrap();
        let frame = StrictRecordFrame::decode_authoritative(&record).unwrap();
        assert_eq!(decoded.record_id(), frame.record_id());
        assert_eq!(
            decoded.record_id().as_bytes().as_slice(),
            Sha256::digest(&record).as_slice()
        );
        let genesis = std::fs::read(output.join("genesis-entry.cbor")).unwrap();
        let entry = GenesisJournalEntry::decode_authoritative(&genesis).unwrap();
        assert_eq!(
            entry.entry_hash().as_bytes().as_slice(),
            Sha256::digest(&genesis).as_slice()
        );
        let ordinary = std::fs::read(output.join("ordinary-verification-standalone.cbor")).unwrap();
        let decoded = OrdinaryVerificationJournalEntry::decode_authoritative(&ordinary).unwrap();
        assert_eq!(decoded.authoritative_cbor(), ordinary);
        assert_eq!(
            decoded.entry_hash().as_bytes().as_slice(),
            Sha256::digest(&ordinary).as_slice()
        );
        let mut journal =
            evidence_registry::RetainedJournal::from_authoritative_genesis(&genesis).unwrap();
        assert_eq!(journal.reconstruct_state().unwrap().entry_count(), 1);
        assert!(
            journal.append_strict_entry(&ordinary).is_err(),
            "index 24 is NOT this Genesis successor"
        );
        assert!(generate_demo(&target, "demo").is_err());
        assert_eq!(
            std::fs::read(output.join("genesis-record.cbor")).unwrap(),
            record
        );
        std::fs::remove_dir_all(target).unwrap();
    }
}
