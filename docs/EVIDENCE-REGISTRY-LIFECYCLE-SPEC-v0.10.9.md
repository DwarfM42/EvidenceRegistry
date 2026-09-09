# EvidenceRegistry Evidence Lifecycle Specification v0.10.9
## Explicit Successor Selection for Terminal Review Admission Replay

**Status:** DRAFT SUCCESSOR CANDIDATE — NOT FROZEN, REVIEWED, ACCEPTED, OR IMPLEMENTATION AUTHORITY

**Frozen semantic predecessor:** EvidenceRegistry Evidence Lifecycle Specification v0.10.2.

**Adopted-contract inputs (not frozen-semantic-predecessor claims):**
- Lifecycle v0.10.4, including Review Admission §§82, 46, and 83;
- the detached Authority-Path Successor Package Adoption Record v0.1 with raw-byte SHA-256 `65813e356eaba89c638475a40c69f728123f4314040f075a09c6be34db14e114`.

**Required structural companion:** proposed Record Schema v0.7.
**Required mechanical companion:** proposed Cross-Reference v0.8.

## 1. Purpose, boundary, and new semantic choice

The adopted Review Admission successor is prospective. It supplies an exact
profile-1, version-1, empty-Scope Rule and evaluator `1015`, but does not select
which retained terminal Records were produced under that successor. Applying it
to every event `302` or `303` would retroactively reinterpret discriminator-less
predecessor history. Inferring selection from a Policy context, Scope, Request,
Result, current configuration, timing, event identity, or generic field would
be an unselected semantic expansion.

This candidate makes one narrowly bounded new semantic choice: only a terminal
`REVIEW_ADMISSION` Record carrying the exact retained selector defined by Record
Schema v0.7 is successor-selected. The selector binds to the exact already
adopted successor package by the adoption-record raw-byte SHA-256 stated above.

No other Record, Journal entry, external package, or runtime fact can select
this route.

## 2. Selected terminal producer rule

For a new terminal Review Admission operation, the Registry MUST preserve all
predecessor and adopted prerequisites. In particular it MUST:

```text
1. obtain Request, Result, Policy, Scope, authority, and lifecycle inputs only
   through the authoritative retained Store;
2. complete every §82 prerequisite before Policy evaluation;
3. for this selected route only, establish the §82 common Scope exactly:
   - authoritative Request.review_scope_ref
       == authoritative Result.review_scope_ref;
   - every participating Scope has profile id 1, version 1, empty payload; and
   - Policy.supported_context_ids == { REVIEW_ADMISSION };
4. execute evaluator 1015 only after those prerequisites. It alone compares
   POLICY.gate_scope_ref with the established common Request/Result Scope;
5. complete §46 and then apply the existing §83 mapping.
```

The selected producer MUST derive the Record Schema v0.7 key-`24` literal
internally. It MUST NOT receive, copy, infer, or default it from a caller,
reviewer, Policy, Request, Result, Scope, generic context membership, source
revision, wall clock, current configuration, or Journal position.

Only after the above has completed may it construct a terminal Record:

```text
§46 == SATISFIED
    -> existing §83 accepted disposition -> event 302

§46 == GATE_UNSATISFIED or GATE_INDETERMINATE
    -> existing §83 rejected disposition -> event 303
```

An uncompleted prerequisite remains pre-terminal. It MUST NOT produce either a
selected `302` or selected `303`. The selector alone never makes a result
terminal, accepted, rejected, published, or authoritative.

The existing Store-owned Record-then-Journal publication and exact retained
readback requirements remain controlling. A selected producer cannot write the
Record or Journal unless all existing operation and publication legality
conditions succeed.

## 3. Replay and readback classification

Replay of a retained terminal event `302` or `303` MUST occur in this order:

```text
1. Strictly replay the Journal and resolve the exact event_record_id through the
   authoritative RecordId-addressed Record namespace.
2. Strictly decode the Record, recompute and compare its RecordId, validate the
   existing type-32 local grammar, event/disposition relation, direct authority
   dependencies, retained references, and predecessor chronology constraints.
3. Read Record key 24 from those exact Record bytes.
4. If key 24 is absent, classify the Record as PREDECESSOR_NOT_SUCCESSOR_SELECTED
   and stop successor-specific replay. Preserve its predecessor treatment; do
   not run §82/§46/§83 or evaluator 1015 under this successor.
5. If key 24 is malformed or differs from the exact Record Schema v0.7 literal,
   fail closed as an invalid successor selection. Do not fall back to predecessor
   treatment.
6. Only for the exact literal, validate every selected-route §82 prerequisite,
   including the common Request/Result Scope, then the exact profile-1 Policy
   context and each participating Scope's local condition. Execute evaluator
   1015 as the exclusive comparison of POLICY.gate_scope_ref with the
   established common Scope, then obtain the complete §46 result.
7. Require the completed result/event relation: SATISFIED only with event 302;
   GATE_UNSATISFIED or GATE_INDETERMINATE only with event 303. Reject a mismatch.
```

The selection field is neither an authority dependency nor a replacement for
any predecessor Record/Journal binding. It merely chooses whether successor
semantic replay is applicable to this exact retained terminal Record.

## 4. Historical preservation

A discriminator-less predecessor event `302` or `303` keeps its historical
Record bytes, RecordId, Journal Entry, event meaning, and predecessor replay
treatment. It MUST NOT become invalid, accepted, rejected, selected, or
evaluator-1015-governed because a later implementation supports this successor.

The absence of key `24` is sufficient to exclude successor semantics. No caller,
replay helper, cache, current Store setting, date, Journal order, Policy context,
Scope, reason code, Request/Result reference, or content similarity may supply
the missing selection.

This preservation does not excuse predecessor structural defects: the existing
predecessor parser and Journal checks remain controlling. It only forbids
prospective successor semantic reclassification.

## 5. Required lifecycle vectors

```text
RA-SELECTOR-PRODUCER-STORE-DERIVED-ONLY-001
RA-SELECTOR-PRODUCER-PRETEMINAL-NO-RECORD-NO-JOURNAL-001
RA-SELECTOR-REPLAY-302-EXACT-SELECTED-SATISFIED-001
RA-SELECTOR-REPLAY-303-EXACT-SELECTED-UNSATISFIED-001
RA-SELECTOR-REPLAY-SELECTED-EVENT-DISPOSITION-MISMATCH-REJECT-001
RA-SELECTOR-REPLAY-HISTORICAL-302-NO-1015-001
RA-SELECTOR-REPLAY-HISTORICAL-303-NO-1015-001
RA-SELECTOR-REPLAY-MALFORMED-OR-WRONG-SELECTOR-NO-FALLBACK-001
RA-SELECTOR-REPLAY-SAME-FACTS-WITHOUT-SELECTOR-NO-SUCCESSOR-001
```

The historical vectors must retain predecessor terminal bytes and demonstrate
that successor replay neither resolves a new profile nor evaluates `1015`.

## 6. Non-claims

This candidate does not change generic Scope semantics, any generic Policy
applicability, Request-creation authority, Policy context rules outside the
adopted exact Review Admission profile, event IDs, disposition grammar, Journal
encoding, terminal dependencies, Identity Format, Store Profile, runtime
implementation, platform qualification, release, or adoption. It does not
backfill historical selectors or authorise a compatibility migration.
