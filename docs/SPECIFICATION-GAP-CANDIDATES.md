# Specification-Gap Candidates

> **Status:** non-normative candidate register. This file does not amend, reinterpret,
> or supersede any frozen baseline specification. A candidate records a bounded
> implementation decision where the frozen authorities do not uniquely determine
> behavior. It is not an authority, admission, lifecycle, or interoperability claim.

## Register rules

- Each candidate is limited to the stated affected lane.
- A missing frozen rule cannot be converted into a successful validation claim.
- The frozen baseline documents remain the only normative sources; their bytes and
  hashes are not modified by this register.
- A later frozen/profile successor may resolve a candidate only by defining the
  missing rule explicitly.

## SG-001 — Opaque authoritative-CBOR container-depth profile

**Affected lane:** structural skipping of opaque Record-body values by
`StrictRecordFrame`; this does not affect type-local Record decoding or Journal parsing.

### Frozen authority and gap

- Identity Format v0.3 §9 permits definite arrays and maps in authoritative
  structures, forbids indefinite forms, and forbids duplicate map keys
  (`docs/IDENTITY-FORMAT-v0.3.md:518-556`).
- Record Schema v0.3 §14 defines bounded primitives and `CanonicalPath`, but does
  not define a global recursive-container-depth or resource-profile limit
  (`docs/RECORD-SCHEMA-v0.3.md:700-728`).
- Record Schema v0.3 §68 declares `CLOSEOUT.formal_support_binding` as an
  `embedded map` (`docs/RECORD-SCHEMA-v0.3.md:2683-2700`), and §74 defines
  its required top-level and Assumption Binding keys
  (`docs/RECORD-SCHEMA-v0.3.md:2930-2946`). Neither section defines a global
  recursive-container-depth or resource-profile limit for generic opaque
  Record-body structural skipping.

The frozen authorities do not provide a global resource profile for an
implementation that performs generic structural-only opaque-value skipping. This
does not establish that any particular known Record schema admits arbitrary
recursive values: type-local decoding remains subject to its closed field matrix
and unknown-key rejection.

### Competing outcomes

1. **An unbounded recursive generic skip algorithm:** recurse according to encoded
   container nesting without a parser resource bound. This permits hostile
   bounded-byte input to consume unbounded call stack and is rejected for the
   structural-parser safety lane.
2. **A local fail-closed resource profile:** accept container nesting through 64 and
   return a controlled decode error at 65. This is the current `StrictRecordFrame`
   implementation policy; it is not claimed to be a frozen normative constant.
3. **A future format/profile rule:** define a normative maximum depth or require an
   iterative bounded decoder. This is the required path for cross-implementation
   interoperability claims about accepted depth.

### Current bounded disposition

The current implementation uses outcome 2 only while structurally skipping opaque
body values. It makes no claim that a structurally valid frame proves a type-local
body schema, Record authority, policy applicability, admission, retained-history
resolution, lifecycle legality, replay correctness, or external trust.

## SG-002 — `FREEZE_RECEIPT.freeze_id` relationship

**Affected lane:** semantic/contextual `FREEZE_COMMITTED` binding and authority
validation. A strict type-local decoder may require the field and validate its
`bstr(32)` representation only.

### Frozen authority and gap

`FREEZE_RECEIPT` requires `freeze_id` as `OpaqueId32`
(`docs/RECORD-SCHEMA-v0.3.md:2253-2278`). The relevant frozen rules define
other bindings: the Receipt contains an exact START JournalReference
(`docs/EVIDENCE-REGISTRY-LIFECYCLE-SPEC-v0.10.2.md:2054-2092`), a committed
Freeze binds the exact Receipt and START (`docs/CROSS-REFERENCE-v0.3.md:1220-1225`),
and the authoritative root derives from `freeze_attempt_id`
(`docs/EVIDENCE-REGISTRY-LIFECYCLE-SPEC-v0.10.2.md:2008-2026`). This candidate
does not deny those bindings. Instead, the frozen baseline does not define an
exact direct semantic equality or derivation rule tying the distinct `freeze_id`
field to `freeze_attempt_id`, a Manifest identity, or a Journal-event identity.
The Receipt identity still commits to its complete canonical bytes, including
`freeze_id`; that identity commitment is not an equality or derivation rule for
the field's semantic value.

