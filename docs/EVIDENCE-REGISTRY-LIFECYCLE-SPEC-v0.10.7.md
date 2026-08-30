# EvidenceRegistry Evidence Lifecycle Specification v0.10.7
## Review Package Journal Anchor Authority-Binding Closure

**Status:** FREEZE CANDIDATE — NOT YET FROZEN
**Frozen semantic predecessor:** EvidenceRegistry Evidence Lifecycle Specification v0.10.2
**Adopted contract inputs (not frozen-semantic-predecessor claims):** Lifecycle v0.10.4 and Lifecycle v0.10.6
**Required companion successors:** Record Schema v0.5 and Cross-Reference v0.6
**Project:** EvidenceRegistry
**Normative language:** MUST / MUST NOT / SHOULD / SHOULD NOT / MAY

---

# 0. Position, exact gap, and preservation

Lifecycle v0.10.2 §117 already requires:

```text
Review Package includes current Journal Anchor
Review Result preserves it unchanged
Admission compares returned Anchor with current history
```

Identity Format v0.3 §§93–96 already defines one canonical portable
`JournalAnchor` representation and its domain-separated `JOURNAL_ANCHOR_ID`.
Record Schema v0.3 §§65–66, however, stores both
`review_package_anchor_id` values only as `OpaqueId32`. The predecessor
requirements do not uniquely select all of these necessary edges:

```text
which typed semantic identity the two carriers represent;
how the authoritative package binds its exact Anchor to its Request;
how a later Admission obtains the exact Anchor body without caller authority;
how identity/body recomputation is required before §82 consumes it.
```

This successor closes only that complete Review Package Journal Anchor chain.
It is not a one-field alias and does not reopen Lifecycle v0.10.6’s separately
Owner-frozen `REVIEW_ADMISSION.operation_start_journal_ref` relation.

Except where expressly stated here, Lifecycle v0.10.2 and adopted v0.10.4 and
v0.10.6 rules retain their prospective and historical meanings. This successor
does not alter the canonical bytes or identities of predecessor Records,
Journal Entries, Journal Anchors, or JournalReferences.

---

# 1. Terms and distinct identity classes

For this successor:

```text
AnchorBytes(A)
    = exact canonical Identity Format v0.3 JournalAnchor encoding.

AnchorId(A)
    = the typed JOURNAL_ANCHOR_ID derived from AnchorBytes(A) by
      Identity Format v0.3 §96.

AnchorCarrier
    = the 32 bytes stored in the version-1 OpaqueId32 field defined by
      Record Schema v0.5. It is transport/storage representation only.

AuthoritativeReviewPackage
    = the exact external Review Package assembled by authoritative review-pack
      under §2, including AnchorBytes(A) and the authoritative Review Request
      whose version-1 carrier equals AnchorId(A).

ResolvedAnchor(A)
    = the one canonical AnchorBytes value recovered under §4 from authoritative
      retained Registry history, not from caller or reviewer material.
```

The following values are distinct and MUST NOT be substituted merely because
some happen to contain equal 32-byte payloads:

```text
AnchorCarrier
!= typed AnchorId
!= raw SHA-256 digest in another identity domain
!= JournalReference
!= REVIEW_ADMISSION.operation_start_journal_ref
!= terminal Journal-entry predecessor identity
```

`AnchorId(A)` uses the existing Identity Format v0.3 domain separator
`EvidenceRegistry.JournalAnchor.v1`; no successor encoding, normalization,
second digest framing, or generic identity framework is introduced here.

---

# 2. Authoritative Review Package creation and binding

## 2.1 Authoritative source

At authoritative `review-pack` creation, the Registry MUST obtain the current
head only from its authoritative retained Journal under the existing Journal
head rules.  The package-Anchor selection occurs at one serializable
package-binding point `L(P)`: at that point the Registry observes one
authoritative Journal head and binds it to the exact package instance `P`.
Every authoritative append racing with `L(P)` MUST be ordered wholly before or
wholly after it; only an append ordered before `L(P)` is reflected in `P`'s
Anchor.  `L(P)` selects only the Review Package Anchor and does not select,
replace, or reinterpret any `operation_start_journal_ref`.

