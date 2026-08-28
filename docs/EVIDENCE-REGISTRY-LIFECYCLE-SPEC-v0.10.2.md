# EvidenceRegistry Evidence Lifecycle Specification v0.10.2

**Status:** FINAL FREEZE CANDIDATE
**Predecessor:** EvidenceRegistry Evidence Lifecycle Specification v0.10.1
**Project:** EvidenceRegistry
**Repository:** `DwarfM42/EvidenceRegistry`
**Implementation candidate:** Rust
**Formal-model candidate:** Dafny
**Primary deployment model:** local-first, cross-platform, filesystem-authoritative CLI/tooling
**Normative language:** MUST / MUST NOT / SHOULD / SHOULD NOT / MAY

---

# 0. Specification Position

This specification is the integrated successor of the EvidenceRegistry Evidence Lifecycle Specification line beginning with v0.2.

The predecessor chain remains historical.

A successor specification MUST NOT retroactively reinterpret Evidence created under an older format unless an explicit compatibility specification authorizes that interpretation.

This specification defines:

```text
Evidence identity
+
Registry authority
+
lifecycle chronology
+
Review admission
+
Closeout
+
succession
+
verification
+
historical preservation
+
support-record authority
+
bootstrap trust authority
```

It does not define the detailed theorem semantics of the separate Formal Verification & Implementation Boundary Specification.

---

## 0.1 Changes from v0.10.1

v0.10.2 introduces no new lifecycle architecture, no new Journal event type, and no new Record type.

It restores normative statements that were present in the v0.6 / v0.6.1 delta line but were lost or weakened during integration, and closes field-location and vocabulary gaps introduced by integration.

```text
FREEZE_COMMIT_REJECTED restored to the transition summary   §20
ONE_PHASE crash-history non-claim restored                  §19.2
intervening-event Policy constraint restored                §75.1
Verification byte-retention dependency restored             §101
probe-cache lifetime non-overclaim restored                 §112.1
rollback-labelling prohibition restored to MUST NOT form    §116.4 / §192
SOURCE_SUBJECT_NODRIFT reinterpretation explicitly barred   §70.1
required_bootstrap_scope_ref location unified               §85 / §88 / §137
formal-finding classification requirement added to Policy   §85 / §145
Journal-replay insufficiency list completed                 §22
GENESIS entry / Record field location clarified             §11.1
directory Artifact treatment decided                        §37.1
INDETERMINATE exit-code mapping stated                      §163.1
adversarial corpus duplicate scenario merged                §168
ER-P / FV-P numbering namespaces separated                  §191.0
```

No Record created under v0.10.1 changes meaning under v0.10.2.

---

# 1. Purpose

EvidenceRegistry is a general-purpose Evidence lifecycle and qualification toolkit.

Its purpose is to preserve and mechanically verify the relationship between:

* a Subject;
* the exact bytes that constituted that Subject;
* the Manifest identity of those bytes;
* the lifecycle event that granted Registry authority;
* Reviews performed against that exact Subject;
* Review Admission or rejection;
* Verification observations;
* Closeout;
* predecessor and successor Evidence;
* formal-support Records where used;
* Bootstrap Trust Declarations where used;
* and the historical order by which authoritative Records came into existence.

Conceptually:

```text
Subject
  ↓
Freeze Attempt
  ↓
Manifest / Freeze Receipt
  ↓
Authority Journal Commit
  ↓
Verification / Review
  ↓
Admission
  ↓
Closeout
  ↓
Postconditions
  ↓
Successor
  ↓
Invalidation / New Evidence
  ↓
Historical Audit
```

The central rule is:

> Evidence can change by succession, but history cannot change by convenience.

---

# 2. Scope

EvidenceRegistry is intended to be reusable for Subjects including:

* software source trees;
* release candidates;
* binaries;
* firmware;
* benchmark outputs;
* research datasets;
* experiment Evidence;
* machine-learning evaluation;
* formal-verification artifacts;
* model qualification;
* security Reviews;
* industrial qualification Records;
* CI Evidence;
* AI-agent work products;
* audit packages;
* specification packages.

EvidenceRegistry MUST NOT contain assumptions specific to:

* onigiranai;
* Hermes;
* RAG;
* Generation;
* a particular benchmark;
* a particular model;
* a particular Project.

Project-specific semantics belong in external:

```text
Policy
Contract
Scope
Method
Claim
governing authority
```

layers.

---

# 3. Fundamental Separation

EvidenceRegistry MUST preserve:

```text
Evidence mechanics
        !=
Project semantics
```

## 3.1 EvidenceRegistry may determine

EvidenceRegistry MAY mechanically determine whether:

* Artifact bytes match a recorded digest;
* a Manifest is valid and reproducible;
* a Record identity recomputes correctly;
* a lifecycle Record appears in a valid Authority Journal;
* a Review Result refers to the correct Subject;
* a Review Result has the required role and Scope;
* a Review Result has been admitted;
* a predecessor identity exists;
* a successor references the exact predecessor;
* a source observation matches a frozen Subject identity;
* a Closeout references exact authoritative support;
* an attempted lifecycle transition is legal;
* an Evidence root was fresh or pre-existing;
* a Journal contains a gap or invalid transition;
* a chronology relation is established;
* a chronology relation remains unproven;
* an Audit covered an exact declared universe;
* an external Journal Anchor agrees with retained history.

## 3.2 EvidenceRegistry must not determine

EvidenceRegistry MUST NOT independently decide:

* whether a product is safe;
* whether a release should ship;
* whether a Claim is legally sufficient;
* whether a human Review was philosophically adequate;
* whether an independent Review is cryptographically independent;
* whether a benchmark demonstrates superiority;
* whether a security property is acceptable to a regulator;
* whether a Bootstrap Trust premise is universally true;
* whether a Policy's human judgment is morally or scientifically correct.

Those decisions belong to the relevant external authority.

---

# 4. Non-Goals

EvidenceRegistry v0.x is not:

* a test runner;
* a benchmark runner;
* a CI platform;
* a build system;
* a RAG engine;
* an ingestion framework;
* an artifact repository;
* a source-control system;
* a distributed consensus system;
* a PKI;
* a TPM abstraction;
* a TEE framework;
* a remote attestation service;
* a mandatory daemon;
* an operating-system security boundary.

EvidenceRegistry v0.x MUST NOT claim to prove that:

* a particular child process actually executed;
* a particular binary was actually loaded;
* a reviewer was cryptographically independent;
* a same-privilege hostile process could not forge all local Evidence;
* an administrator could not replace both Subject and Evidence;
* a local mutable Registry can intrinsically remember a completely deleted unanchored Journal suffix;
* two physical copies of a Registry can be distinguished before their histories diverge;
* two point-in-time source observations establish continuous immutability between those observations.

---

# 5. Threat Model

## 5.1 Profile L: Local Evidence Integrity

The default v0.x profile is intended to detect or prevent classes of failure including:

* accidental overwrite;
* stale Subject identity;
* wrong Review binding;
* role laundering;
* Scope promotion;
* Review substitution;
* predecessor/successor confusion;
* missing Artifacts;
* extra Artifacts;
* Artifact drift;
* unsupported path aliasing;
* partial root reuse;
* crash-created partial Evidence;
* stale Closeout reuse;
* illegal authority transition;
* rejected Admission erasure;
* Policy mutation;
* cooperative writer collision;
* retained Journal modification;
* externally anchored rollback or divergence;
* invalid authority dependency;
* accidental normalization;
* incomplete Audit being presented as global CLEAN.

## 5.2 Explicitly out of scope

The default profile does not establish protection against an adversary that can:

* execute arbitrary code under the same effective security principal;
* directly forge all Registry files;
* modify EvidenceRegistry itself;
* modify kernel or filesystem behavior;
* act as root or local administrator;
* forge all local observations;
* compromise every external Anchor;
* replace all retained bootstrap Evidence.

These require stronger trust primitives.

---

# 6. Normative Principles

## 6.1 Immutable authoritative history

Once a Record becomes authoritative, it MUST NOT be modified in place.

Correction requires succession.

```text
Record A
  ↓
successor_of
  ↓
Record B
```

B MUST NOT rewrite A.

---

## 6.2 Record existence is not authority

A valid Record file is not authoritative merely because it exists.

```text
valid Record bytes
!=
Registry authority
```

Authority requires a legal Authority Journal event.

---

## 6.3 Failed history remains history

Explicit authoritative failures remain historical.

Examples:

```text
REVIEW_ADMISSION_REJECTED
CLOSEOUT_REJECTED
FREEZE_ABORTED_RECOVERY
FREEZE_ABORTED_BY_OPERATOR_ASSERTION
FREEZE_COMMIT_REJECTED
ARTIFACT_EVICTION_BLOCKED
```

A later success MUST NOT erase them.

This principle governs authoritative failure dispositions that were actually published.

It does not create an obligation to preserve crash events that never reached authoritative publication.

See §19.2.

---

## 6.4 Unknown over invention

Missing facts remain:

```text
UNKNOWN
INDETERMINATE
UNSUPPORTED
MISSING_EVIDENCE
```

as appropriate.

EvidenceRegistry MUST NOT infer missing identity or chronology from:

* timestamps;
* filenames;
* nearby files;
* prose;
* expected ordering;
* similar digests;
* human intuition.

---

## 6.5 No authoritative force-overwrite

Ordinary authoritative workflows MUST NOT provide the semantic equivalent of:

```text
--force
--overwrite
--repair-in-place
```

where such an operation would rewrite authoritative history.

A successor is required.

---

## 6.6 Context-required field rule

Authoritative Record validation MUST enforce both directions:

```text
context requires field
    → field MUST be present

context does not permit field
    → field MUST be absent
```

A meaningless-but-present semantic field is a structural contradiction.

Examples:

```text
same Assumption Definition
    → compatibility_authority_ref MUST be absent

cross-Definition application
    → compatibility_authority_ref MUST be present

Verification participating as POST bracket
    → brackets_closeout_authority_ref MUST be present

ordinary Verification
    → brackets_closeout_authority_ref MUST be absent

Policy does not require Bootstrap Trust
    → required_bootstrap_scope_ref MUST be absent

Policy requires Bootstrap Trust
    → required_bootstrap_scope_ref MUST be present
```

This rule is general.

It applies to every optional semantic field defined by this specification, not only to the examples above.

---

# 7. State Model

Semantically different state dimensions MUST remain separate.

---

## 7.1 Method status

```text
VALID
INVALID
ENVIRONMENTALLY_DEGRADED
```

---

## 7.2 Finding state

```text
NO_BLOCKING
BLOCKING
INDETERMINATE
```

---

## 7.3 Derived human verdict

```text
VALID + NO_BLOCKING
    → CLEAN

VALID + BLOCKING
    → NOT_CLEAN

VALID + INDETERMINATE
    → UNKNOWN

INVALID + *
    → INVALID

ENVIRONMENTALLY_DEGRADED + BLOCKING
    → NOT_CLEAN
      with degradation reason

ENVIRONMENTALLY_DEGRADED + NO_BLOCKING
    → UNKNOWN
      with degradation reason

ENVIRONMENTALLY_DEGRADED + INDETERMINATE
    → UNKNOWN
```

Environmental degradation weakens absence claims.

It MUST NOT erase an established blocking finding.

---

## 7.4 Reason codes

Conditions such as:

```text
ENVIRONMENTAL_AMBIGUITY
UNRESOLVED
MISSING_EVIDENCE
```

SHOULD be structured reason codes rather than peer verdict states.

---

## 7.5 Package lifecycle

Derived package states MAY include:

```text
UNCOMMITTED_PARTIAL
NONAUTHORITATIVE_ABORTED
NONAUTHORITATIVE_HISTORICAL
CONTAMINATED
FROZEN_AUTHORITATIVE
```

---

## 7.6 Lineage

Lineage is represented through immutable relations:

```text
successor_of
supersedes
clarifies
requalifies
```

A derived `SUPERSEDED` condition does not erase predecessor state.

---

# 8. Identity Separation

EvidenceRegistry MUST distinguish:

```text
subject_content_identity

record_identity

physical_package_identity

registry_history_identity
```

These are not interchangeable.

Portable Record identity does not imply portable Registry authority.

---

# 9. Registry Identity

Each Registry MUST have a stable `registry_id`.

v0.x SHOULD generate:

```text
256 random bits
from an operating-system CSPRNG
```

Canonical representation:

```text
er-registry:<64 lowercase hexadecimal characters>
```

A human-readable label MAY exist separately.

The label has no authority semantics.

---

# 10. Authority Journal

Each Registry has one authoritative linear Journal history.

Conceptually:

```text
GENESIS
  ↓
ENTRY 1
  ↓
ENTRY 2
  ↓
ENTRY 3
  ↓
...
```

The Journal is the normative source of Registry lifecycle authority and Registry chronology.

---

# 11. GENESIS

The GENESIS **Record** establishes at least:

```text
schema_version
record_type
registry_id
journal_format_version
record_identity_profile
initial storage_capability_class_id
initial environment_observation_id
created_by_tool_version
```

for v0.10-format Registries.

Historical GENESIS schemas MUST NOT be retroactively reinterpreted.

GENESIS defines the first storage capability epoch.

---

## 11.1 GENESIS Entry versus GENESIS Record

The GENESIS **Journal Entry** and the GENESIS **Record** are distinct objects with distinct identities.

The authoritative binding of the initial storage capability epoch lives in the GENESIS Record body, referenced by the GENESIS Journal Entry through `event_record_id`.

The GENESIS Journal Entry MUST additionally carry the common Journal Entry fields defined in §12, including:

```text
storage_capability_class_id
environment_observation_id
```

so that Journal-only replay can determine the capability epoch of every entry, including the first, without opening `records/`.

Both copies refer to the same observation and MUST match.

Mismatch:

```text
INVALID_RECORD_FRAMING
```

A `capability_epoch_ref` pointing at the GENESIS Journal Reference is therefore valid before the first material `STORAGE_CAPABILITY_CHANGED`.

---

# 12. Journal Entry Schema

Every Journal Entry MUST contain at least:

```text
schema_version

registry_id

entry_index
prev_entry_hash

event_type_id
event_record_id

identity_dependencies[]
authority_dependencies[]

lifecycle_object_kind
lifecycle_object_id

storage_capability_class_id
environment_observation_id
```

Event-specific indexing fields MUST be included where required for Journal-only lifecycle reconstruction.

