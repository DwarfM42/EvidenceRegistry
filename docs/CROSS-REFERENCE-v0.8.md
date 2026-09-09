# EvidenceRegistry Normative Event & Transition Cross-Reference v0.8
## Terminal Review Admission Successor-Selection Companion

**Status:** DRAFT SUCCESSOR CANDIDATE — NOT FROZEN, REVIEWED, ACCEPTED, OR IMPLEMENTATION AUTHORITY

**Frozen semantic predecessor:** EvidenceRegistry Normative Event & Transition Cross-Reference v0.3.

**Adopted-contract input (not a frozen-semantic-predecessor claim):** Cross-Reference v0.5.
**Only semantic source for this candidate:** proposed Lifecycle v0.10.9.
**Required Record companion:** proposed Record Schema v0.7.

## 1. Mechanical role and self-limitation

This companion propagates, and does not independently create, Lifecycle v0.10.9's
explicit successor-selection condition for terminal Review Admission events. The
field is a terminal Record carrier; it is not a Journal field or an additional
direct authority dependency.

If this document appears to authorize a condition not present in Lifecycle
v0.10.9, Lifecycle controls and this companion supplies no independent rule.

## 2. Event mapping supplement

For a prospective terminal event `302` or `303`, after every predecessor row
condition and lifecycle prerequisite has been checked, the additional mechanical
condition is:

```text
ReviewAdmissionRecord.key24
  == RecordSchema-v0.7 exact adopted-package SHA-256 literal
```

The condition is required only for Lifecycle v0.10.9 selected-route replay and
new selected-route production. It has these mechanical event consequences:

| Event | Existing required completed relation | Key-24 mapping |
| ---: | --- | --- |
| `302` `REVIEW_ADMISSION_ACCEPTED` | §46 `SATISFIED` and existing accepted disposition | exact selector required for this successor-selected route |
| `303` `REVIEW_ADMISSION_REJECTED` | §46 `GATE_UNSATISFIED` or `GATE_INDETERMINATE` and existing rejected disposition | exact selector required for this successor-selected route |

For a terminal Record without key `24`, the mapping is:

```text
PREDECESSOR_NOT_SUCCESSOR_SELECTED
-> existing predecessor mapping only
-> no Lifecycle-v0.10.9 §82/§46/§83 replay
-> no evaluator 1015 inference
```

A malformed or other key-`24` value is not an alternate predecessor mode. It
mechanically selects no recognized route and fails closed without a fallback.

## 3. Journal and dependency preservation

For either selected event, the existing Journal Entry continues to bind the
terminal Record only through its existing `event_record_id`. Since key `24` is
committed by the Record's canonical bytes and resulting RecordId, this already
commits the selector in Journal replay.

The following do not change:

```text
EventType IDs and names
terminality and state transition
Journal Entry encoding
Journal entry field set
Direct authority dependency set
Identity dependency set
Request / Result / Policy authority-reference rules
```

Do not insert the selector into `authority_dependencies[]`, create a new event,
or treat it as a Policy evaluator output.

## 4. Required companion vectors

```text
XREF-RA-SELECTOR-302-RECORDID-COMMITS-SELECTOR-001
XREF-RA-SELECTOR-303-RECORDID-COMMITS-SELECTOR-001
XREF-RA-SELECTOR-HISTORICAL-302-PREDECESSOR-MAPPING-001
XREF-RA-SELECTOR-HISTORICAL-303-PREDECESSOR-MAPPING-001
XREF-RA-SELECTOR-WRONG-VALUE-NO-PREDECESSOR-FALLBACK-001
XREF-RA-SELECTOR-NO-EXTRA-DEPENDENCY-001
```

## 5. Non-claims

This candidate does not define a generic package registry, a generic Scope
relation, a terminal result, Request-creation authority, new canonical bytes,
RecordId derivation, Journal semantics, historical conversion, implementation,
qualification, publication, or adoption. The full rule exists only if the
companion Record Schema and Lifecycle drafts are reviewed and explicitly adopted
together.
