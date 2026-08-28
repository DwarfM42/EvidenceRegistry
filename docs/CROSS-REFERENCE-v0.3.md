# EvidenceRegistry Normative Event & Transition Cross-Reference Table v0.3

**Status:** FINAL FREEZE CANDIDATE
**Predecessor:** EvidenceRegistry Normative Event & Transition Cross-Reference Table v0.2
**Project:** EvidenceRegistry
**Repository:** `DwarfM42/EvidenceRegistry`
**Normative lifecycle dependency:** `EvidenceRegistry Evidence Lifecycle Specification v0.10.2`
**Normative formal dependency:** `EvidenceRegistry Formal Verification & Implementation Boundary Specification v0.5.4`
**Purpose:** Lifecycle §20.1 normative cross-reference table and direct Dafny/Rust state-machine input
**Normative language:** MUST / MUST NOT / SHOULD / SHOULD NOT / MAY

---

# 0. Purpose

Evidence Lifecycle Specification v0.10.2 §20.1 requires one authoritative cross-reference table containing at least:

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

This document supplies that table.

It additionally freezes, at the semantic level:

```text
complete lifecycle_object_kind vocabulary

lifecycle_object_id meaning by object kind

transition shape

kind-relative state vocabulary

terminality applicability

direct AUTHORITY_DEPENDENCY requirements

IDENTITY_DEPENDENCY requirements

Journal Reference validation

Journal-only event index fields

context-required / context-forbidden relations
```

This document is intended to become direct input to:

```text
Dafny datatypes

Dafny LegalTransition predicates

Rust enums

Rust transition guards

Journal replay

conformance tests

identity-format test-case design
```

This document does NOT yet freeze:

```text
numeric lifecycle_object_kind encoding

CBOR field numeric keys

exact deterministic CBOR bytes

exact Record IDs

exact Journal hashes

canonical dependency-list byte ordering
```

Those belong to the subsequent Identity Format Freeze.

---

## 0.1 Changes from v0.2

v0.3 introduces no new Lifecycle event type and no new Lifecycle Record type.

It closes the final non-blocking review observations before freeze:

```text
POST-bracket Journal-only identification is now based on
the explicit brackets_closeout_authority_ref field rather
than inference from an arbitrary Closeout dependency

brackets_closeout_authority_ref is explicitly required as
an event-specific Journal field for POST Verification

Journal-only replay of POST intent is therefore based on
that exact field

Identity Format Freeze inputs now explicitly include
brackets_closeout_authority_ref placement and encoding

operation_start_journal_ref on Assumption Definition,
Formal Verification, and similar long-running ONE_PHASE
operations is explicitly described as chronology provenance
only under current v0.x Policy semantics

future Policy-constrained formal-execution intervals are
recorded as a deferred successor concern rather than being
silently implied by operation_start_journal_ref
```

No Lifecycle v0.10.3 is required by these clarifications.

---

# 1. Relationship to Lifecycle v0.10.2

This document does not introduce a new common Journal field.

It assigns exact semantics to fields already required by Lifecycle v0.10.2:

```text
event_type_id
event_record_id

lifecycle_object_kind
lifecycle_object_id

identity_dependencies[]
authority_dependencies[]
```

It also specializes event-specific Journal indexing required by Lifecycle §20.1 and §22.

Therefore this document is treated as the normative companion required by Lifecycle §20.1 rather than, by itself, a successor Lifecycle specification.

If review finds that any rule here contradicts rather than specializes Lifecycle v0.10.2, the conflicting rule MUST NOT silently override Lifecycle v0.10.2.

Instead:

```text
Lifecycle successor required
```

---

# 2. Lifecycle Record-Field Supplementation

Lifecycle v0.10.2 contains some minimum Record-field lists that are intentionally incomplete relative to the `operation head` requirements already present in Lifecycle §20.

For purposes of Lifecycle §20.1, this document normatively supplements those minimum lists.

In particular:

```text
Review Result
Review Admission
Policy
```

MUST carry the chronology binding specified by §48 of this document when their stable schemas are frozen.

This is a specialization of the existing:

```text
operation head
```

semantics in Lifecycle §20.

It does not create a new lifecycle rule.

The subsequent Identity / Record Schema Freeze MUST incorporate these fields into the exact Record schemas.

---

# 3. Core Distinction: Event Record vs Lifecycle Object

Every Journal Entry contains both:

```text
event_record_id
```

and:

```text
lifecycle_object_kind
lifecycle_object_id
```

They have different meanings.

---

## 3.1 event_record_id

`event_record_id` identifies:

> The exact deterministic Record containing the authoritative payload associated with this Journal event.

It is an EvidenceRegistry Record identity.

---

## 3.2 lifecycle_object_id

`lifecycle_object_id` identifies:

> The logical lifecycle object whose state is created, transitioned, or observed by this event.

It is not universally a Record ID.

---

## 3.3 Multi-event lifecycle objects

For lifecycle objects having multiple Journal events:

```text
lifecycle_object_id
```

MUST remain stable across every event belonging to that object.

Examples:

```text
FREEZE_ATTEMPT
    → freeze_attempt_id

ARTIFACT_EVICTION_ATTEMPT
    → eviction_attempt_id

REGISTRY
    → registry_id
```

---

## 3.4 Record-backed one-shot lifecycle objects

For an object whose entire authoritative lifecycle consists of one Record-creation event:

```text
lifecycle_object_id
=
event_record_id
```

in v0.x.

This equality is intentional.

The two fields remain semantically distinct:

```text
event_record_id
    = exact event payload Record

lifecycle_object_id
    = logical object identity used by lifecycle indexing
```

A later format MAY choose a distinct logical ID for such objects only through an explicit format/specification successor.

---

## 3.5 One-shot disposition attempts

For ONE_PHASE operations having two possible terminal dispositions but no separate START event:

```text
REVIEW_ADMISSION_ATTEMPT

CLOSEOUT_ATTEMPT
```

the disposition Record itself identifies the attempt.

Therefore:

```text
lifecycle_object_id
=
event_record_id
```

for both accepted/committed and rejected dispositions.

Repeated attempts therefore have distinct lifecycle object IDs.

---

# 4. Complete lifecycle_object_kind Vocabulary

The following symbolic lifecycle object kinds are normative for v0.x:

```text
REGISTRY

FREEZE_ATTEMPT

VERIFICATION

REVIEW_REQUEST

REVIEW_RESULT

REVIEW_ADMISSION_ATTEMPT

POLICY

CLOSEOUT_ATTEMPT

ARTIFACT_EVICTION_ATTEMPT

ASSUMPTION_DEFINITION

ASSUMPTION_ESTABLISHMENT

ASSUMPTION_INVALIDATION

FORMAL_VERIFICATION

ASSUMPTION_VERSION_COMPATIBILITY

ASSUMPTION_VERSION_COMPATIBILITY_INVALIDATION

BOOTSTRAP_TRUST_DECLARATION

BOOTSTRAP_TRUST_INVALIDATION

FORMAL_FINDING_CLASSIFICATION
```

There are exactly:

```text
18
```

v0.x symbolic object kinds.

Every Journal event defined by Lifecycle v0.10.2 §21 MUST use exactly one of these kinds.

Unknown authoritative object kinds MUST fail closed after stable encoding freeze.

---

## 4.1 Object-kind encoding

This document freezes symbolic meaning only.

Numeric or deterministic-CBOR encoding of `lifecycle_object_kind` belongs to the Identity Format Freeze.

The encoding MUST provide a one-to-one stable mapping to the symbolic vocabulary above.

---

# 5. Lifecycle Object ID Semantics

| lifecycle_object_kind                           | lifecycle_object_id semantics |
| ----------------------------------------------- | ----------------------------- |
| `REGISTRY`                                      | `registry_id`                 |
| `FREEZE_ATTEMPT`                                | `freeze_attempt_id`           |
| `VERIFICATION`                                  | `event_record_id`             |
| `REVIEW_REQUEST`                                | `event_record_id`             |
| `REVIEW_RESULT`                                 | `event_record_id`             |
| `REVIEW_ADMISSION_ATTEMPT`                      | disposition `event_record_id` |
| `POLICY`                                        | `event_record_id`             |
| `CLOSEOUT_ATTEMPT`                              | disposition `event_record_id` |
| `ARTIFACT_EVICTION_ATTEMPT`                     | `eviction_attempt_id`         |
| `ASSUMPTION_DEFINITION`                         | `event_record_id`             |
| `ASSUMPTION_ESTABLISHMENT`                      | `event_record_id`             |
| `ASSUMPTION_INVALIDATION`                       | `event_record_id`             |
| `FORMAL_VERIFICATION`                           | `event_record_id`             |
| `ASSUMPTION_VERSION_COMPATIBILITY`              | `event_record_id`             |
| `ASSUMPTION_VERSION_COMPATIBILITY_INVALIDATION` | `event_record_id`             |
| `BOOTSTRAP_TRUST_DECLARATION`                   | `event_record_id`             |
| `BOOTSTRAP_TRUST_INVALIDATION`                  | `event_record_id`             |
| `FORMAL_FINDING_CLASSIFICATION`                 | `event_record_id`             |

This table is normative.

---

# 6. Transition Shapes

Journal events are classified by transition shape separately from operation phase.

Use:

```text
BOOTSTRAP_CREATE

CREATE_OPEN

CREATE_TERMINAL

TRANSITION_TERMINAL

OBSERVE_NO_STATE_CHANGE
```

---

## 6.1 BOOTSTRAP_CREATE

Creates the Registry authority domain.

Conceptually:

```text
ABSENT
→
INITIALIZED_AUTHORITATIVE
```

Used only by:

```text
GENESIS
```

---

## 6.2 CREATE_OPEN

Creates a multi-event lifecycle object in an OPEN state.

Conceptually:

```text
ABSENT
→
OPEN
```

Used by TWO_PHASE START events.

---

## 6.3 CREATE_TERMINAL

Creates a lifecycle object directly in a terminal state.

Conceptually:

```text
ABSENT
→
RECORDED / ACCEPTED / REJECTED / COMMITTED
```

Typical ONE_PHASE operations use this shape.

---