---

# 13. Journal Reference

A Journal Reference contains:

```text
registry_id
entry_index
entry_hash
event_type_id
event_record_id
```

`entry_hash` MUST be recomputed before use.

A Journal Reference binds:

```text
exact event
+
exact position
+
Registry history
```

transitively through the hash chain to GENESIS.

---

# 14. Identity Dependency and Authority Dependency

## 14.1 IDENTITY_DEPENDENCY

Means:

> This Record commits to the exact identity of another Record.

It does not establish that the referenced Record was authoritative.

---

## 14.2 AUTHORITY_DEPENDENCY

Means:

> This lifecycle action requires the referenced Record/Event to have already acquired Registry authority.

It MUST use an exact Journal Reference.

---

## 14.3 No forward authority dependency

An authority-bearing Record MUST NOT require authority that did not yet exist.

Later facts MUST point backward to earlier authority.

Earlier Records MUST NOT point forward to future authority.

---

# 15. Journal Integrity

For Journal Entry `N`:

```text
entry_index = N
```

and:

```text
prev_entry_hash
=
recomputed hash of entry N-1
```

except GENESIS.

The following invalidate authoritative replay beyond the first invalid point:

* missing interior entry;
* duplicate index;
* fork;
* invalid predecessor hash;
* malformed Entry;
* invalid event type;
* impossible lifecycle transition;
* invalid Record identity.

---

# 16. Journal Tail Limitation

A local hash chain cannot intrinsically prove that a deleted tail once existed.

If the final N Journal entries are removed and no surviving external Evidence commits to the previous head, the remaining prefix may still be structurally valid.

Therefore:

> Profile L cannot detect unanchored Journal tail truncation from the truncated Registry alone.

This limitation is normative.

---

# 17. Reconstructed Journal Limitation

If an entity changes an interior Entry and recomputes the complete later suffix, the resulting history is structurally equivalent to another valid chain.

Without surviving external historical commitment, the current Registry alone cannot prove which chain existed earlier.

EvidenceRegistry MUST NOT claim unconditional tamper detection against this case.

---

# 18. Journal Slot Publication

Journal slots MUST use deterministic monotonically increasing indexes.

Publication MUST use atomic no-replace semantics or an equivalent supported primitive.

Existing authoritative Journal Entries MUST NOT be overwritten.

A mutable `HEAD` MAY exist as an optimization.

It is not authoritative.

---

# 19. Operation Phase Classes

Authoritative lifecycle operations use:

```text
BOOTSTRAP
ONE_PHASE
TWO_PHASE
```

The classification of an operation is normative.

It MUST NOT be selected ad hoc by an implementation.

Changing an operation between classes after stable Evidence exists is a lifecycle-format change requiring successor specification review.

---

## 19.1 BOOTSTRAP

Reserved for creation of the Registry authority domain itself.

```text
Registry Init
→ BOOTSTRAP
```

Once GENESIS authority exists, every later authoritative lifecycle operation MUST be ONE_PHASE or TWO_PHASE according to §19.2 and §19.3.

BOOTSTRAP does not weaken any normal lifecycle rule.

---

## 19.2 ONE_PHASE

An operation MAY be ONE_PHASE when, before its terminal authoritative Journal append, it performs no persistent side effect whose crash survival requires lifecycle recovery.

Such an operation MAY:

* read filesystem state;
* hash bytes;
* compute Records in memory;
* validate Policy;
* create nonauthoritative temporary files;
* create nonauthoritative content-addressed Record files.

It MUST NOT perform an irreversible or authority-relevant external mutation before the terminal Journal event.

A crash before terminal publication creates no authoritative lifecycle disposition.

The operation may be retried.

---

### 19.2.1 No crash-history claim for ONE_PHASE operations

For a ONE_PHASE operation, EvidenceRegistry MUST NOT claim that a crash occurring before the terminal Journal append is itself preserved as historical Evidence.

That information does not exist authoritatively.

The normative statement is:

> Completed terminal dispositions are preserved. Crashes before authoritative publication are safely rerunnable and leave no authoritative disposition.

This bounds §6.3.

Any earlier wording suggesting that every operation crossing an early in-memory threshold necessarily leaves immutable crash history is superseded.

---

## 19.3 TWO_PHASE

An operation MUST be TWO_PHASE when it may create crash-surviving persistent state before terminal authority.

Formally:

```text
TWO_PHASE_REQUIRED(operation)
iff
∃ persistent side effect S
such that
  S may survive process failure
  AND
  S occurs before the operation's terminal
  authoritative Journal event
  AND
  Registry history must distinguish S from
  ordinary unrelated filesystem state.
```

Structure:

```text
START
↓
persistent side effects
↓
terminal event
```

Recovery MUST be derivable from Journal history.

---

# 20. Core Transition Summary

This table is normative.

Every event in the Journal Event Type Registry (§21) MUST appear here.

| Operation                        | Phase     | Start binding                       | Terminal                          |
| -------------------------------- | --------- | ----------------------------------- | --------------------------------- |
| Registry Init                    | BOOTSTRAP | none                                | GENESIS                           |
| Freeze                           | TWO_PHASE | FREEZE_ATTEMPT_STARTED              | COMMITTED / ABORTED_RECOVERY / ABORTED_BY_OPERATOR_ASSERTION |
| Rejected Freeze Commit Recording | ONE_PHASE | existing Attempt + conflicting terminal authority | FREEZE_COMMIT_REJECTED |
| Verification Record              | ONE_PHASE | observation head only               | VERIFICATION_RECORDED             |
| Review Request                   | ONE_PHASE | operation head                      | REVIEW_REQUEST_RECORDED           |
| Review Result                    | ONE_PHASE | operation head                      | REVIEW_RESULT_RECORDED            |
| Review Admission                 | ONE_PHASE | operation head                      | ACCEPTED / REJECTED               |
| Policy registration              | ONE_PHASE | operation head                      | POLICY_RECORDED                   |
| Closeout                         | ONE_PHASE | operation head                      | COMMITTED / REJECTED              |
| Artifact eviction                | TWO_PHASE | EVICTION_STARTED                    | COMMITTED / INTERRUPTED / BLOCKED |
| Storage capability change record | ONE_PHASE | current Journal head                | STORAGE_CAPABILITY_CHANGED        |
| Assumption Definition            | ONE_PHASE | operation head                      | RECORDED                          |
| Assumption Establishment         | ONE_PHASE | operation head                      | RECORDED                          |
| Assumption Invalidation          | ONE_PHASE | operation head                      | RECORDED                          |
| Formal Verification              | ONE_PHASE | operation head                      | RECORDED                          |
| Compatibility                    | ONE_PHASE | operation head                      | RECORDED                          |
| Compatibility Invalidation       | ONE_PHASE | operation head                      | RECORDED                          |
| Bootstrap Trust Declaration      | ONE_PHASE | operation head                      | RECORDED                          |
| Bootstrap Trust Invalidation     | ONE_PHASE | operation head                      | RECORDED                          |
| Formal Finding Classification    | ONE_PHASE | operation head                      | RECORDED                          |

---

## 20.1 Full cross-reference table

Before lifecycle freeze, the project MUST publish one authoritative cross-reference table containing at least:

```text
event_type_id
event_type_name
operation_name
phase_classification
lifecycle_object_kind
legal_predecessor_state
resulting_state
terminal
```

Every event in §21 MUST appear in that table.

Every authoritative transition defined in this specification MUST use a registered event identifier.

That table is direct normative input to the Formal Verification & Implementation Boundary Specification.

---

# 21. Journal Event Type Registry

Event type identity uses a normative numeric identifier:

```text
u16 event_type_id
```

v0.x assigns:

```text
1    GENESIS

100  FREEZE_ATTEMPT_STARTED
101  FREEZE_COMMITTED
102  FREEZE_ABORTED_RECOVERY
103  FREEZE_ABORTED_BY_OPERATOR_ASSERTION
104  FREEZE_COMMIT_REJECTED

200  VERIFICATION_RECORDED

300  REVIEW_REQUEST_RECORDED
301  REVIEW_RESULT_RECORDED
302  REVIEW_ADMISSION_ACCEPTED
303  REVIEW_ADMISSION_REJECTED

400  POLICY_RECORDED

500  CLOSEOUT_COMMITTED
501  CLOSEOUT_REJECTED

600  STORAGE_CAPABILITY_CHANGED

700  ARTIFACT_EVICTION_STARTED
701  ARTIFACT_EVICTION_COMMITTED
702  ARTIFACT_EVICTION_INTERRUPTED_RECOVERY
703  ARTIFACT_EVICTION_BLOCKED

800  ASSUMPTION_DEFINITION_RECORDED
801  ASSUMPTION_ESTABLISHMENT_RECORDED
802  ASSUMPTION_INVALIDATION_RECORDED
803  FORMAL_VERIFICATION_RECORDED
804  ASSUMPTION_VERSION_COMPATIBILITY_RECORDED
805  ASSUMPTION_VERSION_COMPATIBILITY_INVALIDATED
806  BOOTSTRAP_TRUST_DECLARATION_RECORDED
807  BOOTSTRAP_TRUST_INVALIDATED
808  FORMAL_FINDING_CLASSIFICATION_RECORDED
```

Assigned identifiers MUST NEVER be reused with different semantics.

Unknown authoritative event IDs MUST fail closed.

---

# 22. Journal-only Reconstruction

The following MUST be derivable from Journal Entry bodies alone:

```text
open Freeze Attempt set

terminal Freeze Attempt set

expected Freeze root set

Attempt → intended root relation

Attempt → terminal disposition

open Artifact Eviction set

capability epoch of every entry

basic Registry authority dependency graph
```

Journal replay alone is NOT sufficient for:

```text
Policy satisfaction

Review Scope contents

Review Method semantics

Verification target semantics

Method status

Finding state

Source Binding contents

Assumption semantics

Assumption Version Compatibility contents

Invalidation predicate contents

Bootstrap Trust contents

Formal Finding Classification contents

formal theorem contents
```

Those require authoritative Record payloads.

Missing required Record payloads MUST produce:

```text
UNKNOWN
MISSING_EVIDENCE
```

or another fail-closed disposition.

They MUST NOT permit a Policy gate to succeed from Journal metadata alone.

---

# 23. Registry On-Disk Layout

Before stable format release, the authoritative namespace layout MUST be frozen.

Conceptually:

```text
<registry-root>/
├─ registry/
│  ├─ genesis.cbor
│  └─ format.cbor
│
├─ journal/
│  ├─ 00000000000000000000.cbor
│  ├─ 00000000000000000001.cbor
│  └─ ...
│
├─ records/
│  └─ <record-id>.cbor
│
├─ roots/
│  └─ <freeze-attempt-id>/
│
├─ coordination/
│
├─ runtime/
│  └─ probes/
│
└─ projections/
```

`coordination/`, `runtime/`, and `projections/` are nonauthoritative operational namespaces.

---

# 24. Partial Registry Initialization

At minimum distinguish:

```text
UNINITIALIZED

INITIALIZATION_PARTIAL_NONAUTHORITATIVE

INITIALIZED_AUTHORITATIVE
```

A partial bootstrap MUST NOT be silently repaired by ordinary `init`.

Where safe, partial residue SHOULD be preserved for inspection as:

```text
unassociated nonauthoritative initialization residue
```

It may lack any Registry authority or usable `registry_id`, and therefore may be incapable of being referenced from any authoritative Registry history.

That is expected and is not itself a defect.

An explicit cleanup operation MAY remove such nonauthoritative residue.

Such cleanup:

* MUST NOT claim to rewrite authoritative history;
* MUST refuse operation if valid Registry authority is discovered;
* SHOULD require explicit operator action.

Ordinary successful `init` SHOULD instead use a fresh Registry root.

---

# 25. Journal Writer Coordination

Journal append operations MUST be serialized.

Long-running filesystem work SHOULD NOT require holding the global Journal append lock for the entire duration.

Preferred structure:

```text
attempt-specific coordination
↓
short Journal lock
↓
START
↓
release Journal lock
↓
long operation
↓
short Journal lock
↓
terminal event
```

A conservative implementation MAY temporarily serialize more broadly.

If it does, it MUST document that this is:

```text
an implementation concurrency limitation
```

and not:

```text
a semantic requirement for historical correctness
```

The normative lifecycle model MUST support reconstruction from a set of concurrently open attempts.

---

# 26. Attempt-Specific Coordination

Long-running TWO_PHASE operations SHOULD use process-scoped coordination associated with their Attempt identity.

The coordination locator MUST be derivable from:

```text
registry_id
+
attempt_id
```

under a normatively specified mapping, so that recovery can locate the primitive without guessing.

Conceptually:

```text
coordination/freeze/<freeze-attempt-id>.lock
```

A persistent lockfile's existence alone MUST NOT prove liveness.

The preferred primitive is an OS-backed lock whose ownership is released when the process terminates.

---

# 27. Open Attempt Recovery

Recovery MUST replay the Journal from GENESIS.

It MUST NOT examine only the current Journal head.

Conceptually:

```text
START events
-
terminal events
=
OPEN attempts
```

For an open Freeze Attempt:

* if writer ownership is established active, leave OPEN;
* if ownership is established released, record `FREEZE_ABORTED_RECOVERY`;
* if liveness cannot be established, classify liveness as UNKNOWN.

A liveness-UNKNOWN Attempt MUST NOT be silently aborted or committed.

---

# 28. Operator Abort Assertion

A liveness-UNKNOWN Freeze Attempt MAY be terminated by:

```text
FREEZE_ABORTED_BY_OPERATOR_ASSERTION
```

Minimum associated Record fields:

```text
schema_version
record_type

freeze_attempt_id
attempt_start_journal_ref

prior_liveness_state = UNKNOWN

operator_assertion

operator_metadata

reason
```

This means:

> Mechanical writer liveness was not proven. An operator explicitly terminated future authority eligibility for this Attempt.

It does NOT mean the previous writer was proven dead.

The state is terminal.

This is explicit historical disposition, not force-overwrite.

---

# 29. Returning Writer

Before every Journal append, terminal transition, or additional destructive Registry operation, a long-running writer MUST verify that its lifecycle object remains eligible for the intended transition.

If it discovers an incompatible terminal state:

```text
commit MUST NOT occur
```

A living writer SHOULD record:

```text
FREEZE_COMMIT_REJECTED
```

