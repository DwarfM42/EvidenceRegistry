# EvidenceRegistry Normative Record Schema Specification v0.3

**Status:** FINAL FROZEN
**Predecessor:** EvidenceRegistry Normative Record Schema Specification v0.2
**Project:** EvidenceRegistry
**Repository:** `DwarfM42/EvidenceRegistry`
**Normative lifecycle dependency:** `EvidenceRegistry Evidence Lifecycle Specification v0.10.2`
**Normative formal dependency:** `EvidenceRegistry Formal Verification & Implementation Boundary Specification v0.5.4`
**Normative transition dependency:** `EvidenceRegistry Normative Event & Transition Cross-Reference Table v0.3`
**Normative identity dependency:** `EvidenceRegistry Identity Format Specification v0.3`
**Purpose:** Stable Record Type Registry, Record-body schemas, Policy schema, evaluator coverage, formal-support schemas, and event-to-Record binding
**Normative language:** MUST / MUST NOT / SHOULD / SHOULD NOT / MAY

---

# 0. Specification Position

This specification defines the normative Record schemas consumed by EvidenceRegistry v0.x.

The upstream specifications already define:

```text id="v6n3fg"
what may become authoritative

when authority arises

which lifecycle transitions are legal

what identity bytes mean

how Journal References behave

how dependencies are encoded

what Policy is allowed to require

what formal support may mean
```

This specification defines:

```text id="a52wtf"
which Record types exist

their numeric Record Type IDs

their type-local field keys

their field types

required / optional / forbidden fields

context-required field symmetry

embedded structure encodings

Policy requirement composition

Policy-supported evaluation contexts

Policy applicability by evaluation context

Policy evaluator registry

Policy field → evaluator → vector coverage

event_type_id → permitted event Record type

event_type_id → Record disposition agreement

direct-authority field typing

chronology-binding classification

failed candidate identity representation

Manifest-relative Artifact eviction selectors
```

This specification MUST NOT silently introduce new lifecycle meaning.

If implementation of a Record schema requires a semantic rule not derivable from the upstream specifications:

```text id="b8p80y"
UPSTREAM_SEMANTIC_GAP
```

MUST be raised for specification review.

The missing rule MUST NOT be invented locally inside serializer or validator code.

---

## 0.1 Changes from v0.2

v0.3 closes all MATERIAL findings and associated MINOR findings from Record Schema review.

Changes include:

```text id="eim6na"
POLICY now declares supported_context_ids

POLICY_CONTEXT_UNSUPPORTED is defined bidirectionally:
the operation context must be declared by the Policy

Policy context support is checked before requirement
evaluation

event-associated Record chronology follows one
explicit classification rule

portable-only Records remain free of operation-start /
observation-start chronology bindings

the Freeze START / Eviction START chronology asymmetry
is explicitly declared intentional and inherited from
Cross-Reference v0.3

named Policy references are typed according to whether
the upstream relation is identity-only or authoritative

FREEZE START / Receipt use policy_record_id

REVIEW_REQUEST / REVIEW_ADMISSION / CLOSEOUT use
policy_authority_ref

REVIEW_RESULT.review_request_id changed to exact
review_request_authority_ref

all named direct-authority fields were audited against
Cross-Reference v0.3

CLOSEOUT Freeze / Manifest / Policy are unconditionally
required for both COMMITTED and REJECTED Records

CLOSEOUT admitted-review and creation-verification
collections are always present and explicitly permit []

failed Record candidates are role-bearing structured
FailedCandidateRecordRef values

MANIFEST added to Candidate Role Registry

PREDECESSOR_CLOSEOUT now explicitly distinguishes
"not requested" from "requested but unresolved"

Freeze START / Receipt / Manifest subject identity
continuity is explicitly required

Record Schema and event-integration vector suite
responsibilities are explicitly separated
```

No Lifecycle, Formal Verification, Cross-Reference, or Identity Format successor is required by these changes.

No known BLOCKING or MATERIAL finding remains open against this frozen revision.

---

# 1. Governing Principle

The central Record-schema rule is:

> A Record may describe only facts or requirements whose semantics are already defined and mechanically evaluable.

For Policy specifically:

> A Policy field MUST NOT exist normatively unless an evaluator exists and positive and negative vectors exist for that evaluator.

For all Records:

> Presence is semantic. Absence is semantic. Null is semantic only where explicitly defined.

---

# 2. Relationship to Identity Format v0.3

Every Record defined here uses Identity Format v0.3 framing:

```text id="5ivm2i"
[
  "EvidenceRegistry.Record.v1",
  record_type_id,
  schema_version,
  record_body
]
```

Every `record_body` contains:

```text id="dm234a"
key 0 = schema_version
key 1 = record_type_id
```

For every Record schema in this specification:

```text id="9m49sl"
schema_version = 1
```

Type-local field keys begin at:

```text id="yfno5f"
16
```

Keys:

```text id="ynqtlr"
2..15
```

are reserved common Record-body keys and MUST be absent.

---

# 3. Record Type Registry Governance

`record_type_id` is an unsigned registered identifier.

Valid range:

```text id="1m45yd"
1..65535
```

Value:

```text id="0vfrj5"
0
```

is invalid/unassigned.

An assigned Record Type ID MUST NEVER be reused for different semantics.

---

## 3.1 Record Type assignment states

Every possible Record Type ID has one machine-readable assignment state:

```text id="vb0j0k"
ASSIGNED

UNASSIGNED

RETIRED
```

`ASSIGNED` means the ID has an active normative meaning.

`UNASSIGNED` means the ID has never been assigned and MAY be assigned by a future explicit Record Schema successor.

`RETIRED` means the ID had a previous normative assignment that is no longer active.

A RETIRED ID MUST NEVER be reused.

---

## 3.2 Current gaps

Numeric gaps in the v0.3 Record Type Registry are:

```text id="n9ne73"
UNASSIGNED
```

not RETIRED.

They may be used by a future explicit successor.

They MUST NOT be interpreted as hidden or reserved semantic Record types.

---

## 3.3 Registry machine metadata

The machine-readable Record Type Registry MUST contain:

```text id="xfgo5b"
record_type_id

record_type_name

assignment_state

first_schema_version

retired_in_schema_version
  where applicable
```

---

# 4. Record Type Registry v0.3

| ID | Record Type                                     |
| -: | ----------------------------------------------- |
|  1 | `GENESIS`                                       |
|  2 | `MANIFEST`                                      |
|  3 | `FREEZE_ATTEMPT_START`                          |
|  4 | `FREEZE_RECEIPT`                                |
|  5 | `FREEZE_ABORT_RECOVERY`                         |
|  6 | `OPERATOR_ABORT_ASSERTION`                      |
|  7 | `FREEZE_COMMIT_REJECTION`                       |
| 10 | `SOURCE_BINDING`                                |
| 11 | `VERIFICATION_OBSERVATION`                      |
| 12 | `VERIFICATION_RECORD`                           |
| 20 | `SCOPE`                                         |
| 21 | `METHOD`                                        |
| 22 | `CHECK`                                         |
| 23 | `CHECK_SET`                                     |
| 30 | `REVIEW_REQUEST`                                |
| 31 | `REVIEW_RESULT`                                 |
| 32 | `REVIEW_ADMISSION`                              |
| 40 | `POLICY`                                        |
| 50 | `CLOSEOUT`                                      |
| 60 | `STORAGE_CAPABILITY_CLASS`                      |
| 61 | `ENVIRONMENT_OBSERVATION`                       |
| 62 | `STORAGE_CAPABILITY_CHANGE`                     |
| 63 | `CAPABILITY_OBSERVATION_PROVENANCE`             |
| 70 | `EVICTION_SCOPE`                                |
| 71 | `ARTIFACT_EVICTION_START`                       |
| 72 | `ARTIFACT_EVICTION_TERMINAL`                    |
| 80 | `ASSUMPTION_DEFINITION`                         |
| 81 | `ASSUMPTION_ESTABLISHMENT`                      |
| 82 | `ASSUMPTION_INVALIDATION`                       |
| 83 | `FORMAL_VERIFICATION`                           |
| 84 | `ASSUMPTION_VERSION_COMPATIBILITY`              |
| 85 | `ASSUMPTION_VERSION_COMPATIBILITY_INVALIDATION` |
| 86 | `BOOTSTRAP_TRUST_DECLARATION`                   |
| 87 | `BOOTSTRAP_TRUST_INVALIDATION`                  |
| 88 | `FORMAL_FINDING_CLASSIFICATION`                 |
| 89 | `PROOF_DEPENDENCY_MANIFEST`                     |
| 90 | `SHARED_DEPENDENCY_MANIFEST`                    |

No Record Type ID outside this table is ASSIGNED in v0.3.

---

# 5. Event Type to Event Record Type Mapping

Every authoritative Journal event MUST use an `event_record_id` whose Record type matches this table.

| Event ID | Event                                          | Required Record Type                            |
| -------: | ---------------------------------------------- | ----------------------------------------------- |
|        1 | `GENESIS`                                      | `GENESIS`                                       |
|      100 | `FREEZE_ATTEMPT_STARTED`                       | `FREEZE_ATTEMPT_START`                          |
|      101 | `FREEZE_COMMITTED`                             | `FREEZE_RECEIPT`                                |
|      102 | `FREEZE_ABORTED_RECOVERY`                      | `FREEZE_ABORT_RECOVERY`                         |
|      103 | `FREEZE_ABORTED_BY_OPERATOR_ASSERTION`         | `OPERATOR_ABORT_ASSERTION`                      |
|      104 | `FREEZE_COMMIT_REJECTED`                       | `FREEZE_COMMIT_REJECTION`                       |
|      200 | `VERIFICATION_RECORDED`                        | `VERIFICATION_RECORD`                           |
|      300 | `REVIEW_REQUEST_RECORDED`                      | `REVIEW_REQUEST`                                |
|      301 | `REVIEW_RESULT_RECORDED`                       | `REVIEW_RESULT`                                 |
|      302 | `REVIEW_ADMISSION_ACCEPTED`                    | `REVIEW_ADMISSION`                              |
|      303 | `REVIEW_ADMISSION_REJECTED`                    | `REVIEW_ADMISSION`                              |
|      400 | `POLICY_RECORDED`                              | `POLICY`                                        |
|      500 | `CLOSEOUT_COMMITTED`                           | `CLOSEOUT`                                      |
|      501 | `CLOSEOUT_REJECTED`                            | `CLOSEOUT`                                      |
|      600 | `STORAGE_CAPABILITY_CHANGED`                   | `STORAGE_CAPABILITY_CHANGE`                     |
|      700 | `ARTIFACT_EVICTION_STARTED`                    | `ARTIFACT_EVICTION_START`                       |
|      701 | `ARTIFACT_EVICTION_COMMITTED`                  | `ARTIFACT_EVICTION_TERMINAL`                    |
|      702 | `ARTIFACT_EVICTION_INTERRUPTED_RECOVERY`       | `ARTIFACT_EVICTION_TERMINAL`                    |
|      703 | `ARTIFACT_EVICTION_BLOCKED`                    | `ARTIFACT_EVICTION_TERMINAL`                    |
|      800 | `ASSUMPTION_DEFINITION_RECORDED`               | `ASSUMPTION_DEFINITION`                         |
|      801 | `ASSUMPTION_ESTABLISHMENT_RECORDED`            | `ASSUMPTION_ESTABLISHMENT`                      |
|      802 | `ASSUMPTION_INVALIDATION_RECORDED`             | `ASSUMPTION_INVALIDATION`                       |
|      803 | `FORMAL_VERIFICATION_RECORDED`                 | `FORMAL_VERIFICATION`                           |
|      804 | `ASSUMPTION_VERSION_COMPATIBILITY_RECORDED`    | `ASSUMPTION_VERSION_COMPATIBILITY`              |
|      805 | `ASSUMPTION_VERSION_COMPATIBILITY_INVALIDATED` | `ASSUMPTION_VERSION_COMPATIBILITY_INVALIDATION` |
|      806 | `BOOTSTRAP_TRUST_DECLARATION_RECORDED`         | `BOOTSTRAP_TRUST_DECLARATION`                   |
|      807 | `BOOTSTRAP_TRUST_INVALIDATED`                  | `BOOTSTRAP_TRUST_INVALIDATION`                  |
|      808 | `FORMAL_FINDING_CLASSIFICATION_RECORDED`       | `FORMAL_FINDING_CLASSIFICATION`                 |

A structurally valid Record of the wrong type MUST NOT serve as the event Record.

---

# 6. Event Type and Record Disposition Agreement

Where multiple Journal events share one Record type, the event and Record disposition MUST agree exactly.

| Event ID | Required disposition                                                        |
| -------: | --------------------------------------------------------------------------- |
|      302 | `REVIEW_ADMISSION.disposition_id = ACCEPTED`                                |
|      303 | `REVIEW_ADMISSION.disposition_id = REJECTED`                                |
|      500 | `CLOSEOUT.disposition_id = COMMITTED`                                       |
|      501 | `CLOSEOUT.disposition_id = REJECTED`                                        |
|      701 | `ARTIFACT_EVICTION_TERMINAL.terminal_disposition_id = COMMITTED`            |
|      702 | `ARTIFACT_EVICTION_TERMINAL.terminal_disposition_id = INTERRUPTED_RECOVERY` |
|      703 | `ARTIFACT_EVICTION_TERMINAL.terminal_disposition_id = BLOCKED`              |

Event Record validation MUST verify:

```text id="2emh7j"
event_type_id
+
record_type_id
+
Record disposition
```

as one coherent tuple.

A valid Record with a disposition inconsistent with its containing Journal event is invalid as that event payload.

---

# 7. Named Authority Binding Rule

A named Record field representing an exact prior Registry authority uses:

```text id="5v2ei5"
JournalReference
```

when the controlling Cross-Reference requires that authority directly.

A named field representing immutable identity without requiring prior Registry authority uses:

```text id="5g61ns"
RecordId
```

or another exact identity type defined by the schema.

The two MUST NOT be interchanged merely because a JournalReference contains an `event_record_id`.

---

# 8. Policy Reference Typing

The following Policy-reference types are normative.

| Record                 | Field meaning                                | Type                               |
| ---------------------- | -------------------------------------------- | ---------------------------------- |
| `FREEZE_ATTEMPT_START` | prospective Policy identity for later COMMIT | `RecordId POLICY`                  |
| `FREEZE_RECEIPT`       | same prospective Policy identity             | `RecordId POLICY`                  |
| `REVIEW_REQUEST`       | exact authoritative Policy                   | `JournalReference POLICY_RECORDED` |
| `REVIEW_ADMISSION`     | exact authoritative Policy                   | `JournalReference POLICY_RECORDED` |
| `CLOSEOUT`             | exact authoritative Policy                   | `JournalReference POLICY_RECORDED` |

Therefore:

