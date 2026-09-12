use serde::{Deserialize, Serialize};

/// Untrusted redundant Request identity, never an execution or ingestion capability.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RequestBinding {
    pub registry_id: String,
    pub entry_index: u64,
    pub entry_hash: String,
    pub event_type_id: u64,
    pub event_record_id: String,
}

/// Parsed Agent claims. Successful parsing establishes no Store authority.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Submission {
    pub schema: u32,
    pub request: RequestBinding,
    pub method_status: u64,
    pub finding_state: u64,
    pub reason_codes: Vec<String>,
    pub findings: Vec<String>,
    pub reviewer_metadata: Option<String>,
    pub artifacts: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubmissionError {
    Limit,
    Grammar,
    RequestMismatch,
    ArtifactName,
}

pub const MAX_SUBMISSION_BYTES: usize = 262_144;

pub(crate) fn exact_hex_id(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

impl RequestBinding {
    pub(crate) fn valid(&self) -> bool {
        self.event_type_id == 300
            && self.entry_index <= evidence_registry::ER_UINT_MAX
            && exact_hex_id(&self.registry_id)
            && exact_hex_id(&self.entry_hash)
            && exact_hex_id(&self.event_record_id)
    }
}

/// v0 accepts a deliberately narrow portable ASCII relative artifact spelling.
/// This is permission syntax only: Store intake performs every filesystem check.
pub fn approved_artifact_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 1024
        && value.split('/').all(|component| {
            if component.is_empty()
                || component == "."
                || component == ".."
                || component.ends_with('.')
                || !component
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
            {
                return false;
            }
            let stem = component
                .split('.')
                .next()
                .unwrap_or("")
                .to_ascii_uppercase();
            !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                && !(stem.len() == 4
                    && (stem.starts_with("COM") || stem.starts_with("LPT"))
                    && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        })
}

/// Parses already captured, bounded bytes; does not read paths, translate prose,
/// resolve findings, dispatch, or publish. `expected` must come from the managed
/// caller's retained Request, not from an Agent-supplied token.
pub fn parse_submission(
    bytes: &[u8],
    expected: &RequestBinding,
) -> Result<Submission, SubmissionError> {
    if bytes.len() > MAX_SUBMISSION_BYTES {
        return Err(SubmissionError::Limit);
    }
    let parsed: Submission = serde_json::from_slice(bytes).map_err(|_| SubmissionError::Grammar)?;
    if parsed.schema != 1
        || !expected.valid()
        || !parsed.request.valid()
        || !matches!(parsed.method_status, 1..=3)
        || !matches!(parsed.finding_state, 1..=3)
        || parsed
            .reason_codes
            .windows(2)
            .any(|p| p[0].as_bytes() >= p[1].as_bytes())
        || parsed.findings.iter().any(|id| !exact_hex_id(id))
    {
        return Err(SubmissionError::Grammar);
    }
    if &parsed.request != expected {
        return Err(SubmissionError::RequestMismatch);
    }
    if parsed.artifacts.len() > 1024
        || parsed.findings.len() > 1024
        || parsed.reason_codes.len() > 1024
    {
        return Err(SubmissionError::Limit);
    }
    let mut names = std::collections::BTreeSet::new();
    if parsed
        .artifacts
        .iter()
        .any(|name| !approved_artifact_name(name) || !names.insert(name))
    {
        return Err(SubmissionError::ArtifactName);
    }
    Ok(parsed)
}
