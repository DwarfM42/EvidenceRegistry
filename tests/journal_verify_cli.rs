use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const GENESIS_JOURNAL_ENTRY_HEX: &str = "82782045766964656e636552656769737472792e4a6f75726e616c456e7472792e7631ac0001015820000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f020003f60401055820202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f068007800801095820000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f0a5820404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f0b5820606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f";
const GENESIS_ENTRY_HASH_HEX: &str =
    "ae1919549c80a0c3708cce0485af881a5223fbcf80c6b27a2bbe602418dabd51";
static TEMP_INPUT_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

fn hex_bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&hex[offset..offset + 2], 16).unwrap())
        .collect()
}

fn cli_path() -> String {
    std::env::var("CARGO_BIN_EXE_evidence-registry")
        .expect("JRN-CLI-TEST-SETUP-001: Cargo must provide the EvidenceRegistry binary")
}

struct TempInputDir {
    path: PathBuf,
}

impl TempInputDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "evidence-registry-journal-verify-cli-{}-{}",
            std::process::id(),
            TEMP_INPUT_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn write(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.path.join(name);
        fs::write(&path, bytes).unwrap();
        path
    }
}

impl Drop for TempInputDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn journal_verify_reports_only_supported_state_only_replay_facts() {
    let inputs = TempInputDir::new();
    let genesis = inputs.write("genesis.cbor", &hex_bytes(GENESIS_JOURNAL_ENTRY_HEX));

    let output = Command::new(cli_path())
        .args(["journal", "verify", "--genesis"])
        .arg(genesis)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "JRN-CLI-STATE-ONLY-001 status={:?} stdout={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "",
        "JRN-CLI-STATE-ONLY-001"
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            concat!(
                "{{\"output_schema_version\":1,",
                "\"operation\":\"journal verify\",",
                "\"outcome\":\"JOURNAL_ONLY_REPLAY\",",
                "\"structural_status\":\"VALID\",",
                "\"authority_status\":\"UNAVAILABLE\",",
                "\"admission_status\":\"UNAVAILABLE\",",
                "\"entry_count\":1,",
                "\"journal_head_index\":0,",
                "\"journal_head_hash\":\"{}\"}}\n"
            ),
            GENESIS_ENTRY_HASH_HEX
        ),
        "JRN-CLI-STATE-ONLY-001"
    );
}

#[test]
fn journal_verify_keeps_invalid_exact_bytes_distinct_from_unavailable_authority() {
    let inputs = TempInputDir::new();
    let genesis = inputs.write("malformed-genesis.cbor", &[0x80]);

    let output = Command::new(cli_path())
        .args(["journal", "verify", "--genesis"])
        .arg(genesis)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        concat!(
            "{\"output_schema_version\":1,",
            "\"operation\":\"journal verify\",",
            "\"outcome\":\"STRUCTURAL_REPLAY_REJECTED\",",
            "\"structural_status\":\"INVALID\",",
            "\"authority_status\":\"UNAVAILABLE\",",
            "\"admission_status\":\"UNAVAILABLE\",",
            "\"error_class\":\"DECODE_ERROR\"}\n"
        ),
        "JRN-CLI-INVALID-001"
    );
}

#[test]
fn journal_verify_reports_unavailable_caller_selected_input_as_indeterminate_evidence() {
    let inputs = TempInputDir::new();
    let missing_genesis = inputs.path.join("missing-genesis.cbor");

    let output = Command::new(cli_path())
        .args(["journal", "verify", "--genesis"])
        .arg(missing_genesis)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(6));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        concat!(
            "{\"output_schema_version\":1,",
            "\"operation\":\"journal verify\",",
            "\"outcome\":\"INPUT_UNAVAILABLE\",",
            "\"structural_status\":\"UNAVAILABLE\",",
            "\"authority_status\":\"UNAVAILABLE\",",
            "\"admission_status\":\"UNAVAILABLE\",",
            "\"error_class\":\"GENESIS_INPUT_UNAVAILABLE\"}\n"
        ),
        "JRN-CLI-MISSING-INPUT-001"
    );
}