```text id="lsy530"
FREEZE_ATTEMPT_START.policy_record_id

FREEZE_RECEIPT.policy_record_id
```

are identity bindings.

Whereas:

```text id="erqm8e"
REVIEW_REQUEST.policy_authority_ref

REVIEW_ADMISSION.policy_authority_ref

CLOSEOUT.policy_authority_ref
```

are authority bindings.

---

## 8.1 No accidental Policy authority

A `RecordId POLICY` does not prove that `POLICY_RECORDED` authority exists.

---

## 8.2 No authority collapse to identity

Where Policy authority is required, the implementation MUST NOT replace the required JournalReference with only its `event_record_id`.

---

# 9. Named Direct-Authority Field Audit

The following named Record relations correspond to direct authority requirements from Cross-Reference v0.3.

| Record                                          | Normative named direct-authority source                                                                                    |
| ----------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `FREEZE_RECEIPT`                                | §56: `attempt_start_journal_ref`                                                                                           |
| `FREEZE_ABORT_RECOVERY`                         | §59: `attempt_start_journal_ref`                                                                                           |
| `OPERATOR_ABORT_ASSERTION`                      | §60: `attempt_start_journal_ref`                                                                                           |
| `FREEZE_COMMIT_REJECTION`                       | §61: `attempt_start_journal_ref`, `conflicting_terminal_journal_ref`                                                       |
| `VERIFICATION_RECORD`                           | §64: `freeze_authority_ref`; POST also `brackets_closeout_authority_ref`                                                   |
| `REVIEW_REQUEST`                                | §65: `freeze_authority_ref`, `policy_authority_ref`                                                                        |
| `REVIEW_RESULT`                                 | §66: `review_request_authority_ref`                                                                                        |
| `REVIEW_ADMISSION`                              | §67: `review_request_ref`, `review_result_ref`, `policy_authority_ref` as context permits                                  |
| `CLOSEOUT`                                      | §§68–74: exact Freeze, Policy, selected Review, Verification, formal, compatibility, Bootstrap, Classification authorities |
| `ARTIFACT_EVICTION_START`                       | §83: `freeze_authority_ref`                                                                                                |
| `ARTIFACT_EVICTION_TERMINAL`                    | §84: `eviction_start_journal_ref`                                                                                          |
| `ASSUMPTION_ESTABLISHMENT`                      | §87: `assumption_definition_authority_ref`                                                                                 |
| `ASSUMPTION_INVALIDATION`                       | §88: explicit-mode target Establishment refs                                                                               |
| `FORMAL_VERIFICATION`                           | §89: Assumption Definition refs; Freeze iff claimed                                                                        |
| `ASSUMPTION_VERSION_COMPATIBILITY`              | §90: source + target Definition refs                                                                                       |
| `ASSUMPTION_VERSION_COMPATIBILITY_INVALIDATION` | §91: explicit-mode Compatibility refs                                                                                      |
| `BOOTSTRAP_TRUST_DECLARATION`                   | §92: predecessor Declaration iff present                                                                                   |
| `BOOTSTRAP_TRUST_INVALIDATION`                  | §93: explicit-mode Declaration refs                                                                                        |
| `FORMAL_FINDING_CLASSIFICATION`                 | §94: exact Formal Verification ref                                                                                         |

This table is a machine-auditable schema cross-reference.

The machine schema registry MUST resolve each cited Record schema into concrete field keys and types.

A mismatch with Cross-Reference v0.3 is:

```text id="pukqjw"
UPSTREAM_AUTHORITY_BINDING_MISMATCH
```

and blocks conformance.

---

# 10. Chronology Binding Principle

Chronology fields are not Policy evaluation contexts.

They describe when an operation or observation began, or bind a multi-event lifecycle object to its prior authoritative event.

Every event-associated Record type MUST belong to one explicit chronology-binding class.

Portable-only Records do not acquire an operation-start or observation-start chronology field merely because they are content-addressed.

---

# 11. Chronology Binding Classes

Use:

```text id="wh7yr9"
BOOTSTRAP_NO_PRIOR

JOURNAL_START_IS_BOUNDARY

OPERATION_START

OBSERVATION_START

ATTEMPT_START

EVICTION_START

CONFLICTING_TERMINAL
```

A Record MAY require more than one class where the upstream operation requires multiple historical relations.

---

# 12. Event Record Chronology Classification

| Record Type                                     | Chronology binding                       |
| ----------------------------------------------- | ---------------------------------------- |
| `GENESIS`                                       | `BOOTSTRAP_NO_PRIOR`                     |
| `FREEZE_ATTEMPT_START`                          | `JOURNAL_START_IS_BOUNDARY`              |
| `FREEZE_RECEIPT`                                | `ATTEMPT_START`                          |
| `FREEZE_ABORT_RECOVERY`                         | `ATTEMPT_START` + `OPERATION_START`      |
| `OPERATOR_ABORT_ASSERTION`                      | `ATTEMPT_START` + `OPERATION_START`      |
| `FREEZE_COMMIT_REJECTION`                       | `ATTEMPT_START` + `CONFLICTING_TERMINAL` |
| `VERIFICATION_RECORD`                           | `OBSERVATION_START`                      |
| `REVIEW_REQUEST`                                | `OPERATION_START`                        |
| `REVIEW_RESULT`                                 | `OPERATION_START`                        |
| `REVIEW_ADMISSION`                              | `OPERATION_START`                        |
| `POLICY`                                        | `OPERATION_START`                        |
| `CLOSEOUT`                                      | `OPERATION_START`                        |
| `STORAGE_CAPABILITY_CHANGE`                     | `OPERATION_START`                        |
| `ARTIFACT_EVICTION_START`                       | `OPERATION_START`                        |
| `ARTIFACT_EVICTION_TERMINAL`                    | `EVICTION_START`                         |
| `ASSUMPTION_DEFINITION`                         | `OPERATION_START`                        |
| `ASSUMPTION_ESTABLISHMENT`                      | `OBSERVATION_START`                      |
| `ASSUMPTION_INVALIDATION`                       | `OPERATION_START`                        |
| `FORMAL_VERIFICATION`                           | `OPERATION_START`                        |
| `ASSUMPTION_VERSION_COMPATIBILITY`              | `OPERATION_START`                        |
| `ASSUMPTION_VERSION_COMPATIBILITY_INVALIDATION` | `OPERATION_START`                        |
| `BOOTSTRAP_TRUST_DECLARATION`                   | `OPERATION_START`                        |
| `BOOTSTRAP_TRUST_INVALIDATION`                  | `OPERATION_START`                        |
| `FORMAL_FINDING_CLASSIFICATION`                 | `OPERATION_START`                        |

---

## 12.1 GENESIS

GENESIS has no prior authoritative Journal history.

No prior chronology Reference exists.

---

## 12.2 Freeze START

`FREEZE_ATTEMPT_STARTED` itself establishes the lower bound of the Freeze Attempt.

Its Record therefore does not contain `operation_start_journal_ref`.

The START Journal Entry is the authoritative lifecycle start.

---

## 12.3 Freeze START / Eviction START asymmetry

The chronology difference between:

```text id="smmbva"
FREEZE_ATTEMPT_START
    → JOURNAL_START_IS_BOUNDARY

ARTIFACT_EVICTION_START
    → OPERATION_START
```

is intentional.

Both are TWO_PHASE START events, but Cross-Reference v0.3 explicitly requires:

```text id="rzv0tz"
ARTIFACT_EVICTION_START.operation_start_journal_ref
```

while it does not require an equivalent field for Freeze START.

Therefore Record Schema preserves that upstream asymmetry exactly.

An implementation MUST NOT:

```text id="zl209c"
add operation_start_journal_ref to Freeze START

or

remove operation_start_journal_ref from Eviction START
```

merely to make the two START schemas visually symmetric.

This asymmetry is inherited from the controlling Cross-Reference and is not evidence of a missing field in either Record.

---

## 12.4 Portable-only Records

Portable-only Records such as:

```text id="8bd1dj"
MANIFEST
SOURCE_BINDING
VERIFICATION_OBSERVATION
SCOPE
METHOD
CHECK
CHECK_SET
STORAGE_CAPABILITY_CLASS
ENVIRONMENT_OBSERVATION
CAPABILITY_OBSERVATION_PROVENANCE
EVICTION_SCOPE
PROOF_DEPENDENCY_MANIFEST
SHARED_DEPENDENCY_MANIFEST
```

do not contain `operation_start_journal_ref` or `observation_start_journal_ref` unless a future explicit successor changes their semantics.

They MAY contain prior JournalReferences for other roles.

---

# 13. Portable Records Without Journal Events

The following Record types MAY exist without Registry authority:

```text id="k5b0f6"
MANIFEST
SOURCE_BINDING
VERIFICATION_OBSERVATION
SCOPE
METHOD
CHECK
CHECK_SET
STORAGE_CAPABILITY_CLASS
ENVIRONMENT_OBSERVATION
CAPABILITY_OBSERVATION_PROVENANCE
EVICTION_SCOPE
PROOF_DEPENDENCY_MANIFEST
SHARED_DEPENDENCY_MANIFEST
```

Their physical existence does not grant authority.

They acquire lifecycle significance only when referenced by an authoritative Record or Journal event according to the upstream specifications.

---

# 14. Common Primitive Types

This specification uses:

```text id="lj5rlf"
RecordId
    = bstr(32)

RegistryId
    = bstr(32)

OpaqueId32
    = bstr(32)

JournalReference
    = Identity Format v0.3 fixed 5-element array

UInt
    = unsigned integer within applicable range

Text
    = valid UTF-8 text without normalization

Bool
    = CBOR true / false

CanonicalPath
    = nonempty array of canonical path-component bstr values
      interpreted under one exact path_identity_profile_id
```

---

# 15. Set Encoding in Record Bodies

Where this specification declares a Record field to be a semantic set:

```text id="5lmrx5"
duplicates MUST be rejected
```

and one canonical order MUST be used.

---

## 15.1 Sets of RecordId

Canonical order:

```text id="yokcc3"
lexicographic ascending by 32 raw bytes
```

---

## 15.2 Sets of JournalReference

Where authority ordering applies:

```text id="n1dkui"
entry_index ascending

then entry_hash bytewise ascending
```

---

## 15.3 Sets of UInt

```text id="63wnr2"
numeric ascending
```

---

## 15.4 Sets of Text

```text id="ylvgo1"
lexicographic ascending by exact UTF-8 payload bytes
```

---

## 15.5 Sets of CanonicalPath

Use exactly:

```text id="32y7wp"
Lifecycle canonical path comparator
```

The same comparator used for Manifest path ordering MUST be used.

---

# 16. Embedded Map Canonicalization

Embedded structures defined by this specification are deterministic CBOR maps with integer keys.

Keys MUST be emitted in ascending numeric order.

Unknown keys are rejected.

Optional fields are absent when not applicable.

Null MUST NOT substitute for absence unless explicitly defined.

---

# 17. Enumerated Semantic Registries

The following registries are normative for v0.3.

---

## 17.1 Method Status Registry

```text id="b8ez5p"
1 = VALID
2 = INVALID
3 = ENVIRONMENTALLY_DEGRADED
```

---

## 17.2 Finding State Registry

```text id="n7x8fb"
1 = NO_BLOCKING
2 = BLOCKING
3 = INDETERMINATE
```

---

## 17.3 Verification Target Registry

```text id="dlgoza"
1 = EMBEDDED_COPY_INTEGRITY
2 = SOURCE_SUBJECT_MATCH
```

`SOURCE_SUBJECT_NODRIFT` has no v0.3 assignment.

---

## 17.4 Review Role Registry

```text id="cpovgn"
1 = HOSTILE
2 = INDEPENDENT
3 = OWNER
4 = SPECIALIST
5 = AUTOMATED
```

Role identity does not prove causal or organizational independence.

---

## 17.5 Custody Mode Registry

```text id="jxf9zy"
1 = EMBEDDED
2 = REFERENCED
```

---

## 17.6 Review Admission Disposition Registry

```text id="h14ixn"
1 = ACCEPTED
2 = REJECTED
```

---

## 17.7 Closeout Disposition Registry

```text id="05werz"
1 = COMMITTED
2 = REJECTED
```

---

## 17.8 Capability State Registry

```text id="rq3u3v"
1 = PRESENT
2 = ABSENT
3 = NOT_PROBED
```

---

## 17.9 Durability Fact State Registry

```text id="ttz1lm"
1 = PERFORMED
2 = NOT_PERFORMED
3 = UNSUPPORTED
```

---

## 17.10 Eviction Terminal Disposition Registry

```text id="6a1j3m"
1 = COMMITTED
2 = INTERRUPTED_RECOVERY
3 = BLOCKED
```

---

## 17.11 Eviction Recovery Observation Registry

```text id="e6l2cq"
1 = NONE_EVICTED
2 = PARTIALLY_EVICTED
3 = FULLY_EVICTED
4 = INDETERMINATE
```

---

## 17.12 Invalidation Target Mode Registry

```text id="uk0vzw"
1 = EXPLICIT_SET
2 = ATTRIBUTE_PREDICATE
```

---

## 17.13 Formal Verification Outcome Registry

```text id="4ug85g"
1 = VERIFIED
2 = COUNTEREXAMPLE
3 = TIMEOUT
4 = RESOURCE_EXHAUSTED
5 = TOOL_ERROR
6 = INVALID_CONFIGURATION
7 = UNSUPPORTED
```

---

## 17.14 Formal Finding Classification Registry

```text id="0zbj4b"
1 = IMPLEMENTATION_PROPERTY_VIOLATION
2 = FORMAL_MODEL_DEFECT
3 = PROPERTY_UNDERSPECIFIED
4 = ASSUMPTION_MODELING_DEFECT
5 = UNRESOLVED
```

---

## 17.15 Judgment Basis Registry

```text id="dp44jb"
1 = HUMAN_JUDGMENT
```

---

## 17.16 Verification Authority Mode Registry

```text id="ud4sde"
1 = AUTHORITATIVE_REQUIRED
```

No portable nonauthoritative Observation can satisfy an authoritative Verification requirement.

---

## 17.17 Verification Phase Registry

```text id="g1v3su"
1 = CREATION_TIME
2 = PRE
3 = POST
```

---

## 17.18 Closeout Postcondition Type Registry

```text id="f9cdz4"
1 = SOURCE_MATCH_BRACKET
```

No generic Boolean requirement algebra exists in v0.3.

---

# 18. Candidate Role Registry

A failed candidate carries one exact semantic role.

```text id="7bx0e5"
1  = REVIEW_REQUEST

2  = REVIEW_RESULT

3  = POLICY

4  = FREEZE

5  = ADMITTED_REVIEW

6  = CREATION_VERIFICATION

7  = PRE_VERIFICATION

8  = PREDECESSOR_CLOSEOUT

9  = FORMAL_VERIFICATION

10 = ASSUMPTION_ESTABLISHMENT

11 = ASSUMPTION_VERSION_COMPATIBILITY

12 = BOOTSTRAP_TRUST_DECLARATION

13 = FORMAL_FINDING_CLASSIFICATION

14 = MANIFEST
```