It MUST deterministically construct `AnchorBytes(A)` from that Registry
identity, head index, head hash, and Journal format version using Identity
Format v0.3 §§93–95, then derive `AnchorId(A)` using §96.

The resulting Review Package MUST include exactly `AnchorBytes(A)` as required
by Lifecycle v0.10.2 §117. Before external review transport, the authoritative
Review Request must be constructed under Record Schema v0.5 with:

```text
review_package_anchor_binding_version = 1
review_package_anchor_id carrier      = bytes(AnchorId(A))
```

The authoritative `review-pack` action binds, as one package-construction fact:

```text
exact Review Package bytes including AnchorBytes(A)
+ exact canonical AnchorBytes(A)
+ typed AnchorId(A)
+ exact version-1 Review Request carrier
+ the exact Review Request instance that is subsequently made authoritative by
  the existing REVIEW_REQUEST_RECORDED path before external review transport
```

No caller, reviewer, helper, cache, fixture, or raw transport field may select
or replace any part of that binding. A structurally valid Request with arbitrary
32 bytes is not an AuthoritativeReviewPackage.  A package that has not reached
the existing Request-authority path is not sufficient later to establish the
authoritative Request input required by §3.

## 2.2 Transport relation

The Reviewer receives the package-bound identity but does not acquire authority
to choose Journal history. A version-1 Review Result MUST preserve the Request’s
binding version and carrier byte-for-byte as specified by Record Schema v0.5
and mechanically propagated by Cross-Reference v0.6.

The reviewer MUST NOT:

```text
compute a new AnchorId
replace a returned AnchorId
refresh to a later Journal head
translate to another semantic domain
supply an Anchor body as authoritative material
```

The Result’s unchanged carrier is only an integrity-preserving transport of the
already-bound typed identity. It is not a grant of Journal authority to the
reviewer.

---

# 3. Admission input binding before resolution

`review-admit` MUST first obtain the exact authoritative Review Request through
the Result’s existing Request-authority path. It MUST validate every existing
§82 prerequisite that precedes or is necessary to establish the Request/Result
binding, including the exact Request authority and Result integrity.

For a version-1 chain, before it can call successful returned-Anchor comparison,
`review-admit` MUST establish all of:

```text
Request.binding_version == 1
Result.binding_version  == 1
Request.anchor_carrier bytes == Result.anchor_carrier bytes
Request.anchor_carrier has exact OpaqueId32 length
```

Only after those conditions are established may the equal carrier bytes be
conceptually decoded as the typed `AnchorId` candidate required by this
successor. Equal raw bytes alone are not sufficient: the Request must be
authoritative and the authoritative resolution in §4 must succeed.

A missing version, unsupported version, malformed carrier, missing authoritative
Request, or Request/Result carrier mismatch is an existing §82 prerequisite
failure. Under adopted Lifecycle v0.10.4, it remains pre-terminal: no
`REVIEW_ADMISSION_ACCEPTED`, no `REVIEW_ADMISSION_REJECTED`, no event 302, and
no event 303 may be published for that attempt. This successor does not convert
the failure to a Policy evaluator result.

---

# 4. One authoritative resolution mechanism

## 4.1 Selection

The sole conforming production resolution mechanism for this successor is
**authoritative Journal-derived resolution**. The implementation MUST derive
candidate Anchor bodies only from the authoritative retained Journal of the
Registry named by the authoritative Review Request context.

For each retained authoritative Journal prefix, it MUST deterministically
construct the exact canonical Identity Format v0.3 JournalAnchor for that
prefix and derive its typed `JOURNAL_ANCHOR_ID`. The resolver candidate set is:

```text
{ canonical AnchorBytes(H) | H is an authoritative retained prefix of the
  exact Registry Journal and AnchorId(H) == carried typed AnchorId }
```

Successful resolution requires candidate-set cardinality exactly one. That sole
value is `ResolvedAnchor(A)`.

This mechanism is restart- and detached-transport-independent: it relies on
retained authoritative Journal history, not a process-local Review Package
object, an in-memory cache, a caller-retained body, an embedded unverified
package copy, or a mutable lookup hint.

## 4.2 Prohibited substitutes

The following must never establish returned-Anchor authority:

```text
review_admit(result, caller_anchor_bytes)
review_admit(result, caller_prevalidated_anchor)
reviewer-supplied Anchor bytes
helper-supplied current or historical head
cache hit without the authoritative Journal-derived reconstruction
arbitrary registry lookup key
raw digest equality without canonical-body recomputation
```

An API may accept candidate bytes only as untrusted diagnostic material if it
independently performs §4.1 and exact comparison. Candidate material cannot
change the candidate set, satisfy its cardinality rule, or become authoritative
because its hash/carrier bytes match.

## 4.3 Identity/body verification

After successful §4.1 resolution, `review-admit` MUST:

```text
1. validate ResolvedAnchor(A) as the canonical Identity Format v0.3
   JournalAnchor structure;
2. derive AnchorId(ResolvedAnchor(A)) using Identity Format v0.3 §96;
3. compare that typed identity to the version-1 Request/Result carrier decoded
   in the named JOURNAL_ANCHOR_ID context.
```

Any canonical-validation failure, zero candidate, more than one candidate, or
identity/body mismatch fails closed before successful §82 returned-Anchor
processing. It MUST NOT be treated as a reviewer choice, a successful raw
32-byte equality check, a valid Anchor merely because the carrier has length
32, or a completed Policy result.

For one exact canonical Anchor body, Identity Format v0.3 §96 yields exactly
one typed `JOURNAL_ANCHOR_ID`. The resolver MUST NOT attach a second semantic
identity, accept an alternate domain label, or let a same-payload value from
another identity domain stand in for that one derived identity.

---

# 5. §82 authoritative returned-Anchor consumption

Only after §§3–4 succeed does §82 receive:

```text
an authoritative ResolvedAnchor(A)
```

It does not receive only `OpaqueId32`, an untrusted caller body, or the
operation-start Journal reference. §82 then performs the existing Lifecycle
v0.10.2 §116 / §117 current-history comparison on `ResolvedAnchor(A)`.

This successor does not invent new Anchor-admission policy. Existing results
remain controlling:

```text
ANCHOR_EQUALS_CURRENT_HEAD
ANCHOR_IS_VALID_ANCESTOR
JOURNAL_DIVERGENCE
JOURNAL_HISTORY_BEHIND_ANCHOR
ANCHOR_FROM_DIFFERENT_REGISTRY
ANCHOR_INVALID
```

In particular, a valid historical Review Package Anchor is not silently
refreshed when the Journal advances. Existing §117 determines that a valid
ancestor is acceptable; its existing potentially-blocking outcomes remain
unchanged. An unknown, malformed, cross-registry, ambiguous, or wrong-domain
candidate cannot pass §4 and therefore cannot be promoted into a successful
§82 returned-Anchor result.

The Review Package Anchor and Lifecycle v0.10.6 operation-start binding remain
independent:

```text
Review Package Anchor
!= REVIEW_ADMISSION.operation_start_journal_ref
!= terminal immediate predecessor
```

No rule here refreshes either one, derives one from another, or changes
Lifecycle v0.10.6 `L(A)` semantics.

---

# 6. Required companion placement

This Lifecycle successor owns the operation-specific authoritative workflow,
resolver selection, pre-terminal §82 boundary, and consumer relation because
those are Review Package / Review Admission lifecycle semantics.