as a ONE_PHASE historical Record, binding at least:

```text
freeze_attempt_id
attempt_start_journal_ref
conflicting_terminal_journal_ref
attempted_transition
rejection_reason = ATTEMPT_ALREADY_TERMINAL
```

The recording actor is the writer that discovers its own attempted transition is illegal.

The Journal serialization layer MAY assist in producing that Record.

EvidenceRegistry does NOT require an always-running independent registrar capable of recording attempted transitions after the originating writer has disappeared.

Therefore, absence of a rejection Record does not imply that no rejected attempt ever occurred.

The normative property is:

> An illegal commit cannot alter an already-terminal Freeze.

The following is explicitly NOT normative:

> Every attempted illegal commit is guaranteed to leave a historical rejection Record.

---

# 30. Chronology

Authoritative chronology is established only by:

```text
Journal order

and

explicit identity / authority commitments
```

Wall-clock timestamps are provenance only.

---

## 30.1 Journal order

Within one Registry:

```text
entry A index < entry B index
```

establishes A-before-B in Registry history.

---

## 30.2 Identity dependency

B referencing A's identity establishes commitment to A's identity.

It does not prove prior authority.

---

## 30.3 Authority dependency

B with AUTHORITY_DEPENDENCY on A establishes that A must already have been authoritative.

---

## 30.4 INVALID_CREATION_CHRONOLOGY

Reserved for mechanically detectable contradictions.

Examples:

* future authority used as creation prerequisite;
* dependency cycle;
* successor requiring predecessor authority before predecessor exists.

It MUST NOT be emitted merely because timestamps appear reversed.

---

## 30.5 CHRONOLOGY_UNPROVEN

If Policy requires A-before-B and retained Evidence does not establish that relation:

```text
CHRONOLOGY_UNPROVEN
```

MUST be returned.

No timestamp inference is permitted.

---

## 30.6 Cross-Registry chronology

Across independent Registries, no chronology may be inferred from timestamps.

Cross-Registry chronology remains UNKNOWN unless established by explicit identity commitment or a separately specified stronger mechanism.

See also §177.

---

# 31. Record Serialization

Authoritative Record identity MUST use:

```text
RFC 8949 deterministic CBOR
```

under a restricted EvidenceRegistry value profile.

Human-readable JSON is a projection.

---

# 32. Restricted Authoritative Value Profile

Core authoritative Records SHOULD use:

* maps;
* arrays;
* unsigned integers;
* explicitly permitted signed integers;
* byte strings;
* UTF-8 text;
* booleans;
* null.

Floating-point values SHOULD NOT appear in foundational identity Records.

Indefinite-length encoding MUST NOT be used.

Duplicate map keys MUST be rejected.

---

# 33. Record Identity Framing

Record identity MUST bind:

```text
domain
record_type
schema_version
record_body
```

Conceptually:

```text
record_id =
SHA256(
  deterministic_cbor(
    [
      "EvidenceRegistry.Record.v1",
      record_type,
      schema_version,
      record_body
    ]
  )
)
```

The Record body MUST itself contain:

```text
record_type
schema_version
```

and both copies MUST match.

Mismatch:

```text
INVALID_RECORD_FRAMING
```

The duplication is intentional domain separation and self-description.

Neither location may silently override the other.

---

# 34. Stored Record Envelope

An envelope MAY contain:

```text
record_id
record_type
schema_version
record_body
```

Stored `record_id` MUST NEVER be trusted without recomputation.

---

# 35. JSON Projection

JSON output MAY be provided to humans, APIs, CI, and Agents.

Every machine-readable JSON output MUST include:

```text
output_schema_version
```

Byte strings require a documented reversible representation.

JSON is not the normative identity encoding.

---

# 36. Digest Algorithm Registry

v0.x requires SHA-256.

Example identifier:

```text
1 = SHA-256
```

Assigned identifiers MUST NOT be reused.

Human representation SHOULD use:

```text
sha256:<lowercase-hex>
```

---

# 37. Artifact Kind Registry

Artifact kinds MUST use a normative stable identifier Registry.

Each Artifact Kind MUST have:

```text
artifact_kind_id
stable symbolic name
normative byte semantics
support status
```

v0.x assigns:

```text
1 = REGULAR_FILE
2 = SYMLINK
3 = RESERVED_DIRECTORY   (reserved; MUST NOT be used in v0.x)
```

SYMLINK support is subject to the filesystem policy in §62.

Changing meaning requires a new ID.

Unassigned IDs fail closed.

The published Artifact Kind Registry is part of the stable identity format and MUST accompany cross-implementation test vectors.

---

## 37.1 Directory treatment in v0.x

Directories are traversal structure, not Manifest entries.

In v0.x:

* directories MUST NOT appear as Artifacts in a Manifest;
* directory structure is implied solely by the canonical path components of Artifact entries;
* an empty directory therefore does not participate in Subject content identity.

This is a known limitation and MUST be documented as such rather than concealed.

Where a Subject's empty-directory structure is material, the Project MUST either:

* include a sentinel Artifact under the directory; or
* await a future Artifact Kind that models directories explicitly.

Identifier `3` is reserved so that such a future kind cannot collide with an existing assignment.

---

# 38. Identifier Generation

## 38.1 Registry ID

Use 256 CSPRNG bits as defined in §9.

## 38.2 Freeze Attempt ID

v0.x SHOULD use:

```text
256 CSPRNG bits
```

represented as:

```text
er-freeze-attempt:<64 lowercase hexadecimal characters>
```

Other lifecycle Attempt IDs SHOULD provide equivalent collision resistance where required.

Randomness provides collision resistance.

It MUST NOT be described as authentication.

---

# 39. Path Identity Model

Path identity is independent of absolute placement.

A path is an ordered sequence of components.

Identity MUST NOT rely on ambiguous slash concatenation.

Identity encoding MUST length-frame each component.

---

# 40. Path Identity Profiles

A `path_identity_profile_id` identifies a complete immutable Path Identity Profile defining at least:

```text
component representation
platform representation where applicable
component byte encoding
path comparison algorithm
normalization behavior
case semantics relevant to identity
```

At minimum:

```text
UTF8_STRICT_V1
NATIVE_LOSSLESS_V1
```

---

## 40.1 UTF8_STRICT_V1

Each component is a Unicode scalar sequence encoded as UTF-8.

No implicit Unicode normalization is permitted.

---

## 40.2 NATIVE_LOSSLESS_V1 on Unix

Component identity bytes are raw filename bytes.

---

## 40.3 NATIVE_LOSSLESS_V1 on Windows

Component identity is the original UTF-16 code-unit sequence.

Normative cross-implementation encoding uses:

```text
u16_be(code_unit)
```

for each code unit.

This representation is independent of host CPU byte order.

A NATIVE_LOSSLESS path identity MAY be non-portable to another platform.

That is permitted.

Lossy conversion is not.

---

# 41. Canonical Path Ordering

Compare paths component-by-component.

For corresponding components:

```text
lexicographic comparison of component identity bytes
```

If one component is an exact prefix of the other:

```text
shorter component first
```

If every common component is equal:

```text
path with fewer components first
```

Component boundaries MUST be respected.

Comparison MUST NOT concatenate all components and then compare across component boundaries.

This ordering is normative for Manifest ordering, derived aggregate construction, test vectors, and cross-implementation reproduction.

---

# 42. Path Rules

Canonical paths MUST reject:

* empty components;
* `.`;
* `..`;
* absolute roots;
* drive prefixes as relative components;
* malformed selected-profile encodings.

Display escaping MAY exist.

Display strings MUST NOT become normative identity.

---

# 43. Filesystem Identity Profile

A Freeze SHOULD record, where detectable:

```text
platform

filesystem type

case sensitivity

normalization behavior

path identity profile

filesystem object identity capability

short-name alias capability

alternate-data-stream capability

filesystem transport

sync management

placeholder capability
```

Unknown values remain explicit.

---

# 44. Filesystem Object Alias Detection

Where supported, freeze and verify SHOULD resolve Artifacts to platform object identity.

Examples:

```text
Unix:
  device + inode

Windows:
  volume identity + file identity
```

This identity is for alias detection.

It is not portable Subject identity.

Distinct canonical paths resolving to the same object MUST fail closed unless a specific Policy explicitly permits the relation.

---

# 45. Cross-Platform Path Verification

A verifier MUST establish that the recorded path profile can be faithfully interpreted on the current platform before byte verification.

If not:

```text
UNSUPPORTED_PATH_PROFILE
```

or an explicit indeterminate result MUST be returned.

Never false CLEAN.

---

# 46. Artifact Byte Semantics

EvidenceRegistry hashes actual bytes.

It MUST NOT silently normalize:

* LF / CRLF;
* BOMs;
* character encoding;
* Unicode normalization;
* JSON whitespace;
* Markdown;
* source formatting.

`size_bytes` MUST mean:

```text
number of bytes actually consumed by the digest operation
```

not an unrelated prior metadata observation.

---

# 47. Manifest

Manifest is a normative EvidenceRegistry Record.

```text
record_type = MANIFEST
```

Minimum fields:

```text
schema_version
record_type

subject_id

artifact_count

path_identity_profile_id

digest_profile_id

artifacts[]
```

`manifest_id` is the Record ID of this deterministic Manifest Record.

It is the sole normative identity of the frozen Artifact inventory.

Manifest deterministic CBOR encoding is part of the identity freeze set (§173) and its test vectors (§174).

---

# 48. Manifest Ordering

Artifacts MUST appear in canonical path order.

Duplicate canonical paths MUST be rejected.

Filesystem enumeration order MUST NOT affect Manifest identity.

---

# 49. ArtifactSet Aggregate v2

ArtifactSet aggregate MAY exist as a stable derived utility value.

It is not independent authority.

Conceptually:

```text
artifact_set_aggregate_v2
=
AggregateV2(Manifest contents)
```

One valid Manifest has exactly one correct derived aggregate.

---

## 49.1 Derived encoding

The stable utility format SHOULD bind:

```text
domain separator

path profile ID

digest profile ID

entry count

artifact kind

canonical path components

hashed byte count

digest algorithm

digest bytes
```

---

## 49.2 No competing authority

Lifecycle Records MUST bind:

```text
manifest_id
```

rather than independently supplied Manifest plus aggregate identities.

A lifecycle Record MUST NOT be accepted or rejected on the basis of an aggregate value supplied independently of the Manifest.

---

## 49.3 Projection mismatch

If a cached aggregate disagrees with recomputation from the authoritative Manifest:

```text
DERIVED_AGGREGATE_CACHE_MISMATCH
```

SHOULD be reported.

Manifest authority remains unchanged.

---

# 50. Fresh Freeze Attempt

Freeze is TWO_PHASE.

Required order:

```text
generate freeze_attempt_id
↓
derive intended root
↓
acquire attempt coordination
↓
append FREEZE_ATTEMPT_STARTED
↓
exclusive-create root
↓
exclusive-create first payload
↓
construct payload
↓
construct Manifest
↓
required verification
↓
Freeze Receipt
↓
durability operations
↓
FREEZE_COMMITTED
```

START occurs before root creation.

The intended root identity MUST therefore be known before root creation and MUST be carried in the START Journal Entry.

---

# 51. Attempt-Derived Root

The authoritative Freeze root MUST use a deterministic mapping from `freeze_attempt_id`.

Conceptually:

```text
roots/<freeze-attempt-id>/
```

If the derived root already exists:

```text
FAIL_CLOSED
```

The Attempt identity is not silently regenerated.

This guarantees that every root created by a conforming Freeze workflow corresponds to a previously Journaled Attempt.

---

# 52. Fresh Root Creation

Root creation MUST require that the root did not previously exist.

A prior existence check MAY be observational Evidence.

It MUST NOT substitute for exclusive creation.

This is a Profile L coordination and integrity primitive.

It is not a security boundary against the excluded same-principal attacker.

---

# 53. First Payload Rule

The first payload MUST use create-new / exclusive semantics.

Existing files MUST NOT be truncated or reused.

Failure keeps the Attempt nonauthoritative.

---

# 54. Freeze Receipt

Freeze Receipt minimum semantics include:

```text
schema_version

freeze_attempt_id
freeze_id

attempt_start_journal_ref

subject_id
manifest_id

custody_mode

creation_profile

filesystem / path profile

policy_ref

payload durability facts

requested commit durability

created_by_tool_version
```

The Receipt MUST bind the exact `FREEZE_ATTEMPT_STARTED` Journal Reference.

`FREEZE_COMMITTED` MUST reject a Receipt whose start reference:

* belongs to another Registry history;
* points to the wrong event type;
* points to another Freeze Attempt;
* cannot be reproduced from the retained Journal;
* has already reached a terminal state.

---

# 55. Freeze Authority

A valid Freeze Receipt is necessary but insufficient.

Authority requires:

```text
valid Freeze Receipt
+
valid FREEZE_COMMITTED
+
legal transition from exact START
```

Therefore:

```text
Receipt exists
but no COMMIT
→ NONAUTHORITATIVE
```

A Receipt copied to another Registry does not recreate prior authority.

---

# 56. Freeze Crash Recovery

Example:

```text
FREEZE_ATTEMPT_STARTED
↓
payload
↓
crash
```

Recovery:

```text
preserve root
↓
FREEZE_ABORTED_RECOVERY
↓
fresh successor Attempt
```

A crash after Freeze Receipt publication but before `FREEZE_COMMITTED` produces the same result.

The aborted Attempt MUST NOT later COMMIT.

v0.x MUST NOT expose a normal command that converts a crash-abandoned Freeze Attempt into an authoritative Freeze by adding the missing Receipt or Journal event later.

---

# 57. Immutable File Publication

Preferred protocol:

```text
create temporary file in target directory
using create-new

↓
write complete bytes

↓
flush file content

↓
publish final name using atomic no-replace operation

↓
flush parent directory metadata
where supported and configured
```

If a required primitive cannot be provided:

```text
UNSUPPORTED_ENVIRONMENT
```

or fail-closed result is required.

---

# 58. Durability Facts

Durability MUST be represented as observed facts rather than a vague success boolean.

Example fields:

```text
file_content_flush:
  PERFORMED
  NOT_PERFORMED
  UNSUPPORTED

atomic_publish_no_replace:
  PERFORMED
  NOT_PERFORMED
  UNSUPPORTED

parent_directory_flush:
  PERFORMED
  NOT_PERFORMED
  UNSUPPORTED

platform_strongest_available:
  true / false
```

