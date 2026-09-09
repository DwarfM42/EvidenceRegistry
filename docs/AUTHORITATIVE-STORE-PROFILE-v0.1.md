# EvidenceRegistry Authoritative Store Profile v0.1
## Local Retained Journal and Immutable Record Publication

**Status:** DRAFT SUCCESSOR CANDIDATE — NOT FROZEN, REVIEWED, ACCEPTED, OR IMPLEMENTATION AUTHORITY

**Frozen semantic inputs:** Identity Format v0.3 exact Record and Journal encodings; Lifecycle v0.10.2 Journal authority and publication boundaries; Record Schema v0.3 typed Record requirements; Cross-Reference v0.3 transitions.

**Candidate-package role:** This profile supplies the missing public-store selection for the bounded authority path. It does not alter any identity or lifecycle semantic rule.

## 1. Scope and Owner choices

The frozen baseline requires exact Record bytes and retained Journal context but does not select a public persistent resolver, namespace, append conflict contract, receipt boundary, or platform profile. This document proposes one deliberately narrow local-store profile.

The following are new Owner choices:

1. one root directory is the authoritative Store locator for this profile;
2. the fixed namespace and filenames in §2 are public profile syntax;
3. writer serialization is cooperative among implementations of this profile, not a hostile same-principal exclusion claim;
4. a terminal result is reported `PUBLISHED` only after exact post-publication Store reopen and retained replay; a visibility/readback ambiguity is `PUBLISHED_RECEIPT_UNCERTAIN`, never success;
5. the profile supports only Windows x86_64, Linux x86_64, and macOS arm64 adapters that separately meet its required operations.
6. `registry/genesis.cbor` is the exact GENESIS Record duplicate; the GENESIS Journal Entry remains only journal slot zero.
7. each physical Freeze root and its attempt-coordination locator use the lower-case hexadecimal projection of the exact 32-byte `freeze_attempt_id`.
8. one Store-owned immutable Record staging and Journal-append mechanism may create prerequisite event-backed inputs; it never substitutes operation-specific lifecycle legality or promotes a staged Record to authority.

No selection here is implied by a source-directory shape, a test fixture, a README example, or an in-memory resolver.

## 2. Canonical namespace

For Store root `R`, the required retained and operational namespace is exactly:

```text
R/
  registry/genesis.cbor
  journal/00000000000000000000.cbor
  journal/00000000000000000001.cbor
  ... contiguous twenty-digit decimal slots ...
  records/<64-lowercase-hex-record-id>.cbor
  roots/<64-lowercase-hex-freeze-attempt-id>/
  coordination/freeze/<64-lowercase-hex-freeze-attempt-id>.lock
```

The `registry`, `journal`, `records`, `roots`, and `coordination` path components are direct children of `R`; `coordination/freeze` is a direct child of `coordination`. No alternate root, case variation, sharding, alias, symlink/reparse substitution, recursive discovery, or filename normalization is permitted by this profile.

`registry/genesis.cbor`, every `journal` slot, and every `records` object MUST be a regular file. Every Record filename must be the lowercase hexadecimal projection of the SHA-256 `RecordId` of its exact file bytes plus `.cbor`. A duplicate RecordId with different bytes is invalid. Root leaves and the required namespace directories MUST be real directories, never reparse points or symlinks.

`registry/genesis.cbor` MUST strictly decode as the GENESIS Record. Its exact bytes MUST also occur at `records/<its-record-id>.cbor`. Journal slot zero MUST strictly decode as the GENESIS Journal Entry; it MUST have event type `1`, name that exact GENESIS Record through `event_record_id`, share its Registry identity, and carry the exact matching initial storage-capability and environment-observation identities required by Lifecycle v0.10.2 §11.1. A mismatch is rejected; neither file is an unchecked duplicate or an alternate source of truth.