## 6.4 TRANSITION_TERMINAL

Transitions an already-open lifecycle object to a terminal state.

Conceptually:

```text
OPEN
→
TERMINAL
```

Used by TWO_PHASE terminal events.

---

## 6.5 OBSERVE_NO_STATE_CHANGE

Records an authoritative event about an existing lifecycle object without changing that object's lifecycle state.

Conceptually:

```text
S
→
S
```

Used by:

```text
FREEZE_COMMIT_REJECTED

STORAGE_CAPABILITY_CHANGED
```

---

# 7. Event Shape Is Separate from Operation Phase

Examples:

```text
FREEZE_ATTEMPT_STARTED
    phase = TWO_PHASE
    shape = CREATE_OPEN

FREEZE_COMMITTED
    phase = TWO_PHASE
    shape = TRANSITION_TERMINAL

FORMAL_VERIFICATION_RECORDED
    phase = ONE_PHASE
    shape = CREATE_TERMINAL

FREEZE_COMMIT_REJECTED
    phase = ONE_PHASE
    shape = OBSERVE_NO_STATE_CHANGE
```

A model MUST NOT infer event shape merely from operation phase.

---

# 8. State Types

Lifecycle state MUST be interpreted relative to `lifecycle_object_kind`.

One universal untagged state enum SHOULD NOT be used as the normative model.

Preferred conceptual structure:

```text
LifecycleObjectState =
    RegistryState
  | FreezeAttemptState
  | OneShotRecordedState
  | ReviewAdmissionState
  | CloseoutAttemptState
  | ArtifactEvictionState
```

---

# 9. RegistryState

```text
ABSENT

INITIALIZED_AUTHORITATIVE
```

`INITIALIZATION_PARTIAL_NONAUTHORITATIVE` from Lifecycle §24 is not an authoritative Journal lifecycle state.

It exists outside authoritative Journal history.

Therefore:

```text
GENESIS:
    ABSENT
    →
    INITIALIZED_AUTHORITATIVE

STORAGE_CAPABILITY_CHANGED:
    INITIALIZED_AUTHORITATIVE
    →
    INITIALIZED_AUTHORITATIVE
```

---

# 10. REGISTRY Has No Terminality Semantics

`REGISTRY` is a long-lived authority domain rather than an attempt or immutable one-shot lifecycle object.

Therefore:

```text
HasTerminality(REGISTRY)
=
false
```

`IsTerminal(kind, state)` is NOT defined for:

```text
kind = REGISTRY
```

This is intentional.

`INITIALIZED_AUTHORITATIVE` means:

> Registry authority has been established.

It does NOT mean either:

```text
terminal
```

or:

```text
nonterminal
```

because terminality is not a meaningful property of the Registry object's lifecycle model.

Registry capability-epoch history may continue indefinitely through state-observing events.

---

# 11. FreezeAttemptState

```text
ABSENT

OPEN

COMMITTED

ABORTED_RECOVERY

ABORTED_BY_OPERATOR_ASSERTION
```

Terminal states:

```text
COMMITTED

ABORTED_RECOVERY

ABORTED_BY_OPERATOR_ASSERTION
```

For:

```text
FREEZE_ATTEMPT
```

terminality is defined.

`FREEZE_COMMIT_REJECTED` does not create another Freeze state.

It observes an already-terminal Freeze Attempt.

---

# 12. OneShotRecordedState

Used by:

```text
VERIFICATION
REVIEW_REQUEST
REVIEW_RESULT
POLICY
ASSUMPTION_DEFINITION
ASSUMPTION_ESTABLISHMENT
ASSUMPTION_INVALIDATION
FORMAL_VERIFICATION
ASSUMPTION_VERSION_COMPATIBILITY
ASSUMPTION_VERSION_COMPATIBILITY_INVALIDATION
BOOTSTRAP_TRUST_DECLARATION
BOOTSTRAP_TRUST_INVALIDATION
FORMAL_FINDING_CLASSIFICATION
```

States:

```text
ABSENT
RECORDED
```

`RECORDED` is terminal for the exact one-shot lifecycle object.

Successor Evidence creates another lifecycle object.

It does not transition the old one.

---

# 13. ReviewAdmissionState

```text
ABSENT

ACCEPTED

REJECTED
```

Both:

```text
ACCEPTED
REJECTED
```

are terminal.

Repeated Admission attempts create distinct `REVIEW_ADMISSION_ATTEMPT` objects.

---

# 14. CloseoutAttemptState

```text
ABSENT

COMMITTED

REJECTED
```

Both:

```text
COMMITTED
REJECTED
```

are terminal for that exact Closeout Attempt.

Derived bracket state is NOT part of `CloseoutAttemptState`.

Bracket state remains:

```text
BracketState(Closeout, Policy)
```

as defined by Lifecycle §§90–95.

---

# 15. ArtifactEvictionState

```text
ABSENT

OPEN

COMMITTED

INTERRUPTED_RECOVERY

BLOCKED
```

Terminal states:

```text
COMMITTED

INTERRUPTED_RECOVERY

BLOCKED
```

`BLOCKED` is legal only before deliberate destructive removal begins.

---

# 16. Terminality Applicability

Define:

```text
HasTerminality(kind)
```

as:

```text
false:
  REGISTRY

true:
  every other v0.x lifecycle_object_kind
```

Conceptually:

```text
IsTerminal(kind, state)
```

is valid only under:

```text
requires HasTerminality(kind)
```

The formal implementation MAY model this using:

```text
a partial function

a function with a precondition

a tagged return such as OPTION<bool>
```

but MUST preserve the same semantics.

---

# 17. Meaning of the `terminal` Column

Lifecycle §20.1 requires a `terminal` column.

This document defines its values as:

```text
true

false

N/A
```

`N/A` means:

> terminality is not a defined semantic property for this lifecycle object kind.

Therefore:

```text
GENESIS
    terminal = N/A

STORAGE_CAPABILITY_CHANGED
    terminal = N/A

FREEZE_ATTEMPT_STARTED
    terminal = false

FREEZE_COMMITTED
    terminal = true

FORMAL_VERIFICATION_RECORDED
    terminal = true

FREEZE_COMMIT_REJECTED
    terminal = true
```

For `FREEZE_COMMIT_REJECTED`, the observing event inherits the already-terminal Freeze Attempt state.

---

# 18. Terminality Consistency Rule

For every row whose object kind satisfies:

```text
HasTerminality(kind)
```

the following MUST hold:

```text
table.terminal
==
IsTerminal(
  table.lifecycle_object_kind,
  table.resulting_state
)
```

For:

```text
kind = REGISTRY
```

the table value MUST be:

```text
N/A
```

and no `IsTerminal` evaluation is performed.

---

# 19. Dependency Semantics

This table distinguishes:

```text
direct AUTHORITY_DEPENDENCY

IDENTITY_DEPENDENCY

Journal Reference field
```

These are not interchangeable.

---

## 19.1 Direct AUTHORITY_DEPENDENCY

A direct authority dependency is an exact prior Journal Reference that MUST appear in:

```text
authority_dependencies[]
```

because the current event requires that exact prior authority.

Transitive dependencies need not be duplicated unless this table requires direct binding.

---

## 19.2 IDENTITY_DEPENDENCY

An identity dependency MUST appear in:

```text
identity_dependencies[]
```

where the event commits to the exact identity of another Record or immutable object but does not require independent Registry authority for that object.

---

## 19.3 Journal Reference field

A Journal Reference used for:

```text
operation interval
chronology bound
START relation
conflicting terminal relation
POST bracket back-reference
```

MAY also be an AUTHORITY_DEPENDENCY where semantics require authority.

Its existence as a Journal Reference field alone does not automatically decide dependency class.

---

# 20. Dependency Collections Are Semantic Sets

For semantic purposes:

```text
authority_dependencies[]

identity_dependencies[]
```

represent duplicate-free sets.

Duplicate dependency entries are invalid.

The Identity Format Freeze MUST define one canonical byte ordering for each serialized dependency collection.

Until that ordering is frozen:

```text
expected Journal Entry bytes

expected Journal Entry hashes
```

MUST NOT be declared stable.

Canonical dependency ordering and duplicate rejection are the first-priority decisions of the Identity Format Freeze.

---

# 21. Accepted vs Rejected Gate Events

An important distinction applies to:

```text
REVIEW_ADMISSION_ACCEPTED
REVIEW_ADMISSION_REJECTED

CLOSEOUT_COMMITTED
CLOSEOUT_REJECTED
```

An accepted/committed disposition requires all authority dependencies whose validity is necessary for success.

A rejected disposition MUST NOT fabricate missing authority merely so that a complete dependency list can be written.

Therefore:

> A rejection Record authority-depends only on prior authorities that were actually established and are required to identify the attempted operation.

Missing or invalid candidate support is recorded as rejection Evidence, not converted into a fake authority dependency.

---

# 22. Failed Candidate Identity Binding

A completed authoritative rejection MUST still identify the exact candidate inputs that caused the rejection wherever those identities were successfully determined.

For a candidate input whose Registry authority is absent, invalid, or unsuitable but whose exact immutable identity is known:

```text
the candidate identity MUST be bound
```

through:

```text
IDENTITY_DEPENDENCY
```

and/or an exact structured rejection-Record field included in Record identity.

The stable schema MUST define one canonical representation.

The candidate MUST NOT be converted into:

```text
AUTHORITY_DEPENDENCY
```

unless its Registry authority actually exists.

Thus:

```text
known exact candidate identity
+
missing / invalid authority

→ identity binding
→ rejection finding
→ no fabricated authority dependency
```

If the candidate identity itself cannot be reconstructed before the Lifecycle historical threshold is crossed, ordinary command/input-error semantics apply according to Lifecycle §83.1 or the analogous Closeout boundary.

This rule applies at minimum to:

```text
REVIEW_ADMISSION_REJECTED

CLOSEOUT_REJECTED
```

---

# 23. Master Normative Cross-Reference Table

Abbreviations used only inside this table:

```text
BC = BOOTSTRAP_CREATE
CO = CREATE_OPEN
CT = CREATE_TERMINAL
TT = TRANSITION_TERMINAL
ON = OBSERVE_NO_STATE_CHANGE
```

