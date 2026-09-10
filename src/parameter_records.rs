//! Portable parameter definitions, not execution or authority attestations.

use super::{
    encode_bstr, encode_text, encode_uint, CborCursor, RecordDecodeError, RecordId, RecordTypeId,
    StrictRecordFrame, ER_UINT_MAX, RECORD_DOMAIN,
};
use sha2::{Digest, Sha256};

/// Exact local CHECK_SET fields from Record Schema v0.3 §50.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckSetRecordInput {
    /// Nonempty, strictly increasing RecordIds. No sorting or deduplication is performed.
    pub check_refs: Vec<RecordId>,
}

/// A locally valid CHECK_SET envelope, not a resolved member closure.
/// Every referenced Record must additionally be resolved and validated as CHECK
/// by the consuming Store. This codec cannot establish that external obligation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckSetRecord {
    record_id: RecordId,
    input: CheckSetRecordInput,
}

impl CheckSetRecord {
    /// Checks local nonempty/sorted/unique constraints without resolving members.
    pub fn new(input: CheckSetRecordInput) -> Result<Self, RecordDecodeError> {
        if input.check_refs.is_empty() || input.check_refs.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(RecordDecodeError);
        }
        let mut record = Self {
            record_id: RecordId([0; 32]),
            input,
        };
        record.record_id = RecordId(Sha256::digest(record.authoritative_cbor()).into());
        Ok(record)
    }

    /// Emits the exact canonical local CHECK_SET bytes.
    pub fn authoritative_cbor(&self) -> Vec<u8> {
        let mut bytes = vec![0x84];
        encode_text(&mut bytes, "EvidenceRegistry.Record.v1");
        bytes.extend_from_slice(&[23, 1, 0xa3, 0, 1, 1, 23, 16]);
        super::encode_array_length(&mut bytes, self.input.check_refs.len());
        for id in &self.input.check_refs {
            super::encode_bstr_32(&mut bytes, id.as_bytes());
        }
        bytes
    }

    /// Strictly decodes local fields only; each member still needs CHECK resolution.
    pub fn decode_authoritative(input: &[u8]) -> Result<Self, RecordDecodeError> {
        let frame = StrictRecordFrame::decode_authoritative(input)?;
        if frame.record_type_id().value() != 23 {
            return Err(RecordDecodeError);
        }
        let mut cursor = CborCursor::new(input);
        cursor.array_exact(4).map_err(|_| RecordDecodeError)?;
        cursor
            .text_exact(RECORD_DOMAIN)
            .map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 23
            || cursor.uint().map_err(|_| RecordDecodeError)? != 1
        {
            return Err(RecordDecodeError);
        }
        cursor.map_exact(3).map_err(|_| RecordDecodeError)?;
        cursor.key(0).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 1 {
            return Err(RecordDecodeError);
        }
        cursor.key(1).map_err(|_| RecordDecodeError)?;
        if cursor.uint().map_err(|_| RecordDecodeError)? != 23 {
            return Err(RecordDecodeError);
        }
        cursor.key(16).map_err(|_| RecordDecodeError)?;
        let count = cursor.array().map_err(|_| RecordDecodeError)?;
        if count == 0 || count > cursor.remaining() / 34 {
            return Err(RecordDecodeError);
        }
        let mut check_refs = Vec::new();
        for _ in 0..count {
            let id = RecordId(cursor.bstr_32().map_err(|_| RecordDecodeError)?);
            if check_refs.last().is_some_and(|previous| previous >= &id) {
                return Err(RecordDecodeError);
            }
            check_refs.push(id);
        }
        if !cursor.finished() {
            return Err(RecordDecodeError);
        }
        Ok(Self {
            record_id: frame.record_id(),
            input: CheckSetRecordInput { check_refs },
        })
    }

    /// The exact-byte content identity.
    pub fn record_id(&self) -> RecordId {
        self.record_id
    }

    /// The assigned CHECK_SET Record Type ID.
    pub fn record_type_id(&self) -> RecordTypeId {
        RecordTypeId::try_from(23).expect("CHECK_SET is assigned in Record Schema v0.3")
    }

    /// The locally checked input, without claiming member resolution.
    pub fn input(&self) -> &CheckSetRecordInput {
        &self.input
    }

    /// The nonempty sorted unique member identities, not resolved CHECK Records.
    pub fn check_refs(&self) -> &[RecordId] {
        &self.input.check_refs
    }
}