The Journal slots must form a contiguous sequence beginning at index zero. Each slot's bytes must strictly decode, append in filename order, and replay from that exact GENESIS Entry. Missing, extra, malformed, incorrectly named, out-of-order, duplicate, or unsupported retained files fail closed. The profile never repairs, renames, sorts, deletes, or synthesizes retained namespace contents.

A root leaf name is the lower-case hexadecimal projection of its exact 32-byte `freeze_attempt_id`. Every retained root leaf MUST resolve to an exact retained `FREEZE_ATTEMPT_STARTED` Entry for the same Registry and Attempt, and that Entry's retained `intended_root_id` MUST equal the frozen `FreezeRoot(registry_id, freeze_attempt_id)` derivation. A started Attempt need not yet have a root: a crash may occur before root creation. Conversely, a root without that retained START, an incorrectly named root, an existing root selected for a new Attempt, or a committed Freeze without its corresponding revalidatable root fails closed. The Store opening validates this locator relation; payload traversal and Manifest verification remain explicit Freeze-operation work and are never inferred by recursive discovery.

`coordination/freeze/<attempt-id>.lock` is a nonauthoritative live OS-backed attempt-coordination locator. Its pathname is derived from the same Registry root and Attempt ID. Persistent lockfile existence alone proves neither liveness nor authority; replay never reads it as an authoritative input. The frozen Lifecycle §23 conceptual `registry/format.cbor`, `runtime/`, and `projections/` sketches are not selected as retained authority objects by this narrow profile, and no missing or extra object is silently treated as one.

## 3. Authoritative input resolution

A production authority operation under this profile MUST obtain every named Record only from `records/<record-id>.cbor`, then:

```text
read exact bytes
-> strict decode
-> recompute RecordId
-> compare to requested RecordId and filename
-> validate the relevant typed schema
-> validate the exact retained Journal event/reference relation required by the operation
```

Caller-supplied Record bytes, a caller-supplied Journal head, a caller-supplied Policy result, an in-memory reconstruction, or a cached resolver result may be retained only as untrusted diagnostic material. None may substitute for the Store-resolved authority input. A Record staged in `records/` without its required Journal event is exact immutable input material, not an authoritative fact or Policy result.

The Store must retain an identity-and-byte witness for the namespace generation it evaluated. If a root, namespace directory, or retained object is replaced, changes identity, changes bytes, becomes inaccessible, or cannot be revalidated, the operation fails closed before positive evaluation or publication.

## 4. Store-owned immutable staging and publication

### 4.1 Immutable Record staging

A Store-owned producer may stage an exact typed Record only by atomic no-replace creation at its RecordId filename, followed by exact readback and retained-generation revalidation. A pre-existing same-identity byte copy may be reused only after exact byte/identity revalidation; a different byte copy, uncertain creation result, or failed readback is non-success. Staging does not create a Journal event, does not establish authority, and does not choose a Policy or terminal disposition.

### 4.2 Journal append for prerequisite and terminal events

Every Store-owned operation that records a lifecycle event, including GENESIS initialization, FREEZE_ATTEMPT_STARTED, Review prerequisite events, and terminal events, uses this sequence after its operation-specific frozen prerequisites are satisfied:

```text
1. acquire the profile's cooperative root publication serialization;
2. re-open and strictly replay the retained Store state;
3. derive the next exact Journal index and previous-entry hash from that current state;
4. construct the canonical event Record and Journal Entry only from Store-derived and operation-owned inputs;
5. preflight the entry against the retained transition and typed-record rules;
6. stage or revalidate the exact event Record with atomic no-replace semantics;
7. publish the exact Journal slot with atomic no-replace semantics;
8. flush required content and parent-directory metadata where the selected platform adapter can establish them;
9. re-open the Store, replay it, and verify the exact Record bytes and exact JournalReference.
```