---

## 18.1 Admission permitted roles

Admission failed candidates MAY use:

```text id="fdtpzb"
REVIEW_REQUEST
REVIEW_RESULT
POLICY
```

---

## 18.2 Closeout permitted roles

Closeout failed candidates MAY use:

```text id="z9tw8e"
MANIFEST
ADMITTED_REVIEW
CREATION_VERIFICATION
PRE_VERIFICATION
PREDECESSOR_CLOSEOUT
FORMAL_VERIFICATION
ASSUMPTION_ESTABLISHMENT
ASSUMPTION_VERSION_COMPATIBILITY
BOOTSTRAP_TRUST_DECLARATION
FORMAL_FINDING_CLASSIFICATION
```

Freeze and Policy authority are mandatory to establish the Closeout context and therefore are not represented as failed Closeout candidates under §69.

---

# 19. Candidate Validation Failure Registry

The registry covers both Journal-reference and Record-identity candidate failures.

---

## 19.1 Journal-reference failure IDs

```text id="94u1rf"
1 = ENTRY_NOT_FOUND

2 = ENTRY_HASH_MISMATCH

3 = EVENT_TYPE_MISMATCH

4 = EVENT_RECORD_ID_MISMATCH

5 = LIFECYCLE_OBJECT_MISMATCH

6 = CROSS_REGISTRY_REFERENCE

7 = FORWARD_REFERENCE

8 = AUTHORITY_RELATION_INVALID

9 = AUTHORITATIVE_RECORD_PAYLOAD_MISSING
```

---

## 19.2 Record-identity failure IDs

```text id="in4buv"
20 = RECORD_NOT_FOUND

21 = RECORD_ID_MISMATCH

22 = RECORD_TYPE_MISMATCH

23 = RECORD_NONAUTHORITATIVE

24 = RECORD_BINDING_MISMATCH

25 = RECORD_PAYLOAD_MISSING
```

Implementations MUST NOT invent free-form replacements where a registered class applies.

---

# 20. FailedCandidateJournalRef Structure

```text id="yept4w"
0 = candidate_role_id

1 = journal_ref

2 = validation_failure_code_id
```

All fields are required.

The `journal_ref` MUST be structurally valid as a five-field JournalReference.

Its contextual or authority validation fails according to key 2.

---

## 20.1 Ordering

Canonical order:

```text id="xoj17e"
candidate_role_id

journal_ref.registry_id bytes

journal_ref.entry_index

journal_ref.entry_hash bytes

journal_ref.event_type_id

journal_ref.event_record_id bytes

validation_failure_code_id
```

---

## 20.2 One controlling failure

For one:

```text id="mmmfjk"
(candidate_role_id, journal_ref)
```

pair, at most one authoritative controlling failure code may appear.

Additional diagnostics MAY be emitted outside the Record.

---

# 21. FailedCandidateRecordRef Structure

A failed Record candidate is:

```text id="cmg6rb"
0 = candidate_role_id

1 = record_id

2 = validation_failure_code_id
```

All fields are required.

Key 2 MUST use one of the Record-identity failure IDs from §19.2.

---

## 21.1 Why Record candidates carry roles

A raw RecordId does not state why that Record was being considered.

Therefore:

```text id="zwzqun"
MANIFEST candidate

FORMAL_VERIFICATION candidate

ADMITTED_REVIEW candidate
```

must remain distinguishable even if all are represented by `bstr(32)` identities.

Role is part of the rejection statement.

---

## 21.2 Canonical ordering

Sort by:

```text id="1u12no"
candidate_role_id

then record_id bytes

then validation_failure_code_id
```

Duplicates are invalid.

For one:

```text id="vx1pr1"
(candidate_role_id, record_id)
```

pair, at most one controlling failure code is authoritative.

---

# 22. Failed Candidate Structural Boundary

A failed candidate may enter authoritative rejection history only if its identity is reconstructable in one of the forms defined by §§20–21.

If a Journal candidate cannot even be represented as a structurally valid JournalReference:

```text id="mqp96c"
it MUST NOT appear in failed_candidate_journal_refs
```

If a Record candidate has no reconstructable exact RecordId:

```text id="70z4vt"
it MUST NOT appear in failed_candidate_record_refs
```

Such failures remain before the relevant lifecycle historical threshold unless another upstream rule explicitly states otherwise.

---

# 23. Reason Code Encoding

`reason_codes[]` uses exact uppercase ASCII/UTF-8 tokens.

It is a semantic set sorted by exact UTF-8 bytes.

Duplicate tokens are invalid.

Reason codes supplement structured fields.

They MUST NOT replace a structured candidate identity or failure class when one is available.

---

# 24. Evaluation Context Registry

Policy evaluation occurs under one explicit mechanical context.

```text id="tc3hqu"
1 = FREEZE_COMMIT

2 = REVIEW_ADMISSION

3 = CLOSEOUT_CREATION

4 = CLOSEOUT_POSTCONDITION

5 = REVIEW_REQUEST_CREATION
```

The context is supplied by the operation.

It MUST NOT be inferred from whichever fields happen to exist in the Policy.

---

# 25. One Policy Record Represents One Qualification Gate

A `POLICY` Record represents one exact qualification gate.

Conceptually:

```text id="bpunjr"
Policy
├─ gate Scope
├─ supported evaluation contexts
├─ Review requirements
├─ Verification requirements
├─ durability requirements
├─ Anchor requirements
├─ Bootstrap requirements
├─ formal support requirements
├─ diversity requirements
└─ Closeout postconditions
```

A different qualification gate SHOULD use a different Policy Record.

---

# 26. Policy Supported Contexts

Every POLICY Record MUST contain:

```text id="zye04o"
supported_context_ids
```

as a nonempty sorted unique set of Evaluation Context IDs.

Before evaluating any requirement:

```text id="4hvzk7"
current_context ∈ supported_context_ids
```

MUST hold.

Otherwise:

```text id="1nsrvn"
POLICY_CONTEXT_UNSUPPORTED
```

and the Policy MUST NOT be evaluated as though the context were supported.

---

## 26.1 No context inference

The existence of fields such as:

```text id="7mjpnf"
review_requirements

verification_requirements

minimum_durability
```

MUST NOT cause the implementation to infer support for their associated contexts.

Support is explicit.

---

## 26.2 Required context-specific structural content

If:

```text id="8j9ak3"
REVIEW_REQUEST_CREATION
```

is supported:

```text id="5s4zz7"
review_requirements MUST be present
```

If:

```text id="30oyw7"
REVIEW_ADMISSION
```

is supported:

```text id="tsihyi"
review_requirements MUST be present
```

If:

```text id="yaq939"
CLOSEOUT_POSTCONDITION
```

is supported:

```text id="bvu3kh"
closeout_postconditions MUST be present
```

If `closeout_postconditions` is present:

```text id="g36deq"
CLOSEOUT_CREATION
and
CLOSEOUT_POSTCONDITION
```

MUST both appear in `supported_context_ids`.

---

## 26.3 Explicit zero-requirement context

`FREEZE_COMMIT` or `CLOSEOUT_CREATION` MAY be explicitly supported even when no additional optional requirement field applies.

That means:

```text id="bwh45s"
this Policy intentionally permits this context
with no additional requirement of those classes
```

not:

```text id="esmvqe"
the implementation forgot which context this Policy was for
```

---

## 26.4 Dead requirement prevention

For every present Policy requirement field, at least one declared supported context MUST make some normative part of that field applicable.

A Policy field whose semantics are unreachable from every supported context is invalid.

Thus:

```text id="z28zss"
field present
+
no supported context can evaluate it
```

is a schema error rather than a dormant requirement.

---

# 27. Policy Requirement Composition

All applicable Policy requirements compose by logical AND.

```text id="v4crqe"
PolicySatisfied(P, Context, Evidence)
iff

ContextSupported(P, Context)

and

for every applicable requirement R in P:

    Evaluate(R, Context, Evidence)
    == SATISFIED
```

One applicable FAIL makes the gate unsatisfied.

One applicable INDETERMINATE prevents success.

No implicit OR exists.

---

# 28. No Implicit Policy OR

An implementation MUST NOT infer:

```text id="moj7dw"
A OR B
```

from multiple fields, requirements, Review paths, support paths, or human-readable descriptions.

A future requirement-expression algebra requires an explicit successor.

---

# 29. Policy Applicability

Applicability is defined by:

```text id="laccwa"
Policy field identity

+
explicit supported context

+
current evaluation context

+
where applicable, the status-bearing Evidence
consumed in that context
```

A present field that is not applicable in the current supported context MUST NOT become a hidden gate.

---

# 30. Policy Applicability Matrix

| Policy requirement                              | REVIEW_REQUEST_CREATION | FREEZE_COMMIT | REVIEW_ADMISSION |        CLOSEOUT_CREATION | CLOSEOUT_POSTCONDITION |
| ----------------------------------------------- | ----------------------: | ------------: | ---------------: | -----------------------: | ---------------------: |
| supported context                               |                     yes |           yes |              yes |                      yes |                    yes |
| Review selector: role / Scope / Method / Checks |                     yes |            no |              yes |           yes, selection |                     no |
| Review `required_count`                         |                      no |            no |               no |                      yes |                     no |
| `required_method_status`                        |                      no |            no |              yes | yes, listed classes only |                    yes |
| `allowed_finding_states`                        |                      no |            no |              yes | yes, listed classes only |                    yes |
| Verification target / authority                 |                      no |            no |               no |                      yes |      via postcondition |
| intervening-event constraint                    |                      no |            no |               no |                      yes |                     no |
| max journal distance to Closeout                |                      no |            no |               no |                      yes |                     no |
| minimum durability                              |                      no |           yes |               no |                       no |                     no |
| Journal Anchor requirements                     |                      no |            no |              yes |                       no |                     no |
| required Bootstrap Scope                        |                      no |            no |               no |                      yes |                     no |
| required formal classification                  |                      no |            no |               no |                      yes |                     no |
| Establishment diversity                         |                      no |            no |               no |                      yes |                     no |
| Closeout postconditions                         |                      no |            no |               no |    creation prerequisite |                    yes |

This table is normative.

---

# 31. REVIEW_REQUEST_CREATION Applicability

During Review Request creation, evaluate:

```text id="6ew7hu"
review_role
review_scope_ref
review_method_ref
required_checks_ref
```

against the selected Review Requirement.

Do NOT evaluate:

```text id="xfvusk"
required_count
required_method_status
allowed_finding_states
Journal Anchor relation
Review Result findings
```

because no Result or Admission exists yet.

---

# 32. Review Requirement Structure

```text id="ig0543"
0 = review_role_id
1 = review_scope_ref
2 = review_method_ref
3 = required_checks_ref
4 = required_count
```

All fields are required.

---

## 32.1 Review Requirement canonical order

Sort by:

```text id="5a26a1"
review_role_id
review_scope_ref
review_method_ref
required_checks_ref
required_count
```

Duplicates are invalid.

---

## 32.2 Exact selector uniqueness

v0.3 contains no Review selector wildcard.

Selector identity is the exact tuple:

```text id="00l25i"
(
  role,
  Scope,
  Method,
  CheckSet
)
```

Because duplicate Review Requirements are forbidden, exact matching can identify at most one distinct selector tuple.

If a future wildcard/range selector is introduced, this property MUST be reconsidered.

---

## 32.3 Admission use

Admission evaluates exact selector compatibility.

---

## 32.4 Closeout use

Closeout additionally evaluates:

```text id="ih63fc"
required_count
```

for every Review Requirement clause.

All clauses compose by AND.

---

## 32.5 Count is not independence

`required_count` proves only a count of distinct accepted Review Admission authorities.

It does not prove independent reviewers, processes, organizations, or failure modes.

---

# 33. Uniform Method Status and Finding State Policy

Policy fields:

```text id="hkoenu"
required_method_status

allowed_finding_states
```

are global v0.3 constraints across a closed list of applicable Evidence classes.

---

## 33.1 Closed applicable class list

The generic constraints apply ONLY to:

```text id="sbf3ki"
REVIEW_RESULT

VERIFICATION_RECORD

ASSUMPTION_ESTABLISHMENT
```

when those Records are consumed by an evaluation context where the corresponding Policy field is applicable.

No other Record type becomes subject to these generic constraints merely because it later gains a similarly named field.

---

## 33.2 Formal Verification exclusion

`FORMAL_VERIFICATION` uses:

```text id="b84m8p"
formal_outcome_id
```

not the generic Method/Finding pair.

Generic Policy status constraints MUST NOT be projected onto Formal Verification outcome.

---

## 33.3 Proof manifest exclusion

`PROOF_DEPENDENCY_MANIFEST.extractor_method_status` is not governed by generic Policy `required_method_status` in v0.3.

It requires an explicit future Policy rule if such gating is desired.

---

## 33.4 Uniform constraint

If the Policy requires:

```text id="0qposc"
required_method_status = { VALID }
```

every applicable Record from the closed class list consumed by that gate must satisfy the same set.

There is no v0.3 per-Evidence-class status syntax.

---

## 33.5 Future class-specific extension

A future successor MAY introduce:

```text id="dmxkkp"
required_method_status_by_evidence_class

allowed_finding_states_by_evidence_class
```

Predecessor v0.3 single-set semantics SHOULD remain representable as the same set applied to every applicable class.

---

# 34. Verification Requirement Structure

```text id="9wnsu9"
0 = verification_phase_id
1 = verification_target_type_id
2 = verification_authority_mode_id
3 = required_count
```

General Verification requirements allow:

```text id="7yf5sp"
CREATION_TIME
PRE
```

but not POST.

POST is represented through Closeout postconditions.

---

# 35. Intervening Event Constraint Structure

```text id="lnq01k"
0 = forbidden_event_type_ids
```

The set is sorted, unique, and evaluated over the exact authoritative observation interval.

No timestamp substitute is permitted.

---

# 36. Minimum Durability Requirement Structure

```text id="23ur6q"
0 = require_file_content_flush
1 = require_atomic_publish_no_replace
2 = require_parent_directory_flush
3 = require_platform_strongest_available
```

All values are Bool.

---

# 37. Journal Anchor Requirement Structure

```text id="3b7qq0"
0 = acceptable_anchor_relation_ids
```

Anchor Relation Registry:

```text id="xuexrd"
1 = ANCHOR_EQUALS_CURRENT_HEAD
2 = ANCHOR_IS_VALID_ANCESTOR
3 = JOURNAL_DIVERGENCE
4 = JOURNAL_HISTORY_BEHIND_ANCHOR
5 = ANCHOR_FROM_DIFFERENT_REGISTRY
6 = ANCHOR_INVALID
```