/// Exact local METHOD fields from Record Schema v0.3 §48.
/// Profile semantics remain external; the payload is opaque definition data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MethodRecordInput {
    pub method_profile_id: u64,
    pub method_profile_version: u64,
    pub method_payload: Vec<u8>,
    pub method_label: Option<String>,
}

/// A strictly typed portable METHOD definition. It does not prove execution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MethodRecord {
    record_id: RecordId,
    input: MethodRecordInput,
}

impl MethodRecord {
    /// Constructs canonical local fields without assigning profile semantics.
    pub fn new(input: MethodRecordInput) -> Result<Self, RecordDecodeError> {
        if input.method_profile_id > ER_UINT_MAX || input.method_profile_version > ER_UINT_MAX {
            return Err(RecordDecodeError);
        }
        let mut record = Self {
            record_id: RecordId([0; 32]),
            input,
        };
        record.record_id = RecordId(Sha256::digest(record.authoritative_cbor()).into());
        Ok(record)
    }

    /// Emits the exact canonical identity-bearing definition, including its label.
    pub fn authoritative_cbor(&self) -> Vec<u8> {
        encode_profile(
            21,
            self.input.method_profile_id,
            self.input.method_profile_version,
            &self.input.method_payload,
            self.input.method_label.as_deref(),
        )
    }

    /// Strictly decodes only METHOD fields; no normalization or profile inference.
    pub fn decode_authoritative(input: &[u8]) -> Result<Self, RecordDecodeError> {
        let (record_id, profile_id, profile_version, payload, label) = decode_profile(input, 21)?;
        Ok(Self {
            record_id,
            input: MethodRecordInput {
                method_profile_id: profile_id,
                method_profile_version: profile_version,
                method_payload: payload,
                method_label: label,
            },
        })
    }

    /// The exact-byte content identity.
    pub fn record_id(&self) -> RecordId {
        self.record_id
    }

    /// The assigned METHOD Record Type ID.
    pub fn record_type_id(&self) -> RecordTypeId {
        RecordTypeId::try_from(21).expect("METHOD is assigned in Record Schema v0.3")
    }

    /// Exact definition data, without an execution or authority claim.
    pub fn input(&self) -> &MethodRecordInput {
        &self.input
    }
}

fn encode_profile(
    record_type: u8,
    profile_id: u64,
    profile_version: u64,
    payload: &[u8],
    label: Option<&str>,
) -> Vec<u8> {
    let mut bytes = vec![0x84];
    encode_text(&mut bytes, "EvidenceRegistry.Record.v1");
    bytes.extend_from_slice(&[
        record_type,
        1,
        if label.is_some() { 0xa6 } else { 0xa5 },
        0,
        1,
        1,
        record_type,
        16,
    ]);
    encode_uint(&mut bytes, profile_id);
    bytes.push(17);
    encode_uint(&mut bytes, profile_version);
    bytes.push(18);
    encode_bstr(&mut bytes, payload);
    if let Some(label) = label {
        bytes.push(19);
        encode_text(&mut bytes, label);
    }
    bytes
}

type DecodedProfile = (RecordId, u64, u64, Vec<u8>, Option<String>);

/// Exact local CHECK fields from Record Schema v0.3 §49.
/// Profile semantics remain external; the payload is opaque definition data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckRecordInput {
    pub check_profile_id: u64,
    pub check_profile_version: u64,
    pub check_payload: Vec<u8>,
    pub check_label: Option<String>,
}

/// A strictly typed portable CHECK definition, not a check result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckRecord {
    record_id: RecordId,
    input: CheckRecordInput,
}