|  ID | Event                                          | Operation                        | Phase     | Shape | Object Kind                                     | Object ID             | Legal predecessor → result                              | Terminal | Direct AUTHORITY_DEPENDENCY                                                                                                                                                                  |
| --: | ---------------------------------------------- | -------------------------------- | --------- | ----- | ----------------------------------------------- | --------------------- | ------------------------------------------------------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
|   1 | `GENESIS`                                      | Registry Init                    | BOOTSTRAP | BC    | `REGISTRY`                                      | `registry_id`         | `ABSENT → INITIALIZED_AUTHORITATIVE`                    | N/A      | none                                                                                                                                                                                         |
| 100 | `FREEZE_ATTEMPT_STARTED`                       | Freeze                           | TWO_PHASE | CO    | `FREEZE_ATTEMPT`                                | `freeze_attempt_id`   | `ABSENT → OPEN`                                         | false    | none                                                                                                                                                                                         |
| 101 | `FREEZE_COMMITTED`                             | Freeze                           | TWO_PHASE | TT    | `FREEZE_ATTEMPT`                                | `freeze_attempt_id`   | `OPEN → COMMITTED`                                      | true     | exact `FREEZE_ATTEMPT_STARTED`                                                                                                                                                               |
| 102 | `FREEZE_ABORTED_RECOVERY`                      | Freeze Recovery                  | TWO_PHASE | TT    | `FREEZE_ATTEMPT`                                | `freeze_attempt_id`   | `OPEN → ABORTED_RECOVERY`                               | true     | exact `FREEZE_ATTEMPT_STARTED`                                                                                                                                                               |
| 103 | `FREEZE_ABORTED_BY_OPERATOR_ASSERTION`         | Operator Abort                   | TWO_PHASE | TT    | `FREEZE_ATTEMPT`                                | `freeze_attempt_id`   | `OPEN → ABORTED_BY_OPERATOR_ASSERTION`                  | true     | exact `FREEZE_ATTEMPT_STARTED`                                                                                                                                                               |
| 104 | `FREEZE_COMMIT_REJECTED`                       | Rejected Freeze Commit Recording | ONE_PHASE | ON    | `FREEZE_ATTEMPT`                                | `freeze_attempt_id`   | `T → T`, where `T` is an existing terminal Freeze state | true     | exact START + exact conflicting terminal event                                                                                                                                               |
| 200 | `VERIFICATION_RECORDED`                        | Verification Record              | ONE_PHASE | CT    | `VERIFICATION`                                  | `event_record_id`     | `ABSENT → RECORDED`                                     | true     | exact Freeze authority; exact Closeout authority iff `brackets_closeout_authority_ref` is present                                                                                            |
| 300 | `REVIEW_REQUEST_RECORDED`                      | Review Request                   | ONE_PHASE | CT    | `REVIEW_REQUEST`                                | `event_record_id`     | `ABSENT → RECORDED`                                     | true     | exact Freeze authority + exact Policy authority                                                                                                                                              |
| 301 | `REVIEW_RESULT_RECORDED`                       | Review Result                    | ONE_PHASE | CT    | `REVIEW_RESULT`                                 | `event_record_id`     | `ABSENT → RECORDED`                                     | true     | exact Review Request authority                                                                                                                                                               |
| 302 | `REVIEW_ADMISSION_ACCEPTED`                    | Review Admission                 | ONE_PHASE | CT    | `REVIEW_ADMISSION_ATTEMPT`                      | `event_record_id`     | `ABSENT → ACCEPTED`                                     | true     | exact Review Request + Review Result + Policy authorities                                                                                                                                    |
| 303 | `REVIEW_ADMISSION_REJECTED`                    | Review Admission                 | ONE_PHASE | CT    | `REVIEW_ADMISSION_ATTEMPT`                      | `event_record_id`     | `ABSENT → REJECTED`                                     | true     | exact successfully resolved Request / Result / Policy authorities required to identify the attempt; failed known candidates are identity-bound per §22                                       |
| 400 | `POLICY_RECORDED`                              | Policy Registration              | ONE_PHASE | CT    | `POLICY`                                        | `event_record_id`     | `ABSENT → RECORDED`                                     | true     | none                                                                                                                                                                                         |
| 500 | `CLOSEOUT_COMMITTED`                           | Closeout                         | ONE_PHASE | CT    | `CLOSEOUT_ATTEMPT`                              | `event_record_id`     | `ABSENT → COMMITTED`                                    | true     | exact Freeze + Policy + every admitted Review Admission + required creation-time Verification + required PRE Verification + exact formal/bootstrap support + predecessor Closeout if present |
| 501 | `CLOSEOUT_REJECTED`                            | Closeout                         | ONE_PHASE | CT    | `CLOSEOUT_ATTEMPT`                              | `event_record_id`     | `ABSENT → REJECTED`                                     | true     | exact established context authorities and every candidate support authority successfully resolved; failed known candidates are identity-bound per §22                                        |
| 600 | `STORAGE_CAPABILITY_CHANGED`                   | Storage Capability Change        | ONE_PHASE | ON    | `REGISTRY`                                      | `registry_id`         | `INITIALIZED_AUTHORITATIVE → INITIALIZED_AUTHORITATIVE` | N/A      | none                                                                                                                                                                                         |
| 700 | `ARTIFACT_EVICTION_STARTED`                    | Artifact Eviction                | TWO_PHASE | CO    | `ARTIFACT_EVICTION_ATTEMPT`                     | `eviction_attempt_id` | `ABSENT → OPEN`                                         | false    | exact Freeze authority whose embedded bytes are targeted                                                                                                                                     |
| 701 | `ARTIFACT_EVICTION_COMMITTED`                  | Artifact Eviction                | TWO_PHASE | TT    | `ARTIFACT_EVICTION_ATTEMPT`                     | `eviction_attempt_id` | `OPEN → COMMITTED`                                      | true     | exact `ARTIFACT_EVICTION_STARTED`                                                                                                                                                            |
| 702 | `ARTIFACT_EVICTION_INTERRUPTED_RECOVERY`       | Eviction Recovery                | TWO_PHASE | TT    | `ARTIFACT_EVICTION_ATTEMPT`                     | `eviction_attempt_id` | `OPEN → INTERRUPTED_RECOVERY`                           | true     | exact `ARTIFACT_EVICTION_STARTED`                                                                                                                                                            |
| 703 | `ARTIFACT_EVICTION_BLOCKED`                    | Artifact Eviction                | TWO_PHASE | TT    | `ARTIFACT_EVICTION_ATTEMPT`                     | `eviction_attempt_id` | `OPEN → BLOCKED`                                        | true     | exact `ARTIFACT_EVICTION_STARTED`                                                                                                                                                            |
| 800 | `ASSUMPTION_DEFINITION_RECORDED`               | Assumption Definition            | ONE_PHASE | CT    | `ASSUMPTION_DEFINITION`                         | `event_record_id`     | `ABSENT → RECORDED`                                     | true     | none                                                                                                                                                                                         |
| 801 | `ASSUMPTION_ESTABLISHMENT_RECORDED`            | Assumption Establishment         | ONE_PHASE | CT    | `ASSUMPTION_ESTABLISHMENT`                      | `event_record_id`     | `ABSENT → RECORDED`                                     | true     | exact Assumption Definition authority                                                                                                                                                        |
| 802 | `ASSUMPTION_INVALIDATION_RECORDED`             | Assumption Invalidation          | ONE_PHASE | CT    | `ASSUMPTION_INVALIDATION`                       | `event_record_id`     | `ABSENT → RECORDED`                                     | true     | explicit mode: exact targeted Establishment authorities; predicate mode: no per-target authority dependency                                                                                  |
| 803 | `FORMAL_VERIFICATION_RECORDED`                 | Formal Verification              | ONE_PHASE | CT    | `FORMAL_VERIFICATION`                           | `event_record_id`     | `ABSENT → RECORDED`                                     | true     | exact Assumption Definition authorities consumed by formal dependency set; Registry-bound formal Subject Freeze authority iff such authority is claimed                                      |
| 804 | `ASSUMPTION_VERSION_COMPATIBILITY_RECORDED`    | Assumption Compatibility         | ONE_PHASE | CT    | `ASSUMPTION_VERSION_COMPATIBILITY`              | `event_record_id`     | `ABSENT → RECORDED`                                     | true     | exact source Definition + exact target Definition authorities                                                                                                                                |
| 805 | `ASSUMPTION_VERSION_COMPATIBILITY_INVALIDATED` | Compatibility Invalidation       | ONE_PHASE | CT    | `ASSUMPTION_VERSION_COMPATIBILITY_INVALIDATION` | `event_record_id`     | `ABSENT → RECORDED`                                     | true     | explicit mode: exact targeted Compatibility authorities; predicate mode: no per-target authority dependency                                                                                  |
| 806 | `BOOTSTRAP_TRUST_DECLARATION_RECORDED`         | Bootstrap Trust Declaration      | ONE_PHASE | CT    | `BOOTSTRAP_TRUST_DECLARATION`                   | `event_record_id`     | `ABSENT → RECORDED`                                     | true     | predecessor Declaration authority iff `predecessor_declaration_ref` present                                                                                                                  |
| 807 | `BOOTSTRAP_TRUST_INVALIDATED`                  | Bootstrap Trust Invalidation     | ONE_PHASE | CT    | `BOOTSTRAP_TRUST_INVALIDATION`                  | `event_record_id`     | `ABSENT → RECORDED`                                     | true     | explicit mode: exact targeted Bootstrap Declaration authorities; predicate mode: no per-target authority dependency                                                                          |
| 808 | `FORMAL_FINDING_CLASSIFICATION_RECORDED`       | Formal Finding Classification    | ONE_PHASE | CT    | `FORMAL_FINDING_CLASSIFICATION`                 | `event_record_id`     | `ABSENT → RECORDED`                                     | true     | exact Formal Verification authority                                                                                                                                                          |

This table is normative.

---

# 24. GENESIS Rules

## 24.1 Object

```text
kind = REGISTRY
object_id = registry_id
```

---

## 24.2 Journal-only index fields