Record Schema v0.5 is required because it owns the two Record fields, their
prospective version boundary, and their structural carrier interpretation.
Identity Format v0.3 remains unchanged because canonical Anchor bytes and typed
identity derivation already exist exactly.

Cross-Reference v0.6 is required as a derivative mechanical companion because
it owns the Review Request/Result recorded-event bindings and must propagate,
without independently augmenting, this Lifecycle relation. It must preserve the
rule that a Journal Anchor is a historical commitment, not an authority
dependency.

The three successors must be reviewed and frozen together as one bounded
contract. A partial adoption does not establish the full authority chain.

---

# 7. Conformance vectors

The following vectors are normative for this authority chain and are additive.
They do not amend predecessor vectors.

## JA-1 — valid authoritative round trip

```text
authoritative retained head -> canonical Anchor A -> AnchorId IA
review-pack binds A + IA + version-1 Request
Result preserves IA/version exactly
review-admit resolves A from authoritative Journal history
rederives IA and compares it
§82 consumes authoritative A
```

## JA-2 — reviewer substitution

```text
Request carries IA
reviewer changes Result carrier to IB
-> Request/Result equality fails before successful §82 Anchor processing
```

## JA-3 — caller body substitution

```text
Result carries IA
caller supplies bytes B with identity IB, or with no authoritative source
-> B cannot enter or satisfy Journal-derived resolution
-> B cannot become returned-Anchor authority
```

## JA-4 — equal raw bytes, wrong semantic source

```text
an unrelated OpaqueId32 contains payload IA
but it was not produced as the authoritative Request’s version-1 carrier
-> it does not prove JournalAnchor source or package binding
-> only authoritative Request binding plus §4 resolution can establish IA
```

## JA-5 — unknown identity

```text
Request/Result carry a syntactically valid 32-byte version-1 carrier
no authoritative retained-Journal prefix derives it
-> zero-candidate resolution
-> fail closed before successful §82 processing
```

## JA-6 — identity/body mismatch

```text
resolver candidate A is obtained
recomputed AnchorId(A) != carried typed identity
-> fail closed before successful §82 processing
```

## JA-7 — intervening append

```text
package binds Anchor A at H0
Journal later appends H1
Result returns IA
-> resolver recovers A, not a fresh H1 Anchor
-> existing §116/§117 history rule decides its relation
```

## JA-8 — restart / later return

```text
Review Package process exits after external transport
later Result return resolves only through retained authoritative Journal prefixes
-> no in-memory package object or caller body is required or authoritative
```

## JA-9 — operation-start distinction

```text
Review Package Anchor A is bound at review-pack
Review Admission later binds Hstart under Lifecycle v0.10.6 L(A)
-> neither is substituted for the other, even if both refer to historical
   Journal states
```

## JA-10 — historical integrity

```text
predecessor Request/Result lack the prospective binding-version keys
-> they are not retroactively classified as version-1 typed Anchor transport
```

Additional rejection vectors MUST prove zero/multiple resolution candidates,
canonicalization failure, wrong Registry, a raw carrier collision from another
domain, and a later-head refresh attempt. None may be recast as Policy
satisfaction, `GATE_UNSATISFIED`, or `GATE_INDETERMINATE`.

---

# 8. Non-claims

This successor does not:

- redefine JournalAnchor bytes, SHA-256, or any Identity Format domain;
- redesign generic object storage, all Journal identities, all OpaqueId32
  fields, or unrelated Review records;
- change Scope, Policy, §46 evaluator semantics, §83 mapping, terminal events,
  operation-start timing, Journal append serialization, or authority-dependency
  semantics;
- authorize an implementation, runtime claim, Policy satisfaction, Admission,
  event publication, qualification, release, push, tag, or remote operation;
- reinterpret predecessor records or historical admissions.

The rule closes exactly the authority-preserving path from authoritative Review
Package creation through detached review transport to authoritative §82 Anchor
validation. It must be independently implemented and reviewed before any
runtime claim.