impl CheckRecord {
    /// Constructs canonical local fields without assigning profile semantics.
    pub fn new(input: CheckRecordInput) -> Result<Self, RecordDecodeError> {
        if input.check_profile_id > ER_UINT_MAX || input.check_profile_version > ER_UINT_MAX {
            return Err(RecordDecodeError);
        }
        let mut record = Self {
            record_id: RecordId([0; 32]),
            input,
        };
        record.record_id = RecordId(Sha256::digest(record.authoritative_cbor()).into());
        Ok(record)
    }

    /// Emits the exact canonical identity-bearing definition, including its label.
    pub fn authoritative_cbor(&self) -> Vec<u8> {
        encode_profile(
            22,
            self.input.check_profile_id,
            self.input.check_profile_version,
            &self.input.check_payload,
            self.input.check_label.as_deref(),
        )
    }

    /// Strictly decodes only CHECK fields; no normalization or profile inference.
    pub fn decode_authoritative(input: &[u8]) -> Result<Self, RecordDecodeError> {
        let (record_id, profile_id, profile_version, payload, label) = decode_profile(input, 22)?;
        Ok(Self {
            record_id,
            input: CheckRecordInput {
                check_profile_id: profile_id,
                check_profile_version: profile_version,
                check_payload: payload,
                check_label: label,
            },
        })
    }

    /// The exact-byte content identity.
    pub fn record_id(&self) -> RecordId {
        self.record_id
    }

    /// The assigned CHECK Record Type ID.
    pub fn record_type_id(&self) -> RecordTypeId {
        RecordTypeId::try_from(22).expect("CHECK is assigned in Record Schema v0.3")
    }

    /// Exact definition data, without an execution or authority claim.
    pub fn input(&self) -> &CheckRecordInput {
        &self.input
    }
}

fn decode_profile(input: &[u8], record_type: u64) -> Result<DecodedProfile, RecordDecodeError> {
    let frame = StrictRecordFrame::decode_authoritative(input)?;
    if u64::from(frame.record_type_id().value()) != record_type {
        return Err(RecordDecodeError);
    }
    let mut cursor = CborCursor::new(input);
    cursor.array_exact(4).map_err(|_| RecordDecodeError)?;
    cursor
        .text_exact(RECORD_DOMAIN)
        .map_err(|_| RecordDecodeError)?;
    if cursor.uint().map_err(|_| RecordDecodeError)? != record_type
        || cursor.uint().map_err(|_| RecordDecodeError)? != 1
    {
        return Err(RecordDecodeError);
    }
    let fields = cursor.map().map_err(|_| RecordDecodeError)?;
    if !matches!(fields, 5 | 6) {
        return Err(RecordDecodeError);
    }
    cursor.key(0).map_err(|_| RecordDecodeError)?;
    if cursor.uint().map_err(|_| RecordDecodeError)? != 1 {
        return Err(RecordDecodeError);
    }
    cursor.key(1).map_err(|_| RecordDecodeError)?;
    if cursor.uint().map_err(|_| RecordDecodeError)? != record_type {
        return Err(RecordDecodeError);
    }
    cursor.key(16).map_err(|_| RecordDecodeError)?;
    let profile_id = cursor.uint().map_err(|_| RecordDecodeError)?;
    cursor.key(17).map_err(|_| RecordDecodeError)?;
    let profile_version = cursor.uint().map_err(|_| RecordDecodeError)?;
    cursor.key(18).map_err(|_| RecordDecodeError)?;
    let payload = cursor.bstr().map_err(|_| RecordDecodeError)?;
    let label = if fields == 6 {
        cursor.key(19).map_err(|_| RecordDecodeError)?;
        Some(cursor.text().map_err(|_| RecordDecodeError)?)
    } else {
        None
    };
    if !cursor.finished() {
        return Err(RecordDecodeError);
    }
    Ok((
        frame.record_id(),
        profile_id,
        profile_version,
        payload,
        label,
    ))
}