Policy MAY require a minimum profile.

---

# 59. Durability Self-Report Limitation

A Record cannot prove a post-publication durability action on its own final directory entry from bytes created before that action.

EvidenceRegistry MUST NOT fake such self-attestation.

A later immutable observation MAY record stronger completed durability facts.

A missing later observation MUST conservatively mean that stronger durability was not established from retained Evidence.

---

# 60. Filesystem Safety

Freeze MUST fail closed on unsupported or ambiguous filesystem objects.

At minimum handle:

* regular files;
* directories;
* symlinks;
* reparse points;
* junctions;
* hardlinks;
* sockets;
* device nodes;
* named pipes;
* unreadable entries;
* alternate data streams where relevant.

Directory handling is defined in §37.1.

---

# 61. Unreadable Entry Rule

Permission denial, sharing denial, transient read failure, or other unreadable Artifact condition MUST NOT be silently skipped.

Freeze fails closed.

---

# 62. Symbolic Links

Default profile SHOULD reject symbolic links.

If explicitly allowed by Policy:

```text
artifact_kind = SYMLINK
```

Digest the link's own lossless target representation.

EvidenceRegistry MUST NOT follow the link to hash target content as though it were the linked path's own regular-file bytes.

Directory symbolic links MUST NOT be recursively traversed.

---

# 63. Reparse Points and Junctions

Generic redirecting Windows reparse structures MUST be rejected unless exact semantics are separately specified.

Permission to handle symbolic links does not imply permission for every reparse type.

---

# 64. Hardlinks

Distinct paths referring to the same mutable filesystem object MUST be detected where supported.

Default profile SHOULD reject this aliasing.

A future explicit hardlink-preserving Artifact model MAY define stronger semantics.

---

# 65. Alternate Data Streams

Where technically detectable, unexpected alternate data streams SHOULD be detected.

A profile claiming complete file identity MUST NOT silently ignore streams that materially exist within its claimed scope.

---

# 66. Custody Modes

At least:

```text
EMBEDDED
REFERENCED
```

The custody mode MUST be explicit.

REFERENCED Evidence MUST NOT be presented as providing the same retention guarantee as EMBEDDED Evidence.

---

# 67. EMBEDDED Custody

For EMBEDDED custody, retained bytes are the canonical frozen byte source.

Required order:

```text
read source
↓
create retained copy
↓
hash retained copy
↓
construct Manifest from retained copy
↓
verify retained copy
↓
separately compare source with copy-derived Manifest
↓
commit Freeze
```

Manifest identity for EMBEDDED custody is therefore derived from the retained embedded bytes.

Source binding remains separate from retained-copy integrity.

A source mismatch MUST NOT silently rewrite the retained copy or the Manifest.

---

# 68. REFERENCED Custody

Referenced bytes remain outside the Evidence package.

A non-authoritative locator MAY be retained:

```text
subject_locator_hint
```

It does not participate in Subject content identity.

If the external bytes later disappear:

```text
historical Evidence remains
current re-verifiability becomes UNKNOWN / UNAVAILABLE
```

The old Evidence does not automatically become false.

---

# 69. Source Binding Record

A Freeze intended for later source matching MUST create an immutable Source Binding Record.

Minimum semantics:

```text
freeze_attempt_id

source_locator_hint

initial_source_object_identity

path_identity_profile_id
```

The locator is provenance only.

---

# 70. Verification Target Types

At minimum:

```text
EMBEDDED_COPY_INTEGRITY

SOURCE_SUBJECT_MATCH
```

## EMBEDDED_COPY_INTEGRITY

> The retained embedded copy matches the recorded Manifest and digests.

This says nothing by itself about whether the external source has changed.

## SOURCE_SUBJECT_MATCH

> The source bound by the Source Binding Record was observed to match the frozen identity during this Verification operation.

A Policy requiring `SOURCE_SUBJECT_MATCH` MUST NOT be satisfied by `EMBEDDED_COPY_INTEGRITY`.

---

## 70.1 Historical target vocabulary

Earlier specifications in this line used the target name:

```text
SOURCE_SUBJECT_NODRIFT
```

That name is retired.

Two rules apply, and they are distinct.

**Interpretation rule.** The historical name MUST NOT be interpreted as establishing continuous immutability between observations. It never established more than a point-in-time match.

**Compatibility rule.** Per §0, a Record created under a predecessor format and carrying the target value `SOURCE_SUBJECT_NODRIFT` MUST NOT be silently reinterpreted as carrying `SOURCE_SUBJECT_MATCH`.

A v0.10.2 Policy requiring `SOURCE_SUBJECT_MATCH` is therefore NOT satisfied by a predecessor-format Verification Record carrying the retired name, unless an explicit compatibility specification authorizes that mapping.

In the absence of such a specification, the correct result is:

```text
UNSUPPORTED_SCHEMA
```

or another explicit fail-closed disposition.

---

# 71. Freeze-Time Source Stability

A live filesystem scan is not an atomic snapshot.

REFERENCED Freeze MUST perform at least:

```text
inventory + hash
↓
Manifest construction
↓
second complete verification against Manifest
```

Mismatch fails closed.

For EMBEDDED Freeze claiming binding to an external source Subject, the same source comparison is required separately from integrity verification of the embedded copy.

This procedure reduces ordinary race and drift risk.

It MUST NOT be described as equivalent to an operating-system atomic snapshot unless such a primitive is explicitly used and recorded.

---

# 72. Verification Modes

## 72.1 Read-only

```text
evidence-registry verify
```

MUST be read-only.

It:

* reads the Registry;
* reads the selected verification target;
* performs mechanical verification;
* emits a result;
* MAY emit a nonauthoritative portable Record of type `VERIFICATION_OBSERVATION`;
* MUST NOT append to the Authority Journal;
* MUST NOT make its result Registry-authoritative.

---

## 72.2 Recorded

```text
evidence-registry verify --record
```

performs a fresh verification and creates:

```text
VERIFICATION_RECORD
+
VERIFICATION_RECORDED
```

`--record` is an authoritative writer operation.

Its Journal publication MUST follow normal Journal append serialization and storage-capability requirements.

---

## 72.3 No promotion

A `VERIFICATION_OBSERVATION` MUST NOT be promoted into a `VERIFICATION_RECORD`.

EvidenceRegistry MUST NOT expose an ordinary workflow equivalent to:

```text
import old VERIFICATION_OBSERVATION
↓
change record_type
↓
journal it as current Verification
```

A historical Observation MAY be referenced as informational Evidence.

Recording an authoritative Verification requires a fresh evaluation.

---

## 72.4 Negative results are not command failures

A recorded verification MAY authoritatively record:

```text
NO_BLOCKING
BLOCKING
INDETERMINATE
```

where the Method state permits such a result.

Recording a negative or indeterminate result MUST NOT be treated as a command failure merely because the evaluated property did not succeed.

See §163.

---

# 73. Verification Observation Interval

Before a recorded verification begins:

```text
observation_start_journal_ref
```

MUST capture the Registry head.

The final:

```text
VERIFICATION_RECORDED
```

event provides the upper bound.

Thus every authoritative Verification has a Registry-history observation interval:

```text
[
  observation_start_journal_ref,
  verification_recorded_journal_ref
]
```

---

## 73.1 Meaning

This establishes:

> This Verification operation began after the referenced Registry history and completed when this Verification became authoritative.

It does NOT establish:

* that the Subject remained unchanged through the interval;
* wall-clock recency;
* that the verification took little wall-clock time.

---

# 74. Verification Record

Minimum lifecycle fields:

```text
schema_version
record_type = VERIFICATION_RECORD

registry_id

verification_target_type

observation_start_journal_ref

freeze_authority_ref
source_binding_ref

manifest_id

examined_source_object_identity
examined_source_locator

path_profile_interpretation

method_status
finding_state
reason_codes[]

coverage_limitations[]

storage_capability_class_id
environment_observation_id

created_by_tool_version

brackets_closeout_authority_ref
  only when structurally applicable
```

`examined_source_object_identity` records the platform object identity actually observed, or `UNKNOWN` where unavailable.

`examined_source_locator` is provenance only and MUST NOT participate in Subject content identity.

A `SOURCE_SUBJECT_MATCH` result MUST reference the exact immutable Source Binding Record.

An arbitrary directory supplied at verification time MUST NOT silently become the original Source.

If the original source association cannot be established:

```text
finding_state = INDETERMINATE
```

---

# 75. Journal-Distance Freshness

Policy MAY constrain:

```text
max_journal_distance_to_closeout
```

Meaning:

```text
closeout_journal_index
-
verification_recorded_journal_index
<= N
```

subject to the exact Policy definition.

This constrains intervening Registry history.

It MUST NOT be described as wall-clock freshness.

If no Journal events occur for a long wall-clock interval, Journal distance may remain zero.

EvidenceRegistry MUST NOT translate Journal-distance requirements into statements such as:

```text
verified within 10 minutes
verified recently
fresh as of today
```

without a separately specified trusted-time mechanism.

---

## 75.1 Intervening-event Policy constraint

Separately from Journal distance, a Policy MAY require:

```text
no relevant Registry event
within the Verification observation interval
```

Relevant event classes MUST be defined mechanically by Policy.

The interval evaluated is the one defined in §73.

Such a requirement MUST be checked using Journal-native lifecycle and authority indexing.

It MUST NOT be inferred from physical entry adjacency, and EvidenceRegistry MUST NOT use timestamps to determine the interval.

---

# 76. Review Request

Review Request minimum semantics:

```text
schema_version
record_type

subject / Freeze authority binding
manifest_id

review_role

required_checks_ref

policy_ref

review_scope_ref

review_method_ref

review_package_anchor

operation_start_journal_ref
```

The reviewer MUST NOT infer Subject identity from directory names, nearby files, or surrounding conversation.

---

# 77. Review Roles

Supported roles MAY include:

```text
HOSTILE
INDEPENDENT
OWNER
SPECIALIST
AUTOMATED
```

Role describes intended procedure.

It does not prove cryptographic independence.

---

# 78. Review Scope

Review Scope MUST be immutable and content-addressed.

v0.x uses exact matching.

Semantic subset/superset inference is prohibited unless a future explicit Scope Algebra defines it.

---

# 79. Review Methods and Checks

Mechanical Policy requirements MUST use:

* stable immutable identifiers whose meaning is versioned; or
* content-addressed Method / Check Records.

Free-form prose alone cannot satisfy a mechanical requirement.

Changing Method semantics requires a new Method identity, not reinterpretation of the old identifier.

---

# 80. Review Result

Minimum semantics:

```text
schema_version

review_request_id

subject / Freeze binding
manifest_id

review_role

review_scope_ref
review_method_ref

method_status
finding_state

reason_codes[]
findings[]

reviewer_metadata

review_package_anchor
```

Redundant role and Scope values MUST exactly match the Request.

Mismatch:

```text
REVIEW_ROLE_MISMATCH
REVIEW_SCOPE_MISMATCH
```

and Admission MUST fail.

---

# 81. Review Transport

Review semantics and Review transport remain separate.

A semantically performed Review may exist while transport identity is insufficient for Admission.

Transport failure does not automatically make the underlying Subject NOT_CLEAN.

---

# 82. Review Admission

Review existence is not Review Admission.

Admission MUST check:

* Request authority;
* exact Subject/Freeze binding;
* Manifest identity;
* Review role;
* Review Scope;
* Method;
* required Checks;
* Result Record integrity;
* Policy identity;
* returned Journal Anchor;
* method status requirements.

Missing required identity MUST NOT be reconstructed from prose.

---

# 83. Admission Outcomes

Every completed authoritative Admission operation produces:

```text
REVIEW_ADMISSION_ACCEPTED
```

or:

```text
REVIEW_ADMISSION_REJECTED
```

Both remain history.

Repeated Admission attempts remain separately visible.

Admission is ONE_PHASE.

A crash before terminal publication creates no authoritative Admission disposition, and the Result MAY be submitted again.

That is not silent rewriting, because no authoritative prior disposition existed.

---

## 83.1 Failure-recording boundary

The historical threshold for Review Admission is:

```text
supported Review Result envelope parsed
+
candidate Review Result record_id successfully recomputed
+
terminal disposition determined
```

Once a terminal disposition is determined, it MUST be published as an immutable Admission Record with its Journal event.

Failures before that point — unreadable file, malformed CBOR, unsupported envelope, identity not reconstructible — MAY remain ordinary command/input errors.

Implementations MUST NOT dynamically move this threshold to avoid preserving inconvenient failure history.

---

# 84. Project Policy

Policy MUST itself be immutable and content-addressed.

A mutable path is not authoritative Policy identity.

Human YAML or JSON MAY be input.

Authoritative use requires deterministic Policy Record identity.

---

# 85. Policy Requirements

The normative Policy schema MAY express mechanically supported requirements including:

```text
Review role

Review Scope

Review Method

required_method_status

allowed finding state

Review count

Verification target

Verification authority

intervening-event constraint

max_journal_distance_to_closeout

minimum durability

Journal Anchor requirements

required_bootstrap_scope_ref

required_formal_finding_classification

Establishment diversity

Closeout postconditions
```

A field MUST NOT appear in the normative supported Policy schema if EvidenceRegistry parses but does not enforce it.

The converse also holds: a requirement that EvidenceRegistry enforces MUST be expressible in the normative Policy schema rather than existing only as implementation behavior.

---

## 85.1 required_formal_finding_classification

Where a Policy accepts formal-verification Evidence, it MAY require that a raw formal result be accompanied by an authoritative Formal Finding Classification before a product-level conclusion may be drawn.

Conceptually:

```yaml
required_formal_finding_classification:
  required: true
  acceptable_classifications:
    - FORMAL_MODEL_DEFECT
    - PROPERTY_UNDERSPECIFIED
  classifier_procedure_identity_allowed:
    - <content-addressed procedure identity>
```

Evaluation semantics are defined in §145.

---

# 86. Default Method Requirement

Default success Policy SHOULD require:

```text
required_method_status:
  - VALID
```

A Policy MAY explicitly allow `ENVIRONMENTALLY_DEGRADED`, but only through an explicit rule.

There MUST be no implicit degraded-environment success.

Therefore:

```text
ENVIRONMENTALLY_DEGRADED
+
NO_BLOCKING
```

