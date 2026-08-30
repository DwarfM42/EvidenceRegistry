# EvidenceRegistry Evidence Lifecycle Specification v0.10.5
## Review Admission Operation-Start Journal Binding Closure

**Status:** FREEZE CANDIDATE — NOT YET FROZEN
**Frozen semantic predecessor:** EvidenceRegistry Evidence Lifecycle Specification v0.10.4
**Project:** EvidenceRegistry
**Repository:** `DwarfM42/EvidenceRegistry`
**Implementation candidate:** Rust
**Formal-model candidate:** Dafny
**Primary deployment model:** local-first, cross-platform, filesystem-authoritative CLI/tooling
**Normative language:** MUST / MUST NOT / SHOULD / SHOULD NOT / MAY

---

# 0. Specification Position

This specification is a narrow prospective Lifecycle successor of frozen Lifecycle
v0.10.4. Except where this successor explicitly states otherwise, every
normative rule of Lifecycle v0.10.4 remains unchanged, including its inherited
v0.10.2 rules.

This successor closes only one semantic edge:

```text
At what exact point is REVIEW_ADMISSION.operation_start_journal_ref
captured and frozen for one Review Admission operation?
```

The field already exists as required `REVIEW_ADMISSION` Record key `23` in
Record Schema v0.3 §67. This successor does not change that field, its type,
its CBOR encoding, Record identity, JournalReference identity, evaluator IDs,
Policy requirements, §82 prerequisites, §46 evaluation semantics, §83
outcome mapping, event 302 meaning, event 303 meaning, Journal publication
format, or Cross-Reference event mappings.

The predecessor rules already require Review Admission to be ONE_PHASE with
an `operation head` start binding, and Cross-Reference v0.3 §48 already gives
`operation_start_journal_ref` ordinary-operation-start chronology provenance.
They do not, however, identify a machine-distinguishable capture boundary
relative to Review Admission input processing. This successor supplies that
Lifecycle-owned boundary prospectively and only for `REVIEW_ADMISSION`.

---

# 1. Applicability

This successor applies only to one Review Admission operation governed by
Lifecycle §82 and §83 and materialized, if it reaches the existing terminal
threshold, as either event 302 or event 303.

It does not:

```text
change the meaning of historical Record or Journal bytes

reinterpret a historical operation_start_journal_ref

generalize this rule to Review Request, Review Result, Policy registration,
Closeout, Verification, or another operation

create a START event, a new lifecycle state, or a second Journal append

change the existing pre-terminal no-publication rule for §82 prerequisite failure
```

---

# 2. Exact Capture Boundary

For this specification, the existing Review Admission operation begins at the
existing authoritative command / request acceptance for processing. The
implementation MUST capture the authoritative Journal head exactly once:

```text
immediately after authoritative Review Admission command / request acceptance

and

before decoding REVIEW_REQUEST, REVIEW_RESULT, or POLICY bytes for that
Review Admission operation.
```

The captured JournalReference is:

```text
Hstart
=
REVIEW_ADMISSION.operation_start_journal_ref
```

`Hstart` MUST identify the authoritative Registry Journal head observed at that
boundary. A mutable `HEAD` optimization is not authoritative and MUST NOT be
used as a substitute for the authoritative Journal head.

The capture order is normative:

```text
authoritative Review Admission command / request acceptance
↓
capture authoritative Journal head Hstart exactly once
↓
decode REVIEW_REQUEST
↓
decode REVIEW_RESULT
↓
perform every existing §82 Review Admission prerequisite
↓
decode the identified POLICY as required by the existing prerequisites and
applicable Policy requirements
↓
perform §46 EvaluatePolicy only after the existing §82 prerequisite boundary
has succeeded
↓
apply the existing §83 completion / pre-threshold boundary
↓
if the existing Lifecycle rules determine a terminal disposition,
construct the Admission Record using Hstart
↓
serialize the existing terminal Journal publication against the then-current
Journal state
```

This ordering does not alter which existing inputs §82, §46, or §83 require.
It specifies only the capture boundary for this one required chronology field.

The following are forbidden for the same operation:

```text
capturing operation_start_journal_ref before authoritative command / request
acceptance

capturing it after REVIEW_REQUEST decode

capturing it after REVIEW_RESULT decode

capturing it after any §82 prerequisite evaluation

capturing it after POLICY decode or §46 evaluation

capturing it after §83 disposition determination

capturing it immediately before terminal Journal publication instead of at the
specified operation-start boundary
```

---

# 3. Capture-Once and No-Refresh Rule

An implementation MUST retain the first and only `Hstart` captured under §2
for the lifetime of that Review Admission operation.

It MUST NOT:

```text
re-read the Journal head to replace Hstart after capture

replace Hstart after §82 evaluation

replace Hstart after §46 evaluation

replace Hstart after §83 disposition determination

refresh Hstart immediately before terminal Journal publication

select a different historical JournalReference through a publication helper
```

The caller MUST NOT provide, prevalidate, select, or mark trusted an
`operation_start_journal_ref` that becomes authoritative for this purpose.
Production authority for `Hstart` comes only from the authoritative Journal
head captured at §2.