---

# 38. Required Formal Finding Classification Structure

```text id="2hutcp"
0 = required
1 = acceptable_classification_ids
2 = classifier_procedure_identity_allowed
```

If `required = false`, keys 1 and 2 MUST be absent.

If `required = true`, keys 1 and 2 MUST be nonempty.

---

# 39. Establishment Diversity Requirement Structure

```text id="412dte"
0 = dimension_id
1 = distinct_count
```

Initial dimensions:

```text id="azn2b3"
1 = IMPLEMENTATION_IDENTITY
2 = HOST_IDENTITY
```

Recorded diversity does not prove causal independence.

---

# 40. Closeout Postcondition Structure

v0.3 supports only:

```text id="htdp8k"
SOURCE_MATCH_BRACKET
```

Encoding:

```text id="kmd491"
0 = postcondition_type_id
1 = verification_authority_mode_id
2 = pre_target_type_id
3 = post_target_type_id
```

For this postcondition:

```text id="xklt88"
verification_authority_mode_id
    = AUTHORITATIVE_REQUIRED

pre_target_type_id
    = SOURCE_SUBJECT_MATCH

post_target_type_id
    = SOURCE_SUBJECT_MATCH
```

---

## 40.1 Creation-time effect

Closeout creation requires an exact authoritative PRE Verification.

---

## 40.2 Post-Closeout effect

The first structurally eligible POST Verification is decisive under the upstream bracket rules.

---

# 41. Policy Evaluator Registry

```text id="bo61wu"
1001 = POLICY_REVIEW_REQUIREMENT_MATCH

1002 = POLICY_REVIEW_REQUIRED_COUNT

1003 = POLICY_REQUIRED_METHOD_STATUS

1004 = POLICY_ALLOWED_FINDING_STATE

1005 = POLICY_VERIFICATION_REQUIREMENT

1006 = POLICY_INTERVENING_EVENT_CONSTRAINT

1007 = POLICY_MAX_JOURNAL_DISTANCE

1008 = POLICY_MINIMUM_DURABILITY

1009 = POLICY_JOURNAL_ANCHOR_REQUIREMENT

1010 = POLICY_BOOTSTRAP_SCOPE_REQUIREMENT

1011 = POLICY_FORMAL_FINDING_CLASSIFICATION_REQUIREMENT

1012 = POLICY_ESTABLISHMENT_DIVERSITY_REQUIREMENT

1013 = POLICY_CLOSEOUT_POSTCONDITION

1014 = POLICY_SUPPORTED_CONTEXT
```

Assignments MUST NOT be reused.

---

# 42. Policy Field to Evaluator Mapping

| Policy field              | Evaluator ID | Positive vector                 | Negative vector                    |
| ------------------------- | -----------: | ------------------------------- | ---------------------------------- |
| Review selector           |         1001 | `POL-REVIEW-MATCH-VALID-001`    | `POL-REVIEW-MATCH-ROLE-FAIL-001`   |
| Review count              |         1002 | `POL-REVIEW-COUNT-VALID-001`    | `POL-REVIEW-COUNT-LOW-001`         |
| `required_method_status`  |         1003 | `POL-METHOD-VALID-001`          | `POL-METHOD-STATUS-FAIL-001`       |
| `allowed_finding_states`  |         1004 | `POL-FINDING-VALID-001`         | `POL-FINDING-FAIL-001`             |
| Verification requirements |         1005 | `POL-VERIFY-VALID-001`          | `POL-VERIFY-TARGET-FAIL-001`       |
| intervening events        |         1006 | `POL-INTERVENING-NONE-001`      | `POL-INTERVENING-FORBIDDEN-001`    |
| journal distance          |         1007 | `POL-DISTANCE-BOUNDARY-001`     | `POL-DISTANCE-EXCEEDED-001`        |
| minimum durability        |         1008 | `POL-DURABILITY-VALID-001`      | `POL-DURABILITY-MISSING-001`       |
| Journal Anchor            |         1009 | `POL-ANCHOR-ANCESTOR-001`       | `POL-ANCHOR-DIVERGENCE-001`        |
| Bootstrap Scope           |         1010 | `POL-BOOTSTRAP-SCOPE-VALID-001` | `POL-BOOTSTRAP-SCOPE-MISMATCH-001` |
| formal classification     |         1011 | `POL-FORMAL-CLASS-VALID-001`    | `POL-FORMAL-CLASS-MISSING-001`     |
| Establishment diversity   |         1012 | `POL-DIVERSITY-VALID-001`       | `POL-DIVERSITY-COUNT-FAIL-001`     |
| Closeout postcondition    |         1013 | `POL-BRACKET-SATISFIED-001`     | `POL-BRACKET-FIRST-POST-FAIL-001`  |
| `supported_context_ids`   |         1014 | `POL-CONTEXT-SUPPORTED-001`     | `POL-CONTEXT-UNSUPPORTED-001`      |

---

# 43. Policy Evaluator Coverage CI

CI MUST verify:

```text id="tmtybv"
every normative Policy requirement has evaluator metadata

every evaluator exists

every evaluator has >= 1 positive vector

every evaluator has >= 1 negative vector

material failure branches have negative vectors

no normative Policy field is parsed but silently ignored

no evaluator invents an unexpressed Policy rule
```

---

# 44. Evaluator Minimum vs Branch Coverage

One positive and one negative vector per evaluator is only the minimum registration floor.

Evaluator 1001, for example, requires material negative coverage for:

```text id="vz2xjb"
role mismatch

Scope mismatch

Method mismatch

CheckSet mismatch
```

One evaluator may therefore own several negative vectors.

---

# 45. POLICY Record

Record Type:

```text id="eknr8h"
40 = POLICY
```

| Key | Field                                    | Type                         | Presence |
| --: | ---------------------------------------- | ---------------------------- | -------- |
|  16 | `gate_scope_ref`                         | RecordId `SCOPE`             | required |
|  17 | `review_requirements`                    | Review Requirement set       | optional |
|  18 | `required_method_status`                 | sorted UInt set              | optional |
|  19 | `allowed_finding_states`                 | sorted UInt set              | optional |
|  20 | `verification_requirements`              | Verification Requirement set | optional |
|  21 | `intervening_event_constraints`          | embedded map                 | optional |
|  22 | `max_journal_distance_to_closeout`       | UInt                         | optional |
|  23 | `minimum_durability`                     | embedded map                 | optional |
|  24 | `journal_anchor_requirements`            | embedded map                 | optional |
|  25 | `required_bootstrap_scope_ref`           | RecordId `SCOPE`             | optional |
|  26 | `required_formal_finding_classification` | embedded map                 | optional |
|  27 | `required_establishment_diversity`       | diversity requirement set    | optional |
|  28 | `closeout_postconditions`                | embedded map                 | optional |
|  29 | `operation_start_journal_ref`            | JournalReference             | required |
|  30 | `supported_context_ids`                  | sorted UInt set              | required |

`supported_context_ids` MUST be nonempty.

---

## 45.1 Policy registration chronology

`operation_start_journal_ref` records the Policy registration operation's chronology.

It is not itself a Policy evaluation context.

The Policy may be registered without evaluating itself.

---

## 45.2 Requirement coverage

Keys 17–28 implement the supported upstream Policy requirement classes.

A missing upstream requirement class is a schema defect.

---

## 45.3 No empty present requirements

The following MUST be nonempty if present:

```text id="dwbg3n"
review_requirements
required_method_status
allowed_finding_states
verification_requirements
required_establishment_diversity
closeout_postconditions
```

---

# 46. Policy Applicability Algorithm

```text id="1k9m1a"
EvaluatePolicy(P, Context, Evidence):

    load exact POLICY

    validate identity

    validate authority where the operation requires
    authoritative Policy

    if Context not in P.supported_context_ids:
        return POLICY_CONTEXT_UNSUPPORTED

    validate context-specific structural obligations

    determine requirements applicable to Context

    for each applicable requirement:
        invoke registered evaluator

        FAIL
            → GATE_UNSATISFIED

        INDETERMINATE
            → GATE_INDETERMINATE

    return SATISFIED
```

A context not declared by the Policy MUST fail before ordinary requirement evaluation begins.

---

# 47. SCOPE Record

Record Type:

```text id="nk5706"
20 = SCOPE
```

| Key | Field                   | Type | Presence |
| --: | ----------------------- | ---- | -------- |
|  16 | `scope_profile_id`      | UInt | required |
|  17 | `scope_profile_version` | UInt | required |
|  18 | `scope_payload`         | bstr | required |
|  19 | `scope_label`           | Text | optional |

---

# 48. METHOD Record

Record Type:

```text id="blmmjx"
21 = METHOD
```

| Key | Field                    | Type | Presence |
| --: | ------------------------ | ---- | -------- |
|  16 | `method_profile_id`      | UInt | required |
|  17 | `method_profile_version` | UInt | required |
|  18 | `method_payload`         | bstr | required |
|  19 | `method_label`           | Text | optional |

---

# 49. CHECK Record

Record Type:

```text id="wjfrgv"
22 = CHECK
```

| Key | Field                   | Type | Presence |
| --: | ----------------------- | ---- | -------- |
|  16 | `check_profile_id`      | UInt | required |
|  17 | `check_profile_version` | UInt | required |
|  18 | `check_payload`         | bstr | required |
|  19 | `check_label`           | Text | optional |

---

# 50. CHECK_SET Record

Record Type:

```text id="nvfht0"
23 = CHECK_SET
```

| Key | Field        | Type                       | Presence |
| --: | ------------ | -------------------------- | -------- |
|  16 | `check_refs` | sorted unique RecordId set | required |

The set MUST be nonempty.

Every referenced Record MUST be type `CHECK`.

---

# 51. GENESIS Record

Record Type:

```text id="6capcq"
1 = GENESIS
```

| Key | Field                         | Type       | Presence |
| --: | ----------------------------- | ---------- | -------- |
|  16 | `registry_id`                 | RegistryId | required |
|  17 | `journal_format_version`      | UInt       | required |
|  18 | `record_identity_profile_id`  | UInt       | required |
|  19 | `storage_capability_class_id` | RecordId   | required |
|  20 | `environment_observation_id`  | RecordId   | required |
|  21 | `created_by_tool_version`     | Text       | required |

Journal GENESIS:

```text id="pgsrwf"
registry_id

storage_capability_class_id

environment_observation_id
```

MUST match this Record exactly.

---

# 52. MANIFEST Record

Record Type:

```text id="wy4x70"
2 = MANIFEST
```

| Key | Field                      | Type                 | Presence |
| --: | -------------------------- | -------------------- | -------- |
|  16 | `subject_id`               | OpaqueId32           | required |
|  17 | `artifact_count`           | UInt                 | required |
|  18 | `path_identity_profile_id` | UInt                 | required |
|  19 | `digest_profile_id`        | UInt                 | required |
|  20 | `artifacts`                | Artifact Entry array | required |

`artifact_count` MUST equal the actual array length.

---

# 53. Artifact Entry Structure

```text id="e0n14f"
[
  artifact_kind_id,
  path_components,
  size_bytes,
  digest_algorithm_id,
  digest_bytes
]
```

`path_components` is `CanonicalPath`.

Directories are not Manifest Artifacts in v0.x.

---

# 54. Manifest Artifact Ordering

`artifacts[]` uses the Lifecycle canonical path comparator.

Duplicate canonical paths are invalid.

Therefore:

```text id="p0dq71"
(path_identity_profile_id, canonical path)
```

identifies at most one Artifact Entry in one Manifest.

---

# 55. FREEZE_ATTEMPT_START Record

Record Type:

```text id="gy1j46"
3 = FREEZE_ATTEMPT_START
```

| Key | Field               | Type              | Presence |
| --: | ------------------- | ----------------- | -------- |
|  16 | `freeze_attempt_id` | OpaqueId32        | required |
|  17 | `intended_root_id`  | OpaqueId32        | required |
|  18 | `subject_id`        | OpaqueId32        | required |
|  19 | `policy_record_id`  | RecordId `POLICY` | required |

`intended_root_id` MUST satisfy Identity Format derivation.

---

## 55.1 Policy role

`policy_record_id` prospectively binds the exact Policy identity intended to govern the later COMMIT.

It does not itself assert Policy authority.

---

## 55.2 Supported context requirement

The bound Policy Record MUST declare:

```text id="b5eizl"
FREEZE_COMMIT
```

in `supported_context_ids`.

---

# 56. FREEZE_RECEIPT Record

Record Type:

```text id="0bfugm"
4 = FREEZE_RECEIPT
```

| Key | Field                             | Type                | Presence |
| --: | --------------------------------- | ------------------- | -------- |
|  16 | `freeze_attempt_id`               | OpaqueId32          | required |
|  17 | `freeze_id`                       | OpaqueId32          | required |
|  18 | `attempt_start_journal_ref`       | JournalReference    | required |
|  19 | `subject_id`                      | OpaqueId32          | required |
|  20 | `manifest_id`                     | RecordId `MANIFEST` | required |
|  21 | `custody_mode_id`                 | UInt                | required |
|  22 | `creation_profile_ref`            | RecordId            | required |
|  23 | `path_identity_profile_id`        | UInt                | required |
|  24 | `filesystem_profile_ref`          | RecordId            | required |
|  25 | `policy_record_id`                | RecordId `POLICY`   | required |
|  26 | `file_content_flush_state`        | UInt                | required |
|  27 | `atomic_publish_no_replace_state` | UInt                | required |
|  28 | `parent_directory_flush_state`    | UInt                | required |
|  29 | `platform_strongest_available`    | Bool                | required |
|  30 | `requested_commit_durability_ref` | RecordId            | optional |
|  31 | `created_by_tool_version`         | Text                | required |

---

# 57. Freeze Policy Continuity

Required:

```text id="4yy7af"
FREEZE_RECEIPT.policy_record_id
==
FREEZE_ATTEMPT_START.policy_record_id
```

No substitute Policy is permitted.

Semantic equivalence, stricter requirements, or later registration do not permit identity substitution.

---

# 58. Freeze Subject and Manifest Continuity

For a committed Freeze:

```text id="56yxug"
FREEZE_ATTEMPT_START.subject_id
==
FREEZE_RECEIPT.subject_id
==
MANIFEST.subject_id
```

MUST hold.

Additionally:

```text id="vlviic"
FREEZE_RECEIPT.path_identity_profile_id
==
MANIFEST.path_identity_profile_id
```

MUST hold.

A Receipt MUST NOT commit a Manifest for a different Subject or path identity interpretation than the bound Freeze Attempt.

---

# 59. FREEZE_ABORT_RECOVERY Record

Record Type:

```text id="6v0pm8"
5 = FREEZE_ABORT_RECOVERY
```