does not satisfy the default success gate, and the derived human verdict `UNKNOWN` (§7.3) remains consistent with the mechanical Policy disposition.

---

# 87. Establishment Diversity Policy

Policy MAY express:

```yaml
required_establishment_diversity:
  - dimension: implementation_identity
    distinct_count: 2
  - dimension: host_identity
    distinct_count: 2
```

Only registered machine-readable dimensions may be used.

A rule is satisfied only when the admitted Establishments contain at least `N` distinct authoritative values for the named dimension.

Missing identity values MUST NOT count as automatically distinct.

They yield:

```text
DIVERSITY_REQUIREMENT_UNSATISFIED
```

unless Policy explicitly permits unknown values.

Free-form statements such as "use independent implementations" MUST NOT satisfy a normative diversity requirement.

The detailed meaning of these diversity identities belongs to the Formal Verification specification.

---

# 88. Closeout

A Closeout records a disposition over exact authoritative support.

Minimum lifecycle fields SHOULD include:

```text
schema_version

subject / Freeze authority
manifest_id

admitted_review_authority_refs[]

creation_time_verification_authority_refs[]

pre_verification_authority_ref
  where Policy requires a bracket

policy_ref

required_bootstrap_scope_ref
  where Policy requires Bootstrap Trust

predecessor_closeout_id

formal_support_binding
  where applicable

bootstrap_trust_declaration_authority_refs[]
  where applicable

disposition

operation_start_journal_ref
```

A Closeout MUST NOT forward-reference future Evidence.

---

## 88.1 Bootstrap scope field location

The **requirement** originates in Policy (§85).

The **exact value satisfied** is recorded on the Closeout as `required_bootstrap_scope_ref`, so that the Closeout's own support snapshot is self-contained and does not depend on later reinterpretation of a Policy Record.

Both copies MUST match exactly.

Per §6.6:

```text
Policy requires Bootstrap Trust
    → required_bootstrap_scope_ref MUST be present

Policy does not require Bootstrap Trust
    → required_bootstrap_scope_ref MUST be absent
```

Validation of the pairing occurs at Closeout creation (§137).

---

# 89. Closeout Rejection

A mechanically completed rejected Closeout creates:

```text
CLOSEOUT_REJECTED
```

and remains historical.

A later successful Closeout MUST NOT erase prior rejected attempts.

---

# 90. Bracketed Source Observation

Where Policy requires PRE and POST source observations:

```text
PRE Verification
↓
Closeout
↓
POST Verification
```

The semantics are:

> The bound source matched the required frozen identity at both authoritative observation points bracketing the Closeout.

This does NOT establish continuous immutability between them.

Journal adjacency is NOT required.

Unrelated Journal events MAY occur between them.

---

# 91. PRE Verification

The Closeout references exact authoritative PRE Verification through:

```text
pre_verification_authority_ref
```

The PRE Verification MUST already be authoritative.

This is an AUTHORITY_DEPENDENCY.

---

# 92. POST Verification

The POST Verification references the exact Closeout through:

```text
brackets_closeout_authority_ref
```

and MUST have an AUTHORITY_DEPENDENCY on the exact `CLOSEOUT_COMMITTED` event.

This backward dependency is chronologically valid.

No separate bracket-completion event is required in v0.x.

The POST Verification event itself is sufficient Evidence.

---

# 93. Bracket State Is Policy-Relative

Bracket state is not a free-standing property of a Closeout.

Define:

```text
BracketState(Closeout C, Policy P)
```

Possible derived states:

```text
NOT_REQUIRED

PENDING_POSTCONDITION

SATISFIED

FAILED
```

A Closeout binds exactly one normative `policy_ref`, so the pair `(C, P)` is mechanically fixed.

The two-argument form is retained so that future formal or API layers cannot treat bracket semantics as intrinsic to a Closeout independently of Policy.

---

## 93.1 Claim discipline

A Closeout Record itself MUST NOT claim bracket completion, because that fact did not exist when the Closeout was created.

Immediately after `CLOSEOUT_COMMITTED` under a bracket-requiring Policy, the derived state is:

```text
PENDING_POSTCONDITION
```

`inspect`, `audit`, export, or Claim-layer operations MAY derive the completed state from retained history.

---

# 94. POST Eligibility

Eligibility is structural.

It MUST NOT depend on whether the result is favorable.

Structural requirements include:

* exact Closeout reference;
* correct Freeze authority;
* correct Source Binding;
* correct verification target type;
* correct Policy context;
* production after Closeout authority;
* valid chronology;
* structural Record validity.

The following MUST NOT determine eligibility:

```text
method_status
finding_state
whether Policy success would result
```

Otherwise an implementation could classify an unfavorable POST as "ineligible" and continue searching until a favorable one appears.

Eligibility answers: *is this the structurally relevant POST observation?*

Policy evaluation answers: *did that decisive POST satisfy the required condition?*

---

# 95. First Eligible POST Is Decisive

For one Closeout and Policy:

```text
FirstEligiblePost(C, P)
```

is the eligible authoritative POST with the lowest Journal index.

That POST alone determines bracket success or failure.

Retry-until-success semantics are forbidden.

---

## 95.1 Success

If the decisive POST satisfies Policy:

```text
SATISFIED
```

---

## 95.2 Failure

If the decisive POST does not satisfy Policy:

```text
FAILED
```

and that state is terminal for the exact Closeout.

There is no `FAILED → SATISFIED` transition for the same Closeout.

Later successful Verification remains valid historical Evidence.

It does not rehabilitate the original Closeout.

A successor Closeout is required.

---

# 96. Successor Rule

Correction, clarification, replacement, or requalification uses a successor.

Never mutate the predecessor.

```text
v1 failure
→
v2 success
```

remains exactly that history.

It MUST NOT become:

```text
v1 success
```

---

# 97. Non-Authoritative Evidence

Valid portable Records MAY exist without Registry authority.

Examples:

```text
VERIFICATION_OBSERVATION

valid unjournaled Record

partial root

historical contaminated Evidence
```

Presence under `records/` alone MUST NOT grant authority.

A nonauthoritative Record MAY remain useful historical Evidence.

It MUST NOT silently regain authority.

Authority can arise only through a legal new Journal transition.

---

# 98. Contaminated Root Rule

A root with uncertain prior state MUST NOT be repaired into fresh authority.

Preserve where useful.

Create a fresh successor Attempt.

The successor SHOULD explicitly reference the contaminated predecessor, the reason for contamination, and retained observations.

---

# 99. Artifact Byte Eviction

EMBEDDED Evidence may later require explicit byte removal.

Deletion of retained Evidence is a TWO_PHASE lifecycle.

Required structure:

```text
ARTIFACT_EVICTION_STARTED
↓
dependency arbitration
↓
deletion
↓
ARTIFACT_EVICTION_COMMITTED
```

or a terminal BLOCKED / INTERRUPTED state.

The historical fact that bytes were retained at Freeze time MUST NOT be rewritten because retained bytes are later removed.

---

# 100. Eviction Scope

Before START, the exact intended target scope must be known and bound:

```text
freeze_authority_ref
eviction_scope
pre_eviction_manifest_id
```

`eviction_scope` MUST mechanically identify exactly what is to be removed.

Examples:

```text
ALL_EMBEDDED_ARTIFACT_BYTES

explicit Artifact identity set
```

A vague statement such as "old files deleted" is insufficient.

---

# 101. Eviction Dependency Arbitration

`ARTIFACT_EVICTION_STARTED` establishes the ordering point visible to other Registry operations through Journal replay.

Therefore the dependency check MUST NOT occur only before START.

After START and immediately before the first destructive removal, EvidenceRegistry MUST determine whether the targeted bytes are required by a mechanically known open lifecycle dependency.

At minimum:

```text
pending Closeout postcondition requiring embedded bytes

open Verification operation with an explicit byte-retention
dependency, where such dependency is authoritatively
represented

another open eviction over overlapping bytes

other registered lifecycle dependency explicitly requiring
the target byte scope
```

If such a dependency exists:

```text
ARTIFACT_EVICTION_BLOCKED
```

MUST be recorded as a terminal non-destructive disposition.

No target byte may have been deliberately removed by that Attempt before this disposition.

If destructive removal already began, `ARTIFACT_EVICTION_BLOCKED` MUST NOT be used.

---

## 101.1 Dependency detection limitation

EvidenceRegistry detects only mechanically represented Registry dependencies.

It MUST NOT claim detection of:

```text
unknown external consumers
human expectations
unregistered copies
unregistered processes
external filesystem readers
```

The dependency safety property is Registry-history scoped.

---

# 102. Symmetric Closeout Check

Before creating a pending Closeout dependency on embedded bytes, Closeout creation MUST verify that no conflicting open eviction already owns the earlier lifecycle ordering point.

Ordering rule:

```text
Eviction START first
    → later Closeout cannot acquire a dependency
      on bytes targeted by that open eviction

Closeout pending-byte dependency first
    → later Eviction START may exist,
      but destructive deletion MUST be blocked
```

This eliminates the check-before-START race in both directions.

---

# 103. Eviction Crash Recovery

If crash occurs after deletion begins:

```text
ARTIFACT_EVICTION_INTERRUPTED_RECOVERY
```

records observed state:

```text
NONE_EVICTED
PARTIALLY_EVICTED
FULLY_EVICTED
INDETERMINATE
```

The interrupted event is terminal for that eviction Attempt.

---

# 104. Successor Eviction

If previous recovery is PARTIAL or INDETERMINATE, a successor eviction MUST first reconstruct exact currently retained bytes:

```text
enumerate retained embedded namespace
↓
reconcile against authoritative Manifest
↓
reconcile against completed / interrupted eviction history
↓
derive exact currently retained Artifact set
↓
construct successor eviction scope
```

A successor MUST NOT guess the remaining scope as:

```text
intended original scope
-
known deleted subset
```

when retained Evidence does not establish that result.

If remaining scope cannot be established exactly:

```text
destructive successor eviction MUST NOT begin
```

---

# 104.1 Eviction Historical Semantics

After eviction:

```text
historical Freeze identity
    remains authoritative

historical custody_mode = EMBEDDED
    remains true for Freeze time

current embedded-byte availability
    = UNAVAILABLE / PARTIALLY_AVAILABLE
```

Loss of present re-verifiability MUST NOT retroactively make the historical Freeze INVALID.

If embedded bytes disappear without a prior authoritative eviction event:

```text
MISSING_PAYLOAD
```

or equivalent unexpected Evidence loss MUST be reported.

Audit MUST distinguish expected recorded eviction from unexpected payload loss.

---

# 105. Storage Capability Class

Storage mechanics MUST use an immutable:

```text
STORAGE_CAPABILITY_CLASS
```

Record.

Each capability value uses:

```text
PRESENT
ABSENT
NOT_PROBED
```

Its content-addressed identity is `storage_capability_class_id`.

Policy gating and material change detection operate on this identity and its structured capability fields.

---

# 106. Capability Semantics

## PRESENT

A conclusive supported observation, or a normatively accepted platform fact, establishes availability under the observed storage context.

## ABSENT

The capability was evaluated and found unavailable.

This MUST NOT be used for:

```text
probe failed
probe unsupported
probe not executed
result ambiguous
```

## NOT_PROBED

No conclusive evaluation exists.

Reasons MAY include probe unsupported, probe skipped, probe failed before disposition, insufficient permissions, or an implementation that does not provide the probe.

Detailed reasons belong in Environment Observation or probe provenance.

---

# 107. Storage Capability Fields

Candidate fields include:

```text
filesystem_transport

sync_management

placeholder_capability

exclusive_create_capability

no_replace_publication_capability

locking_capability

atomic_rename_capability

file_flush_capability

directory_flush_capability
```

---

# 108. Environment Observation

Descriptive provenance belongs in a separate:

```text
ENVIRONMENT_OBSERVATION
```

Record with identity `environment_observation_id`.

Examples:

* OS name/version;
* filesystem-reported name;
* driver details;
* mount details;
* probe tool version.

Descriptive change alone does not prove capability change.

---

# 109. Material Capability Change

A proven material transition, for a given capability, is:

```text
PRESENT → ABSENT
ABSENT  → PRESENT
```

The following alone are NOT material capability changes:

```text
NOT_PROBED → PRESENT
NOT_PROBED → ABSENT
PRESENT    → NOT_PROBED
ABSENT     → NOT_PROBED
```

because these may represent changes in observation strength rather than proven changes in underlying storage behavior.

A transition involving `NOT_PROBED` SHOULD be surfaced separately as observation-strength provenance.

Policy MAY still reject a `NOT_PROBED` state, but MUST do so because the Policy requires stronger observation — not because EvidenceRegistry falsely claims a proven storage capability transition.

Note that `storage_capability_class_id` inequality does not by itself imply material change; materiality uses the transition rule above.

---

# 110. STORAGE_CAPABILITY_CHANGED

Each authoritative writer operation MUST use a relevant current storage capability observation before final Journal commitment.

If a material capability change is established:

```text
STORAGE_CAPABILITY_CHANGED
```

MUST become authoritative before or as part of continuing the requested workflow.

Its lifecycle object is:

```text
lifecycle_object_kind = REGISTRY
lifecycle_object_id   = registry_id
```

Policy MAY permit, warn on, or reject specific capability transitions.

---

## 110.1 No recursive observation requirement

Creation of `STORAGE_CAPABILITY_CLASS` and `ENVIRONMENT_OBSERVATION` Records is exempt from the requirement to perform another storage capability observation for their own creation.

Otherwise the model would recurse indefinitely.

These Records acquire lifecycle significance only when referenced by an authoritative Journal event.

---

## 110.2 Historical meaning of the initial observation

The GENESIS storage capability observation represents:

```text
storage environment observed at initialization
```

not:

```text
storage environment guaranteed for the lifetime of the Registry
```

---

# 111. Capability Probe Namespace

Capability probes MUST use a nonauthoritative namespace such as:

```text
runtime/probes/
```

They MUST NOT create temporary probe files in:

```text
journal/
records/
roots/
```

---

# 112. Probe Cache

Within:

```text
same process
+
same mount/volume identity
+
same relevant Registry storage context
```

probe results MAY be reused.

A probe cache entry MUST retain, where available:

```text
mount_identity
volume_identity
resolved_registry_storage_identity
```

Cache reuse ends when the relevant storage identity changes, including:

```text
mount identity differs
volume identity differs
resolved filesystem transport differs
sync-management classification differs
```

A Registry move matters exactly when it causes the storage identity or capability context used by the cached result to no longer match.

No separate detection of "Registry root moved" independent of those observations is required.

Process restart SHOULD invalidate the default v0.x probe cache unless a stronger persistent probe-cache contract is separately specified.

---

## 112.1 No lifetime overclaim

A cached probe result means:

> EvidenceRegistry reused a capability observation considered valid under its declared cache scope.

It MUST NOT mean:

> The filesystem capability was continuously re-proven before every Journal append.

This bounds §110.

Where a Policy requires stronger observation freshness than the declared cache scope provides, that requirement MUST be expressed in Policy rather than assumed from §110 alone.

---

# 113. Probe Residue

Probe residue is nonauthoritative operational state.

EvidenceRegistry SHOULD clean it:

* after normal probe completion;
* during later startup/maintenance.

Stale residue MAY be classified by Audit as:

```text
STALE_PROBE_RESIDUE
```

It MUST NOT be interpreted as an Evidence Record or a Subject Artifact.

Cleanup does not rewrite Evidence history.

Failure to remove residue SHOULD be reported operationally but MUST NOT alter Registry authority.

---

# 114. Network and Synchronized Storage

EvidenceRegistry MUST NOT assume local-filesystem semantics on:

* NFS;
* SMB;
* cloud-synchronized folders;
* placeholder/on-demand storage;
* unknown transport.

`init` and authoritative operations SHOULD observe such facts and record them.

If required primitives cannot be established:

```text
UNSUPPORTED_ENVIRONMENT
```

is preferred to false equivalence.

Filesystem locality alone MUST NOT be treated as proof that a primitive is reliable.

---

# 115. Journal Anchor

An externally portable Journal Anchor contains:

```text
schema_version

registry_id

journal_head_index

journal_head_hash

journal_format_version
```

It uses deterministic domain-separated encoding.

Conceptually:

```text
anchor_id =
SHA256(
  deterministic_cbor(
    [
      "EvidenceRegistry.JournalAnchor.v1",
      schema_version,
      anchor_body
    ]
  )
)
```

An Anchor is a portable historical commitment.

It is not automatically a Registry-authoritative Record.

It is not itself proof that the external holder is trustworthy.

Every reliance-oriented artifact exported from the Registry MUST include a Journal Anchor, including Audit Results, Review Packages, Registry exports, Closeout export packages, bootstrap packages, and machine-readable qualification receipts.

Human-only transient diagnostic output MAY omit the Anchor where no relying party is expected to preserve it.

---

# 116. Anchor Comparison

Possible results:

```text
ANCHOR_EQUALS_CURRENT_HEAD

ANCHOR_IS_VALID_ANCESTOR

JOURNAL_DIVERGENCE

JOURNAL_HISTORY_BEHIND_ANCHOR

ANCHOR_FROM_DIFFERENT_REGISTRY

ANCHOR_INVALID
```

---

## 116.1 Same head

Same index and same hash.

## 116.2 Valid ancestor

Current Journal is longer and reproduces the Anchor hash at the anchored index.

## 116.3 Divergence

Current Journal contains an Entry at the anchored index but the hash differs.

Divergence is thereby established.

## 116.4 History behind Anchor

Current Journal ends before the Anchor index.

The standalone Anchor does not prove whether the current Registry is a rolled-back prefix of the anchored history, or whether divergence occurred after the current head.

Therefore the result MUST be:

```text
JOURNAL_HISTORY_BEHIND_ANCHOR
```

EvidenceRegistry MUST NOT label this case specifically as:

```text
ROLLBACK
```

without additional historical proof.

## 116.5 Different Registry

If `registry_id` differs:

```text
ANCHOR_FROM_DIFFERENT_REGISTRY
```

No ancestry comparison is implied.

---

# 117. Review Anchor Round Trip

Review Package MUST include the current Journal Anchor.

Review Result MUST preserve it unchanged.

Admission MUST compare the returned Anchor with current history.

Acceptable:

```text
ANCHOR_EQUALS_CURRENT_HEAD
ANCHOR_IS_VALID_ANCESTOR
```

Potentially blocking:

```text
JOURNAL_DIVERGENCE
JOURNAL_HISTORY_BEHIND_ANCHOR
ANCHOR_INVALID
```

The Policy response MUST be fail-closed by default.

This automatically exercises Anchor-relative historical consistency in ordinary Review workflow, so users need not remember to compare Anchors manually.

Any workflow that exports Registry-bound material for external processing and later admits it authoritatively SHOULD use the same Anchor round-trip model.

---

# 118. Registry Duplication

A complete copy of a Registry copies:

```text
registry_id
GENESIS
Journal history
```

Before histories diverge, Profile L cannot distinguish one Registry accessed at two locations from two physical copies.

EvidenceRegistry MUST NOT claim detection of duplication before divergence.

Once histories diverge, a surviving Anchor from one branch MAY expose the divergence when compared against the other.

Physical Registry uniqueness requires a stronger external authority or attestation profile.

---

# 119. Audit

`audit` MUST be read-only.

It evaluates:

```text
Journal integrity

Record integrity

root inventory

lineage

chronology

Review bindings

Closeout bindings

Policy bindings

filesystem aliases

Artifact availability

Anchor relations

support-record authority
```

within exact declared coverage.

---

# 120. Audit Coverage Record

Audit results MUST bind actual coverage.

At minimum:

```text
registry_id

journal_head_index
journal_head_hash

journal_entry_count

scanned_root_count
scanned_root_identity_set

root_inventory_digest

coverage_limitations[]
```

Filesystem locator strings MAY be emitted as nonauthoritative hints.

---

# 121. Coverage-Bound Negative Conclusions

A statement:

```text
no historical rewrite detected
```

means only:

> No such defect was detected within this exact Audit coverage.

It MUST NOT mean no undiscovered root or Evidence exists elsewhere.

---

# 122. Audit Root Discovery

Audit MUST use both:

```text
Journal-derived expected roots
+
filesystem enumeration of authoritative root namespace
```

It MUST detect:

```text
root with no corresponding START

expected root missing

unexpected authoritative namespace object
```

A conforming v0.x Freeze workflow should not normally produce an unjournaled root, because START precedes root creation.

Audit MUST still detect such structures, because older implementations, manual modification, partial migration, or external tools may create them.

---

# 123. Record Namespace Classification

Audit MUST distinguish:

```text
VALID_NONAUTHORITATIVE_RECORD

AUTHORITATIVE_RECORD

MALFORMED_RECORD

UNKNOWN_OBJECT_IN_RECORD_NAMESPACE

AUTHORITATIVE_RECORD_PAYLOAD_MISSING
```

A valid unjournaled Record is not automatically corruption.

Its presence alone does not grant authority.

---

# 124. Initial Audit Detection Classes

At minimum:

```text
IDENTITY_DRIFT

MISSING_PAYLOAD
EXTRA_PAYLOAD

MANIFEST_MISMATCH

DERIVED_AGGREGATE_CACHE_MISMATCH

JOURNAL_GAP
JOURNAL_FORK
JOURNAL_HASH_MISMATCH
JOURNAL_ILLEGAL_TRANSITION
JOURNAL_DIVERGENCE
JOURNAL_HISTORY_BEHIND_ANCHOR

REVIEW_SUBJECT_MISMATCH
REVIEW_ROLE_MISMATCH
REVIEW_SCOPE_MISMATCH
REVIEW_METHOD_MISMATCH

PREDECESSOR_MISSING
SUCCESSOR_BINDING_INVALID

CHRONOLOGY_UNPROVEN
INVALID_CREATION_CHRONOLOGY

PARTIAL_ROOT_REUSE
IN_PLACE_REWRITE

CLOSEOUT_WITH_UNADMITTED_REVIEW

NONAUTHORITATIVE_PROMOTION

UNSUPPORTED_FILESYSTEM_OBJECT
CANONICAL_PATH_COLLISION
FILESYSTEM_OBJECT_ALIAS

STALE_PROBE_RESIDUE

POLICY_IDENTITY_MISMATCH

BOOTSTRAP_SCOPE_MISMATCH

AUDIT_COVERAGE_INCOMPLETE
```

Audit MUST additionally distinguish:

```text
expected Journal root  vs  expected root absent

recorded Artifact eviction  vs  unexpected Artifact disappearance

valid portable unjournaled Record  vs  malformed Record

Storage Capability transition  vs  unexplained environment mismatch

externally anchored rollback  vs  unanchored rollback unknowable
```

Cross-Registry identity laundering heuristics MAY remain future work.

---

# 125. Authoritative Storage

Authoritative truth consists of:

```text
immutable Record files

+
immutable retained Artifact Evidence

+
Authority Journal
```

A database MAY serve as:

* index;
* search projection;
* UI projection;
* cache;
* report store.

It MUST NOT redefine authority.

---

# 126. Projection Independence

If projection and authoritative derivation disagree:

```text
Journal + authoritative Records win
```

Projection MUST be rebuilt, repaired, or discarded.

Its disagreement MUST NOT change historical Evidence.

---

# 127. Claim Registry Boundary

Evidence remains distinct from Claims.

```text
Evidence
↓
Property evaluation
↓
Claim authorization
```

Evidence existence alone MUST NOT authorize external product wording.

---

# 128. Assumption Definition Authority

Formal-verification layers MAY define immutable Assumption Definition Records.

Authority requires:

```text
ASSUMPTION_DEFINITION_RECORDED
```

A portable definition without Journal authority has no Registry authority.

Detailed Assumption semantics belong to the Formal Verification specification.

---

# 129. Assumption Establishment Authority

An Assumption Establishment requires:

```text
ASSUMPTION_ESTABLISHMENT_RECORDED
```

and MUST bind, as an AUTHORITY_DEPENDENCY, the exact authoritative Assumption Definition being evaluated.

It MAY bind Scope, storage capability class, environment observation, Method, Evidence, capability epoch, and diversity metadata as defined by the Formal Verification specification.

An Establishment existing only under `records/` has no Registry authority.

---

# 130. Assumption Invalidation

Later Evidence may create:

```text
ASSUMPTION_INVALIDATION_RECORDED
```

This does not rewrite historical Establishment bytes.

Target modes MAY include:

```text
EXPLICIT_ESTABLISHMENT_SET

ATTRIBUTE_PREDICATE
```

Predicate evaluation MUST be versioned and coverage-bound.

Free-form prose MUST NOT determine affected Establishments.

By default in v0.x, an invalidation applies only to Establishments authoritative at or before its Journal position.

It MUST NOT automatically govern future Establishments.

A future affected Establishment requires a successor invalidation or another explicit authoritative invalidation Record.

This prevents an old immutable Record from acting as a mutable future policy rule.

---

# 131. Formal Verification Authority

A Formal Verification Record acquires authority through:

```text
FORMAL_VERIFICATION_RECORDED
```

It MUST bind exact formal Subject, model, theorem, dependency-manifest, and toolchain identities required by the separate Formal Verification specification.

A portable formal report has no Registry authority merely by existing.

---

# 132. Assumption Version Compatibility

Cross-version Assumption reuse requires an exact authoritative:

```text
ASSUMPTION_VERSION_COMPATIBILITY_RECORDED
```

Record.

Compatibility is directional.

```text
v1 → v2
```

does not imply:

```text
v2 → v1
```

Identical symbolic name or related version numbers establish nothing.

A compatibility Record MUST define a mechanical scope mapping between the two Definitions.

Implicit dimension dropping, invention, or defaulting is forbidden.

---

# 133. Compatibility Binding Rule

If the Establishment and requirement use the same exact Assumption Definition:

```text
compatibility_authority_ref MUST be absent
```

If Definitions differ:

```text
compatibility_authority_ref MUST be present
```

and authoritative.

Any other combination is a structural contradiction and MUST be rejected, per §6.6.

---

# 134. Compatibility Invalidation

Later Evidence may create:

```text
ASSUMPTION_VERSION_COMPATIBILITY_INVALIDATED
```

The historical Compatibility Record remains unchanged.

The invalidated object is the cross-version application edge.

The original Establishment is not automatically invalidated.

Predicate horizon rules follow §130.

---

# 135. Bootstrap Trust Declaration

EvidenceRegistry MUST provide a way to explicitly state where recursive qualification stops.

A Bootstrap Trust Declaration means:

> These exact premises and judgment points are accepted as trust boundary inputs for this declared qualification Scope.

It is not proof that the premises are universally true.

---

# 136. Bootstrap Trust Declaration Fields

Minimum conceptual fields:

```text
schema_version

record_type =
  BOOTSTRAP_TRUST_DECLARATION

declaration_scope_ref

trusted_mathematical_assumptions[]

trusted_physical_assumptions[]

trusted_computational_assumptions[]

human_judgment_points[]

externally_governed_trust_points[]

rationale_refs[]

known_limitations[]

bootstrap_evidence_refs[]

predecessor_declaration_ref
```

Authority requires:

```text
BOOTSTRAP_TRUST_DECLARATION_RECORDED
```

---

# 137. Bootstrap Trust Scope

A Bootstrap Trust Declaration MUST bind an exact content-addressed Scope Record as `declaration_scope_ref`.

A Policy requiring Bootstrap Trust expresses `required_bootstrap_scope_ref` (§85).

A Closeout satisfying that requirement records the same exact value (§88.1).

v0.x applicability requires:

```text
declaration_scope_ref
==
required_bootstrap_scope_ref
```

No semantic subset/superset inference is permitted.

The following do NOT establish applicability:

```text
similar scope name
same product family
later milestone
prose asserting broader coverage
shared predecessor declaration chain
```

Failure of exact matching yields:

```text
BOOTSTRAP_SCOPE_MISMATCH
```

unless a future explicitly specified Scope Algebra is in force.

---

## 137.1 Evaluation actor

Closeout creation MUST perform the Bootstrap Trust applicability check, verifying:

```text
exact Declaration authority exists

declaration_scope_ref == required_bootstrap_scope_ref

Policy-required trust categories present

Declaration not already authoritatively invalidated

all required trust points present
```

Missing or mismatched trust boundary:

```text
BOOTSTRAP_TRUST_REQUIREMENT_UNSATISFIED
```

MUST fail closed.

The Closeout MUST NOT delegate this check to later human interpretation.

---

