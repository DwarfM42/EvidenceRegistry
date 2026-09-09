# EvidenceRegistry Record Schema Successor v0.7
## Terminal Review Admission Successor-Selection Carrier

**Status:** DRAFT SUCCESSOR CANDIDATE — NOT FROZEN, REVIEWED, ACCEPTED, OR IMPLEMENTATION AUTHORITY

**Frozen semantic predecessor:** EvidenceRegistry Record Schema v0.3.

**Adopted-contract input (not a frozen-semantic-predecessor claim):** EvidenceRegistry Authority-Path Successor Package Adoption Record v0.1, SHA-256 `65813e356eaba89c638475a40c69f728123f4314040f075a09c6be34db14e114`.

**Required semantic companion:** proposed Lifecycle v0.10.9.
**Required mechanical companion:** proposed Cross-Reference v0.8.

## 1. Purpose and explicit new choice

The adopted Authority-Path package defines the prospective exact Review Scope
profile and evaluator `1015`, but deliberately adds no terminal Record or
Journal carrier that selects that package for a particular event `302` or `303`.
The adoption record is prospective and non-retroactive. A terminal record cannot
therefore acquire successor semantics from its Policy context, Scope bytes,
referenced records, event type, date, Journal position, configuration, code
version, or any other existing field.

This candidate proposes one new semantic choice for future Owner adoption: a
future terminal Review Admission Record can explicitly select exactly the
already adopted Authority-Path successor package by carrying its exact
adoption-record digest.
It does not add generic Scope meaning, a new Policy requirement, an event, a
Journal field, or a new identity derivation.

## 2. Type-32 key and canonical value grammar

For prospective `REVIEW_ADMISSION` Record type `32`, this successor assigns the
next unused body key:

| Key | Field | Type | Presence |
| ---: | --- | --- | --- |
| `24` | `review_admission_successor_package_sha256` | `bstr` of exactly 32 bytes | absent for predecessor records; required for this successor-selected terminal route |

The value has exactly one defined canonical byte value:

```text
SHA-256(raw bytes of
  docs/FREEZE-RECORD-AUTHORITY-PATH-SUCCESSOR-PACKAGE-v0.1.md)
= 65813e356eaba89c638475a40c69f728123f4314040f075a09c6be34db14e114
```

The field is encoded as the canonical CBOR byte string carrying those 32 bytes.
No integer profile alias, text spelling, shortened digest, wrapper map, array,
null, duplicate key, alternate hash algorithm, or alternate value is defined.
The existing canonical Record map ordering remains controlling; a selected record
therefore has the normal type-32 fields plus key `24` in canonical key order.

The field is a retained **selector**, not an asserted Policy conclusion. It
means only:

```text
this terminal Record requests the one prospective Review Admission semantic lane
bound by the named adopted package
```

It does not itself establish `PolicySatisfied`, §82 completion, evaluator output,
§83 disposition, Record authority, Journal authority, publication, or replay
success.

## 3. Authoritative source and prohibition on inference

The source of the selector value is this successor package's exact adopted-
package binding. It is not a value supplied by a caller, reviewer, Policy,
Request, Result, Scope, Journal entry, configuration file, system clock, source
tree, or current implementation setting.

A conforming terminal producer may place key `24` only by deriving the one
literal above internally after the proposed Lifecycle v0.10.9 selected-route
conditions have been established. A public API MUST NOT accept the selector as a
caller-chosen terminal input. A Store MUST re-read key `24` from the exact
RecordId-addressed retained Record bytes during replay; it MUST NOT obtain the
value from a caller presentation, cache, or ambient configuration.

No relationship of any other value to this literal is defined. In particular,
equality of a historical Policy/SCOPE/Request/Result chain to the prospective
profile does not select this route.

## 4. Predecessor preservation and invalid selected forms

The existing canonical eight-field type-32 form without key `24` remains a valid
predecessor Record form. Its absence means only:

```text
PREDECESSOR_NOT_SUCCESSOR_SELECTED
```

It MUST NOT be treated as a missing required field, upgraded to the successor
route, or evaluated by evaluator `1015` merely because its referenced content
resembles the prospective profile. The Record and Journal retain exactly their
predecessor structural and semantic treatment.

A Record containing key `24` whose local shape is not §2, or whose bytes differ
from the one literal in §2, is neither a predecessor form nor a valid selected
form. It MUST fail closed as an unsupported or malformed successor selection;
there is no fallback to predecessor treatment.

## 5. Mechanical consequences

Key `24` changes the canonical Record bytes, so existing RecordId derivation
commits it automatically. The existing terminal Journal Entry already commits
that RecordId through `event_record_id`. No Journal field, event type, direct
authority dependency, identity dependency, or duplicate selector is added.

This successor does not change the existing type-32 keys `16` through `23`, the
accepted/rejected field combinations, EventType IDs `302` and `303`, the
Journal encoding, canonical-byte rules, or RecordId and JournalReference
algorithms.

## 6. Required vectors

The package must include at least:

```text
RA-SELECTOR-RECORD-HISTORICAL-ABSENT-VALID-001
RA-SELECTOR-RECORD-EXACT-ADOPTED-PACKAGE-VALID-001
RA-SELECTOR-RECORD-WRONG-DIGEST-REJECT-001
RA-SELECTOR-RECORD-MALFORMED-OR-NONCANONICAL-REJECT-001
RA-SELECTOR-RECORD-DUPLICATE-KEY-REJECT-001
RA-SELECTOR-RECORD-SAME-REFERENCES-ABSENT-NOT-SELECTED-001
```

The final vector must show that matching or profile-shaped referenced data cannot
replace the absent retained selector.

## 7. Non-claims

This candidate does not define generic Scope applicability, containment,
coverage, labels, payload semantics, Policy context membership, a generic
package resolver, a new SHA-256 identity domain, historical reinterpretation,
runtime implementation, publication, qualification, or adoption. The exact
literal becomes authority only if this entire draft package is independently
reviewed and explicitly adopted by the Owner.