Beyond common Journal Entry fields, GENESIS MUST permit Journal-only reconstruction of the first capability epoch.

The common fields therefore include:

```text
storage_capability_class_id
environment_observation_id
```

and these MUST agree with the GENESIS Record.

---

## 24.3 State rule

No second:

```text
GENESIS
```

is legal in the same Registry history.

---

## 24.4 Terminality

Terminality does not apply to `REGISTRY`.

Therefore:

```text
GENESIS.terminal = N/A
```

---

# 25. Freeze START Rules

`FREEZE_ATTEMPT_STARTED` MUST expose sufficient Journal-native indexing for:

```text
open Freeze Attempt set

Attempt → intended root relation
```

Required event-specific Journal fields:

```text
freeze_attempt_id

intended_root_id
```

where `intended_root_id` is the canonical identity of the root deterministically derived for this Attempt.

The exact byte encoding of `intended_root_id` is deferred to Identity Format Freeze.

---

# 26. Freeze Terminal Rules

Events:

```text
FREEZE_COMMITTED

FREEZE_ABORTED_RECOVERY

FREEZE_ABORTED_BY_OPERATOR_ASSERTION
```

MUST include event-specific Journal fields:

```text
freeze_attempt_id

attempt_start_journal_ref
```

and:

```text
attempt_start_journal_ref
```

MUST be an AUTHORITY_DEPENDENCY on the exact `FREEZE_ATTEMPT_STARTED`.

---

## 26.1 FREEZE_COMMITTED

`FREEZE_COMMITTED` additionally requires successful validation of the exact Freeze Receipt and its START binding.

The event's Record payload MUST bind the exact Receipt identity or contain the exact terminal payload defined by the later identity/schema freeze.

---

# 27. FREEZE_COMMIT_REJECTED Rules

Required event-specific fields:

```text
freeze_attempt_id

attempt_start_journal_ref

conflicting_terminal_journal_ref

attempted_transition

rejection_reason
```

Required authority dependencies:

```text
attempt_start_journal_ref

conflicting_terminal_journal_ref
```

Legal predecessor state:

```text
COMMITTED

ABORTED_RECOVERY

ABORTED_BY_OPERATOR_ASSERTION
```

Resulting state:

```text
same state
```

The event MUST NOT create a new Freeze Attempt state.

---

## 27.1 Repeated rejection events

More than one:

```text
FREEZE_COMMIT_REJECTED
```

MAY legally refer to the same terminal Freeze Attempt.

Example:

```text
writer A returns
→ rejected

writer B returns
→ rejected
```

Each rejection is a distinct one-shot event Record observing the same terminal Freeze Attempt.

Every such event MUST preserve the existing Freeze state.

No duplicate-rejection event changes lifecycle state or reopens the Attempt.

---

# 28. Verification Rules

`VERIFICATION_RECORDED` is a one-shot terminal lifecycle object.

Required Journal-specific chronology field:

```text
observation_start_journal_ref
```

Required direct authority dependency:

```text
freeze_authority_ref
```

For POST bracket Verification:

```text
brackets_closeout_authority_ref
```

MUST be present as an event-specific Journal Reference field and MUST also be represented as a direct AUTHORITY_DEPENDENCY on the exact:

```text
CLOSEOUT_COMMITTED
```

event.

For non-bracket Verification:

```text
brackets_closeout_authority_ref
```

MUST be absent.

The presence or absence of this exact field is the normative Journal-level discriminator for POST-bracket intent.

For:

```text
SOURCE_SUBJECT_MATCH
```

the exact immutable:

```text
source_binding_ref
```

is a required IDENTITY_DEPENDENCY.

For:

```text
EMBEDDED_COPY_INTEGRITY
```

a Source Binding is not automatically required.

---

## 28.1 Journal-only POST-bracket classification

Journal-only replay determines POST-bracket intent through:

```text
brackets_closeout_authority_ref present
    → structurally declared POST-bracket intent

brackets_closeout_authority_ref absent
    → ordinary Verification
```

The field MUST identify an exact prior:

```text
CLOSEOUT_COMMITTED
```

Journal event and MUST satisfy all common Journal Reference validation rules.

The same reference MUST appear consistently in:

```text
brackets_closeout_authority_ref

and

authority_dependencies[]
```

according to the stable schema.

Journal-only replay does NOT establish full POST eligibility because it cannot reconstruct from Journal metadata alone:

```text
verification_target_type

Source Binding contents

Freeze / Source semantic equality

Policy satisfaction

finding_state

method_status
```

Therefore:

```text
Journal-only:
    POST intent and exact Closeout back-reference identifiable

Record payload required:
    full EligiblePost(...)
```

The implementation MUST NOT infer POST intent merely from the presence of some unrelated Closeout authority dependency.

---

# 29. Review Request Rules

`REVIEW_REQUEST_RECORDED` requires direct authority dependencies on:

```text
exact Freeze authority

exact Policy authority
```

Required identity dependencies include, where represented as content-addressed objects:

```text
manifest_id

required_checks_ref

review_scope_ref

review_method_ref

review_package_anchor
```

The Anchor is a historical commitment, not Registry authority.

It therefore MUST NOT be inserted into `authority_dependencies[]` merely because it is required by Review transport.

---

# 30. Review Result Rules

`REVIEW_RESULT_RECORDED` MUST directly authority-depend on:

```text
exact Review Request authority
```

Its Record payload MUST redundantly bind and validate the Request-defined:

```text
Freeze / Subject identity

manifest_id

review_role

review_scope_ref

review_method_ref

review_package_anchor
```

where required by Lifecycle.

It MUST additionally carry:

```text
operation_start_journal_ref
```

as required by §48.

This requirement supplements the minimum field list in Lifecycle §80.

The redundant equality checks do not create separate lifecycle states.

---

# 31. Review Admission Rules

Admission is one-shot.

There is no:

```text
REVIEW_ADMISSION_STARTED
```

event.

Therefore:

```text
ABSENT
→
ACCEPTED
```

or:

```text
ABSENT
→
REJECTED
```

occurs in one terminal Journal append.

Admission Record schemas MUST include:

```text
operation_start_journal_ref
```

as required by §48.

This supplements Lifecycle §§82–83.

---

## 31.1 Accepted Admission

`REVIEW_ADMISSION_ACCEPTED` MUST directly authority-depend on:

```text
Review Request

Review Result

Policy
```

that were actually evaluated.

---

## 31.2 Rejected Admission

`REVIEW_ADMISSION_REJECTED` records a completed mechanical Admission disposition.

It MUST bind exact successfully resolved authority inputs.

It MUST NOT fabricate an authority dependency for an input that failed because:

```text
authority missing

identity invalid

schema unsupported

binding mismatched
```

Such failure remains part of the rejection payload.

Every exact failed candidate identity that was successfully determined MUST be bound according to §22.

Failures before Lifecycle §83.1's historical threshold remain ordinary input/command errors rather than authoritative Admission objects.

---

# 32. Policy Rules

`POLICY_RECORDED` is:

```text
ABSENT
→
RECORDED
```

and terminal.

Policy registration has no mandatory prior Registry authority dependency beyond the existing Registry itself.

The Policy Record schema MUST include:

```text
operation_start_journal_ref
```

as required by §48.

This supplements Lifecycle §§84–85.

Where the Policy body contains exact content-addressed references such as:

```text
Scope Records

Method Records

Check Records

classifier procedure identities

other frozen semantic definitions
```

those identities MUST be represented as IDENTITY_DEPENDENCIES where the identity format defines them as Records.

Policy authority does not make those referenced semantics mutable.

---

# 33. Closeout Rules

Closeout is one-shot per attempt.

There is no:

```text
CLOSEOUT_STARTED
```

event.

Therefore:

```text
ABSENT
→
COMMITTED
```

or:

```text
ABSENT
→
REJECTED
```

occurs at one terminal Journal append.

---

## 33.1 CLOSEOUT_COMMITTED direct dependencies

A successful Closeout MUST directly authority-depend on every exact authority whose validity is required to establish that Closeout.

At minimum, where applicable:

```text
Freeze authority

Policy authority

each REVIEW_ADMISSION_ACCEPTED authority
used by the Closeout

each creation-time Verification authority

PRE Verification authority
where bracket Policy requires PRE

predecessor Closeout authority
where predecessor_closeout_id is present

Formal Verification authorities

Assumption Establishment authorities

Assumption Version Compatibility authorities
where cross-version support was used

Bootstrap Trust Declaration authorities

Formal Finding Classification authorities
where Policy requires Classification
```

No required support authority may be represented only as prose.

---

## 33.2 Formal support closure

A Closeout MAY have many direct support authorities.

These references MUST appear in:

```text
authority_dependencies[]
```

rather than relying exclusively on transitive inference.

This preserves the exact historical support snapshot.

---

## 33.3 CLOSEOUT_REJECTED

A rejected Closeout MUST directly authority-depend on the authorities required to identify the attempted qualification context:

```text
Freeze authority

Policy authority

predecessor Closeout authority
where present
```

and every candidate support authority the rejection Record states was successfully resolved.

If a required candidate support item was absent, invalid, unadmitted, or otherwise failed:

```text
it MUST remain a rejection finding
```

and MUST NOT be invented as an authority dependency.

Every exact failed candidate identity that was successfully determined MUST additionally be bound according to §22.

---

# 34. Closeout Bracket Is Not a Closeout Lifecycle Transition

After:

```text
CLOSEOUT_COMMITTED
```

the Closeout Attempt remains:

```text
COMMITTED
```

regardless of later POST.

Derived:

```text
BracketState(Closeout, Policy)
```

may become:

```text
PENDING_POSTCONDITION
SATISFIED
FAILED
```

without changing:

```text
CloseoutAttemptState
```

Therefore no POST event transitions `CLOSEOUT_ATTEMPT`.

Instead, the POST Verification is its own:

```text
VERIFICATION
```

lifecycle object with an AUTHORITY_DEPENDENCY back to the Closeout.

---

# 35. Storage Capability Change Rules

`STORAGE_CAPABILITY_CHANGED` uses:

```text
kind = REGISTRY

object_id = registry_id
```

and:

