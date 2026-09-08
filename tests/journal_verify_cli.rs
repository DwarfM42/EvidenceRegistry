use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(unix)]
use std::{ffi::OsString, os::unix::ffi::OsStringExt};

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

    #[cfg(target_os = "linux")]
    fn write_native(&self, name: OsString, bytes: &[u8]) -> PathBuf {
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

#[cfg(target_os = "linux")]
#[test]
fn journal_verify_accepts_non_utf8_genesis_path_without_changing_structural_output() {
    let inputs = TempInputDir::new();
    let genesis = inputs.write_native(
        OsString::from_vec(b"native-\xff-genesis.cbor".to_vec()),
        &hex_bytes(GENESIS_JOURNAL_ENTRY_HEX),
    );
    let output = Command::new(cli_path())
        .args(["journal", "verify", "--genesis"])
        .arg(genesis)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            concat!(
                "{{\"output_schema_version\":1,\"operation\":\"journal verify\",",
                "\"outcome\":\"JOURNAL_ONLY_REPLAY\",\"structural_status\":\"VALID\",",
                "\"authority_status\":\"UNAVAILABLE\",\"admission_status\":\"UNAVAILABLE\",",
                "\"entry_count\":1,\"journal_head_index\":0,\"journal_head_hash\":\"{}\"}}\n"
            ),
            GENESIS_ENTRY_HASH_HEX
        )
    );
}

#[cfg(unix)]
#[test]
fn journal_verify_reports_native_byte_missing_inputs_without_argument_panics() {
    let inputs = TempInputDir::new();
    let missing_genesis = inputs
        .path
        .join(OsString::from_vec(b"missing-\xff-genesis.cbor".to_vec()));
    let genesis = inputs.write("genesis.cbor", &hex_bytes(GENESIS_JOURNAL_ENTRY_HEX));
    let missing_entry = inputs
        .path
        .join(OsString::from_vec(b"missing-\xff-entry.cbor".to_vec()));
    let genesis_output = Command::new(cli_path())
        .args(["journal", "verify", "--genesis"])
        .arg(missing_genesis)
        .output()
        .unwrap();
    assert_eq!(genesis_output.status.code(), Some(6));
    assert!(String::from_utf8(genesis_output.stdout)
        .unwrap()
        .contains("GENESIS_INPUT_UNAVAILABLE"));
    let entry_output = Command::new(cli_path())
        .args(["journal", "verify", "--genesis"])
        .arg(genesis)
        .arg("--entry")
        .arg(missing_entry)
        .output()
        .unwrap();
    assert_eq!(entry_output.status.code(), Some(6));
    assert!(String::from_utf8(entry_output.stdout)
        .unwrap()
        .contains("ENTRY_INPUT_UNAVAILABLE"));
}

#[cfg(unix)]
#[test]
fn journal_verify_native_byte_command_and_option_match_ascii_usage_output() {
    let invalid_command = Command::new(cli_path())
        .arg(OsString::from_vec(b"journal-\xff".to_vec()))
        .output()
        .unwrap();
    let ascii_command = Command::new(cli_path()).arg("invalid").output().unwrap();
    assert_eq!(invalid_command.status.code(), Some(2));
    assert_eq!(invalid_command.stdout, ascii_command.stdout);
    let invalid_option = Command::new(cli_path())
        .args(["journal", "verify"])
        .arg(OsString::from_vec(b"--bad-\xff".to_vec()))
        .output()
        .unwrap();
    let ascii_option = Command::new(cli_path())
        .args(["journal", "verify", "--bad"])
        .output()
        .unwrap();
    assert_eq!(invalid_option.status.code(), Some(2));
    assert_eq!(invalid_option.stdout, ascii_option.stdout);
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

#[cfg(target_os = "macos")]
#[test]
fn journal_verify_handles_non_utf8_native_argv_without_panicking() {
    let inputs = TempInputDir::new();
    let mut missing = inputs.path.as_os_str().as_encoded_bytes().to_vec();
    missing.extend_from_slice(b"/missing-");
    missing.push(0xff);
    let output = Command::new(cli_path())
        .args(["journal", "verify", "--genesis"])
        .arg(OsString::from_vec(missing))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(6));
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("GENESIS_INPUT_UNAVAILABLE"));
    let output = Command::new(cli_path())
        .args(["journal", "verify"])
        .arg(OsString::from_vec(vec![b'-', b'-', 0xff]))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("\"error_class\":\"USAGE_ERROR\""));
}

#[cfg(target_os = "macos")]
#[test]
fn journal_verify_probes_existing_non_utf8_filename_support() {
    let inputs = TempInputDir::new();
    let mut bytes = inputs.path.as_os_str().as_encoded_bytes().to_vec();
    bytes.extend_from_slice(b"/genesis-");
    bytes.push(0xff);
    let path = PathBuf::from(OsString::from_vec(bytes));
    if let Err(error) = fs::write(&path, hex_bytes(GENESIS_JOURNAL_ENTRY_HEX)) {
        if error.raw_os_error() == Some(92) {
            eprintln!("non_utf8_filename_probe=unsupported errno=92");
            return;
        }
        panic!("non-UTF-8 filename probe failed unexpectedly: {error}");
    }
    let output = Command::new(cli_path())
        .args(["journal", "verify", "--genesis"])
        .arg(path)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("\"structural_status\":\"VALID\""));
}

#[cfg(windows)]
#[test]
fn journal_verify_reports_unpaired_utf16_missing_path_without_argument_panic() {
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};

    let inputs = TempInputDir::new();
    let mut missing: Vec<u16> = inputs.path.as_os_str().encode_wide().collect();
    missing.extend("\\missing-".encode_utf16());
    missing.push(0xd800);
    let output = Command::new(cli_path())
        .args(["journal", "verify", "--genesis"])
        .arg(OsString::from_wide(&missing))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(6));
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        concat!(
            "{\"output_schema_version\":1,\"operation\":\"journal verify\",",
            "\"outcome\":\"INPUT_UNAVAILABLE\",\"structural_status\":\"UNAVAILABLE\",",
            "\"authority_status\":\"UNAVAILABLE\",\"admission_status\":\"UNAVAILABLE\",",
            "\"error_class\":\"GENESIS_INPUT_UNAVAILABLE\"}\n"
        )
    );
}