# 138. Bootstrap Trust as Support

Where qualification depends on Bootstrap Trust, the exact authoritative Declaration MUST appear in the Closeout support binding.

A Closeout MUST NOT depend on ambient "current Bootstrap Trust" without naming the exact Declaration.

---

# 139. Bootstrap Trust Succession

A successor Declaration does not revoke its predecessor.

Strengthening the trust boundary MUST NOT automatically degrade old Closeouts.

The predecessor remains the exact support Record originally used.

A successor MUST NOT silently substitute for its predecessor inside an existing Closeout support snapshot.

It may support a successor Closeout.

---

# 140. Bootstrap Trust Invalidation

Later Evidence may create:

```text
BOOTSTRAP_TRUST_INVALIDATED
```

The original Declaration remains historical and unchanged.

The invalidation may change current downstream support as specified by the Formal Verification layer.

Target modes and predicate horizon follow §130.

Future Declarations are not automatically targeted by an old predicate invalidation.

Recording a Bootstrap Trust invalidation does not require proving the invalidation from another, deeper Bootstrap Trust Declaration indefinitely.

The invalidation carries its own Evidence, Method, basis, and declared limitations.

Where that basis is itself uncertain, the resulting downstream support may become indeterminate rather than invalid.

EvidenceRegistry records that uncertainty rather than inventing an ultimate foundation.

---

# 141. Bootstrap Trust Is Not Self-Proving

Authority of a Bootstrap Trust Declaration establishes:

> This exact trust boundary was declared.

It does NOT establish:

> Every declared premise is proven true.

This distinction is normative.

---

# 142. Formal Finding Classification

Raw formal Verification and human semantic classification MUST remain separate.

A Formal Verification Record contains the raw formal event and MUST NOT wait for later human interpretation.

Human classification uses:

```text
FORMAL_FINDING_CLASSIFICATION
```

Authority requires:

```text
FORMAL_FINDING_CLASSIFICATION_RECORDED
```

Conceptually:

```text
FORMAL_VERIFICATION_RECORDED
↓
raw result exists
↓
human / specialist analysis
↓
FORMAL_FINDING_CLASSIFICATION_RECORDED
```

A long-running human investigation therefore does not delay preservation of the original solver result.

---

# 143. Formal Finding Classification Fields

Minimum conceptual fields:

```text
schema_version
record_type

formal_verification_authority_ref

classification

classifier_procedure_identity

classifier_role

judgment_basis = HUMAN_JUDGMENT

rationale_ref

supporting_evidence_refs[]

known_uncertainties[]
```

Initial classification vocabulary MAY include:

```text
IMPLEMENTATION_PROPERTY_VIOLATION

FORMAL_MODEL_DEFECT

PROPERTY_UNDERSPECIFIED

ASSUMPTION_MODELING_DEFECT

UNRESOLVED
```

The classification vocabulary MUST be versioned.

`classifier_procedure_identity` SHOULD be a content-addressed review-procedure identity, so that the judgment process is inspectable without pretending to formalize the judgment itself.

---

# 144. Classification Is Human Judgment

The classification authority establishes:

> This exact human judgment was recorded under this procedure.

It does not mathematically establish the judgment as true.

This distinction MUST remain machine-visible through:

```text
judgment_basis = HUMAN_JUDGMENT
```

A Classification MUST NOT rewrite the raw Formal Verification result.

Correct history is:

```text
counterexample occurred
+
it was later classified as a model defect
```

not:

```text
counterexample never happened
```

---

# 145. Default Unclassified Formal Finding Rule

A raw formal counterexample MUST NOT automatically become:

```text
IMPLEMENTATION_PROPERTY_VIOLATION
```

The default is fail-closed.

Where a Policy expresses `required_formal_finding_classification` with `required: true` and the required Classification is absent or is not among the acceptable classifications:

```text
INDETERMINATE
```

and the Policy gate is not satisfied.

Where a Policy does not express the requirement at all, the default remains fail-closed for product-level conclusions: EvidenceRegistry MUST NOT derive a product-level defect conclusion from an unclassified raw formal result.

A Policy MAY define a mechanically unambiguous mapping from a raw formal outcome to a product-level conclusion, but only through an explicit frozen Policy rule.

---

# 146. Formal Support Binding in Closeout

Where formal Evidence is used, a Closeout SHOULD preserve the exact support graph.

Conceptually:

```text
formal_verification_authority_refs[]

assumption_bindings[]

bootstrap_trust_declaration_authority_refs[]
```

An Assumption binding contains:

```text
required_assumption_definition_authority_ref

establishment_authority_ref

compatibility_authority_ref
  iff required per §133
```

This support snapshot is historical Evidence.

A Closeout MUST NOT silently reconstruct a cross-version compatibility dependency later.

---

# 147. Current Support Versus Historical Support

The Closeout records the support basis accepted at creation.

Later applicability or invalidation may change what the historical graph supports now.

It MUST NOT rewrite which support graph existed then.

Detailed support aggregation belongs to the Formal Verification specification.

---

# 148. CLI Surface

Proposed commands include:

```text
evidence-registry init

evidence-registry freeze

evidence-registry verify
evidence-registry verify --record

evidence-registry inspect

evidence-registry review-pack
evidence-registry review-admit

evidence-registry closeout

evidence-registry successor

evidence-registry audit

evidence-registry journal verify
evidence-registry journal head
evidence-registry journal export
evidence-registry journal check-anchor
```

Additional formal-support commands MAY be added without changing lifecycle semantics.

---

# 149. `init`

`init` establishes a fresh Registry and GENESIS.

It MUST NOT overwrite an existing authoritative Registry.

It establishes `registry_id`, journal format, record identity profile, initial on-disk layout, and the initial storage capability observation (§11).

---

# 150. `freeze`

Responsibilities include:

* generate Attempt identity;
* acquire attempt coordination;
* START before root creation;
* exclusive fresh-root creation;
* exclusive first-payload creation;
* filesystem checks;
* Artifact retention/reference handling;
* Manifest;
* required verification;
* Receipt;
* durability operations;
* terminal Journal event.

---

# 151. `verify`

Read-only.

No Registry mutation.

MUST require or derive an explicit verification target.

---

# 152. `verify --record`

Fresh verification plus authoritative Verification Record and `VERIFICATION_RECORDED` event.

---

# 153. `inspect`

Read-only.

Must distinguish:

```text
authoritative fact

portable nonauthoritative Record

derived state

locator hint

UNKNOWN

coverage limitation
```

---

# 154. `review-pack`

Creates exact Review Request package and Journal Anchor, binding Subject, Policy, role, Scope, Method, and required Checks.

---

# 155. `review-admit`

Mechanically validates returned Review Result and Anchor.

Accepted and rejected completed dispositions become history.

MUST fail closed on missing or mismatched required identity.

---

# 156. `closeout`

Creates immutable Closeout only when creation-time Policy requirements pass, including Bootstrap Trust applicability (§137.1).

Postconditions may leave derived state pending.

MUST NOT overwrite a prior Closeout.

---

# 157. `successor`

Creates fresh successor history.

Never overwrites predecessor.

MUST retain predecessor identity, preserve predecessor bytes, and identify the succession relation and reason.

---

# 158. `journal verify`

Read-only full Journal-chain validation.

---

# 159. `journal head`

Returns:

```text
registry_id
entry_index
entry_hash
```

---

# 160. `journal export`

Produces portable Journal Anchor and optionally an immutable Journal prefix package.

Export MUST NOT mutate the Registry.

---

# 161. `journal check-anchor`

Read-only comparison against a retained Anchor.

Output distinguishes the result classes of §116.

---

# 162. Error Taxonomy

Suggested families:

```text
ER_FS_*

ER_PATH_*

ER_IDENTITY_*

ER_RECORD_*

ER_MANIFEST_*

ER_JOURNAL_*

ER_REVIEW_*

ER_VERIFY_*

ER_CLOSEOUT_*

ER_LINEAGE_*

ER_CHRONOLOGY_*

ER_POLICY_*

ER_AUDIT_*

ER_FORMAL_*

ER_BOOTSTRAP_*

ER_UNSUPPORTED_*

ER_INTERNAL_*
```

Unknown failure MUST NOT be misclassified as a more specific false diagnosis.

An explicit unknown class within the correct family is preferred.

---

# 163. Exit Codes

Recommended classes:

```text
0
operation completed and requested condition satisfied

1
operation completed and established a negative result

2
usage / configuration error

3
unsupported environment

4
Policy gate mechanically unsatisfied

5
internal EvidenceRegistry failure

6
operation could not collect enough Evidence
to determine the requested property
```

Negative result and inability to determine MUST remain distinct.

---

## 163.1 Mapping of indeterminate outcomes

```text
finding_state = BLOCKING, verification completed
    → 1

finding_state = INDETERMINATE
because Evidence could not be collected
(missing payload, permission, transient I/O,
 source unavailable, required Record absent)
    → 6

Policy gate not satisfied
(including required Classification absent per §145,
 required_method_status unmet, diversity unmet,
 BOOTSTRAP_TRUST_REQUIREMENT_UNSATISFIED)
    → 4

required path profile or capability not supported
    → 3
```

Where more than one class applies, the more specific mechanical cause takes precedence in the order:

```text
5  >  2  >  3  >  6  >  4  >  1  >  0
```

Recording a negative or indeterminate result through `verify --record` is a successful command execution with respect to the recording operation itself; the exit code reflects the evaluated property, not the success of Journal publication.

---

# 164. Tool Output

Human-readable output MAY exist.

Automation-facing output MUST be machine-readable and stable enough for CI and Agent use.

Every machine-readable output MUST include `output_schema_version`.

Human prose MUST NOT be the sole authoritative representation.

---

# 165. PerformanceEvidenceProbe Relationship

PerformanceEvidenceProbe is an Evidence Producer.

EvidenceRegistry is the lifecycle registrar.

```text
PerformanceEvidenceProbe
        ↓
measurement Evidence
        ↓
EvidenceRegistry
        ↓
authority / review / closeout / succession
```

Workload-specific measurement logic MUST remain outside EvidenceRegistry core.

---

# 166. Agent Skill Architecture

Preferred:

```text
Agent / Hermes / CI
       ↓
Thin Skill
       ↓
EvidenceRegistry CLI
       ↓
Rust Core
```

Critical lifecycle semantics belong in the executable core.

They MUST NOT exist only as free-form Agent instructions.

---

# 167. Repository Structure

Proposed:

```text
EvidenceRegistry/
├─ Cargo.toml
│
├─ crates/
│  ├─ evidence-registry-core/
│  ├─ evidence-registry-cli/
│  ├─ evidence-registry-schema/
│  ├─ evidence-registry-journal/
│  └─ evidence-registry-audit/
│
├─ specs/
│  ├─ EVIDENCE-REGISTRY-LIFECYCLE-SPEC-v0.10.2.md
│  └─ ...
│
├─ schemas/
│
├─ test-vectors/
│
├─ formal/
│  └─ dafny/
│
├─ tests/
│  ├─ fixtures/
│  ├─ crash/
│  ├─ concurrency/
│  ├─ filesystem/
│  └─ adversarial/
│
├─ skills/
└─ docs/
```

Exact crate boundaries MAY be simplified without changing semantics.

---

# 168. Adversarial Test Corpus

The permanent corpus SHOULD include at least:

```text
pre-existing root

root creation race

first-payload overwrite

crash after START, before root creation
crash after root creation
crash after payload creation
crash after Manifest
crash after Receipt
crash before COMMIT

recovery attempt tries to reuse crashed root

Receipt without COMMIT
COMMIT with invalid Receipt

Journal gap
Journal fork
Journal hash mismatch
Journal illegal transition

Journal Anchor divergence
Journal history behind Anchor

extra payload
deleted payload
changed payload

Manifest drift

wrong Subject Review

Review role laundering
Review Scope promotion

mutated Policy

rejected Admission retry

NOT_CLEAN / BLOCKING predecessor rewrite attempt

Unicode composed / decomposed path

invalid UTF-8 Unix filename

Windows unpaired surrogate where constructible

Windows reserved names (CON, AUX, aux.txt)

trailing dot / trailing space

MAX_PATH-exceeding paths

NTFS 8.3 aliases

alternate data streams

symlink
directory symlink cycle
junction
reparse point
hardlink alias

permission denied

file disappears during scan

file grows during hash

file changes between passes

case-only collision

empty Subject

database projection loss

concurrent writer collision

operator abort while writer survives,
followed by that writer attempting a later
Freeze commit
    (both phases of one scenario:
     the assertion, and the rejected commit)

Eviction blocked by active dependency

crash during eviction

indeterminate eviction recovery

successor eviction with unknown remaining scope

NFS lock failure
NFS exclusive-create ambiguity

SMB concurrency behavior

sync-client conflicted copy
sync-client restoration of old Journal
sync-client rename of authoritative file

placeholder file with metadata but unavailable bytes

partial cloud hydration during Manifest scan

stale synchronized Registry snapshot

Registry duplication and later divergence

cross-platform verify under incompatible path profile

audit with incomplete root coverage

Bootstrap Scope mismatch

invalid Compatibility edge
invalidated Compatibility

invalidated Bootstrap Trust

unclassified formal counterexample under a Policy
requiring classification
```

Each historic real failure SHOULD be considered for permanent regression conversion.

---

# 169. Cross-Platform Requirement

Milestone 1 SHOULD target:

```text
Windows
Linux
macOS
```

Platform-specific code is permitted.

Externally visible semantics MUST remain explicit.

EvidenceRegistry MUST NOT claim equivalent protection where primitives materially differ, in particular for:

```text
path behavior
alias behavior
reparse behavior
durability
directory metadata persistence
atomic publication behavior
```

---

# 170. Network Filesystem Requirement

Where NFS, SMB, synchronization clients, or other non-local storage are used, capability must be established rather than assumed.

Unsupported capability Scope MUST fail closed where authority depends on it.

---

# 171. Algorithm Agility

SHA-256 is required for v0.x.

Algorithm identity is explicit.

Old identities MUST retain old meaning forever.

Algorithm identifiers MUST NOT be recycled.

---

# 172. Schema Versioning

Every authoritative Record contains:

```text
schema_version
record_type
```

Old semantic meaning MUST NOT silently change.

Material reinterpretation requires:

```text
new schema version
```

or:

```text
new Record type
```

or successor specification.

---