```text
INITIALIZED_AUTHORITATIVE
→
INITIALIZED_AUTHORITATIVE
```

It is an observing event over Registry lifecycle state.

Terminality is not applicable.

It changes the Registry's authoritative capability epoch history, not its initialization state.

Identity dependencies SHOULD bind the exact:

```text
prior / new Storage Capability Class Records

Environment Observation Records
```

used by the event payload.

The current Entry common fields MUST expose the capability class/environment observation applicable to that Entry.

---

# 36. Artifact Eviction Attempt Identity

Artifact eviction is TWO_PHASE and therefore requires a stable Attempt identity.

Define:

```text
eviction_attempt_id
```

as the `lifecycle_object_id` for:

```text
ARTIFACT_EVICTION_ATTEMPT
```

All events 700–703 belonging to the same Attempt MUST carry the same value.

Its exact generation and byte encoding belong to Identity Format Freeze.

It SHOULD provide collision resistance equivalent to other lifecycle Attempt IDs.

---

# 37. Artifact Eviction START Journal Indexing

`ARTIFACT_EVICTION_STARTED` MUST include event-specific Journal fields:

```text
eviction_attempt_id

freeze_authority_ref

pre_eviction_manifest_id

eviction_scope_identity
```

where:

```text
eviction_scope_identity
```

mechanically identifies the exact intended eviction scope.

Required direct authority dependency:

```text
freeze_authority_ref
```

Required identity dependencies include:

```text
pre_eviction_manifest_id

eviction_scope_identity
```

where the scope is represented as an immutable Record.

---

# 38. Artifact Eviction Terminal Journal Indexing

Events:

```text
ARTIFACT_EVICTION_COMMITTED

ARTIFACT_EVICTION_INTERRUPTED_RECOVERY

ARTIFACT_EVICTION_BLOCKED
```

MUST contain:

```text
eviction_attempt_id

eviction_start_journal_ref
```

`eviction_start_journal_ref` MUST be a direct AUTHORITY_DEPENDENCY on the exact:

```text
ARTIFACT_EVICTION_STARTED
```

event.

This is sufficient for Journal-only reconstruction of:

```text
open Artifact Eviction set

terminal Artifact Eviction set

Eviction Attempt → terminal disposition
```

---

## 38.1 BLOCKED guard

`ARTIFACT_EVICTION_BLOCKED` is legal only while:

```text
no target byte has been deliberately removed
```

by that Attempt.

If deletion began:

```text
BLOCKED
```

is not a legal resulting state.

Recovery must use:

```text
ARTIFACT_EVICTION_INTERRUPTED_RECOVERY
```

where applicable.

---

# 39. Assumption Definition Rules

`ASSUMPTION_DEFINITION_RECORDED` creates a terminal one-shot object.

No prior Assumption authority is required.

Identity dependencies include exact referenced semantic Records such as:

```text
scope_coverage_rules_ref

allowed establishment Method identities
```

where those are represented as content-addressed Records.

---

# 40. Assumption Establishment Rules

`ASSUMPTION_ESTABLISHMENT_RECORDED` MUST directly authority-depend on:

```text
exact Assumption Definition authority
```

Required identity dependencies include exact identities for:

```text
Scope

Method

Storage Capability Class

Environment Observation

capability observation provenance
where required

supporting Evidence
where represented as identity Records
```

It MUST contain:

```text
observation_start_journal_ref
```

as defined by Formal Verification Specification v0.5.4 §27.

No alternative:

```text
operation_start_journal_ref
```

is permitted for the Assumption Establishment observation lower bound in v0.x.

The observation reference is a chronology field.

It does not replace the Assumption Definition authority dependency.

---

# 41. Assumption Invalidation Rules

`ASSUMPTION_INVALIDATION_RECORDED` is one-shot.

Two modes exist.

---

## 41.1 EXPLICIT_ESTABLISHMENT_SET

For every explicitly targeted Establishment:

```text
exact Establishment authority
```

MUST be present as a direct AUTHORITY_DEPENDENCY.

---

## 41.2 ATTRIBUTE_PREDICATE

Predicate mode MUST NOT enumerate future Establishments as dependencies.

Per-target authority dependencies are not required.

Instead the invalidation Record MUST identity-bind:

```text
exact predicate language/version

exact selector

exact immutable metadata fields/rules
```

and its authoritative Journal position defines the evaluation horizon.

Future Establishments are outside that horizon.

---

# 42. Formal Verification Rules

`FORMAL_VERIFICATION_RECORDED` creates a terminal one-shot Formal Verification object.

It MUST directly authority-depend on each exact:

```text
Assumption Definition
```

consumed by the formal dependency set.

It MUST identity-bind:

```text
formal Subject identity

formal model identity

theorem identity

Proof Dependency Manifest

Dafny identity/configuration

SMT solver identity/configuration
```

Where the Formal Verification explicitly claims that its formal Subject is a Registry-authoritative frozen Subject, the exact:

```text
Freeze authority
```

MUST additionally be a direct AUTHORITY_DEPENDENCY.

A theorem proof does NOT authority-depend on concrete Assumption Establishments merely to prove the theorem under abstract Assumptions.

Establishments are consumed later during applicability/Closeout evaluation.

---

# 43. Compatibility Rules

`ASSUMPTION_VERSION_COMPATIBILITY_RECORDED` MUST directly authority-depend on:

```text
source Assumption Definition authority

target Assumption Definition authority
```

It MUST identity-bind:

```text
scope_mapping_ref

Method identity

other immutable compatibility procedure inputs
```

Compatibility is directional.

The dependency pair does not imply reverse Compatibility.

---

# 44. Compatibility Invalidation Rules

`ASSUMPTION_VERSION_COMPATIBILITY_INVALIDATED` follows the same two target modes as Assumption Invalidation.

---

## 44.1 Explicit mode

Every exact targeted Compatibility Record authority MUST be a direct AUTHORITY_DEPENDENCY.

---

## 44.2 Predicate mode

No per-target authority dependency is required.

The predicate and its immutable evaluation vocabulary MUST be identity-bound.

The Journal position defines the horizon.

---

# 45. Bootstrap Trust Declaration Rules

`BOOTSTRAP_TRUST_DECLARATION_RECORDED` creates a terminal one-shot object.

Required identity dependencies include:

```text
declaration_scope_ref

rationale refs

bootstrap Evidence identities
where represented as identity Records
```

If:

```text
predecessor_declaration_ref
```

is present, the exact predecessor Bootstrap Trust Declaration MUST be a direct AUTHORITY_DEPENDENCY.

Succession does not invalidate the predecessor.

---

# 46. Bootstrap Trust Invalidation Rules

`BOOTSTRAP_TRUST_INVALIDATED` uses:

```text
EXPLICIT_DECLARATION_SET

ATTRIBUTE_PREDICATE
```

semantics.

Explicitly targeted Declaration authorities are direct AUTHORITY_DEPENDENCIES.

Predicate mode has no per-target dependency and is bounded by the invalidation's Journal position.

---

# 47. Formal Finding Classification Rules

`FORMAL_FINDING_CLASSIFICATION_RECORDED` MUST directly authority-depend on:

```text
exact Formal Verification authority
```

It MUST identity-bind:

```text
classifier_procedure_identity

rationale_ref

supporting Evidence identities
where represented as Records
```

Supporting Evidence explicitly typed as authoritative Journal References MUST additionally appear in:

```text
authority_dependencies[]
```

Supporting evidence that is merely content-addressed does not acquire authority through this Classification event.

---

# 48. Required Journal Chronology Fields

ONE_PHASE operation-start or observation bounds are not all lifecycle states.

The following fields are nevertheless normative chronology inputs.

| Event class                   | Required chronology field       |
| ----------------------------- | ------------------------------- |
| `VERIFICATION_RECORDED`       | `observation_start_journal_ref` |
| Review Request                | `operation_start_journal_ref`   |
| Review Result                 | `operation_start_journal_ref`   |
| Review Admission              | `operation_start_journal_ref`   |
| Policy registration           | `operation_start_journal_ref`   |
| Closeout                      | `operation_start_journal_ref`   |
| Assumption Definition         | `operation_start_journal_ref`   |
| Assumption Establishment      | `observation_start_journal_ref` |
| Assumption Invalidation       | `operation_start_journal_ref`   |
| Formal Verification           | `operation_start_journal_ref`   |
| Compatibility                 | `operation_start_journal_ref`   |
| Compatibility Invalidation    | `operation_start_journal_ref`   |
| Bootstrap Trust Declaration   | `operation_start_journal_ref`   |
| Bootstrap Trust Invalidation  | `operation_start_journal_ref`   |
| Formal Finding Classification | `operation_start_journal_ref`   |

Exact field placement is frozen by the subsequent Record / Identity Schema work.

The semantic distinction remains:

```text
observation_start_journal_ref
    = lower bound of an observation interval

operation_start_journal_ref
    = Registry head observed at ordinary operation start
```

---

## 48.1 Current Policy Semantics of operation_start_journal_ref

For operations using:

```text
operation_start_journal_ref
```

the field currently serves as:

```text
chronology provenance
+
operation interval lower-bound evidence
```

unless another specification explicitly assigns stronger Policy semantics.

Lifecycle v0.10.2 does not currently define a general Policy mechanism constraining the operation interval of:

```text
Assumption Definition

Formal Verification

Compatibility

Bootstrap Trust Declaration

Formal Finding Classification
```

in the same way that Verification uses:

```text
max_journal_distance_to_closeout

intervening-event constraint
```

Therefore the presence of `operation_start_journal_ref` MUST NOT by itself be interpreted as:

```text
Policy-constrained freshness

continuous environment stability

no relevant intervening event

formal execution interval qualification
```

For v0.x it preserves chronology for future analysis.

A future requirement to constrain a long-running Formal Verification execution interval MAY require:

```text
new Policy semantics

and/or
a dedicated observation_start_journal_ref-style field

and/or
a specification successor
```

That future possibility does not alter current semantics.

---

# 49. Common Validation of Every Journal Reference

Every Journal Reference field, regardless of whether it is:

```text
an AUTHORITY_DEPENDENCY

an observation_start_journal_ref

an operation_start_journal_ref

a START relation

a conflicting-terminal relation

a brackets_closeout_authority_ref

another chronology reference
```

MUST be structurally validated.

At minimum:

```text
referenced entry_hash recomputes correctly

referenced registry_id == current registry_id

referenced entry exists in retained authoritative Journal

referenced entry_index < current entry_index

referenced event_type_id matches the field's expected
event class where that field constrains event type

referenced lifecycle_object_id matches the required
logical object where applicable
```

Cross-Registry chronology or authority is not valid in Profile L v0.x.

A chronology reference does not become an AUTHORITY_DEPENDENCY merely because it passes these checks.

Authority semantics remain separately defined.

---

# 50. Journal-only Replay Requirements

Journal Entry bodies alone MUST support reconstruction of at least:

```text
Registry initialized/not initialized

open Freeze Attempt set

terminal Freeze Attempt set

Freeze Attempt → intended root

Freeze Attempt → terminal disposition

open Artifact Eviction Attempt set

terminal Artifact Eviction Attempt set

Artifact Eviction Attempt → terminal disposition

capability epoch of every Journal Entry

direct Registry authority dependency graph

whether a Verification explicitly contains
brackets_closeout_authority_ref

the exact Closeout Journal Reference carried by that field
```

The last two items establish only:

```text
POST-bracket intent

and exact historical Closeout back-reference
```

not complete POST eligibility.

Record payloads remain required for semantic Policy evaluation.

---

# 51. Journal-only Replay Algorithm: Freeze

Conceptually:

```text
for entry in Journal order:

  if event == FREEZE_ATTEMPT_STARTED:
      require object kind == FREEZE_ATTEMPT
      require object id == freeze_attempt_id
      require attempt absent
      state[attempt] = OPEN

  if event in {
      FREEZE_COMMITTED,
      FREEZE_ABORTED_RECOVERY,
      FREEZE_ABORTED_BY_OPERATOR_ASSERTION
  }:
      require matching START
      require state == OPEN
      require exact START authority dependency
      state[attempt] = terminal result

  if event == FREEZE_COMMIT_REJECTED:
      require matching START
      require current state terminal
      require conflicting terminal ref
      require resulting state == prior state

      // repeated rejection events are allowed
      // state remains unchanged
```

There is no maximum number of valid `FREEZE_COMMIT_REJECTED` observations for one terminal Attempt in v0.x.

---

# 52. Journal-only Replay Algorithm: Eviction

Conceptually:

```text
for entry in Journal order:

  if event == ARTIFACT_EVICTION_STARTED:
      require object kind == ARTIFACT_EVICTION_ATTEMPT
      require object id == eviction_attempt_id
      require attempt absent
      state[attempt] = OPEN

  if event in {
      ARTIFACT_EVICTION_COMMITTED,
      ARTIFACT_EVICTION_INTERRUPTED_RECOVERY,
      ARTIFACT_EVICTION_BLOCKED
  }:
      require matching START
      require state == OPEN
      require exact START authority dependency
      state[attempt] = terminal result
```

Semantic legality of BLOCKED versus destructive-progress state requires the event Record payload and recovery observations.

Journal replay alone establishes terminal disposition, not the full physical deletion history.

---

# 53. Journal-only Replay Algorithm: Verification Bracket Intent

Conceptually:

```text
if event == VERIFICATION_RECORDED:

    if brackets_closeout_authority_ref present:
        validate Journal Reference
        require referenced event == CLOSEOUT_COMMITTED
        require same Registry
        require strictly prior
        require same exact Journal Reference present in
            authority_dependencies[]

        bracket_intent = POST

    else:
        bracket_intent = NONE
```

This does NOT evaluate:

```text
EligiblePost(...)
```

which still requires authoritative Record payloads and Policy.

---

# 54. Direct Authority Graph Rule

For every authoritative Journal Entry:

```text
for dep in authority_dependencies:

    dep.entry_index < current.entry_index
```

and every dependency MUST:

```text
recompute validly

belong to same registry_id

identify the expected event/object class
```

unless a future federation profile explicitly permits cross-Registry authority.

Profile L v0.x does not.

---

# 55. No Forward Authority Dependency

Normative rule:

```text
∀ event E
∀ authority dependency D of E:

    D.entry_index < E.entry_index
```

Cycles are therefore impossible in a valid same-Registry authority graph.

This is direct input to:

```text
NoForwardAuthorityDependency
```

in the Dafny model.

---

# 56. Chronology References Are Also Strictly Prior

For every Journal Reference field used as a chronology bound or prior-event reference:

```text
R.entry_index < current.entry_index
```

MUST hold.

This includes:

```text
observation_start_journal_ref

operation_start_journal_ref

attempt_start_journal_ref

eviction_start_journal_ref

conflicting_terminal_journal_ref

brackets_closeout_authority_ref
```

subject to their exact field semantics.

A future reference is invalid even when it is not an authority dependency.

---

# 57. State-Observing Events Do Not Transition

Define:

```text
IsStateObservingEvent(e)
```

for:

```text
FREEZE_COMMIT_REJECTED

STORAGE_CAPABILITY_CHANGED
```

Then the formal model SHOULD prove:

```text
StateObservingEventsDoNotTransition:

  LegalTransition(kind, s, e, s')
  ∧ IsStateObservingEvent(e)

  ⇒

  s == s'
```

This is a PURE_FORMAL property.

---

# 58. Create-Open Rule

For every `CREATE_OPEN` event:

```text
predecessor = ABSENT

result = OPEN
```

and the lifecycle object MUST NOT already exist.

Candidate theorem:

```text
CreateOpenRequiresAbsence
```

---

# 59. Transition-Terminal Rule

For every `TRANSITION_TERMINAL` event:

```text
predecessor = OPEN
```

and where terminality applies:

```text
IsTerminal(kind, result) = true
```

The exact START authority MUST already exist.

Candidate theorem:

```text
TerminalTransitionRequiresOpen
```

---

# 60. Create-Terminal Rule

For every `CREATE_TERMINAL` event:

```text
predecessor = ABSENT
```

and:

```text
IsTerminal(kind, result) = true
```

There is no separate OPEN state for that exact one-shot lifecycle object.

Candidate theorem:

```text
OneShotCreationIsTerminal
```

---

# 61. One-Shot Object Non-Transition Rule

Once a one-shot Record-backed lifecycle object reaches:

```text
RECORDED
ACCEPTED
REJECTED
COMMITTED
```

as applicable, that exact `lifecycle_object_id` MUST NOT later transition.

Correction creates a new lifecycle object.

Candidate theorem:

```text
OneShotTerminalObjectsDoNotTransition
```

---

# 62. Registry Observation Rule

The Registry object is intentionally long-lived.

`STORAGE_CAPABILITY_CHANGED` does not transition:

```text
RegistryState
```

but may change the authoritative capability epoch used by later entries.

Therefore:

```text
Registry lifecycle state
!=
Registry capability epoch
```

and:

```text
Registry lifecycle state
!=
terminality
```

These MUST be modeled separately.

---

# 63. Closeout Bracket Separation Rule

Dafny MUST NOT encode:

```text
SATISFIED
FAILED
PENDING_POSTCONDITION
```

as `CloseoutAttemptState`.

Instead model:

```text
CloseoutAttemptState
```

and:

```text
BracketState(Closeout, Policy, JournalState)
```

separately.

This prevents a later POST Verification from appearing to mutate the historical Closeout Record.

---

# 64. Lifecycle Datatype Skeleton

This table is intended to permit a near-mechanical first Dafny model.

Conceptually:

```text
datatype LifecycleObjectKind =
    Registry
  | FreezeAttempt
  | Verification
  | ReviewRequest
  | ReviewResult
  | ReviewAdmissionAttempt
  | Policy
  | CloseoutAttempt
  | ArtifactEvictionAttempt
  | AssumptionDefinition
  | AssumptionEstablishment
  | AssumptionInvalidation
  | FormalVerification
  | AssumptionVersionCompatibility
  | AssumptionVersionCompatibilityInvalidation
  | BootstrapTrustDeclaration
  | BootstrapTrustInvalidation
  | FormalFindingClassification
```

Exact Dafny syntax is non-normative.

The symbolic vocabulary is normative.

---

# 65. Candidate State Datatypes

Conceptually:

```text
datatype RegistryState =
    RegistryAbsent
  | InitializedAuthoritative

datatype FreezeAttemptState =
    FreezeAbsent
  | FreezeOpen
  | FreezeCommitted
  | FreezeAbortedRecovery
  | FreezeAbortedByOperatorAssertion

datatype OneShotRecordedState =
    OneShotAbsent
  | Recorded

datatype ReviewAdmissionState =
    AdmissionAbsent
  | AdmissionAccepted
  | AdmissionRejected

datatype CloseoutAttemptState =
    CloseoutAbsent
  | CloseoutCommitted
  | CloseoutRejected

datatype ArtifactEvictionState =
    EvictionAbsent
  | EvictionOpen
  | EvictionCommitted
  | EvictionInterruptedRecovery
  | EvictionBlocked
```

Exact syntax is non-normative.

---

# 66. Candidate Event Shape Datatype

Conceptually:

```text
datatype EventShape =
    BootstrapCreate
  | CreateOpen
  | CreateTerminal
  | TransitionTerminal
  | ObserveNoStateChange
```

This is recommended because event shape and operation phase are independent.

---

# 67. Candidate Terminality API

Conceptually:

```text
function HasTerminality(
    kind: LifecycleObjectKind
): bool
```

with:

```text
HasTerminality(Registry) = false

HasTerminality(k) = true
for every other v0.x kind
```

Then:

```text
function IsTerminal(
    kind: LifecycleObjectKind,
    state: LifecycleObjectState
): bool
    requires HasTerminality(kind)
```

or an equivalent type-safe construction.

A Dafny model MUST NOT call `IsTerminal` for `Registry`.

---

# 68. Candidate Core Predicates

The first formal core SHOULD be able to derive:

```text
EventShapeOf(event)

ObjectKindOf(event)

HasTerminality(kind)

LegalPredecessor(kind, event, state)

ResultingState(kind, event, state)

IsTerminal(kind, state)
  where applicable

RequiredAuthorityDependencyClass(event)

JournalReferencesValid(event)

LegalTransition(kind, before, event, after)
```

