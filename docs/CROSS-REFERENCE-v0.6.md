# EvidenceRegistry Normative Event & Transition Cross-Reference v0.6
## Review Package Journal Anchor Authority-Binding Companion

**Status:** FREEZE CANDIDATE — NOT YET FROZEN
**Frozen semantic predecessor:** EvidenceRegistry Normative Event & Transition Cross-Reference v0.3
**Adopted contract input (not a frozen-semantic-predecessor claim):** Cross-Reference v0.5
**Normative semantic source:** EvidenceRegistry Evidence Lifecycle Specification v0.10.7
**Required Record companion:** EvidenceRegistry Record Schema Successor v0.5
**Purpose:** mechanical propagation of the Review Package Journal Anchor typed-carrier and authoritative-resolution rules

---

# 0. Companion position and self-limitation

Lifecycle v0.10.7 is the only semantic source for the prospective Review Package
Journal Anchor authority chain. This Cross-Reference successor mechanically
propagates that rule at the existing Review Request, Review Result, and Review
Admission mapping boundaries.

It does not independently create:

```text
JournalAnchor canonical bytes
JOURNAL_ANCHOR_ID derivation
Review Package creation authority
an Anchor resolver
an Admission prerequisite
an event disposition
an authority dependency
```

If interpreted to add a Lifecycle rule that v0.10.7 does not contain, this
successor is invalid. Record Schema v0.5 continues to own the versioned Record
field supplement; Identity Format v0.3 continues to own canonical Anchor bytes
and the `JOURNAL_ANCHOR_ID` domain.

---

# 1. REVIEW_REQUEST_RECORDED propagation

For a prospective `REVIEW_REQUEST_RECORDED` governed by Lifecycle v0.10.7 and
Record Schema v0.5 binding version `1`, the existing required identity
commitment named `review_package_anchor` in Cross-Reference v0.3 §29 is
mechanically represented as:

```text
IdentityDependencyKind = JOURNAL_ANCHOR_ID
IdentityDependency value = typed identity carried by
    ReviewRequest.review_package_anchor_id
binding version = ReviewRequest.review_package_anchor_binding_version == 1
```

This is an identity dependency and historical commitment only. It MUST NOT be
inserted into `authority_dependencies[]`, used as a substitute for Request
authority, or treated as Registry authority merely because it is required by
Review transport.

The event mapping does not create a second Anchor encoding. The carrier bytes
are interpreted only through Record Schema v0.5 and the Identity Format v0.3
`JOURNAL_ANCHOR_ID` domain selected by version `1`.

---

# 2. REVIEW_RESULT_RECORDED propagation

For a prospective `REVIEW_RESULT_RECORDED` governed by Lifecycle v0.10.7 and
Record Schema v0.5 binding version `1`, the Result MUST directly
authority-depend on the exact Review Request as already required by
Cross-Reference v0.3 §30.

Its existing redundant Request binding now mechanically includes both:

```text
ReviewResult.review_package_anchor_binding_version
    == ReviewRequest.review_package_anchor_binding_version
    == 1

ReviewResult.review_package_anchor_id carrier bytes
    == ReviewRequest.review_package_anchor_id carrier bytes
```

The Result’s event-level identity commitment for `review_package_anchor` MUST
therefore equal the Request’s typed `JOURNAL_ANCHOR_ID` commitment. The Result
MUST NOT create a new Anchor dependency, select a newer Journal head, or use an
untrusted body to establish the commitment.

This equality confirms transport preservation only. It does not itself resolve
canonical Anchor bytes, compare history, establish Anchor authority, satisfy
Policy, or create an Admission result.

---

# 3. REVIEW_ADMISSION prerequisite propagation

For a prospective Review Admission governed by Lifecycle v0.10.7, the
Cross-Reference mechanically requires the following sequence before successful
returned-Anchor §82 processing:

```text
authoritative Review Request resolution
-> version-1 Request/Result carrier equality
-> Lifecycle v0.10.7 authoritative Journal-derived Anchor recovery
-> canonical Anchor validation and typed identity/body equality
-> existing §82 Anchor-history comparison
```

The anchor recovery source is the authoritative retained Registry Journal only,
exactly as selected by Lifecycle v0.10.7. A caller, reviewer, helper, cache,
fixture, raw digest equality, or candidate Anchor body is not an alternative
Cross-Reference route.

Failure to establish the sequence is an existing §82 prerequisite failure. The
pre-terminal boundary propagated by Cross-Reference v0.5 §31.3 remains
controlling:

```text
no REVIEW_ADMISSION_ACCEPTED
no REVIEW_ADMISSION_REJECTED
no event 302
no event 303
```

This companion does not reclassify such a failure as an evaluator result or
alter the completed-result mappings already derived from Lifecycle v0.10.4.

---

# 4. Identity and authority separation

The following distinctions are mandatory at the mechanical mapping boundary:

```text
typed JOURNAL_ANCHOR_ID
!= Review Request authority JournalReference
!= Review Result authority JournalReference
!= Policy authority JournalReference
!= REVIEW_ADMISSION.operation_start_journal_ref
!= terminal publication predecessor
```

`JOURNAL_ANCHOR_ID` identifies canonical portable Anchor content. It is not
Registry authority, and it has no authority-dependency role. The existing Review
Request/Result authorities establish which Record payloads are authoritative;
the Lifecycle resolver establishes which canonical Anchor body is authoritative
for §82 comparison. Neither relation may be replaced by the other.

---

# 5. Mechanical conformance vectors

The following companion vectors are required in addition to Lifecycle v0.10.7
JA-1 through JA-10 and Record Schema v0.5 RS-JA-1 through RS-JA-6:

```text
CR-JA-1  REVIEW_REQUEST_RECORDED version 1 carries an identity dependency whose
         kind/value equal the typed JOURNAL_ANCHOR_ID encoded in its carrier.

CR-JA-2  REVIEW_RESULT_RECORDED authority-depends on the exact Request and
         preserves both version and carrier bytes exactly.

CR-JA-3  A reviewer carrier substitution cannot reach authoritative resolver or
         successful §82 Anchor comparison.

CR-JA-4  Caller-supplied Anchor bytes are absent from the authoritative resolver
         input set and cannot satisfy it.

CR-JA-5  Zero, multiple, malformed, wrong-Registry, or identity/body-mismatched
         resolution candidates reach neither terminal Admission event.

CR-JA-6  A later Journal head does not replace the Request-bound Anchor identity
         or Lifecycle v0.10.6 operation-start reference.
```

The vectors prove only this companion’s mechanical propagation. They do not
prove implementation, runtime qualification, Policy satisfaction, authority, or
legal event publication.

---

# 6. Historical boundary and non-claims

This successor applies prospectively only to exact version-`1` Review records
and Lifecycle v0.10.7 operations. It MUST NOT retroactively classify a
predecessor `OpaqueId32` carrier as `JOURNAL_ANCHOR_ID`, infer a missing binding
version, or modify historical event semantics.

It does not change:

- EventType IDs/names, operation names, phases, states, terminality, or Journal
  serialization;
- Record field bytes, Record IDs, JournalReference bytes, AuthorityDependency
  semantics, or Identity Format bytes;
- Lifecycle v0.10.6 operation-start semantics;
- Policy evaluator IDs, Policy result grammar, §46, §83, or event 302/303
  meanings;
- generic resolver architecture, generic object storage, or any unrelated
  `OpaqueId32` field.

This Cross-Reference has no independent authority to preserve an incompatible
rule if its Lifecycle v0.10.7 semantic source changes. It is a bounded
mechanical companion, not an implementation or qualification claim.