### Competing outcomes

1. **Infer a relationship** from field names or surrounding lifecycle wording. This
   would invent semantics and is prohibited.
2. **Retain an opaque typed value** and validate only required presence, width, and
   canonical CBOR framing. This is the required fail-closed boundary for any
   future type-local decoder.
3. **Define a future explicit relationship** in a frozen successor, then add the
   corresponding contextual equality/derivation validation.

### Current bounded disposition

A binding or authority-shaped API must treat `freeze_id` as unavailable semantic
context until outcome 3 exists. It must not return authority/admission success from
this field.

## SG-003 — MANIFEST path and digest profile resolution

**Affected lane:** type-local semantic validation of `MANIFEST.artifacts[]` and
the structural-continuity path that depends on a semantically validated Manifest.
This does not affect strict Record-frame decoding, exact Record identity, or
retention of opaque Manifest bytes.

### Frozen authority and gap

- `MANIFEST` requires a numeric `path_identity_profile_id`, a numeric
  `digest_profile_id`, and an `artifacts` array whose entries contain canonical
  paths and digest algorithm identifiers
  (`docs/RECORD-SCHEMA-v0.3.md:2168-2176`,
  `docs/RECORD-SCHEMA-v0.3.md:2180-2210`).
- The Lifecycle specification names `UTF8_STRICT_V1` and `NATIVE_LOSSLESS_V1`
  and specifies their representations, but supplies no numeric registry binding
  either profile to a `path_identity_profile_id`
  (`docs/EVIDENCE-REGISTRY-LIFECYCLE-SPEC-v0.10.2.md:1656-1710`). The selected
  profile defines component encoding and the path comparison algorithm
  (`docs/EVIDENCE-REGISTRY-LIFECYCLE-SPEC-v0.10.2.md:1656-1667`), including the
  selected-profile malformed-encoding rejection required for a canonical path
  (`docs/EVIDENCE-REGISTRY-LIFECYCLE-SPEC-v0.10.2.md:1744-1753`).
- The frozen baseline also provides no numeric `digest_profile_id` registry or
  rule relating that field to each Artifact Entry's `digest_algorithm_id`. The
  latter has a separately frozen algorithm registry
  (`docs/IDENTITY-FORMAT-v0.3.md:756-766`).

The frozen authorities therefore do not uniquely determine how a runtime selects
the path parser/comparator for a numeric Manifest profile or evaluates
digest-profile-to-algorithm coherence.

### Competing outcomes

1. **Invent a numeric mapping or profile/algorithm relationship:** infer that a
   numeric value selects a named path profile, or that a digest profile implies a
   particular artifact algorithm. This invents frozen semantics and is rejected.
2. **Treat raw component bytes as a universally validated canonical path:** this
   would bypass the required selected-profile encoding and comparator semantics,
   and could accept an out-of-order, duplicate, or malformed path under the
   actual selected profile. This is rejected for the semantic-MANIFEST lane.
3. **Fail closed for semantic Manifest validation:** retain only exact Record
   identity/framing facts and report profile semantics unavailable until a frozen
   profile registry and digest-profile relation are supplied. This is the current
   bounded disposition.
4. **A future frozen profile successor:** assign numeric profile IDs and define
   digest-profile/algorithm coherence, then permit strict typed Manifest
   validation and Receipt-to-Manifest semantic continuity checks.

### Current bounded disposition

No current runtime path may return a successful semantic-MANIFEST or
Receipt-to-Manifest continuity result by selecting an unstated profile mapping or
by treating bytewise parsing as the selected-profile validation. This does not
weaken the independent requirement that every successful Freeze continuity check
uses exact identities where the frozen rules do define them.