# 173. Format Stability Gate

Before stable identity format release, publish:

```text
deterministic CBOR profile

Record identity framing

Record Type Registry

Manifest encoding

Digest Algorithm Registry

Artifact Kind Registry

Path Identity Profiles

canonical path comparator

Registry ID generation and encoding

Attempt ID generation and encoding

Journal Entry schema

Journal Event Type numeric Registry

Journal Entry event-specific index fields

Journal Reference encoding

Journal Anchor encoding

IDENTITY_DEPENDENCY encoding

AUTHORITY_DEPENDENCY encoding

observation_start_journal_ref semantics

operation_start_journal_ref semantics

GENESIS encoding

Storage Capability Class schema

Environment Observation schema

Source Binding schema

Verification Observation schema

Verification Record schema

Operator Abort Assertion Record schema

Eviction Record family schemas

Bootstrap Trust Declaration schema

Formal Finding Classification schema

formal-support authority Record schemas

on-disk authoritative namespace layout

probe namespace

coordination naming

attempt-derived root naming
```

ArtifactSet v2 MAY be frozen separately as a stable utility format.

It is not required to establish Registry authority.

---

# 174. Identity Test Vectors

Stable format release MUST include:

* valid vectors;
* expected Record IDs;
* Manifest IDs;
* Journal hashes;
* Journal Reference and Anchor vectors;
* path ordering vectors;
* malformed-input rejection vectors;
* boundary vectors;
* cross-implementation vectors.

Reproduction MUST be possible without reading the Rust implementation.

---

# 175. Journal Compaction

Destructive logical compaction is a non-goal for v0.x.

EvidenceRegistry MUST NOT delete historical Journal entries and claim unchanged history semantics.

Specifically, it MUST NOT convert a full historical Journal into a summary checkpoint plus a deleted prefix while claiming unchanged Profile L history semantics.

Physical compression, indexes, caches, and immutable archive packaging are permitted only if the complete logical Journal remains exactly reconstructable and exact historical identities remain verifiable.

---

# 176. Trust Profiles

## 176.1 Profile L

Local Evidence Integrity.

Default v0.x.

Does not establish independent execution attestation.

## 176.2 Profile H

Future High-Assurance / Independent Attestation.

May include:

* separately administered authority;
* signed Records;
* TPM;
* TEE;
* remote verifier;
* external witness;
* trust federation.

Profile H MUST be separately specified and reviewed.

Profile L Evidence MUST NOT silently become Profile H Evidence.

---

# 177. Cross-Registry Federation

Cross-Registry federation and mutual trust recognition are OUT OF SCOPE for v0.x.

In particular, v0.x does not define:

```text
foreign Assumption Establishment acceptance

foreign Bootstrap Trust recognition

cross-Registry Scope equivalence

cross-Registry authority federation

cross-Registry chronology
```

These require a future Profile H or federation specification.

---

# 178. Milestone 1

Milestone 1 SHOULD establish the foundational identity and authority layer.

Required:

```text
Registry init

GENESIS

Authority Journal

Journal replay

Journal-only open-attempt reconstruction

event registry

writer coordination

attempt-specific coordination

operator-abort assertion path

deterministic CBOR

Record identity

Path profiles

canonical path comparator

filesystem alias detection

Manifest

SHA-256

START-before-root Freeze

attempt-derived root namespace

Freeze Attempt lifecycle

Receipt → START Journal Reference binding

durability facts

crash recovery

typed Verification

read-only verify and verify --record

Source Binding

Storage Capability re-observation

Artifact-byte eviction

Journal Anchor export

Journal Anchor checking

rollback / divergence detection

basic Audit

cross-platform tests
```

The first goal is:

> Evidence must be born through an unambiguous authoritative event, and an incomplete birth must never later become authoritative by convenience.

---

# 179. Milestone 2

Add complete Review lifecycle:

```text
content-addressed Policy Record

content-addressed Review Scope

Review Method / Check identities

Review Request

Review Result

Review Anchor round-trip

role equality enforcement

Scope equality enforcement

Admission (accepted and rejected)

Closeout

Closeout rejection history

bracket postconditions
```

---

# 180. Milestone 3

Add:

```text
expanded Policy engine

lineage graph Audit

coverage-bound Audit reports

chronology validation

support-record authority

Assumption Records

Compatibility Records

Bootstrap Trust Records

Formal Finding Classification

formal-support lifecycle integration

machine-readable disposition ledger
```

---

# 181. Milestone 4

Potential advanced work:

```text
full Dafny formal model

Rust / model conformance

signed Records

external witness

Profile H

remote attestation

TPM / TEE research

federation research

Claim Registry
```

These are not required to make Profile L useful.

Dafny MAY be started earlier if useful.

Milestone placement does not mean the state machine may remain semantically undefined until Milestone 4; the state machine is normative from this specification.

---

# 182. Formal Verification Boundary

Detailed proof semantics belong to:

```text
EvidenceRegistry Formal Verification
& Implementation Boundary Specification
```

The Lifecycle Specification defines:

* which formal-support Records may acquire authority;
* which Journal events provide that authority;
* how chronology and succession apply;
* how those Records bind into Closeout.

It does not redefine the mathematical theorem layer.

---

# 183. onigiranai Integration Boundary

EvidenceRegistry MUST NOT retroactively rewrite pre-adoption onigiranai qualification history.

Existing Evidence MAY be used as:

```text
read-only design input

read-only historical failure corpus

adversarial test source
```

Adoption requires an explicit later integration gate.

---

# 184. Compatibility Principle

Prefer explicit incompatibility over silent reinterpretation.

When an old Record cannot be faithfully interpreted:

```text
UNSUPPORTED_SCHEMA
```

or another explicit result is preferred to guessed compatibility.

Predecessor-format Records MUST NOT silently acquire successor-format semantics.

---

# 185. Privacy Principle

EvidenceRegistry SHOULD minimize required disclosure.

Subject contents need not leave user-controlled storage.

Records SHOULD retain only metadata necessary for identity, authority, lifecycle, verification, and audit.

Opaque procedural identities SHOULD be preferred where real-world personal identity is unnecessary.

Remote or federated profiles require separate privacy specifications.

---

# 186. Licensing Requirement

Before public stable release, define repository license treatment of:

* commercial use;
* modification;
* redistribution;
* binary distribution;
* linking;
* bundled CLI use;
* patent terms;
* NOTICE;
* trademark boundaries.

Dependency licensing requires separate audit.

---

# 187. Documentation Requirement

Maintain:

```text
specs/
  normative specifications

docs/
  architecture / usage

test-vectors/
  stable identity vectors

docs/history/
  superseded specification history

formal/
  formal models / assumptions
```

Superseded specifications remain available.

---

# 188. Development Governance

Material changes to foundational semantics SHOULD require:

```text
design
↓
frozen candidate
↓
hostile review
↓
independent review
↓
accepted successor specification
```

Foundational areas include:

* Journal authority;
* Record identity;
* canonical serialization;
* path identity;
* Manifest identity;
* Freeze semantics;
* durability semantics;
* Verification authority;
* Review Admission;
* Policy identity;
* Closeout semantics;
* operation phase classification;
* succession;
* chronology;
* Bootstrap Trust authority.

Implementation-only changes MAY use lighter verification where semantics remain unchanged.

---

# 189. Bootstrap Rule

EvidenceRegistry cannot use an unqualified version of itself as the sole proof of that same version.

Initial releases MUST retain an external bootstrap path.

Possible bootstrap Evidence includes:

```text
source-control commit / archive identity

published identity test vectors

independent recomputation

external hostile review

external independent review

environmental observation diversity

Bootstrap Trust Declaration
```

Self-hosting MAY supplement but MUST NOT erase this chain.

---

# 190. Bootstrap Trust Boundary Principle

Recursive qualification terminates at explicit trust boundaries.

EvidenceRegistry MUST record where qualification stops.

It MUST NOT turn:

```text
accepted premise
```

into:

```text
proven universal truth
```

merely because the premise appears in an authoritative Declaration.

---

# 191. Core Safety Properties

## 191.0 Numbering namespace

Properties prefixed `ER-P` belong to this Lifecycle Specification.

Properties prefixed `FV-P` belong to the Formal Verification & Implementation Boundary Specification.

The two namespaces are independent and MUST NOT be cross-referenced by bare number.

Where an earlier delta document in the v0.6 line introduced a property under an `ER-P` number that this specification assigns differently, the assignment in this section governs for Lifecycle purposes, and the Formal Verification specification governs its own `FV-P` numbering.

Citations across specifications MUST use the qualified form, for example:

```text
Lifecycle ER-P21
FV FV-P21
```

---

## ER-P1: Identity Integrity

An authoritative Record refers to one exact identity.

## ER-P2: Scoped No Silent Rewrite

Non-recomputed modification of retained Journal history is detectable from the retained chain.

A reconstructed valid replacement history requires external historical commitment for detection.

## ER-P3: Explicit Succession

Correction occurs through explicit successor history.

## ER-P4: Binding Integrity

Review, Verification, and Closeout cannot mechanically bind to wrong authority without rejection.

## ER-P5: Fail-Closed Missing Identity

Missing required identity is not inferred.

## ER-P6: Partial Is Not Authoritative

Crash-created partial packages do not become Freeze authority.

## ER-P7: Sticky Terminal Non-Authority

An aborted Freeze Attempt cannot later COMMIT.

## ER-P8: Claim Separation

Evidence mechanics do not automatically authorize semantic Claims.

## ER-P9: Historical Failure Preservation

Explicit published failure dispositions remain inspectable.

## ER-P10: Projection Independence

Projection failure does not redefine authority.

## ER-P11: Journal Authority

Authority derives from legal Journal commitment, not file presence.

## ER-P12: Chronology by Commitment

Chronology derives from Journal order and explicit dependency.

## ER-P13: Typed Verification

Verification states exactly what was examined.

## ER-P14: Policy Immutability

Gates bind exact Policy identity.

## ER-P15: Scope Integrity

Narrow Scope does not silently satisfy broader Scope.

## ER-P16: Role Integrity

Review Result does not promote its Review role.

## ER-P17: Coverage-Bound Audit

Negative Audit claims are limited to actual coverage.

## ER-P18: Platform Honesty

Cross-platform equivalence is not claimed beyond established primitives.

## ER-P19: Registry-History-Bound Authority

Portable Record identity does not transfer Registry authority.

## ER-P20: Anchor-Relative Historical Detection

External Journal Anchors permit detection of inconsistent later history relative to those Anchors.

## ER-P21: Method-State Gate Integrity

Policy evaluates Method status separately from Finding state.

## ER-P22: Recovery Replay Sufficiency

Journal replay is sufficient only for the explicitly indexed lifecycle recovery facts enumerated in §22.

It is not sufficient for semantic Policy evaluation.

## ER-P23: Explicit Evidence Availability Loss

Intentional removal of embedded bytes requires explicit eviction history.

## ER-P24: Observation Does Not Equal Continuity

Two observations do not establish continuous unchanged state.

## ER-P25: No Forward Authority Dependency

Later facts refer backward to prior authority.

## ER-P26: Manifest-Sole Inventory Authority

Manifest identity is the sole normative frozen Artifact inventory identity.

## ER-P27: Destructive Operation Visibility

Crash-visible destructive operations require TWO_PHASE history.

## ER-P28: Capability / Environment Separation

Descriptive environment change is distinct from material capability change.

## ER-P29: Bootstrap Trust Is Explicit

Qualification roots of trust are recorded rather than hidden.

## ER-P30: Human Judgment Is Typed

Human semantic classification is not disguised as mechanical proof.

## ER-P31: One-Phase Crash Leaves No Authority

A crash before the terminal Journal append of a ONE_PHASE operation leaves no authoritative disposition, and none is claimed.

## ER-P32: Context-Required Field Symmetry

An optional semantic field is present exactly when its context requires it, and absent otherwise.

---

# 192. Claim Boundary

EvidenceRegistry v0.x MAY claim:

```text
exact deterministic Record identity

Manifest-bound Subject inventory

Registry-history-bound authority

Journal-replayable lifecycle recovery

explicit chronology where committed

typed Verification

Journal-position-bound observation intervals

Review role / Scope binding

Policy-bound Closeout

first-eligible-POST bracket semantics

explicit Artifact eviction history

proven Journal divergence where a conflicting entry
at the anchored index is retained

external-Anchor-relative historical inconsistency detection

capability-sensitive storage transition history

coverage-bound Audit

explicit Bootstrap Trust boundary
```

It MUST NOT claim:

```text
intrinsic detection of completely deleted unanchored history

that every same-registry Anchor mismatch is provably a rollback

that every attempted illegal transition is always
historically recorded

same-principal tamper resistance

continuous Subject immutability from two observations

wall-clock freshness from Journal distance

that a capability was absent merely because it was not probed

that every changed Capability Class identity represents a
material storage change

cryptographic reviewer independence

physical Registry uniqueness before divergence

automatic truth of Bootstrap Trust premises

automatic product defect from an unclassified formal
counterexample

cross-Registry trust federation
```

---

# 193. Success Criterion for v0.x

EvidenceRegistry v0.x succeeds if workflows such as:

```text
init
↓
freeze exact Subject
↓
commit authority
↓
verify exact target
↓
preserve history
↓
recover from crashes without authority promotion
↓
review
↓
admit / reject
↓
closeout
↓
postcondition
↓
successor
↓
invalidate support where later Evidence requires
↓
audit
```

can be performed mechanically without every Project reinventing its own Evidence lifecycle.

---

# 194. Core Institutional Statement

EvidenceRegistry exists to enforce:

> Evidence can change by succession, but history cannot change by convenience.

More precisely:

> A later fact may change what an existing historical Evidence graph supports now. It may not change which Evidence graph existed then.

And:

> EvidenceRegistry does not claim memory that no surviving Evidence possesses.

UNKNOWN is not a failure to be hidden.

It is a first-class result when Evidence cannot justify a stronger conclusion.

---

# 195. Final Lifecycle Principle

EvidenceRegistry records:

```text
what bytes existed

what identity they had

what acquired authority

what was observed

what failed

what later succeeded

what was invalidated

what was removed

what trust boundary was accepted

what human judgment was recorded

and what cannot be known
```

The purpose of the lifecycle is not to eliminate uncertainty.

It is to make uncertainty, authority, succession, and historical change mechanically distinguishable.