---

# 69. LegalTransition Must Permit State Preservation

Do NOT define:

```text
LegalTransition(...)
```

in a way that requires:

```text
before != after
```

because:

```text
FREEZE_COMMIT_REJECTED

STORAGE_CAPABILITY_CHANGED
```

are valid authoritative events with:

```text
before == after
```

---

# 70. Suggested LegalTransition Shape

Conceptually:

```text
LegalTransition(kind, before, event, after)
iff

  EventAppliesToKind(event, kind)

  ∧ LegalPredecessor(kind, event, before)

  ∧ after == ResultState(kind, before, event)

  ∧ RequiredAuthorityDependenciesSatisfied(event)

  ∧ JournalReferencesValid(event)

  ∧ ContextFieldsValid(event)
```

For state-observing events:

```text
ResultState(kind, before, event)
=
before
```

---

# 71. `terminal` Is a Conformance Value

The master table retains `terminal` because Lifecycle §20.1 requires it.

For kinds having terminality, formal code SHOULD derive it as:

```text
terminal
=
IsTerminal(
  lifecycle_object_kind,
  resulting_state
)
```

For `REGISTRY`:

```text
terminal = N/A
```

The table's value acts as a conformance assertion.

---

# 72. Cross-Reference Completeness Invariant

Define:

```text
RegisteredEvents
```

from Lifecycle §21.

Define:

```text
CrossReferenceEvents
```

from this document.

Required:

```text
RegisteredEvents
==
CrossReferenceEvents
```

For v0.x:

```text
count = 28
```

No registered authoritative Journal event may lack a row.

No cross-reference row may invent an unregistered authoritative Journal event.

---

# 73. Object-Kind Coverage Invariant

Every authoritative event has exactly one lifecycle object kind:

```text
∀ e ∈ RegisteredEvents:

  exists exactly one k:
    ObjectKindOf(e) = k
```

No event may dynamically choose its lifecycle object kind from payload prose.

---

# 74. Event Shape Coverage Invariant

Every authoritative event has exactly one EventShape:

```text
∀ e ∈ RegisteredEvents:

  exists exactly one shape:
    EventShapeOf(e) = shape
```

---

# 75. Phase Coverage Invariant

Every authoritative event belongs to exactly one operation whose phase is:

```text
BOOTSTRAP
ONE_PHASE
TWO_PHASE
```

The phase MUST agree with Lifecycle v0.10.2 §20.

---

# 76. Terminality Coverage Invariant

Required:

```text
∀ kind:

  either
    HasTerminality(kind)

  or
    terminality is explicitly N/A
```

For v0.x:

```text
REGISTRY
```

is the only object kind for which:

```text
HasTerminality(kind) = false
```

Every master-table row MUST therefore have a terminal column value reproducible by:

```text
HasTerminality

and, where applicable,

IsTerminal
```

---

# 77. Authority Dependency Completeness Rule

If an event semantically claims that prior Registry authority was required to establish its disposition, the exact prior Journal Reference MUST appear in:

```text
authority_dependencies[]
```

The implementation MUST NOT rely only on:

```text
a Record ID

a filename

a transitive guess

a nearby Journal entry
```

where this table requires direct authority binding.

---

# 78. Identity Dependency Completeness Rule

If an event's semantics materially depend on an immutable identity but not prior Registry authority, that identity MUST be captured through:

```text
identity_dependencies[]
```

or an equivalently frozen identity-bearing field whose encoding is normatively included in the event Record.

The Identity Format Freeze MUST decide one canonical encoding.

It MUST NOT leave two semantically equivalent ways to bind the same required identity unless the distinction is intentional.

---

# 79. Journal Index Field Principle

An event-specific field belongs directly in the Journal Entry rather than only inside the Record payload when Journal-only replay requires that fact.

This principle currently requires direct Journal indexing for at least:

```text
freeze_attempt_id

Freeze intended-root identity

Freeze START references on terminal events

eviction_attempt_id

Eviction START references on terminal events

brackets_closeout_authority_ref for POST Verification

capability class/environment IDs on every entry
```

Chronology references require stable schema placement but are not necessarily all required for the minimal open-set replay algorithm.

---

# 80. No Journal Semantic Overloading

The fact that a value is indexed in the Journal does not make all of its semantics Journal-replayable.

Example:

```text
policy_ref
```

may be authority-indexed.

But:

```text
Policy satisfaction
```

still requires the Policy Record payload.

Likewise:

```text
Formal Verification authority ref
```

does not make theorem semantics replayable without its Record.

And:

```text
brackets_closeout_authority_ref
```

makes POST intent and exact historical Closeout reference Journal-visible, but does not make full POST eligibility Journal-replayable.

---

# 81. Identity Format Freeze Inputs Produced by This Table

The next Identity Format Freeze MUST assign deterministic representations for:

```text
lifecycle_object_kind

lifecycle_object_id

eviction_attempt_id

intended_root_id

eviction_scope_identity

event-specific Journal index fields

authority_dependencies[]

identity_dependencies[]

every Journal Reference field

observation_start_journal_ref

operation_start_journal_ref

attempt_start_journal_ref

eviction_start_journal_ref

conflicting_terminal_journal_ref

brackets_closeout_authority_ref

kind-local state values
  where serialized

terminal column representation
  if externally serialized
```

`EventShape` need not be serialized into authoritative Journal bytes unless the Identity Format specification intentionally chooses to do so.

It may remain derived from `event_type_id`.

---

# 82. Identity Freeze Priority

Before any expected Journal bytes or Journal hashes are frozen, the Identity Format Freeze MUST first decide:

```text
1. semantic dependency collection encoding

2. canonical ordering of authority_dependencies[]

3. canonical ordering of identity_dependencies[]

4. duplicate dependency rejection

5. Journal Reference deterministic encoding

6. chronology-reference field placement

7. brackets_closeout_authority_ref placement and encoding

8. event-specific Journal field encoding
```

A logically identical dependency set MUST NOT admit multiple authoritative Journal byte encodings.

---

# 83. Parallel Identity Vector Case Design

Identity-vector case DESIGN MAY proceed in parallel with review of this table.

Expected bytes and expected hashes MUST NOT be frozen until Identity Format Freeze completes.

Initial case families SHOULD include:

```text
one valid Journal Entry for every event_type_id

one valid state path for every multi-event lifecycle object

one invalid predecessor state per event

one wrong lifecycle_object_kind per event family

one wrong lifecycle_object_id continuation

missing required START authority dependency

forward authority dependency

duplicate authority dependency

duplicate identity dependency

wrong-registry authority dependency

wrong-registry chronology reference

future chronology reference

one-shot object reuse

state-observing event that incorrectly changes state

Eviction terminal event with wrong eviction_attempt_id

Freeze terminal event with wrong freeze_attempt_id

Verification POST with missing brackets_closeout_authority_ref

Verification ordinary with unexpected
brackets_closeout_authority_ref

Verification bracket field points to non-Closeout event

Verification bracket field differs from authority dependency

Review Result bound to wrong Request

Admission accepted with rejected/mismatched Review support

Admission rejected with known nonauthoritative candidate identity

Closeout committed with missing admitted Review authority

Closeout rejected with known failed candidate identity

Formal Verification bound to wrong Assumption Definition

Compatibility bound to wrong source/target Definition

Formal Finding Classification bound to wrong Formal Verification
```

These are case definitions only.

They do not yet contain normative expected CBOR bytes or hashes.

---

# 84. Preliminary Journal Vector Families

At minimum:

```text
JRN-GENESIS-VALID-001

JRN-FREEZE-HAPPY-001

JRN-FREEZE-RECOVERY-001

JRN-FREEZE-OPERATOR-ABORT-001

JRN-FREEZE-REJECTED-LATE-COMMIT-001

JRN-FREEZE-MULTIPLE-REJECTIONS-001

JRN-VERIFICATION-ORDINARY-001

JRN-VERIFICATION-POST-BRACKET-001

JRN-REVIEW-HAPPY-001

JRN-REVIEW-ADMISSION-REJECTED-001

JRN-CLOSEOUT-COMMITTED-001

JRN-CLOSEOUT-REJECTED-001

JRN-CAPABILITY-OBSERVE-001

JRN-EVICTION-COMMITTED-001

JRN-EVICTION-BLOCKED-001

JRN-EVICTION-INTERRUPTED-001

JRN-ASSUMPTION-ESTABLISHMENT-001

JRN-ASSUMPTION-INVALIDATION-EXPLICIT-001

JRN-ASSUMPTION-INVALIDATION-PREDICATE-001

JRN-FORMAL-VERIFICATION-001

JRN-COMPATIBILITY-001

JRN-COMPATIBILITY-INVALIDATION-001

JRN-BOOTSTRAP-SUCCESSION-001

JRN-BOOTSTRAP-INVALIDATION-001

JRN-FORMAL-CLASSIFICATION-001
```

---

# 85. Preliminary Negative Transition Vectors

At minimum:

```text
JRN-NEG-FREEZE-COMMIT-WITHOUT-START

JRN-NEG-FREEZE-COMMIT-AFTER-ABORT

JRN-NEG-FREEZE-SECOND-TERMINAL

JRN-NEG-FREEZE-REJECT-ON-OPEN

JRN-NEG-EVICTION-COMMIT-WITHOUT-START

JRN-NEG-EVICTION-SECOND-TERMINAL

JRN-NEG-ONE-SHOT-REUSE

JRN-NEG-WRONG-OBJECT-KIND

JRN-NEG-WRONG-OBJECT-ID

JRN-NEG-FORWARD-AUTHORITY-DEPENDENCY

JRN-NEG-CROSS-REGISTRY-AUTHORITY-DEPENDENCY

JRN-NEG-CROSS-REGISTRY-CHRONOLOGY-REF

JRN-NEG-FUTURE-CHRONOLOGY-REF

JRN-NEG-DUPLICATE-AUTHORITY-DEPENDENCY

JRN-NEG-DUPLICATE-IDENTITY-DEPENDENCY

JRN-NEG-POST-BRACKET-MISSING-REF

JRN-NEG-POST-BRACKET-WRONG-EVENT-TYPE

JRN-NEG-POST-BRACKET-REF-NOT-IN-AUTHORITY-DEPS

JRN-NEG-ORDINARY-VERIFICATION-UNEXPECTED-BRACKET-REF

JRN-NEG-REVIEW-RESULT-WRONG-REQUEST

JRN-NEG-CLASSIFICATION-WRONG-VERIFICATION
```

