//! Owner-facing, read-only Journal verification surface.
//!
//! This binary accepts caller-selected exact Journal Entry byte files. It does not
//! define a Registry storage layout, Record resolver, authority, admission, Policy,
//! durability, custody, or lifecycle-truth contract. A successful command reports
//! only the currently supported structural/state-only replay facts.

use evidence_registry::{RetainedJournal, RetainedJournalError};
use std::path::PathBuf;
use std::process::ExitCode;

const OUTPUT_SCHEMA_VERSION: u8 = 1;
const OPERATION: &str = "journal verify";

fn main() -> ExitCode {
    match parse_arguments(std::env::args().skip(1)) {
        Ok(input) => verify_journal(input),
        Err(()) => {
            print_unavailable("USAGE_ERROR", "INPUT_UNAVAILABLE");
            ExitCode::from(2)
        }
    }
}

struct JournalVerifyInput {
    genesis_path: PathBuf,
    entry_paths: Vec<PathBuf>,
}

fn parse_arguments(arguments: impl Iterator<Item = String>) -> Result<JournalVerifyInput, ()> {
    let mut arguments = arguments;
    if arguments.next().as_deref() != Some("journal")
        || arguments.next().as_deref() != Some("verify")
    {
        return Err(());
    }

    let mut genesis_path = None;
    let mut entry_paths = Vec::new();
    while let Some(option) = arguments.next() {
        match option.as_str() {
            "--genesis" => {
                if genesis_path.is_some() {
                    return Err(());
                }
                genesis_path = arguments.next().map(PathBuf::from);
                if genesis_path.is_none() {
                    return Err(());
                }
            }
            "--entry" => {
                let path = arguments.next().map(PathBuf::from).ok_or(())?;
                entry_paths.push(path);
            }
            _ => return Err(()),
        }
    }

    Ok(JournalVerifyInput {
        genesis_path: genesis_path.ok_or(())?,
        entry_paths,
    })
}

fn verify_journal(input: JournalVerifyInput) -> ExitCode {
    let genesis_bytes = match std::fs::read(input.genesis_path) {
        Ok(bytes) => bytes,
        Err(_) => {
            print_unavailable("GENESIS_INPUT_UNAVAILABLE", "INPUT_UNAVAILABLE");
            return ExitCode::from(2);
        }
    };
    let mut journal = match RetainedJournal::from_authoritative_genesis(&genesis_bytes) {
        Ok(journal) => journal,
        Err(error) => return print_retained_error(error),
    };

    for entry_path in input.entry_paths {
        let entry_bytes = match std::fs::read(entry_path) {
            Ok(bytes) => bytes,
            Err(_) => {
                print_unavailable("ENTRY_INPUT_UNAVAILABLE", "INPUT_UNAVAILABLE");
                return ExitCode::from(2);
            }
        };
        if let Err(error) = journal.append_strict_entry(&entry_bytes) {
            return print_retained_error(error);
        }
    }

    let replay = match journal.reconstruct_state() {
        Ok(replay) => replay,
        Err(error) => return print_retained_error(error),
    };
    println!(
        concat!(
            "{{\"output_schema_version\":{},",
            "\"operation\":\"{}\",",
            "\"outcome\":\"JOURNAL_ONLY_REPLAY\",",
            "\"structural_status\":\"VALID\",",
            "\"authority_status\":\"UNAVAILABLE\",",
            "\"admission_status\":\"UNAVAILABLE\",",
            "\"entry_count\":{},",
            "\"journal_head_index\":{},",
            "\"journal_head_hash\":\"{}\"}}"
        ),
        OUTPUT_SCHEMA_VERSION,
        OPERATION,
        replay.entry_count(),
        replay.journal_head_index().value(),
        lowercase_hex(replay.journal_head_hash().as_bytes())
    );
    ExitCode::SUCCESS
}

fn print_retained_error(error: RetainedJournalError) -> ExitCode {
    match error {
        RetainedJournalError::UnsupportedEntry => {
            print_unavailable("UNSUPPORTED_ENTRY", "SUPPORTED_SUBSET_UNAVAILABLE");
            ExitCode::from(3)
        }
        RetainedJournalError::RegistryMismatch => print_rejected("REGISTRY_MISMATCH"),
        RetainedJournalError::UnexpectedEntryIndex => print_rejected("UNEXPECTED_ENTRY_INDEX"),
        RetainedJournalError::PreviousHashMismatch => print_rejected("PREVIOUS_HASH_MISMATCH"),
        RetainedJournalError::MissingReference => print_rejected("MISSING_REFERENCE"),
        RetainedJournalError::ReferenceMismatch => print_rejected("REFERENCE_MISMATCH"),
        RetainedJournalError::LifecycleTransition => print_rejected("LIFECYCLE_TRANSITION"),
        RetainedJournalError::DecodeError => print_rejected("DECODE_ERROR"),
    }
}

fn print_rejected(error_class: &str) -> ExitCode {
    println!(
        concat!(
            "{{\"output_schema_version\":{},",
            "\"operation\":\"{}\",",
            "\"outcome\":\"STRUCTURAL_REPLAY_REJECTED\",",
            "\"structural_status\":\"INVALID\",",
            "\"authority_status\":\"UNAVAILABLE\",",
            "\"admission_status\":\"UNAVAILABLE\",",
            "\"error_class\":\"{}\"}}"
        ),
        OUTPUT_SCHEMA_VERSION, OPERATION, error_class
    );
    ExitCode::from(1)
}

fn print_unavailable(error_class: &str, outcome: &str) {
    println!(
        concat!(
            "{{\"output_schema_version\":{},",
            "\"operation\":\"{}\",",
            "\"outcome\":\"{}\",",
            "\"structural_status\":\"UNAVAILABLE\",",
            "\"authority_status\":\"UNAVAILABLE\",",
            "\"admission_status\":\"UNAVAILABLE\",",
            "\"error_class\":\"{}\"}}"
        ),
        OUTPUT_SCHEMA_VERSION, OPERATION, outcome, error_class
    );
}

fn lowercase_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
