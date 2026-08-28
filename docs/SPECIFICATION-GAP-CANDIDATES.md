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
- Record Schema v0.3 §68 declares `CLOSEOUT.formal_support_binding` only as an
  `embedded map`; its nested grammar is not defined in the frozen baseline
  (`docs/RECORD-SCHEMA-v0.3.md:2683-2700`).

The frozen authorities therefore do not uniquely select an interoperable maximum
container depth for every possible opaque Record-body value.

### Competing outcomes

1. **Unlimited recursive parsing:** accept every otherwise canonical definite
   nesting depth. This permits hostile bounded-byte input to consume unbounded call
   stack and is rejected for the structural-parser safety lane.
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
(`docs/RECORD-SCHEMA-v0.3.md:2255-2278`). The frozen baseline reviewed for this
candidate does not define a derivation, equality, or linkage rule between that field
and `freeze_attempt_id`, the Receipt identity, a Manifest, or a Journal event.

### Competing outcomes

1. **Infer a relationship** from field names or surrounding lifecycle wording. This
   would invent semantics and is prohibited.
2. **Retain an opaque typed value** and validate only required presence, width, and
   canonical CBOR framing. This is the current fail-closed decoder boundary.
3. **Define a future explicit relationship** in a frozen successor, then add the
   corresponding contextual equality/derivation validation.

### Current bounded disposition

A binding or authority-shaped API must treat `freeze_id` as unavailable semantic
context until outcome 3 exists. It must not return authority/admission success from
this field.

## SG-003 — `FREEZE_RECEIPT` custody and durability numeric semantics

**Affected lane:** type-local semantic interpretation, Policy/durability evaluation,
and authority/admission. Raw canonical integer decoding is not semantic evaluation.

### Frozen authority and gap

`FREEZE_RECEIPT` declares `custody_mode_id`, `file_content_flush_state`,
`atomic_publish_no_replace_state`, and `parent_directory_flush_state` as `UInt`
(`docs/RECORD-SCHEMA-v0.3.md:2261-2278`). Record Schema v0.3 §14 defines `UInt`
as an unsigned integer within its applicable range but supplies no numeric registry
for those fields (`docs/RECORD-SCHEMA-v0.3.md:700-728`).

### Competing outcomes

1. **Assign numeric meanings locally** (for example, mapping a number to a
   durability state). This would invent authority-relevant semantics and is
   prohibited.
2. **Decode canonical unsigned integers as opaque values** without treating any
   number as `PERFORMED`, `NOT_PERFORMED`, `UNSUPPORTED`, custody-qualified, or
   Policy-satisfying. This is the current fail-closed decoder boundary.
3. **Define a future frozen numeric registry and applicability rules**, then evaluate
   the values only with the required Policy, profile, storage/environment, and
   workflow evidence.

### Current bounded disposition

Without outcome 3 and the separate required evidence, a contextual API must return
an explicit non-success such as `AuthorityEvidenceUnavailable`; it must never infer
Policy satisfaction, durability truth, authority, or admission from these integers.
