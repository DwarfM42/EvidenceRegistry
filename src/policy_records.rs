//! Structural Policy construction only; event400 authority remains Store-owned.

use super::{
    authoritative_store, encode_array_length, encode_bstr_32, encode_journal_reference,
    encode_map_length, encode_uint, JournalReference, MinimalPolicyRecord, RecordDecodeError,
    RecordId, ReviewAdmissionPolicyRecord, ReviewAdmissionReviewRequirement, ER_UINT_MAX,
    RECORD_DOMAIN,
};

/// Exact local fields for a Policy with no optional requirements.
///
/// The start reference is for structural local construction, not a Binder-selected
/// authority head. A Store registration producer must supply its own captured start.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MinimalPolicyRecordInput {
    pub gate_scope_ref: RecordId,
    pub supported_context_ids: Vec<u64>,
    pub operation_start_journal_ref: JournalReference,
}

impl MinimalPolicyRecord {
    /// Constructs strict local Policy bytes without resolving Scope or Journal authority.
    /// Sets must already be canonical: this does not sort, deduplicate, or infer contexts.
    pub fn new(input: MinimalPolicyRecordInput) -> Result<Self, RecordDecodeError> {
        if input
            .supported_context_ids
            .iter()
            .any(|id| *id > ER_UINT_MAX)
        {
            return Err(RecordDecodeError);
        }
        let mut bytes = policy_prefix(5, input.gate_scope_ref);
        policy_tail(
            &mut bytes,
            &input.operation_start_journal_ref,
            &input.supported_context_ids,
        );
        authoritative_store::validate_policy_record_schema(&bytes)?;
        Self::decode_authoritative(&bytes)
    }

    /// Emits the exact canonical bytes of this minimal Policy, without publication.
    pub fn authoritative_cbor(&self) -> Vec<u8> {
        let mut bytes = policy_prefix(5, self.gate_scope_ref);
        policy_tail(
            &mut bytes,
            &self.operation_start_journal_ref,
            &self.supported_context_ids,
        );
        bytes
    }
}

/// Exact local Review Policy fields; optional sets distinguish absence from invalid emptiness.
///
/// Gate and selector identities are deliberately independent and remain unresolved.
/// The Store, not a Binder caller, owns the registration operation's start reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewAdmissionPolicyRecordInput {
    pub gate_scope_ref: RecordId,
    pub review_requirements: Vec<ReviewAdmissionReviewRequirement>,
    pub required_method_statuses: Option<Vec<u64>>,
    pub allowed_finding_states: Option<Vec<u64>>,
    pub acceptable_anchor_relation_ids: Option<Vec<u64>>,
    pub supported_context_ids: Vec<u64>,
    pub operation_start_journal_ref: JournalReference,
}

impl ReviewAdmissionPolicyRecord {
    /// Constructs the locally supported Review Policy field subset using the existing
    /// schema validator. No context inference, evaluation, registration, or resolution occurs.
    pub fn new(input: ReviewAdmissionPolicyRecordInput) -> Result<Self, RecordDecodeError> {
        if input
            .supported_context_ids
            .iter()
            .chain(input.required_method_statuses.iter().flatten())
            .chain(input.allowed_finding_states.iter().flatten())
            .chain(input.acceptable_anchor_relation_ids.iter().flatten())
            .any(|value| *value > ER_UINT_MAX)
        {
            return Err(RecordDecodeError);
        }
        Self::decode_authoritative(&input.encode_local_fields())
    }

    /// Returns exact decoded bytes, including schema-valid fields outside this
    /// constructor's subset. This preserves identity without extending the evaluator.
    pub fn authoritative_cbor(&self) -> Vec<u8> {
        self.authoritative_cbor.clone()
    }
}

impl ReviewAdmissionPolicyRecordInput {
    fn encode_local_fields(&self) -> Vec<u8> {
        let field_count = 6
            + usize::from(self.required_method_statuses.is_some())
            + usize::from(self.allowed_finding_states.is_some())
            + usize::from(self.acceptable_anchor_relation_ids.is_some());
        let mut bytes = policy_prefix(field_count, self.gate_scope_ref);
        encode_uint(&mut bytes, 17);
        encode_array_length(&mut bytes, self.review_requirements.len());
        for requirement in &self.review_requirements {
            bytes.extend_from_slice(&[0xa5, 0]);
            encode_uint(&mut bytes, requirement.review_role_id);
            bytes.push(1);
            encode_bstr_32(&mut bytes, requirement.review_scope_ref.as_bytes());
            bytes.push(2);
            encode_bstr_32(&mut bytes, requirement.review_method_ref.as_bytes());
            bytes.push(3);
            encode_bstr_32(&mut bytes, requirement.required_checks_ref.as_bytes());
            bytes.push(4);
            encode_uint(&mut bytes, requirement.required_count);
        }
        for (key, values) in [
            (18, &self.required_method_statuses),
            (19, &self.allowed_finding_states),
        ] {
            if let Some(values) = values {
                encode_uint(&mut bytes, key);
                uint_set(&mut bytes, values);
            }
        }
        if let Some(values) = &self.acceptable_anchor_relation_ids {
            encode_uint(&mut bytes, 24);
            bytes.extend_from_slice(&[0xa1, 0]);
            uint_set(&mut bytes, values);
        }
        policy_tail(
            &mut bytes,
            &self.operation_start_journal_ref,
            &self.supported_context_ids,
        );
        bytes
    }
}

impl ReviewAdmissionReviewRequirement {
    /// Constructs an exact local selector; references are not resolved or executed.
    /// Count is preserved as UInt, including zero, not interpreted as an Admission gate.
    pub fn new(
        review_role_id: u64,
        review_scope_ref: RecordId,
        review_method_ref: RecordId,
        required_checks_ref: RecordId,
        required_count: u64,
    ) -> Result<Self, RecordDecodeError> {
        if !matches!(review_role_id, 1..=5) || required_count > ER_UINT_MAX {
            return Err(RecordDecodeError);
        }
        Ok(Self {
            review_role_id,
            review_scope_ref,
            review_method_ref,
            required_checks_ref,
            required_count,
        })
    }
}

fn policy_prefix(field_count: usize, gate_scope_ref: RecordId) -> Vec<u8> {
    let mut bytes = vec![0x84, 0x78, 0x1a];
    bytes.extend_from_slice(RECORD_DOMAIN);
    encode_uint(&mut bytes, 40);
    encode_uint(&mut bytes, 1);
    encode_map_length(&mut bytes, field_count);
    bytes.extend_from_slice(&[0, 1, 1, 0x18, 40, 16]);
    encode_bstr_32(&mut bytes, gate_scope_ref.as_bytes());
    bytes
}

fn uint_set(bytes: &mut Vec<u8>, values: &[u64]) {
    encode_array_length(bytes, values.len());
    for value in values {
        encode_uint(bytes, *value);
    }
}

fn policy_tail(bytes: &mut Vec<u8>, start: &JournalReference, contexts: &[u64]) {
    encode_uint(bytes, 29);
    encode_journal_reference(bytes, start);
    encode_uint(bytes, 30);
    uint_set(bytes, contexts);
}