For a Freeze operation, the producer additionally acquires the attempt-specific coordination locator, appends and reads back the exact `FREEZE_ATTEMPT_STARTED` event, exclusively creates only the corresponding derived root, exclusively creates the first payload, retains and revalidates the root generation through Receipt and terminal decision, and releases coordination only according to the frozen recovery boundary. It MUST NOT create a root before START or silently regenerate an Attempt after a root conflict.

An observed competing publication, missing no-replace primitive, failed flush, changed retained generation, or pre-visibility error is not success. An event Record whose bytes become durable before its Journal event is nonauthoritative until the Journal event is committed and read back. A nonterminal append readback establishes only the event authority defined by frozen lifecycle rules; it does not establish any downstream Freeze, Policy, Review, or terminal result.

If a Journal slot may have become visible but the implementation cannot complete the specified readback, it MUST return an explicit uncertain receipt. For a terminal operation it is `PUBLISHED_RECEIPT_UNCERTAIN`. It MUST expose candidate Record and Journal identities only as diagnostic candidates; it MUST NOT claim authority, completed publication, or a successful terminal result.

## 5. Platform profile

Windows x86_64, Linux x86_64, and macOS arm64 may implement this profile only when the adapter establishes all of the following for the exact local Store root:

```text
regular-file and directory identity checks
no-replace immutable Record and Journal-slot publication
content flush before success reporting
parent-directory metadata flush before PUBLISHED receipt
root-level cooperative writer serialization
post-publication exact-byte and retained-replay readback
```

On Windows, a writer may use deny-share handles. On Linux and macOS, a writer may use `flock` plus exact object/byte revalidation. Neither mechanism is a universal security boundary against a same-principal writer that ignores the protocol. A platform or filesystem lacking any required operation is unsupported for positive publication under this profile; it must not fall back to a best-effort write.

This profile does not establish universal crash durability, external custody, remote replication, consensus, ACL enforcement, or protection against a hostile principal with equivalent filesystem authority.

## 6. Required Store-profile vectors

The package must prove at minimum:

```text
STORE-OPEN-EXACT-NAMESPACE-VALID-001
STORE-OPEN-GENESIS-RECORD-ENTRY-BINDING-VALID-001
STORE-OPEN-GENESIS-DUPLICATE-MISMATCH-REJECT-001
STORE-OPEN-NONCANONICAL-NAME-REJECT-001
STORE-OPEN-ROOT-START-LOCATOR-VALID-001
STORE-OPEN-ORPHAN-ROOT-REJECT-001
STORE-OPEN-REPLACED-RETAINED-GENERATION-REJECT-001
STORE-STAGE-RECORD-NO-REPLACE-VALID-001
STORE-STAGE-RECORD-CONFLICT-REJECT-001
STORE-APPEND-PREREQUISITE-REPLAY-READBACK-VALID-001
STORE-FREEZE-START-BEFORE-EXCLUSIVE-ROOT-001
STORE-FREEZE-PREEXISTING-ROOT-NO-SUCCESS-001
STORE-PUBLISH-RECORD-THEN-JOURNAL-VALID-001
STORE-PUBLISH-RECORD-NO-REPLACE-CONFLICT-001
STORE-PUBLISH-JOURNAL-NO-REPLACE-CONFLICT-001
STORE-PUBLISH-PRE-VISIBILITY-FAIL-NO-SUCCESS-001
STORE-PUBLISH-POST-VISIBILITY-UNCERTAIN-001
STORE-PUBLISH-REPLAY-READBACK-VALID-001
```

The vectors must be run against every profile-qualified platform adapter. They are implementation and qualification evidence, not adoption evidence.

## 7. Non-retroactivity and non-claims

This candidate changes no Record encoding, RecordId, JournalReference, Journal Entry encoding, lifecycle object identity, event ID, direct authority dependency, terminal disposition, or existing record's historical authority.

It is not a general database, directory scanner, repair tool, networked registry, custody system, policy evaluator, release declaration, or runtime qualification. A frozen profile still requires a separate production implementation, RED-to-GREEN vectors, hostile review, and platform qualification before any public positive-authority claim.