# EvidenceRegistry Evidence Lifecycle Specification v0.10.6
## Review Admission Atomic Operation-Start Journal Binding Closure

**Status:** FREEZE CANDIDATE — NOT YET FROZEN
**Frozen semantic predecessor:** EvidenceRegistry Evidence Lifecycle Specification v0.10.2
**Adopted contract input (not a frozen-semantic-predecessor claim):** EvidenceRegistry Evidence Lifecycle Specification v0.10.4
**Remediates rejected candidate:** EvidenceRegistry Evidence Lifecycle Specification v0.10.5 (`89a9b9475e63c3eaef0e9990a9c276c6396c991c`)
**Project:** EvidenceRegistry
**Repository:** `DwarfM42/EvidenceRegistry`
**Implementation candidate:** Rust
**Formal-model candidate:** Dafny
**Primary deployment model:** local-first, cross-platform, filesystem-authoritative CLI/tooling
**Normative language:** MUST / MUST NOT / SHOULD / SHOULD NOT / MAY

---

# 0. Specification Position and Provenance

This is a narrow prospective Lifecycle candidate over the frozen Lifecycle
v0.10.2 semantic baseline. The v0.10.4 bytes are an adopted contract input
whose own retained text remains a freeze candidate and names v0.10.2 as its
frozen semantic predecessor. This candidate does not rewrite that retained
provenance or represent v0.10.4 as its frozen semantic predecessor.

The rejected v0.10.5 candidate is historical, non-authoritative remediation
context only. It remains unmodified. Its hostile review found three independent
binding gaps: undefined acceptance linearization, acceptance-to-head concurrent
append TOCTOU, and ambiguous continuation after a pre-terminal stop. This
candidate resolves only those gaps and the v0.10.5 provenance defect.

Except where this successor expressly states otherwise, the normative rules of
Lifecycle v0.10.2 and the adopted v0.10.4 contract input remain unchanged.
This successor changes neither their historical meaning nor any historical
Record or Journal bytes.

The sole new prospective rule is the exact atomic selection of the already
required:

```text
REVIEW_ADMISSION.operation_start_journal_ref
```

Record Schema v0.3 §67 already requires that field at REVIEW_ADMISSION Record
key `23`; Cross-Reference v0.3 §48 already assigns it ordinary-operation-start
chronology provenance; Lifecycle already makes Review Admission ONE_PHASE with
an operation-head start binding. This candidate supplies only the previously
unselected runtime-independent Lifecycle relation between authoritative command
/ request acceptance and the operation-head observation.

This candidate does not change Record Schema field shape, CBOR encoding, Record
identity, JournalReference identity, evaluator IDs, Policy requirements, §82
prerequisites, §46 evaluation semantics, §83 outcome mapping, event 302/event
303 meaning, Journal append format, Journal serialization validity, or
Cross-Reference event mappings.

---

# 1. Applicability and Non-Expansion

This candidate applies only to one Review Admission operation governed by
Lifecycle §82 and §83. If existing rules reach a terminal threshold, the
operation can be materialized only by the existing event 302 or event 303
boundary.

It does not:

```text
change the meaning of historical Record or Journal bytes

reinterpret a historical operation_start_journal_ref

generalize the rule to Review Request, Review Result, Policy registration,
Closeout, Verification, or another operation

create a START event, a new lifecycle state, a logical/effective start concept,
or a second Journal append

change existing pre-terminal no-publication behavior for §82 prerequisite failure

select whether a retry, recovery, or re-presentation is permitted by another
existing rule
```

---

# 2. Exact Acceptance-and-Snapshot Boundary

## 2.1 Acceptance instance

For this candidate only, a **Review Admission acceptance instance** is one
runtime execution instance of one authoritative Review Admission command /
request. It is identified by a fresh opaque local execution token, `A`, which
MUST be unique among simultaneously live Review Admission acceptance instances.
`A` is not a Record field, Journal field, event, lifecycle state, logical start,
effective start, policy input, caller-supplied identity, or authoritative
history object.

An implementation has accepted the command / request for processing exactly
when it performs the single `accept-and-snapshot` action in §2.2 for `A`.
Before that action, the candidate Review Request, Review Result, and Policy
inputs MUST remain undecoded opaque input bytes for this operation. This
candidate does not add a separate acceptance phase: it defines the existing
authoritative command / request acceptance boundary by the one required action.

## 2.2 Atomic accept-and-snapshot action

For each acceptance instance `A`, authoritative command / request acceptance
and selection of `Hstart` MUST occur at one and the same serializable
linearization point, `L(A)`, in the authoritative Registry Journal order.

At `L(A)`, atomically and exactly once, the implementation MUST:

```text
1. accept A for Review Admission processing;
2. observe the authoritative Registry Journal head; and
3. bind that observed head as Hstart for A.
```

The authoritative head observed at `L(A)` is:

```text
Hstart(A)
=
REVIEW_ADMISSION.operation_start_journal_ref for A
```

An implementation MUST ensure that every authoritative Journal append racing
with this action is serialized wholly before or wholly after `L(A)`:

```text
append ordered before L(A)  → it is reflected in Hstart(A)
append ordered after  L(A)  → it is not reflected in Hstart(A)
```

It is non-conforming to accept `A` at one point and later separately read a
head for `A`, or to read a head before accepting `A`. “Immediately after” does
not permit a non-atomic acceptance-to-head interval; the single linearization
point is the complete required meaning of the boundary.

A mutable cache, advisory `HEAD` optimization, caller assertion, fixture input,
or publication helper selection is not the authoritative observation required
at `L(A)` and MUST NOT substitute for it.

## 2.3 Required ordering after L(A)

After `L(A)`, the implementation MUST preserve this order for `A`:

```text
accept-and-snapshot at L(A), binding Hstart(A)
↓
decode REVIEW_REQUEST
↓
decode REVIEW_RESULT
↓
perform every existing §82 Review Admission prerequisite
↓
decode the identified POLICY only as required by the existing prerequisites and
applicable Policy requirements
↓
perform §46 EvaluatePolicy only after the existing §82 prerequisite boundary
has succeeded
↓
apply the existing §83 completion / pre-threshold boundary
↓
if existing Lifecycle rules determine a terminal disposition, construct the
Admission Record using Hstart(A)
↓
serialize the existing terminal Journal publication against then-current
Journal state
```

This ordering does not alter which inputs §82, §46, or §83 require. It defines
only the timing and provenance of the one existing required chronology field.

The following are non-conforming for `A`:

```text
accepting A without atomically binding an authoritative Hstart(A)

reading the head before acceptance or separately after acceptance

decoding REVIEW_REQUEST, REVIEW_RESULT, or POLICY bytes before L(A)

capturing or replacing Hstart(A) after any §82 prerequisite, §46 evaluation,
§83 disposition determination, or immediately before terminal publication
```

---

# 3. Capture-Once, Retention, and No-Refresh

`Hstart(A)` is the first and only operation-start reference for acceptance
instance `A`. The implementation MUST retain the binding of `A` to `Hstart(A)`
for as long as `A` remains live.

It MUST NOT:

```text
perform a second accept-and-snapshot action for A

re-read the Journal head to replace Hstart(A)

replace Hstart(A) after §82 evaluation, §46 evaluation, or §83 disposition

determine Hstart(A) through a publication helper

allow a caller to provide, prevalidate, select, or mark trusted an
operation_start_journal_ref for A
```

A test fixture MAY describe historical Journal states, but it MUST NOT create a
production API that accepts fixture-supplied operation-start-reference
authority.

---

# 4. Concurrent Appends and Terminal-Publication Distinction

After `L(A)`, another operation MAY append one or more authoritative Journal
Entries before Review Admission terminal publication. Those later entries do
not change `Hstart(A)`.

```text
Hstart(A)
<
Hintervening[0..n]
<
Hterminal
```

is valid when existing Journal serialization and transition rules permit
`Hterminal`.

`operation_start_journal_ref` is operation-start provenance. It is not the
terminal-publication predecessor identity. Existing Journal serialization and
current-head validation determine any predecessor relationship needed to append
the terminal Journal Entry; this candidate neither introduces a Record field
for that relationship nor requires it to equal `Hstart(A)`.

The Identity Format strictly-prior requirement remains distinct from an
immediate-predecessor requirement. `Hstart(A)` can satisfy that existing
requirement when it is historical and strictly prior to the exact authority
Journal Entry that later makes the Admission Record authoritative, even if one
or more authoritative Journal Entries intervene.

---

# 5. Pre-Completion, Stop, and Re-Presentation Boundary

This candidate does not change existing outcome semantics. If an existing §82
prerequisite fails, including a Review Scope prerequisite failure, it does not
authorize a terminal Admission Record, event 302, or event 303. It preserves
the existing pre-terminal no-publication boundary.

If existing Lifecycle rules reach a terminal threshold and require an Admission
Record and terminal event, the Record MUST use `Hstart(A)` for that same
acceptance instance regardless of whether the existing disposition is accepted
or rejected.

A pre-terminal stop ends `A` when the process crashes, terminates, returns the
uncompleted operation to its caller, or otherwise loses the live binding of
opaque token `A` and `Hstart(A)` before a terminal Journal append. Such a stop
creates no new authoritative disposition and no new publication.