| Key | Field                         | Type             | Presence |
| --: | ----------------------------- | ---------------- | -------- |
|  16 | `freeze_attempt_id`           | OpaqueId32       | required |
|  17 | `attempt_start_journal_ref`   | JournalReference | required |
|  18 | `recovery_observation_ref`    | RecordId         | optional |
|  19 | `reason_codes`                | sorted Text set  | required |
|  20 | `operation_start_journal_ref` | JournalReference | required |

---

# 60. OPERATOR_ABORT_ASSERTION Record

Record Type:

```text id="8ri4v5"
6 = OPERATOR_ABORT_ASSERTION
```

| Key | Field                         | Type             | Presence |
| --: | ----------------------------- | ---------------- | -------- |
|  16 | `freeze_attempt_id`           | OpaqueId32       | required |
|  17 | `attempt_start_journal_ref`   | JournalReference | required |
|  18 | `prior_liveness_state`        | Text             | required |
|  19 | `operator_assertion`          | Text             | required |
|  20 | `operator_metadata`           | Text             | optional |
|  21 | `reason`                      | Text             | required |
|  22 | `operation_start_journal_ref` | JournalReference | required |

`prior_liveness_state` MUST equal:

```text id="ms53d3"
UNKNOWN
```

in v0.x.

---

# 61. FREEZE_COMMIT_REJECTION Record

Record Type:

```text id="jx7f93"
7 = FREEZE_COMMIT_REJECTION
```

| Key | Field                                | Type             | Presence |
| --: | ------------------------------------ | ---------------- | -------- |
|  16 | `freeze_attempt_id`                  | OpaqueId32       | required |
|  17 | `attempt_start_journal_ref`          | JournalReference | required |
|  18 | `conflicting_terminal_journal_ref`   | JournalReference | required |
|  19 | `attempted_transition_event_type_id` | UInt             | required |
|  20 | `rejection_reason`                   | Text             | required |

v0.x returning-writer reason:

```text id="zqxvwp"
ATTEMPT_ALREADY_TERMINAL
```

---

# 62. SOURCE_BINDING Record

Record Type:

```text id="c5bh1n"
10 = SOURCE_BINDING
```

| Key | Field                            | Type       | Presence |
| --: | -------------------------------- | ---------- | -------- |
|  16 | `freeze_attempt_id`              | OpaqueId32 | required |
|  17 | `source_locator_hint`            | Text       | optional |
|  18 | `initial_source_object_identity` | bstr       | required |
|  19 | `path_identity_profile_id`       | UInt       | required |

Locator hint is provenance only.

---

# 63. VERIFICATION_OBSERVATION Record

Record Type:

```text id="wquv13"
11 = VERIFICATION_OBSERVATION
```

Portable and nonauthoritative.

| Key | Field                             | Type                | Presence         |
| --: | --------------------------------- | ------------------- | ---------------- |
|  16 | `registry_id`                     | RegistryId          | required         |
|  17 | `verification_target_type_id`     | UInt                | required         |
|  18 | `freeze_authority_ref`            | JournalReference    | required         |
|  19 | `source_binding_ref`              | RecordId            | context-required |
|  20 | `manifest_id`                     | RecordId `MANIFEST` | required         |
|  21 | `examined_source_object_identity` | bstr                | required         |
|  22 | `examined_source_locator`         | Text                | optional         |
|  23 | `path_profile_interpretation`     | UInt                | required         |
|  24 | `method_status`                   | UInt                | required         |
|  25 | `finding_state`                   | UInt                | required         |
|  26 | `reason_codes`                    | sorted Text set     | required         |
|  27 | `coverage_limitations`            | sorted Text set     | required         |
|  28 | `storage_capability_class_id`     | RecordId            | required         |
|  29 | `environment_observation_id`      | RecordId            | required         |
|  30 | `created_by_tool_version`         | Text                | required         |

It MUST NOT contain:

```text id="69pg3z"
observation_start_journal_ref

brackets_closeout_authority_ref
```

---

## 63.1 Why the chronology field is absent

A portable Observation has no authoritative Registry observation interval.

The absence is intentional and structurally separates it from `VERIFICATION_RECORD`.

This structural difference reinforces the prohibition on converting a portable Observation into an authoritative Verification merely by changing Record type and re-hashing.

---

# 64. VERIFICATION_RECORD Record

Record Type:

```text id="15nr71"
12 = VERIFICATION_RECORD
```

| Key | Field                             | Type                      | Presence         |
| --: | --------------------------------- | ------------------------- | ---------------- |
|  16 | `registry_id`                     | RegistryId                | required         |
|  17 | `verification_target_type_id`     | UInt                      | required         |
|  18 | `observation_start_journal_ref`   | JournalReference          | required         |
|  19 | `freeze_authority_ref`            | JournalReference          | required         |
|  20 | `source_binding_ref`              | RecordId `SOURCE_BINDING` | context-required |
|  21 | `manifest_id`                     | RecordId `MANIFEST`       | required         |
|  22 | `examined_source_object_identity` | bstr                      | required         |
|  23 | `examined_source_locator`         | Text                      | optional         |
|  24 | `path_profile_interpretation`     | UInt                      | required         |
|  25 | `method_status`                   | UInt                      | required         |
|  26 | `finding_state`                   | UInt                      | required         |
|  27 | `reason_codes`                    | sorted Text set           | required         |
|  28 | `coverage_limitations`            | sorted Text set           | required         |
|  29 | `storage_capability_class_id`     | RecordId                  | required         |
|  30 | `environment_observation_id`      | RecordId                  | required         |
|  31 | `created_by_tool_version`         | Text                      | required         |
|  32 | `brackets_closeout_authority_ref` | JournalReference          | context-required |

---

## 64.1 Source Binding symmetry

For `SOURCE_SUBJECT_MATCH`:

```text id="wrdmjv"
source_binding_ref MUST be present
```

For `EMBEDDED_COPY_INTEGRITY`:

```text id="75hcc1"
source_binding_ref MUST be absent
```

unless an explicit successor states otherwise.

---

## 64.2 POST symmetry

POST Verification:

```text id="e8p9kh"
brackets_closeout_authority_ref present
```

Ordinary Verification:

```text id="ncv7x8"
brackets_closeout_authority_ref absent
```

The Record and Journal copies MUST match exactly.

---

# 65. REVIEW_REQUEST Record

Record Type:

```text id="e5jbh7"
30 = REVIEW_REQUEST
```

| Key | Field                         | Type                 | Presence |
| --: | ----------------------------- | -------------------- | -------- |
|  16 | `freeze_authority_ref`        | JournalReference     | required |
|  17 | `manifest_id`                 | RecordId `MANIFEST`  | required |
|  18 | `review_role_id`              | UInt                 | required |
|  19 | `required_checks_ref`         | RecordId `CHECK_SET` | required |
|  20 | `policy_authority_ref`        | JournalReference     | required |
|  21 | `review_scope_ref`            | RecordId `SCOPE`     | required |
|  22 | `review_method_ref`           | RecordId `METHOD`    | required |
|  23 | `review_package_anchor_id`    | OpaqueId32           | required |
|  24 | `operation_start_journal_ref` | JournalReference     | required |

---

## 65.1 Policy authority

`policy_authority_ref` MUST identify exact event:

```text id="gazqn0"
400 POLICY_RECORDED
```

and the corresponding Policy MUST support:

```text id="t8o89n"
REVIEW_REQUEST_CREATION
```

---

## 65.2 Request selector evaluation

Before authority is recorded, evaluator 1001 checks:

```text id="43lpyh"
role
Scope
Method
CheckSet
```

No Result/count evaluation occurs yet.

---

# 66. REVIEW_RESULT Record

Record Type:

```text id="9vhv6p"
31 = REVIEW_RESULT
```

| Key | Field                          | Type                | Presence |
| --: | ------------------------------ | ------------------- | -------- |
|  16 | `review_request_authority_ref` | JournalReference    | required |
|  17 | `freeze_authority_ref`         | JournalReference    | required |
|  18 | `manifest_id`                  | RecordId `MANIFEST` | required |
|  19 | `review_role_id`               | UInt                | required |
|  20 | `review_scope_ref`             | RecordId `SCOPE`    | required |
|  21 | `review_method_ref`            | RecordId `METHOD`   | required |
|  22 | `method_status`                | UInt                | required |
|  23 | `finding_state`                | UInt                | required |
|  24 | `reason_codes`                 | sorted Text set     | required |
|  25 | `findings`                     | RecordId array      | required |
|  26 | `reviewer_metadata`            | Text                | optional |
|  27 | `review_package_anchor_id`     | OpaqueId32          | required |
|  28 | `operation_start_journal_ref`  | JournalReference    | required |

`review_request_authority_ref` MUST identify exact event 300.

All redundant Request bindings MUST match.

---

# 67. REVIEW_ADMISSION Record

Record Type:

```text id="k6rn33"
32 = REVIEW_ADMISSION
```

| Key | Field                           | Type                          | Presence          |
| --: | ------------------------------- | ----------------------------- | ----------------- |
|  16 | `disposition_id`                | UInt                          | required          |
|  17 | `review_request_ref`            | JournalReference              | context-dependent |
|  18 | `review_result_ref`             | JournalReference              | context-dependent |
|  19 | `policy_authority_ref`          | JournalReference              | context-dependent |
|  20 | `failed_candidate_record_refs`  | FailedCandidateRecordRef set  | optional          |
|  21 | `failed_candidate_journal_refs` | FailedCandidateJournalRef set | optional          |
|  22 | `reason_codes`                  | sorted Text set               | required          |
|  23 | `operation_start_journal_ref`   | JournalReference              | required          |

---

## 67.1 ACCEPTED

Required:

```text id="9gxgvq"
review_request_ref
review_result_ref
policy_authority_ref
```

Failed-candidate fields MUST be absent.

Policy MUST support:

```text id="pgp52h"
REVIEW_ADMISSION
```

---

## 67.2 REJECTED

Successfully resolved authorities are recorded by JournalReference.

Known failed Record candidates use:

```text id="igbkan"
failed_candidate_record_refs
```

Known structurally valid but context-invalid Journal candidates use:

```text id="4kwk8s"
failed_candidate_journal_refs
```

Unknown identity MUST NOT be invented.

Only Candidate Roles permitted by §18.1 are legal.

---

# 68. CLOSEOUT Record

Record Type:

```text id="adwf2w"
50 = CLOSEOUT
```

| Key | Field                                          | Type                          | Presence                 |
| --: | ---------------------------------------------- | ----------------------------- | ------------------------ |
|  16 | `disposition_id`                               | UInt                          | required                 |
|  17 | `freeze_authority_ref`                         | JournalReference              | required                 |
|  18 | `manifest_id`                                  | RecordId `MANIFEST`           | required                 |
|  19 | `admitted_review_authority_refs`               | JournalReference set          | required, `[]` permitted |
|  20 | `creation_time_verification_authority_refs`    | JournalReference set          | required, `[]` permitted |
|  21 | `pre_verification_authority_ref`               | JournalReference              | context-required         |
|  22 | `policy_authority_ref`                         | JournalReference              | required                 |
|  23 | `required_bootstrap_scope_ref`                 | RecordId `SCOPE`              | context-required         |
|  24 | `predecessor_closeout_id`                      | RecordId `CLOSEOUT`           | optional                 |
|  25 | `formal_support_binding`                       | embedded map                  | context-required         |
|  26 | `bootstrap_trust_declaration_authority_refs`   | JournalReference set          | context-required         |
|  27 | `formal_finding_classification_authority_refs` | JournalReference set          | context-required         |
|  28 | `failed_candidate_record_refs`                 | FailedCandidateRecordRef set  | optional                 |
|  29 | `failed_candidate_journal_refs`                | FailedCandidateJournalRef set | optional                 |
|  30 | `reason_codes`                                 | sorted Text set               | required                 |
|  31 | `operation_start_journal_ref`                  | JournalReference              | required                 |

---

# 69. Closeout Context Identity

An authoritative Closeout, including REJECTED, requires exact:

```text id="q29w8l"
freeze_authority_ref

policy_authority_ref

manifest_id
```

Without exact Freeze and Policy authority, the attempted Closeout qualification context is not sufficiently identified for event 500/501 authority.

Such failure remains before authoritative Closeout disposition.

---

## 69.1 Manifest derivation

`manifest_id` MUST equal the Manifest bound by the authoritative Freeze Receipt identified through `freeze_authority_ref`.

The Closeout MUST NOT substitute another Manifest.

---

## 69.2 Policy support

The Policy MUST declare:

```text id="uq9vyd"
CLOSEOUT_CREATION
```

in `supported_context_ids`.

---

# 70. Closeout Always-Present Support Collections

Keys:

```text id="b7xu0w"
19 admitted_review_authority_refs

20 creation_time_verification_authority_refs
```

are always present.

If no such authority was resolved or required:

```text id="ywj1gb"
[]
```

is the canonical representation.

Absence and empty are not interchangeable.

---

## 70.1 REJECTED Closeout

If required Review or Verification candidates could not be resolved, keys 19/20 contain only successfully resolved authorities.

Failed candidates are represented separately in keys 28/29.

---

## 70.2 COMMITTED Closeout

The collections must contain enough exact authorities to satisfy every applicable Policy requirement.

An empty array is valid only where Policy does not require that support class.

---

# 71. Closeout Failed Candidate Roles

`failed_candidate_record_refs` and `failed_candidate_journal_refs` may use only roles permitted by §18.2.

Typical examples:

```text id="0f4oh4"
MANIFEST

ADMITTED_REVIEW

CREATION_VERIFICATION

PRE_VERIFICATION

PREDECESSOR_CLOSEOUT

FORMAL_VERIFICATION

ASSUMPTION_ESTABLISHMENT
```

Freeze and Policy are mandatory Closeout context authorities and therefore cannot be relegated to failed-candidate fields.

---

## 71.1 Predecessor Closeout not requested

If no predecessor Closeout was intended for this Closeout attempt:

```text id="bjip3m"
predecessor_closeout_id MUST be absent

and

no FailedCandidateRecordRef with
candidate_role_id = PREDECESSOR_CLOSEOUT
may be present
```

Absence means:

```text id="mqlos2"
no predecessor was requested
```

---

## 71.2 Predecessor Closeout requested and resolved

If a predecessor Closeout was intended and its exact Record identity was successfully resolved:

```text id="x8852h"
predecessor_closeout_id MUST be present
```

with that exact RecordId.

A corresponding predecessor authority dependency MUST additionally be present where required by the controlling lifecycle relation.

---

## 71.3 Predecessor Closeout requested but unresolved

If a predecessor Closeout was intended but its exact candidate Record could not satisfy the required resolution or authority relation:

```text id="4iqxm9"
predecessor_closeout_id MUST be absent
```

and the failed candidate MUST be represented, where its exact RecordId is reconstructable, as:

```text id="v97f18"
FailedCandidateRecordRef {
    candidate_role_id = PREDECESSOR_CLOSEOUT,
    record_id = exact candidate RecordId,
    validation_failure_code_id = exact registered failure
}
```

Thus:

```text id="j3dpv0"
key 24 absent
+
no PREDECESSOR_CLOSEOUT failure
```

means:

```text id="2j8x8z"
no predecessor requested
```

while:

```text id="1b82rk"
key 24 absent
+
PREDECESSOR_CLOSEOUT failed candidate present
```

means:

```text id="xnvxhz"
predecessor intended but unresolved
```

The implementation MUST NOT collapse these states.

---

# 72. Closeout Bootstrap Symmetry

If Policy requires Bootstrap Trust:

```text id="xcs7z1"
required_bootstrap_scope_ref present

bootstrap_trust_declaration_authority_refs present
```

Otherwise both are absent unless another explicit support rule requires the authority set.

The Scope value MUST equal the Policy requirement exactly.

---

# 73. Closeout PRE Symmetry

If Policy requires `SOURCE_MATCH_BRACKET`:

```text id="0tj7sg"
pre_verification_authority_ref present
```

Otherwise it is absent.

If postconditions are present, Policy MUST support both:

```text id="bsvbm0"
CLOSEOUT_CREATION

CLOSEOUT_POSTCONDITION
```

---

# 74. Formal Support Binding Structure

```text id="4crsvg"
0 = formal_verification_authority_refs

1 = assumption_bindings
```

Each Assumption Binding:

```text id="krlgda"
0 = required_assumption_definition_authority_ref

1 = establishment_authority_ref

2 = compatibility_authority_ref
```

---

## 74.1 Compatibility symmetry

Same Definition:

```text id="u8y7n6"
compatibility_authority_ref absent
```

Different Definition:

```text id="087wk3"
compatibility_authority_ref required
```

No implicit compatibility exists.

---

# 75. STORAGE_CAPABILITY_CLASS Record

Record Type:

```text id="acnf7l"
60 = STORAGE_CAPABILITY_CLASS
```

| Key | Field                               | Type |
| --: | ----------------------------------- | ---- |
|  16 | `filesystem_transport`              | Text |
|  17 | `sync_management`                   | Text |
|  18 | `placeholder_capability`            | UInt |
|  19 | `exclusive_create_capability`       | UInt |
|  20 | `no_replace_publication_capability` | UInt |
|  21 | `locking_capability`                | UInt |
|  22 | `atomic_rename_capability`          | UInt |
|  23 | `file_flush_capability`             | UInt |
|  24 | `directory_flush_capability`        | UInt |

All fields are required.

Capability values use PRESENT / ABSENT / NOT_PROBED.

---

# 76. ENVIRONMENT_OBSERVATION Record

Record Type:

```text id="1th0mz"
61 = ENVIRONMENT_OBSERVATION
```

| Key | Field                                | Type            | Presence |
| --: | ------------------------------------ | --------------- | -------- |
|  16 | `os_name`                            | Text            | required |
|  17 | `os_version`                         | Text            | optional |
|  18 | `filesystem_reported_name`           | Text            | optional |
|  19 | `driver_details`                     | Text            | optional |
|  20 | `mount_identity`                     | bstr            | optional |
|  21 | `volume_identity`                    | bstr            | optional |
|  22 | `resolved_registry_storage_identity` | bstr            | optional |
|  23 | `probe_tool_version`                 | Text            | required |
|  24 | `observation_limitations`            | sorted Text set | required |

---

# 77. CAPABILITY_OBSERVATION_PROVENANCE Record

Record Type:

```text id="x2erx5"
63 = CAPABILITY_OBSERVATION_PROVENANCE
```

| Key | Field                                | Type     | Presence |
| --: | ------------------------------------ | -------- | -------- |
|  16 | `storage_capability_class_id`        | RecordId | required |
|  17 | `environment_observation_id`         | RecordId | required |
|  18 | `cache_reused`                       | Bool     | required |
|  19 | `process_scope_identity`             | bstr     | required |
|  20 | `mount_identity`                     | bstr     | optional |
|  21 | `volume_identity`                    | bstr     | optional |
|  22 | `resolved_registry_storage_identity` | bstr     | optional |
|  23 | `cache_scope_description`            | Text     | optional |

---

# 78. STORAGE_CAPABILITY_CHANGE Record

Record Type:

```text id="3pwo68"
62 = STORAGE_CAPABILITY_CHANGE
```

| Key | Field                                        | Type             | Presence |
| --: | -------------------------------------------- | ---------------- | -------- |
|  16 | `prior_storage_capability_class_id`          | RecordId         | required |
|  17 | `new_storage_capability_class_id`            | RecordId         | required |
|  18 | `prior_environment_observation_id`           | RecordId         | required |
|  19 | `new_environment_observation_id`             | RecordId         | required |
|  20 | `material_transition_capability_ids`         | sorted UInt set  | required |
|  21 | `observation_strength_change_capability_ids` | sorted UInt set  | required |
|  22 | `operation_start_journal_ref`                | JournalReference | required |

Material transitions contain only proven:

```text id="t9iruj"
PRESENT ↔ ABSENT
```

changes.

---

# 79. ArtifactPathSelector

An ArtifactPathSelector is exactly one:

```text id="celfqi"
CanonicalPath
```

under one exact `path_identity_profile_id`.

It does not duplicate:

```text id="m871o5"
digest
kind
size
Manifest entry index
```

---

# 80. ArtifactPathSelector Canonical Order

Selector sets use the Lifecycle canonical path comparator.

Duplicate canonical paths are invalid.

Caller declaration order is not authoritative.

---

# 81. EVICTION_SCOPE Record

Record Type:

```text id="d8i0sz"
70 = EVICTION_SCOPE
```

| Key | Field                      | Type              | Presence         |
| --: | -------------------------- | ----------------- | ---------------- |
|  16 | `scope_mode`               | UInt              | required         |
|  17 | `path_identity_profile_id` | UInt              | required         |
|  18 | `artifact_path_selectors`  | CanonicalPath set | context-required |

Modes:

```text id="3t7fgy"
1 = ALL_EMBEDDED_ARTIFACT_BYTES

2 = EXPLICIT_ARTIFACT_SET
```

---

## 81.1 ALL mode

`path_identity_profile_id` remains required.

`artifact_path_selectors` MUST be absent.

---

## 81.2 EXPLICIT mode

`artifact_path_selectors` MUST be present and nonempty.

---

## 81.3 Self-description

EVICTION_SCOPE carries its own path profile so that the portable Record does not change interpretation depending on which Manifest later consumes it.

---

# 82. Manifest-relative Eviction Scope Validation

Given:

```text id="kzq4l6"
pre_eviction_manifest_id

eviction_scope_ref
```

load exact MANIFEST and EVICTION_SCOPE.

Required:

```text id="j2psmt"
scope.path_identity_profile_id
==
manifest.path_identity_profile_id
```

No cross-profile conversion is allowed.

For each explicit selector:

```text id="bwaaz6"
exactly one Manifest Artifact Entry
must have equal path_components
```

Missing selector is invalid.

Manifest remains sole authority for digest, kind, and size.

---

# 83. ARTIFACT_EVICTION_START Record

Record Type:

```text id="9gl2uc"
71 = ARTIFACT_EVICTION_START
```

| Key | Field                         | Type                      | Presence |
| --: | ----------------------------- | ------------------------- | -------- |
|  16 | `eviction_attempt_id`         | OpaqueId32                | required |
|  17 | `freeze_authority_ref`        | JournalReference          | required |
|  18 | `pre_eviction_manifest_id`    | RecordId `MANIFEST`       | required |
|  19 | `eviction_scope_ref`          | RecordId `EVICTION_SCOPE` | required |
|  20 | `operation_start_journal_ref` | JournalReference          | required |

Scope validation MUST complete before destructive work begins.

The presence of `operation_start_journal_ref` is intentional under §12.3.

---

# 84. ARTIFACT_EVICTION_TERMINAL Record

Record Type:

```text id="0qnlmg"
72 = ARTIFACT_EVICTION_TERMINAL
```

| Key | Field                        | Type             | Presence         |
| --: | ---------------------------- | ---------------- | ---------------- |
|  16 | `eviction_attempt_id`        | OpaqueId32       | required         |
|  17 | `eviction_start_journal_ref` | JournalReference | required         |
|  18 | `terminal_disposition_id`    | UInt             | required         |
|  19 | `observed_eviction_state_id` | UInt             | context-required |
|  20 | `reason_codes`               | sorted Text set  | required         |

COMMITTED:

```text id="w56ita"
observed_eviction_state_id absent
```

INTERRUPTED_RECOVERY:

```text id="ecr98e"
observed_eviction_state_id present
```

BLOCKED:

```text id="583ari"
observed_eviction_state_id absent
```

BLOCKED is valid only before deliberate target deletion begins.

---

# 85. ASSUMPTION_DEFINITION Record

Record Type:

```text id="b8kp2t"
80 = ASSUMPTION_DEFINITION
```

| Key | Field                         | Type             |
| --: | ----------------------------- | ---------------- |
|  16 | `assumption_symbol`           | Text             |
|  17 | `assumption_version`          | UInt             |
|  18 | `assumption_category`         | Text             |
|  19 | `scope_ref`                   | RecordId `SCOPE` |
|  20 | `definition_semantics_ref`    | RecordId         |
|  21 | `operation_start_journal_ref` | JournalReference |

All fields required.

---

# 86. Diversity Identity Structure

```text id="trf0el"
[
  dimension_id,
  identity_bytes
]
```

Canonical order:

```text id="c5pygh"
dimension_id
then identity_bytes
```

---

# 87. ASSUMPTION_ESTABLISHMENT Record

Record Type:

```text id="ubdlou"
81 = ASSUMPTION_ESTABLISHMENT
```

| Key | Field                                   | Type                   | Presence         |
| --: | --------------------------------------- | ---------------------- | ---------------- |
|  16 | `assumption_definition_authority_ref`   | JournalReference       | required         |
|  17 | `scope_ref`                             | RecordId `SCOPE`       | required         |
|  18 | `method_ref`                            | RecordId `METHOD`      | required         |
|  19 | `method_status`                         | UInt                   | required         |
|  20 | `finding_state`                         | UInt                   | required         |
|  21 | `evidence_refs`                         | sorted RecordId set    | required         |
|  22 | `storage_capability_class_id`           | RecordId               | required         |
|  23 | `environment_observation_id`            | RecordId               | required         |
|  24 | `capability_epoch_ref`                  | JournalReference       | required         |
|  25 | `capability_observation_provenance_ref` | RecordId               | context-required |
|  26 | `diversity_identities`                  | diversity identity set | required         |
|  27 | `shared_dependency_manifest_ref`        | RecordId               | optional         |
|  28 | `observation_start_journal_ref`         | JournalReference       | required         |
|  29 | `reason_codes`                          | sorted Text set        | required         |

Cached capability Evidence requires key 25.

Fresh noncached capability Evidence forbids key 25.

---

# 88. ASSUMPTION_INVALIDATION Record

Record Type:

```text id="du5buq"
82 = ASSUMPTION_INVALIDATION
```

| Key | Field                                   | Type                 | Presence         |
| --: | --------------------------------------- | -------------------- | ---------------- |
|  16 | `target_mode_id`                        | UInt                 | required         |
|  17 | `explicit_establishment_authority_refs` | JournalReference set | context-required |
|  18 | `attribute_predicate_ref`               | RecordId             | context-required |
|  19 | `method_ref`                            | RecordId `METHOD`    | required         |
|  20 | `evidence_refs`                         | sorted RecordId set  | required         |
|  21 | `reason_codes`                          | sorted Text set      | required         |
|  22 | `operation_start_journal_ref`           | JournalReference     | required         |

EXPLICIT_SET:

```text id="0x6y6e"
17 present
18 absent
```

ATTRIBUTE_PREDICATE:

```text id="i1dr29"
17 absent
18 present
```

---

# 89. FORMAL_VERIFICATION Record

Record Type:

```text id="alr9jp"
83 = FORMAL_VERIFICATION
```

| Key | Field                                  | Type                 | Presence         |
| --: | -------------------------------------- | -------------------- | ---------------- |
|  16 | `formal_subject_identity`              | OpaqueId32           | required         |
|  17 | `formal_model_ref`                     | RecordId             | required         |
|  18 | `theorem_ref`                          | RecordId             | required         |
|  19 | `proof_dependency_manifest_ref`        | RecordId             | required         |
|  20 | `dafny_identity_ref`                   | RecordId             | required         |
|  21 | `solver_identity_ref`                  | RecordId             | required         |
|  22 | `formal_outcome_id`                    | UInt                 | required         |
|  23 | `counterexample_ref`                   | RecordId             | context-required |
|  24 | `raw_tool_output_ref`                  | RecordId             | required         |
|  25 | `assumption_definition_authority_refs` | JournalReference set | required         |
|  26 | `freeze_authority_ref`                 | JournalReference     | context-required |
|  27 | `shared_dependency_manifest_ref`       | RecordId             | optional         |
|  28 | `operation_start_journal_ref`          | JournalReference     | required         |
|  29 | `reason_codes`                         | sorted Text set      | required         |

COUNTEREXAMPLE requires `counterexample_ref`.

Other outcomes forbid it.

Formal outcome is distinct from generic Method/Finding status and from human Classification.

---

# 90. ASSUMPTION_VERSION_COMPATIBILITY Record

Record Type:

```text id="mqx8t2"
84 = ASSUMPTION_VERSION_COMPATIBILITY
```

| Key | Field                             | Type                |
| --: | --------------------------------- | ------------------- |
|  16 | `source_definition_authority_ref` | JournalReference    |
|  17 | `target_definition_authority_ref` | JournalReference    |
|  18 | `scope_mapping_ref`               | RecordId            |
|  19 | `method_ref`                      | RecordId `METHOD`   |
|  20 | `evidence_refs`                   | sorted RecordId set |
|  21 | `operation_start_journal_ref`     | JournalReference    |

All fields required.

Compatibility is directional.

---

# 91. ASSUMPTION_VERSION_COMPATIBILITY_INVALIDATION Record

Record Type:

```text id="tj7gny"
85 = ASSUMPTION_VERSION_COMPATIBILITY_INVALIDATION
```

| Key | Field                                   | Type                 | Presence         |
| --: | --------------------------------------- | -------------------- | ---------------- |
|  16 | `target_mode_id`                        | UInt                 | required         |
|  17 | `explicit_compatibility_authority_refs` | JournalReference set | context-required |
|  18 | `attribute_predicate_ref`               | RecordId             | context-required |
|  19 | `method_ref`                            | RecordId `METHOD`    | required         |
|  20 | `evidence_refs`                         | sorted RecordId set  | required         |
|  21 | `reason_codes`                          | sorted Text set      | required         |
|  22 | `operation_start_journal_ref`           | JournalReference     | required         |