A test fixture MAY describe historical Journal states, but it MUST NOT create a
production API that accepts fixture-supplied start-reference authority.

---

# 4. Concurrent Append and Terminal-Publication Distinction

After `Hstart` is captured, another operation MAY append one or more
authoritative Journal Entries before Review Admission terminal publication.
Those intervening entries do not change `Hstart`.

```text
Hstart
<
Hintervening[0..n]
<
Hterminal
```

is valid when the existing Journal serialization and transition rules permit
`Hterminal`.

`operation_start_journal_ref` is the operation-start identity. It is not the
terminal publication predecessor identity. Existing Journal serialization and
current-head validation determine any predecessor relationship needed to append
the terminal Journal Entry; this successor neither introduces a Record field
for that relationship nor requires it to equal `Hstart`.

Therefore an implementation MUST NOT reinterpret the Identity Format
strictly-prior requirement as an immediate-predecessor requirement. The
captured `Hstart` is valid when it is historical and strictly prior to the
exact authority Journal Entry that later makes the Admission Record
authoritative, even if one or more authoritative Journal Entries intervene.

---

# 5. Pre-Completion and Terminal Boundaries

This successor does not change the existing outcome semantics.

If an existing §82 prerequisite fails, including a Review Scope prerequisite
failure, this successor does not authorize a terminal Admission Record, event
302, or event 303. It preserves the existing pre-terminal no-publication
boundary.

If existing Lifecycle rules reach their terminal threshold and require an
Admission Record and terminal event, the Record MUST use the same captured
`Hstart` regardless of whether the existing terminal disposition is accepted
or rejected.

If the operation crashes or otherwise stops before an existing terminal Journal
append, this successor creates no new authoritative disposition and no new
publication. The existing ONE_PHASE rule remains controlling.

This successor does not create a retry taxonomy. Within one continuing Review
Admission operation, a publication retry or any other continuation MUST retain
its captured `Hstart`. A later authoritative command / request acceptance is a
new Review Admission operation and captures a new start head under §2.

---

# 6. Conformance Examples

The examples in this section are normative conformance vectors for the
operation-start binding relation. `Hterminal` denotes the Journal Entry that,
only when existing Lifecycle conditions are satisfied, makes the Admission
Record authoritative.

## OS-1 — No concurrent append

```text
start head = H0
Review Admission command / request accepted
capture H0
no other append
terminal publication = Hterminal
operation_start_journal_ref = H0
```

## OS-2 — One intervening append

```text
start head = H0
Review Admission command / request accepted
capture H0
another operation appends H1
Review Admission terminal publication occurs after H1
operation_start_journal_ref = H0
H0 < H1 < Hterminal
```

## OS-3 — Multiple intervening appends

```text
start head = H0
Review Admission command / request accepted
capture H0
other operations append H1, H2, H3
Review Admission terminal publication occurs after H3
operation_start_journal_ref = H0
H0 < H1 < H2 < H3 < Hterminal
```

## OS-4 — Refresh forbidden

```text
captured H0
latest current head becomes H2
attempt to replace H0 with H2 before terminal publication
→ non-conforming
```

## OS-5 — Caller injection forbidden

```text
caller supplies an arbitrary historical JournalReference Hx
Hx was not captured under §2 for this operation
→ Hx is non-authoritative as operation_start_journal_ref
```

## OS-6 — Strictly prior

```text
captured Hstart
terminal authority event = Hterminal
Hstart.entry_index < Hterminal.entry_index
→ chronology reference may satisfy the existing strictly-prior relation
```

A Record-local decoder alone still cannot claim that this relation holds. The
existing Identity Format authority-context validation remains required.

## OS-7 — Immediate predecessor is not required

```text
captured H0
concurrent append H1
terminal Review Admission event H2
H0 < H1 < H2
operation_start_journal_ref = H0
```

`H0` need not equal the immediate predecessor of `H2`. The existing Journal
append mechanism remains responsible for validating the terminal publication
against current Journal state.

---

# 7. Relationship to Existing Authorities

This successor specializes only Review Admission Lifecycle timing semantics.
It leaves the existing authority ownership split intact:

```text
Lifecycle
    owns the operation-specific capture boundary and its prospective
    Review Admission consequence

Cross-Reference
    continues to mechanically represent Lifecycle event semantics and gains
    no independent authority to create this Lifecycle rule

Record Schema
    continues to own the existing required field and its structure; no Record
    Schema successor is required because field shape and presence do not change

Identity Format
    continues to own JournalReference encoding and historical / strictly-prior
    authority-context validation; no Identity Format successor is required
```

---

# 8. Non-Claims

A conforming implementation of this capture rule alone does not establish:

```text
§82 prerequisite success

Policy applicability, satisfaction, or authority

legal event 302 or event 303 issuance

Admission authority

terminal publication success

runtime, system, or production qualification
```

The rule closes only the exact semantic choice for
`REVIEW_ADMISSION.operation_start_journal_ref`.