This candidate defines no persisted acceptance-instance or continuation record.
It follows that a later re-presentation, retry, recovery, restart, or new
invocation after a pre-terminal stop is a new acceptance instance `B`, not a
continuation of `A`, and MUST execute a new `accept-and-snapshot` action at
`L(B)` to bind `Hstart(B)`. It MUST NOT reuse `Hstart(A)`.

This is solely a chronology-binding classification. It does not authorize the
new invocation, decide whether a retry/recovery is allowed, change any
operation's logical/effective semantics, or produce a terminal event without
all existing §82, §46, §83, and Journal requirements.

An ordinary in-process pause that retains the live, unique binding of `A` and
`Hstart(A)` is not a pre-terminal stop; resuming that same `A` MUST retain its
already-bound `Hstart(A)` and MUST NOT recapture.

---

# 6. Conformance Vectors

The vectors in this section are normative for the operation-start binding
relation only. `Hterminal` denotes the Journal Entry that, only if existing
Lifecycle conditions are satisfied, makes the Admission Record authoritative.

## OS-1 — No concurrent append

```text
current head = H0
perform accept-and-snapshot at L(A)
Hstart(A) = H0
no other append
terminal publication = Hterminal
operation_start_journal_ref = H0
```

## OS-2 — One append after acceptance snapshot

```text
current head = H0
perform accept-and-snapshot at L(A)
Hstart(A) = H0
another operation appends H1 after L(A)
terminal publication occurs after H1
operation_start_journal_ref = H0
H0 < H1 < Hterminal
```

## OS-3 — Multiple appends after acceptance snapshot

```text
current head = H0
perform accept-and-snapshot at L(A)
Hstart(A) = H0
other operations append H1, H2, H3 after L(A)
terminal publication occurs after H3
operation_start_journal_ref = H0
H0 < H1 < H2 < H3 < Hterminal
```

## OS-4 — Concurrent append race is serialized at L(A)

```text
case 1: concurrent append H1 is ordered before L(A)
        Hstart(A) = H1

case 2: concurrent append H1 is ordered after L(A)
        Hstart(A) remains the prior head H0
```

For a single authoritative Journal serialization order, exactly one case
applies. No implementation may accept the same `A` at one point and choose
between H0 and H1 later through an acceptance-to-head gap.

## OS-5 — Refresh forbidden

```text
Hstart(A) = H0
latest current head becomes H2
attempt to replace H0 with H2 before terminal publication
→ non-conforming
```

## OS-6 — Caller injection forbidden

```text
caller supplies arbitrary historical JournalReference Hx
Hx was not atomically bound as Hstart(A) at L(A)
→ Hx is non-authoritative as operation_start_journal_ref for A
```

## OS-7 — Strictly prior, not immediate predecessor

```text
Hstart(A) = H0
concurrent append = H1
terminal Review Admission event = H2
H0 < H1 < H2
operation_start_journal_ref = H0
```

`H0` need not equal the immediate predecessor of `H2`. Existing Identity
Format authority-context validation remains required; a Record-local decoder
alone cannot establish that relation.

## OS-8 — Pre-terminal stop creates no continuation binding

```text
perform accept-and-snapshot at L(A) with Hstart(A) = H0
stop before a terminal Journal append
later re-present the command / request
perform accept-and-snapshot for new B at L(B)
→ B has independently selected Hstart(B)
→ B MUST NOT reuse Hstart(A)
```

## OS-9 — Live pause retains the one binding

```text
perform accept-and-snapshot at L(A) with Hstart(A) = H0
pause without losing A or Hstart(A)
resume A
→ operation_start_journal_ref remains H0
→ no second head capture occurs
```

---

# 7. Relationship to Existing Authorities

This candidate specializes only Review Admission Lifecycle timing semantics. It
leaves authority ownership intact:

```text
Lifecycle
    owns this operation-specific acceptance-and-snapshot timing rule and its
    prospective Review Admission consequence

Cross-Reference
    continues to mechanically represent Lifecycle event semantics and gains
    no independent authority to create this rule

Record Schema
    continues to own the existing required field and its structure; no Record
    Schema successor is required because field shape and presence do not change

Identity Format
    continues to own JournalReference encoding and historical / strictly-prior
    authority-context validation; no Identity Format successor is required
```

---

# 8. Non-Claims

Conforming to this candidate alone does not establish:

```text
§82 prerequisite success

Policy applicability, satisfaction, or authority

legal event 302 or event 303 issuance

Admission authority

terminal publication success

runtime, system, implementation, or production qualification
```

The rule closes only the exact prospective semantic choice for
`REVIEW_ADMISSION.operation_start_journal_ref`.