Mode symmetry follows §88.

---

# 92. BOOTSTRAP_TRUST_DECLARATION Record

Record Type:

```text id="zy5wrg"
86 = BOOTSTRAP_TRUST_DECLARATION
```

| Key | Field                               | Type                | Presence |
| --: | ----------------------------------- | ------------------- | -------- |
|  16 | `declaration_scope_ref`             | RecordId `SCOPE`    | required |
|  17 | `trusted_mathematical_assumptions`  | sorted RecordId set | required |
|  18 | `trusted_physical_assumptions`      | sorted RecordId set | required |
|  19 | `trusted_computational_assumptions` | sorted RecordId set | required |
|  20 | `human_judgment_points`             | sorted RecordId set | required |
|  21 | `externally_governed_trust_points`  | sorted RecordId set | required |
|  22 | `rationale_refs`                    | sorted RecordId set | required |
|  23 | `known_limitations`                 | sorted Text set     | required |
|  24 | `bootstrap_evidence_refs`           | sorted RecordId set | required |
|  25 | `predecessor_declaration_ref`       | JournalReference    | optional |
|  26 | `operation_start_journal_ref`       | JournalReference    | required |

Declaration authority records an explicit trust boundary.

It does not prove every premise.

---

# 93. BOOTSTRAP_TRUST_INVALIDATION Record

Record Type:

```text id="96vb5d"
87 = BOOTSTRAP_TRUST_INVALIDATION
```

| Key | Field                                 | Type                 | Presence         |
| --: | ------------------------------------- | -------------------- | ---------------- |
|  16 | `target_mode_id`                      | UInt                 | required         |
|  17 | `explicit_declaration_authority_refs` | JournalReference set | context-required |
|  18 | `attribute_predicate_ref`             | RecordId             | context-required |
|  19 | `method_ref`                          | RecordId `METHOD`    | required         |
|  20 | `evidence_refs`                       | sorted RecordId set  | required         |
|  21 | `known_limitations`                   | sorted Text set      | required         |
|  22 | `reason_codes`                        | sorted Text set      | required         |
|  23 | `operation_start_journal_ref`         | JournalReference     | required         |

---

# 94. FORMAL_FINDING_CLASSIFICATION Record

Record Type:

```text id="x82ohf"
88 = FORMAL_FINDING_CLASSIFICATION
```

| Key | Field                                | Type                 | Presence |
| --: | ------------------------------------ | -------------------- | -------- |
|  16 | `formal_verification_authority_ref`  | JournalReference     | required |
|  17 | `classification_id`                  | UInt                 | required |
|  18 | `classifier_procedure_identity`      | RecordId `METHOD`    | required |
|  19 | `classifier_role_id`                 | UInt                 | required |
|  20 | `judgment_basis_id`                  | UInt                 | required |
|  21 | `rationale_ref`                      | RecordId             | required |
|  22 | `supporting_evidence_refs`           | sorted RecordId set  | required |
|  23 | `supporting_evidence_authority_refs` | JournalReference set | optional |
|  24 | `known_uncertainties`                | sorted Text set      | required |
|  25 | `operation_start_journal_ref`        | JournalReference     | required |

`judgment_basis_id` is `HUMAN_JUDGMENT` in v0.3.

Classification does not rewrite Formal Verification outcome.

---

# 95. PROOF_DEPENDENCY_MANIFEST Record

Record Type:

```text id="76g6m3"
89 = PROOF_DEPENDENCY_MANIFEST
```

| Key | Field                     | Type                |
| --: | ------------------------- | ------------------- |
|  16 | `formal_model_ref`        | RecordId            |
|  17 | `theorem_ref`             | RecordId            |
|  18 | `dependency_refs`         | sorted RecordId set |
|  19 | `extractor_identity_ref`  | RecordId            |
|  20 | `extractor_method_status` | UInt                |
|  21 | `extraction_limitations`  | sorted Text set     |

All fields required.

Generic Policy Method status does not apply to `extractor_method_status` in v0.3.

---

# 96. SHARED_DEPENDENCY_MANIFEST Record

Record Type:

```text id="and0oj"
90 = SHARED_DEPENDENCY_MANIFEST
```

| Key | Field                          | Type                |
| --: | ------------------------------ | ------------------- |
|  16 | `component_identity_refs`      | sorted RecordId set |
|  17 | `shared_dependency_refs`       | sorted RecordId set |
|  18 | `known_dependency_limitations` | sorted Text set     |

All fields required.

This Record discloses shared dependencies.

It does not prove independence.

---

# 97. Context-Required Field Symmetry

For every contextual field:

```text id="2jfo02"
context requires
    → present

context forbids
    → absent
```

A meaningless present field is invalid.

A required missing field is invalid.

---

# 98. Record-contained JournalReference Validation

Every Record-contained JournalReference receives structural validation from the Record alone.

Historical, authority, same-Registry, and strictly-prior validation require the exact authority context defined by Identity Format.

---

# 99. Authority Fields and Journal authority_dependencies

Where the same named authority role appears both:

```text id="bqpzei"
inside the Record

and

inside Journal authority_dependencies[]
```

the exact JournalReference MUST match.

Named-role and generic-dependency copies serve different purposes and are both required where specified.

---

# 100. Identity Fields and Journal identity_dependencies

Where an exact Record identity is also required as Journal identity dependency, the exact bytes MUST match:

```text id="m64orn"
IdentityDependency(
  RECORD_ID,
  exact RecordId
)
```

No approximate matching is permitted.

---

# 101. Event Record Validation

Given Journal Entry `E`:

```text id="qwmaor"
load E.event_record_id

verify exact Record identity

determine Record type

validate §5 event/type mapping

validate §6 event/disposition mapping

validate Record schema

validate named authority typing

validate redundant event bindings

validate lifecycle_object_id relation

validate direct authority dependencies

validate identity dependencies

validate Record-contained JournalReferences
against E where applicable

validate cross-Record lifecycle continuity
where specified
```

---

# 102. Cross-Record Freeze Continuity

For successful Freeze:

```text id="p6yda6"
START.freeze_attempt_id
==
RECEIPT.freeze_attempt_id

START.subject_id
==
RECEIPT.subject_id
==
MANIFEST.subject_id

START.policy_record_id
==
RECEIPT.policy_record_id

RECEIPT.manifest_id
==
exact loaded MANIFEST identity

RECEIPT.path_identity_profile_id
==
MANIFEST.path_identity_profile_id
```

Every equality is exact.

---

# 103. No Hidden Record-Type Conversion

The implementation MUST NOT convert:

```text id="u7lnx1"
VERIFICATION_OBSERVATION
→
VERIFICATION_RECORD
```

or any portable Record into an authoritative event Record merely by changing type/framing and re-hashing.

A new authoritative operation must produce the authoritative Record.

---

# 104. Record-local Unknown Fields

Unknown type-local keys in a known `(record_type_id, schema_version)` are invalid.

They MUST NOT be ignored.

---

# 105. Record Schema Succession

Changing any of the following requires Record Schema succession:

```text id="3dkmch"
Record Type assignment

field-key meaning

field requiredness

embedded structure

enum semantics

Policy supported-context semantics

Policy applicability

Policy evaluator mapping

Record-event mapping

event-disposition mapping

direct-authority field type

chronology-binding classification

failed candidate representation

Artifact selector representation
```

---

# 106. No Retroactive Semantic Upgrade

A predecessor Record MUST NOT acquire successor semantics because a newer parser recognizes similar fields.

If faithful interpretation cannot be established:

```text id="k780ud"
UNSUPPORTED_SCHEMA
```

is preferred.

---

# 107. Record Schema Machine Registry

The repository MUST maintain machine-readable schema metadata containing:

```text id="l4kh9x"
record_type_id
record_type_name
assignment_state
schema_version

field_key
field_name
field_type
presence_rule
context_rule
set_ordering_rule

referenced_record_type
where constrained

associated_event_type_ids
where applicable

required_disposition_id
where applicable

authority_binding_kind
where applicable

chronology_binding_class
where applicable
```

This registry SHOULD generate:

```text id="u0jrh5"
Rust schema constants

Dafny schema constants

documentation tables

test-vector templates

collision checks
```

---

# 108. Policy Machine Registry

Policy metadata MUST additionally contain:

```text id="sojqze"
policy_field_identity

applicable_contexts[]

applicable_subfield_rules

evaluator_id

positive_vector_ids[]

negative_vector_ids[]

material_failure_dimensions[]
```

---

# 109. Schema Coverage Invariant

Every ASSIGNED Record Type has exactly one schema.

Every Journal event maps to exactly one permitted Record Type.

Every shared-disposition event additionally has one required disposition mapping.

Every event-associated Record has one explicit chronology classification.

Every named direct-authority relation in §9 resolves to concrete machine-readable Record field metadata.

---

# 110. Policy Requirement Coverage Invariant

The Policy schema requirement classes MUST equal the controlling Lifecycle Policy requirement classes.

The equality MUST be machine-checked.

`supported_context_ids` is schema/gate metadata in addition to those requirement classes.

---

# 111. Policy Evaluator Coverage Invariant

Every mechanically enforced Policy field or structural context guard has registered evaluator coverage.

This includes:

```text id="uehkcr"
supported_context_ids
```

through evaluator 1014.

---

# 112. Positive/Negative Vector Coverage

Every evaluator requires:

```text id="obzhkb"
>= 1 positive vector

>= 1 negative vector
```

Every declared material failure dimension requires a dedicated negative vector.

Every context-required field requires presence/absence symmetry vectors where both contextual forms exist.

---

# 113. Record Schema Vector Families

At minimum:

```text id="yv0um2"
REC-GENESIS-VALID-001

REC-MANIFEST-VALID-001
REC-MANIFEST-PATH-ORDER-FAIL-001

REC-FREEZE-START-VALID-001
REC-FREEZE-RECEIPT-VALID-001
REC-FREEZE-RECEIPT-WRONG-START-001
REC-FREEZE-POLICY-MISMATCH-001
REC-FREEZE-SUBJECT-MISMATCH-001
REC-FREEZE-MANIFEST-SUBJECT-MISMATCH-001

REC-VERIFY-OBSERVATION-VALID-001
REC-VERIFY-OBSERVATION-UNEXPECTED-START-REF-001

REC-VERIFY-EMBEDDED-VALID-001
REC-VERIFY-SOURCE-MATCH-VALID-001
REC-VERIFY-SOURCE-MATCH-MISSING-BINDING-001
REC-VERIFY-ORDINARY-UNEXPECTED-BRACKET-001

REC-REVIEW-REQUEST-VALID-001
REC-REVIEW-REQUEST-POLICY-AUTHORITY-REQUIRED-001
REC-REVIEW-REQUEST-POLICY-SELECTOR-FAIL-001

REC-REVIEW-RESULT-VALID-001
REC-REVIEW-RESULT-WRONG-REQUEST-AUTHORITY-001
REC-REVIEW-RESULT-ROLE-MISMATCH-001

REC-ADMISSION-ACCEPTED-001
REC-ADMISSION-REJECTED-KNOWN-RECORD-001
REC-ADMISSION-REJECTED-CONTEXT-INVALID-JREF-001

REC-POLICY-MINIMAL-001
REC-POLICY-CONTEXT-DECLARATION-001
REC-POLICY-CONTEXT-UNSUPPORTED-001
REC-POLICY-DEAD-REQUIREMENT-FAIL-001
REC-POLICY-UNKNOWN-FIELD-001
REC-POLICY-EMPTY-REQUIREMENT-001

REC-CLOSEOUT-COMMITTED-001
REC-CLOSEOUT-REJECTED-001
REC-CLOSEOUT-REJECTED-EMPTY-REVIEWS-001
REC-CLOSEOUT-REJECTED-EMPTY-CREATION-VERIFY-001
REC-CLOSEOUT-FREEZE-MISSING-NOT-HISTORICAL-001
REC-CLOSEOUT-POLICY-MISSING-NOT-HISTORICAL-001
REC-CLOSEOUT-MANIFEST-MISMATCH-001
REC-CLOSEOUT-BOOTSTRAP-MISSING-001
REC-CLOSEOUT-BRACKET-PRE-MISSING-001
REC-CLOSEOUT-NO-PREDECESSOR-001
REC-CLOSEOUT-PREDECESSOR-RESOLVED-001
REC-CLOSEOUT-PREDECESSOR-UNRESOLVED-001
REC-CLOSEOUT-PREDECESSOR-STATE-AMBIGUOUS-FAIL-001

REC-STORAGE-CAPABILITY-001
REC-ENVIRONMENT-OBSERVATION-001
REC-CAPABILITY-PROVENANCE-CACHED-001

REC-EVICTION-SCOPE-ALL-001
REC-EVICTION-SCOPE-EXPLICIT-001
REC-EVICTION-SCOPE-PATH-ORDER-FAIL-001
REC-EVICTION-SCOPE-PROFILE-MISMATCH-001
REC-EVICTION-SCOPE-MISSING-PATH-001

REC-EVICTION-START-CHRONOLOGY-REQUIRED-001

REC-ASSUMPTION-DEFINITION-001
REC-ASSUMPTION-ESTABLISHMENT-001
REC-ASSUMPTION-ESTABLISHMENT-CACHE-PROVENANCE-MISSING-001
REC-ASSUMPTION-INVALIDATION-EXPLICIT-001
REC-ASSUMPTION-INVALIDATION-PREDICATE-001

REC-FORMAL-VERIFICATION-VERIFIED-001
REC-FORMAL-VERIFICATION-COUNTEREXAMPLE-001
REC-FORMAL-VERIFICATION-COUNTEREXAMPLE-MISSING-REF-001

REC-COMPATIBILITY-001
REC-COMPATIBILITY-INVALIDATION-001

REC-BOOTSTRAP-DECLARATION-001
REC-BOOTSTRAP-INVALIDATION-001

REC-FORMAL-CLASSIFICATION-001
REC-FORMAL-CLASSIFICATION-WRONG-BASIS-001

REC-PROOF-DEPENDENCY-MANIFEST-001
REC-SHARED-DEPENDENCY-MANIFEST-001
```

---

# 114. Policy Vector Families

At minimum:

```text id="j9kgxh"
POL-CONTEXT-SUPPORTED-001
POL-CONTEXT-UNSUPPORTED-001

POL-REQUEST-SELECTOR-VALID-001
POL-REQUEST-SELECTOR-ROLE-FAIL-001
POL-REQUEST-COUNT-NOT-EVALUATED-001

POL-REVIEW-MATCH-VALID-001
POL-REVIEW-MATCH-ROLE-FAIL-001
POL-REVIEW-MATCH-SCOPE-FAIL-001
POL-REVIEW-MATCH-METHOD-FAIL-001
POL-REVIEW-MATCH-CHECKS-FAIL-001

POL-REVIEW-COUNT-VALID-001
POL-REVIEW-COUNT-LOW-001

POL-METHOD-VALID-001
POL-METHOD-STATUS-FAIL-001
POL-METHOD-UNIFORM-REVIEW-FAIL-001
POL-METHOD-UNIFORM-VERIFICATION-FAIL-001
POL-METHOD-UNIFORM-ESTABLISHMENT-FAIL-001

POL-FINDING-VALID-001
POL-FINDING-FAIL-001
POL-FINDING-UNIFORM-REVIEW-FAIL-001
POL-FINDING-UNIFORM-VERIFICATION-FAIL-001
POL-FINDING-UNIFORM-ESTABLISHMENT-FAIL-001

POL-VERIFY-VALID-001
POL-VERIFY-TARGET-FAIL-001
POL-VERIFY-NONAUTHORITATIVE-FAIL-001

POL-INTERVENING-NONE-001
POL-INTERVENING-FORBIDDEN-001

POL-DISTANCE-BELOW-001
POL-DISTANCE-BOUNDARY-001
POL-DISTANCE-EXCEEDED-001

POL-DURABILITY-VALID-001
POL-DURABILITY-MISSING-001
POL-DURABILITY-UNSUPPORTED-001

POL-ANCHOR-HEAD-001
POL-ANCHOR-ANCESTOR-001
POL-ANCHOR-DIVERGENCE-001

POL-BOOTSTRAP-SCOPE-VALID-001
POL-BOOTSTRAP-SCOPE-MISMATCH-001

POL-FORMAL-CLASS-VALID-001
POL-FORMAL-CLASS-MISSING-001
POL-FORMAL-CLASS-NOT-ALLOWED-001

POL-DIVERSITY-VALID-001
POL-DIVERSITY-COUNT-FAIL-001
POL-DIVERSITY-UNKNOWN-NOT-DISTINCT-001

POL-BRACKET-PRE-VALID-001
POL-BRACKET-SATISFIED-001
POL-BRACKET-FIRST-POST-FAIL-001
POL-BRACKET-LATER-SUCCESS-NO-REHAB-001
```

---

# 115. Event Integration Vector Families

The event-integration suite owns Journal-event ↔ Record semantic agreement.

At minimum:

```text id="xs4vcz"
EVT-302-RECORD-REJECTED-FAIL-001
EVT-303-RECORD-ACCEPTED-FAIL-001

EVT-500-RECORD-REJECTED-FAIL-001
EVT-501-RECORD-COMMITTED-FAIL-001

EVT-701-RECORD-INTERRUPTED-FAIL-001
EVT-702-RECORD-COMMITTED-FAIL-001
EVT-703-RECORD-COMMITTED-FAIL-001

EVT-300-POLICY-AUTHORITY-BINDING-FAIL-001
EVT-301-REQUEST-AUTHORITY-BINDING-FAIL-001
```

---

## 115.1 Suite responsibility

`REC-*` vectors primarily assert:

```text id="igqndn"
Record schema validity

field symmetry

Record-internal structure

cross-Record consistency
```

`EVT-*` vectors primarily assert:

```text id="off929"
Journal event type
↔
Record type/disposition/authority role
```

The same fixture MAY be reused by both suites.

A reused fixture is not duplicate qualification when the asserted property differs.

---

## 115.2 Disposition vector ownership

Where a `REC-*` fixture and an `EVT-*` fixture both contain the same disposition mismatch:

```text id="lt87q7"
REC suite claim:
    Record/context construction and schema-side setup

EVT suite claim:
    Journal event_type_id and Record disposition
    cannot disagree
```

The normative event/disposition integration claim belongs to the `EVT-*` suite.

Record fixtures MAY reuse the same malformed Record only as supporting schema input.

This distinction prevents duplicate vector names from being mistaken for duplicate normative responsibilities.

---

# 116. Failed Candidate Vector Families

At minimum:

```text id="erv186"
FAILCAND-ADMISSION-RECORD-NONAUTHORITATIVE-001
FAILCAND-ADMISSION-RECORD-ROLE-PRESERVED-001

FAILCAND-ADMISSION-JREF-ENTRY-MISSING-001
FAILCAND-ADMISSION-JREF-HASH-MISMATCH-001
FAILCAND-ADMISSION-JREF-WRONG-EVENT-001
FAILCAND-ADMISSION-JREF-CROSS-REGISTRY-001

FAILCAND-CLOSEOUT-MANIFEST-RECORD-FAIL-001
FAILCAND-CLOSEOUT-ADMITTED-REVIEW-JREF-FAIL-001
FAILCAND-CLOSEOUT-PRE-VERIFICATION-JREF-FAIL-001
FAILCAND-CLOSEOUT-PREDECESSOR-RECORD-FAIL-001
FAILCAND-CLOSEOUT-BOOTSTRAP-JREF-FAIL-001

FAILCAND-STRUCTURALLY-INVALID-JREF-NOT-RECORDABLE-001
FAILCAND-UNKNOWN-RECORD-ID-NOT-INVENTED-001
```

---

# 117. Eviction Selector Vector Families

At minimum:

```text id="1rzfr3"
EVICT-SELECTOR-ALL-PROFILE-MATCH-001

EVICT-SELECTOR-EXPLICIT-SINGLE-001

EVICT-SELECTOR-EXPLICIT-MULTI-CANONICAL-001

EVICT-SELECTOR-DUPLICATE-PATH-FAIL-001

EVICT-SELECTOR-UNSORTED-PATHS-FAIL-001

EVICT-SELECTOR-PROFILE-MISMATCH-FAIL-001

EVICT-SELECTOR-MISSING-MANIFEST-PATH-FAIL-001

EVICT-SELECTOR-MANIFEST-DIGEST-IS-AUTHORITY-001
```

---

# 118. Identity Vector Interaction

Identity vectors establish byte mechanics:

```text id="6p5hyn"
canonical CBOR

Record framing

Journal framing

Reference encoding

dependency ordering

hash derivation
```

Record Schema vectors establish semantic schema structure:

```text id="mgnxzk"
field set

field types

context symmetry

Record dispatch

cross-Record equality

Policy applicability

Artifact selector interpretation
```

Claims MUST remain distinct.

---

# 119. Bootstrap of EvidenceRegistry Itself

EvidenceRegistry's first qualified implementation cannot use itself as the sole authority for this Record Schema.

Initial qualification SHOULD include:

```text id="1ixvtp"
source-control identity

frozen normative specifications

Identity expected-byte vectors

independent CBOR/hash reproduction

hostile Record Schema review

independent Record Schema review

external test-run evidence

Bootstrap Trust Declaration
```

Self-hosted output MAY be supplemental.

It MUST NOT erase the bootstrap chain.

---

# 120. Bootstrap Recursion Stop

The initial Bootstrap Trust Declaration SHOULD explicitly identify accepted premises such as:

```text id="qkw9em"
SHA-256 implementation

CBOR implementation

compiler/toolchain

filesystem primitives

reviewer judgment

external source-control identity
```

A recorded premise is an accepted trust boundary, not proof of universal truth.

---

# 121. Conformance Successor Handoff

A future conformance schema SHOULD distinguish:

```text id="rhm8gk"
BYTE_FORMAT_COMPATIBLE

SEMANTICS_UNCHANGED

CONFORMANCE_REQUIREMENT_TIGHTENED

VECTORS_INVALIDATED
```

Example:

```text id="5qltdp"
Identity Format v0.2 → v0.3

BYTE_FORMAT_COMPATIBLE = true
SEMANTICS_UNCHANGED = true
CONFORMANCE_REQUIREMENT_TIGHTENED = true
VECTORS_INVALIDATED = false
```

This does not modify the frozen Identity Format.

---

# 122. Vector Invalidation Principle

A successor MUST explicitly declare whether predecessor frozen vectors remain valid.

```text id="5r4sx8"
authoritative bytes changed
    → VECTORS_INVALIDATED = true

only validation/conformance obligations tightened
while valid bytes remain identical
    → VECTORS_INVALIDATED = false
```

Predecessor vectors MUST NOT be silently overwritten.

---

# 123. Open Design Detection Rule

During vector construction:

> If the frozen specifications do not determine one unique authoritative representation or one unique normative validation outcome, vector construction MUST stop and report a specification gap.

The implementation MUST NOT make a local convenience choice.

Vector construction is therefore a specification-completeness test, not merely a serialization test.

---

# 124. Record Schema Freeze Criterion

Record Schema v0.3 is frozen because review established:

```text id="91zr3d"
every ASSIGNED Record Type has one exact schema

UNASSIGNED and RETIRED IDs are distinct

Record Type IDs are collision-free

event-to-Record mapping covers all 28 Journal events

event_type_id and Record disposition agreement
is normatively required

all field keys are unique within each Record type

all required / optional / forbidden rules are explicit

all contextual field symmetry is explicit

Policy declares supported evaluation contexts

Policy context is checked before requirements

Policy applicability is defined for every requirement

dead Policy requirement fields are rejected

REVIEW_REQUEST_CREATION has selector-only semantics

Policy composition is AND

no implicit OR exists

generic Method/Finding constraints use one closed
Evidence-class list

Policy fields have evaluator IDs

every evaluator has positive and negative vectors

material evaluator branches have vector coverage

no parsed Policy requirement lacks enforcement

chronology-field presence follows one stated rule
across all event-associated Record types

portable-only Record chronology behavior is explicit

Freeze START / Eviction START chronology asymmetry
is explicitly intentional and upstream-derived

Policy reference typing is consistent with
Cross-Reference authority requirements

all named direct-authority Record fields agree with
Cross-Reference v0.3

the direct-authority audit is machine-resolvable to
concrete Record fields

Review Request binds authoritative Policy

Review Result binds authoritative Review Request

Freeze START / Receipt Policy binding is identity-based
and remains exact across the lifecycle object

GENESIS Record / Journal redundant fields match

Manifest encoding is deterministic

Freeze Receipt binds exact START

Freeze Subject identity is consistent across
START / Receipt / Manifest

Verification Observation and authoritative
Verification are structurally distinct

Verification Source Binding symmetry is explicit

Verification POST symmetry is explicit

Admission rejection preserves role-bearing Record and
Journal candidate identities without fabricating authority

Closeout Freeze / Policy / Manifest presence is
unambiguous for both dispositions

Closeout Review and creation-Verification collections
are always present and have explicit empty semantics

Closeout rejection preserves role-bearing failed
candidate identities

Closeout predecessor states distinguish:
not requested,
resolved,
and requested-but-unresolved

failed candidate Journal roles and failure codes
are registered

failed candidate Record roles and failure codes
are registered

Bootstrap Scope symmetry is explicit

Formal compatibility symmetry is explicit

cached capability provenance symmetry is explicit

invalidations have exact target-mode symmetry

formal counterexample does not become product defect
without classification

Bootstrap Trust remains an explicit premise boundary

all Record-contained Journal References use
two-stage validation

explicit Artifact eviction selectors have one
deterministic Manifest-relative representation

Eviction selector ordering uses the Lifecycle path comparator

Eviction Scope path profile exactly matches
pre-eviction Manifest profile

Eviction Scope cannot override Manifest digest/kind/size

Record-schema and event-integration vector claims
are explicitly separated

expected-byte generation encounters no unresolved
representation choice
```

No known BLOCKING or MATERIAL finding remains open against this frozen revision.

---

# 125. Next Work After Record Schema Freeze

Once frozen:

```text id="ymlw80"
Lane A — deterministic vectors

1. FreezeRoot expected bytes/hash

2. JournalReference vectors

3. IdentityDependency vectors

4. dependency collection vectors

5. GENESIS Journal Entry vectors

6. remaining Identity vectors

7. Record expected-byte vectors

8. Policy evaluator vectors

9. event-integration vectors


Lane B — PURE_FORMAL

1. SupportImpact algebra

2. CombineImpact properties

3. SupportStateIsOrderIndependent

4. lifecycle datatypes

5. EventShapeOf / ObjectKindOf

6. HasTerminality / IsTerminal

7. LegalPredecessor / LegalTransition

8. remaining Cross-Reference PURE_FORMAL properties
```

Lane A and Lane B MAY proceed independently.

Lane B consumes no filesystem, runtime, storage, probe, or platform assumption at this stage.

---

## 125.1 Recommended PURE_FORMAL entry point

Before implementing the larger lifecycle transition core, the Dafny work SHOULD begin with the SupportImpact algebra.

Using the exact order:

```text id="y545wj"
UNAFFECTED
<
AFFECTED_UNKNOWN
<
DEGRADED
<
AFFECTED
```

and:

```text id="xe9v60"
CombineImpact(a, b)
=
max(a, b)
```

the first proofs SHOULD establish:

```text id="jbs69d"
commutativity

associativity

idempotence

UNAFFECTED as identity
```

These are lightweight PURE_FORMAL properties.

Once established, order-independence of accumulated SupportImpact follows directly or nearly directly from the same algebra.

This provides a small proof kernel before the larger lifecycle state machine is introduced.

---

## 125.2 Vector-first specification gap detection

Lane A MUST preserve §123.

If any expected-byte vector cannot be constructed uniquely:

```text id="4pltvv"
stop vector construction

classify as specification gap

do not choose an implementation-local encoding
```

A newly discovered specification gap MAY require a Record Schema successor, Identity Format successor, or upstream semantic successor depending on the layer in which the ambiguity originates.

---

# 126. Final Record Principle

> A Record is an immutable statement with one exact byte representation. Journal authority may later make that statement part of Registry history, but authority does not change what the Record says.

> A Policy is one exact qualification gate. It explicitly declares the evaluation contexts in which it may be used. A caller may not repurpose a Policy merely because some of its fields happen to fit another operation.

> Every applicable Policy requirement composes by AND. No implementation may invent an unstated OR, an undeclared context, a dead requirement, or a hidden requirement.

> A named prior authority is represented as authority, not collapsed into a Record identity. A pure identity binding remains an identity and does not silently acquire authority.

> Chronology is explicit but is not Policy applicability. An operation-start or observation-start reference records where an operation began; it does not, by itself, invent a gate. Apparent asymmetries inherited from the controlling lifecycle specification are preserved rather than "repaired" for visual consistency.

> A rejected operation preserves failed candidates only when their exact identities are reconstructable. Record candidates and Journal candidates retain both their role and their controlling validation failure. Absence of a predecessor is not the same historical fact as an intended predecessor that failed qualification.

> Artifact eviction selects canonical paths relative to one exact pre-eviction Manifest and one exact path identity profile. The Scope selects; the Manifest states the Artifact facts.

> A schema field that cannot be evaluated, validated, contextualized, or given one canonical representation does not belong in the frozen schema.

> EvidenceRegistry must be able to explain not only why a gate passed or failed, but which context was authorized, which Policy requirement was applicable, which evaluator handled it, which Evidence satisfied or failed it, and which frozen vectors demonstrate both outcomes.