---

# 86. Preliminary State-Observation Vectors

At minimum:

```text
JRN-OBS-FREEZE-COMMIT-REJECTED-PRESERVES-COMMITTED

JRN-OBS-FREEZE-COMMIT-REJECTED-PRESERVES-ABORTED-RECOVERY

JRN-OBS-FREEZE-COMMIT-REJECTED-PRESERVES-OPERATOR-ABORT

JRN-OBS-FREEZE-MULTIPLE-COMMIT-REJECTIONS-PRESERVE-STATE

JRN-OBS-STORAGE-CAPABILITY-CHANGE-PRESERVES-REGISTRY-STATE
```

Each state-observation vector MUST assert:

```text
before_state == after_state
```

For the Registry case:

```text
terminality = N/A
```

must also be asserted.

---

# 87. Preliminary Dependency Vectors

At minimum:

```text
DEP-VALID-DIRECT-PRIOR

DEP-INVALID-FORWARD

DEP-INVALID-WRONG-REGISTRY

DEP-INVALID-WRONG-EVENT-TYPE

DEP-INVALID-WRONG-OBJECT-ID

DEP-INVALID-DUPLICATE-AUTHORITY

DEP-INVALID-DUPLICATE-IDENTITY

DEP-VALID-TRANSITIVE-NOT-DUPLICATED

DEP-CLOSEOUT-DIRECT-SUPPORT-SNAPSHOT

DEP-INVALID-MISSING-CLOSEOUT-SUPPORT

DEP-INVALID-MISSING-EVICTION-START

DEP-INVALID-MISSING-FREEZE-START

DEP-REJECTION-FAILED-CANDIDATE-IDENTITY-WITHOUT-AUTHORITY

DEP-POST-BRACKET-EXACT-CLOSEOUT-REF

DEP-POST-BRACKET-REF-AND-AUTHORITY-DEP-MUST-MATCH
```

---

# 88. Preliminary Chronology Reference Vectors

At minimum:

```text
CHR-VALID-OBSERVATION-START

CHR-VALID-OPERATION-START

CHR-VALID-POST-BRACKET-CLOSEOUT-REF

CHR-INVALID-WRONG-REGISTRY

CHR-INVALID-FUTURE-INDEX

CHR-INVALID-HASH

CHR-INVALID-MISSING-ENTRY

CHR-INVALID-WRONG-EXPECTED-EVENT-TYPE

CHR-INVALID-WRONG-LIFECYCLE-OBJECT
```

These cases are required because chronology references receive the same structural Journal Reference validation as authority dependencies even though they do not necessarily establish authority.

---

## 88.1 Deferred Future Interval Semantics

The following is intentionally NOT decided by this table:

> Whether long-running ONE_PHASE operations such as Formal Verification should in a future specification gain Policy-constrained execution-interval semantics comparable to recorded Verification.

Current v0.x semantics are:

```text
Formal Verification:
  operation_start_journal_ref
  = chronology provenance only

Assumption Definition:
  operation_start_journal_ref
  = chronology provenance only
```

A future specification MAY add:

```text
formal execution interval constraints

intervening-event constraints for Formal Verification

trusted-time constraints

dedicated observation_start_journal_ref semantics
```

but such semantics require an explicit successor or Policy extension.

They MUST NOT be inferred from current fields.

---

# 89. Questions Intentionally Deferred to Identity Format Freeze

The following are no longer semantic questions, but their byte representation remains open:

```text
numeric lifecycle_object_kind IDs

numeric or textual state encodings

field-key representation

Journal event-specific field ordering

canonical dependency-list ordering rule

Journal Reference byte encoding

chronology-reference placement and encoding

brackets_closeout_authority_ref exact field placement
and deterministic encoding

eviction_attempt_id textual prefix

intended_root_id exact path representation

eviction_scope_identity exact Record/domain framing
```

These MUST be settled before expected-byte vectors are frozen.

---

# 90. Questions NOT Deferred

The following semantics are fixed by this document:

```text
GENESIS belongs to REGISTRY

REGISTRY has no terminality semantics

Review Request kind = REVIEW_REQUEST

Review Result kind = REVIEW_RESULT

Policy kind = POLICY

Artifact Eviction kind = ARTIFACT_EVICTION_ATTEMPT

eviction uses stable eviction_attempt_id

lifecycle_object_id means logical lifecycle object identity

record-backed one-shot lifecycle_object_id = event_record_id

FREEZE_COMMIT_REJECTED does not transition Freeze state

multiple FREEZE_COMMIT_REJECTED observations are legal

STORAGE_CAPABILITY_CHANGED does not transition Registry state

one-shot RECORDED objects are terminal

Closeout bracket state is derived and separate

Eviction START/terminal relation is Journal-indexed

failed known candidate identities are identity-bound

Assumption Establishment uses observation_start_journal_ref

every Journal Reference is structurally validated

dependency collections reject duplicates

POST Verification is identified Journal-side by
brackets_closeout_authority_ref

brackets_closeout_authority_ref must match the exact
Closeout authority dependency

ordinary Verification must not carry that field

operation_start_journal_ref does not silently create
Policy freshness semantics

required direct authority dependencies are explicit
```

---

# 91. Need for Lifecycle v0.10.3

This document does not require a Lifecycle successor because:

```text
Lifecycle §20.1 explicitly requires this normative table

Lifecycle already contains lifecycle_object_kind

Lifecycle already contains lifecycle_object_id

Lifecycle already contains authority_dependencies[]

Lifecycle already contains identity_dependencies[]

Lifecycle already requires Journal-only open Eviction replay

Lifecycle already defines brackets_closeout_authority_ref
for POST Verification

Lifecycle §20 already defines operation-head bindings
for Review Result, Admission, and Policy
```

This document supplies missing enumerations and per-event specialization.

The chronology fields specified in §48 supplement incomplete minimum field listings but do not alter the existing Lifecycle semantics.

The Journal-side placement of `brackets_closeout_authority_ref` specializes an already-defined Lifecycle semantic relation so that §20.1 / §22 replay can identify POST intent without guessing.

Therefore:

```text
Lifecycle v0.10.3 is not required
```

for the issues identified through v0.3 review.

A future review finding a genuine contradiction would still require an explicit Lifecycle successor.

---

# 92. Initial Dafny Extraction

From this table, the first Dafny layer can be generated nearly mechanically:

```text
EventType

LifecycleObjectKind

EventShape

RegistryState

FreezeAttemptState

OneShotRecordedState

ReviewAdmissionState

CloseoutAttemptState

ArtifactEvictionState
```

followed by:

```text
ObjectKindOf(event)

EventShapeOf(event)

HasTerminality(kind)

IsTerminal(kind, state)

LegalPredecessor(kind, event, state)

JournalReferencesValid(event)

LegalTransition(kind, before, event, after)
```

---

# 93. Initial PURE_FORMAL Properties Exposed by the Table

In addition to properties already named by the Formal Verification specification, this table exposes:

```text
StateObservingEventsDoNotTransition

CreateOpenRequiresAbsence

TerminalTransitionRequiresOpen

OneShotCreationIsTerminal

OneShotTerminalObjectsDoNotTransition

RegisteredEventCoverageIsComplete

EachEventHasExactlyOneObjectKind

EachEventHasExactlyOneShape

TerminalityIsDefinedExactlyForApplicableKinds

TerminalColumnMatchesKindTerminality

AuthorityDependenciesAreStrictlyPrior

ChronologyReferencesAreStrictlyPrior

FreezeTerminalReferencesExactStart

EvictionTerminalReferencesExactStart

RepeatedFreezeCommitRejectionPreservesState

PostBracketReferenceIdentifiesExactCloseout

OrdinaryVerificationHasNoBracketReference
```

These are candidate PURE_FORMAL properties.

---

# 94. Cross-Reference Table Freeze Criterion

This document is ready to freeze when hostile review establishes:

```text
all 28 Lifecycle event IDs represented exactly once

all 18 lifecycle object kinds consistently defined

every event has exactly one object kind

every event has exactly one shape

every event has one legal predecessor/result rule

HasTerminality is defined for every object kind

IsTerminal reproduces every applicable table terminal value

REGISTRY terminal column is consistently N/A

Freeze Journal-only replay is sufficient

Eviction Journal-only replay is sufficient

POST-bracket intent is Journal-identifiable without
overclaiming full POST eligibility

all Journal Reference fields have one common validation rule

direct authority dependencies are complete

failed candidate identities remain traceable without
fabricated authority

no rejection event fabricates missing authority

Closeout support dependencies preserve exact historical support

no Lifecycle state is silently used for derived bracket/support state

lifecycle_object_id semantics are unambiguous

chronology-field supplementation is represented in the
schema-freeze input

dependency collections have duplicate-free semantics

operation_start_journal_ref does not silently acquire
Policy semantics not defined by Lifecycle
```

---

# 95. Freeze Disposition

At this revision:

```text
Known BLOCKING findings:
    NONE

Known MATERIAL findings:
    NONE outstanding
```

The remaining open work belongs to:

```text
Identity Format Freeze

Record Schema Freeze

expected-byte / hash test-vector population

Dafny implementation

Rust implementation
```

and not to the semantic event/transition table itself.

---

# 96. Final Principle

> A Journal event is not merely a named occurrence. It belongs to one exact lifecycle object, has one exact legal state relation, and commits to every prior authority and identity required to make that occurrence meaningful.

And:

> Some authoritative events change state. Some create state already terminal. Some record facts while state remains unchanged. The Registry itself is a continuing authority domain for which terminality is not a meaningful property. Chronology fields preserve what the Registry knew when an operation began, but they do not silently acquire stronger Policy meaning merely because they were recorded.
