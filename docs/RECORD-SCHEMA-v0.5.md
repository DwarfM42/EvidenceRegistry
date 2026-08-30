# EvidenceRegistry Record Schema Successor v0.5
## Review Package Journal Anchor Typed-Carrier Binding

**Status:** Draft successor candidate — not frozen, reviewed, accepted, or implementation authority
**Frozen semantic predecessor:** EvidenceRegistry Record Schema v0.3
**Adopted contract input (not a frozen-semantic-predecessor claim):** EvidenceRegistry Record Schema Successor v0.4
**Companion semantic source:** EvidenceRegistry Evidence Lifecycle Specification v0.10.7
**Companion mechanical propagation:** EvidenceRegistry Normative Event & Transition Cross-Reference v0.6
**Semantic scope:** prospective Review Package Journal Anchor carrier/version binding for `REVIEW_REQUEST` and `REVIEW_RESULT` only
**Normative language:** MUST / MUST NOT / SHOULD / SHOULD NOT / MAY

---

## 1. Purpose, predecessor preservation, and placement

Lifecycle v0.10.2 §117 requires a Review Package to include the current Journal
Anchor, a Review Result to preserve it unchanged, and Admission to compare the
returned Anchor with authoritative current history. Identity Format v0.3 §§93–96
already owns the canonical `JournalAnchor` bytes and the domain-separated
`JOURNAL_ANCHOR_ID` derivation. Record Schema v0.3, however, represents the two
transport fields only as `OpaqueId32`.

This successor supplies the minimum Record-Schema-owned bridge without changing
the `OpaqueId32` wire/storage carrier:

```text
canonical JournalAnchor bytes
    -> Identity Format v0.3 JOURNAL_ANCHOR_ID
    -> typed Review Package JournalAnchor identity
    -> OpaqueId32 carrier in the versioned Review records
```

This successor does not make all `OpaqueId32` values Journal Anchor identities.
It applies only to the two named fields when their accompanying binding version
is exactly `1` and Lifecycle v0.10.7 governs the operation.

Lifecycle v0.10.7 owns authoritative package creation, recovery, `review-admit`
boundary, and §82 consumption. Cross-Reference v0.6 mechanically propagates
that Lifecycle meaning. Identity Format v0.3 remains the sole owner of Anchor
framing, canonical encoding, domain separation, and digest derivation.

---

## 2. New prospective binding-version registry

This successor assigns one stable binding-version value:

| Value | Name |
| ---: | --- |
| `1` | `REVIEW_PACKAGE_JOURNAL_ANCHOR_V1` |

`1` means exactly the prospective authority chain defined by Lifecycle v0.10.7:

```text
Review Package current authoritative JournalAnchor
-> canonical Identity Format v0.3 bytes
-> JOURNAL_ANCHOR_ID
-> exact carrier transport in Request and Result
-> authoritative Journal-derived resolution at review-admit
```

No other value is defined by this successor. A decoder or operation that cannot
consume value `1` MUST stop before it claims the Lifecycle v0.10.7 chain. It
MUST NOT treat an unknown value as `0`, as an unversioned predecessor record, or
as a successful Policy or Admission result.

---

## 3. REVIEW_REQUEST structural supplement

For a prospective `REVIEW_REQUEST` governed by Lifecycle v0.10.7, the v0.3
§65 table is supplemented by the following required key:

| Key | Field | Type | Presence |
| --: | --- | --- | --- |
| `25` | `review_package_anchor_binding_version` | UInt | required |

The value MUST equal `1`.

The existing key remains unchanged structurally:

| Key | Field | Type | Presence |
| --: | --- | --- | --- |
| `23` | `review_package_anchor_id` | `OpaqueId32` | required |

For binding version `1`, the bytes at key `23` are a carrier for exactly the
32-byte authoritative representation of the typed `JOURNAL_ANCHOR_ID` derived
from the Review Package’s exact canonical JournalAnchor bytes under Identity
Format v0.3 §96. The carrier is not itself the typed identity and does not prove
that arbitrary caller-provided bytes are authoritative.

Before the Request is made authoritative or externally transported, Lifecycle
v0.10.7 requires the authoritative `review-pack` path to establish the package
Anchor, derive the typed identity, and bind this version/value pair to that
Request. Merely constructing a structurally valid Request map does not establish
that binding.

---

## 4. REVIEW_RESULT structural supplement

For a prospective `REVIEW_RESULT` governed by Lifecycle v0.10.7, the v0.3 §66
table is supplemented by the following required key:

| Key | Field | Type | Presence |
| --: | --- | --- | --- |
| `29` | `review_package_anchor_binding_version` | UInt | required |

The value MUST equal `1`.

The existing key remains unchanged structurally:

| Key | Field | Type | Presence |
| --: | --- | --- | --- |
| `27` | `review_package_anchor_id` | `OpaqueId32` | required |

For binding version `1`, Result construction MUST transport, byte-for-byte,
both the Request’s version and its `review_package_anchor_id` carrier. It MUST
NOT derive a fresh Anchor identity, replace the identity, refresh to a newer
Journal head, translate it into another domain, or permit a reviewer to select a
substitute.

The mechanical equality obligation is:

```text
ReviewResult.review_package_anchor_binding_version
    == ReviewRequest.review_package_anchor_binding_version
    == 1

ReviewResult.review_package_anchor_id bytes
    == ReviewRequest.review_package_anchor_id bytes
```

Equality alone is transport integrity, not authority. The authoritative recovery
and identity/body validation required before §82 remain exclusively governed by
Lifecycle v0.10.7.

---

## 5. Semantic-domain and parser boundary

For binding version `1` only, an implementation MUST conceptually decode the
field in this order:

```text
OpaqueId32 carrier with exact length 32
-> context/version selects the Review Package JournalAnchor carrier rule
-> typed JOURNAL_ANCHOR_ID candidate
-> Lifecycle-authoritative recovery of canonical Anchor bytes
-> Identity Format v0.3 recomputation and equality check
```

It is non-conforming to infer the typed domain from equal raw bytes held in an
unrelated field, an unrelated identity type, a reviewer assertion, a caller
parameter, a cache, a fixture, or a digest coincidence. An unrelated
`OpaqueId32` with the same 32-byte payload is not evidence that its source was a
JournalAnchor and cannot establish package binding or recovery authority.

The carrier remains `OpaqueId32` in the record schema. This successor does not
rename its global type, change CBOR encoding, add a generic raw-digest identity
domain, alter Identity Format’s fixed domain-separator registry, or permit
semantic-domain substitution.

---

## 6. Historical applicability

The new required keys and version-`1` interpretation are prospective only.
They apply only where Lifecycle v0.10.7 expressly governs Review Package
creation and Review Admission.

Predecessor `REVIEW_REQUEST` and `REVIEW_RESULT` records lacking keys `25` and
`29` MUST NOT be retroactively described as carrying a typed
`JOURNAL_ANCHOR_ID`, even if a historical `OpaqueId32` payload happens to equal
one. This successor defines neither a compatibility upgrade nor a fallback
admission path for such records. Their historical meaning remains governed by
predecessor authority.

---

## 7. Required structural vectors

The following vectors are additive requirements for the exact versioned carrier
relation. Full authority-chain vectors are in Lifecycle v0.10.7.

```text
RS-JA-1  Version-1 Request carries the 32-byte encoding of a known typed
         JOURNAL_ANCHOR_ID and includes key 25 = 1.

RS-JA-2  Version-1 Result exactly preserves Request keys 27/29 from Request
         keys 23/25.

RS-JA-3  A missing, unknown, or non-1 binding version cannot enter the
         Lifecycle v0.10.7 authority chain.

RS-JA-4  A Result carrier differing by one byte from its authoritative Request
         carrier is rejected before successful §82 processing.

RS-JA-5  Equal payload bytes in an unrelated OpaqueId32 context do not prove
         a JournalAnchor source, package binding, or authority.

RS-JA-6  A predecessor Request or Result without the new binding-version key
         is not retroactively classified as version 1.
```

These vectors do not replace the required Identity Format canonical-byte vectors
or establish runtime qualification.

---

## 8. Non-claims

This successor does not define or authorize:

- a new generic meaning for `OpaqueId32`;
- a new JournalAnchor encoding, normalization, hash, or identity domain;
- a new Record type, EventType, Journal event, authority dependency, lifecycle
  state, Policy evaluator, or disposition;
- caller-provided Anchor-body authority;
- an in-memory-only continuation requirement;
- Journal-history comparison policy beyond the existing Lifecycle rules;
- retrospective record reinterpretation;
- runtime implementation, Admission, Policy satisfaction, authority,
  qualification, release, publication, or remote operation.

The exact `OpaqueId32` carrier and typed semantic identity remain distinct
concepts. Conformance requires the companion Lifecycle recovery and §82 rules;
structural decoding of these new keys alone is insufficient.
