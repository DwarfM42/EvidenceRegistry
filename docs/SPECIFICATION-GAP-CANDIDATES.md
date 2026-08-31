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
uses exact identities where the frozen rules do define them. Exact duplicate raw
path-component sequences are rejected as a profile-independent local contradiction,
but that narrow check does not establish selected-profile ordering, equivalence, or
canonicality. Positive Freeze authority remains unavailable until both the selected
Manifest-profile semantics and SG-005's generic Policy gate-Scope relation are
frozen and implemented.

## SG-004 — Runtime Record-byte resolver contract

**Affected lane:** supplying exact authoritative Record bytes to a bounded
runtime composition adapter. This candidate does not define a persistent Record
store, namespace ownership, discovery protocol, cache, retry, retention, or
authorization behavior.

### Frozen authority and gap

- Event Record Validation requires a runtime to load a Journal Entry's exact
  `event_record_id` and verify its exact Record identity before later
  event-specific checks (`docs/RECORD-SCHEMA-v0.3.md:3597-3628`).
- A Journal Entry may remain structurally valid while its authoritative payload
  is unavailable; that absence must be reported distinctly and cannot validate
  Record-contained Journal References (`docs/IDENTITY-FORMAT-v0.3.md:344-370`).
- The Lifecycle specification sketches a `records/<record-id>.cbor` namespace,
  but says the authoritative namespace layout must be frozen before stable-format
  release and labels operational namespaces nonauthoritative
  (`docs/EVIDENCE-REGISTRY-LIFECYCLE-SPEC-v0.10.2.md:1103-1134`). It does not
  define a runtime resolver's persistence, discovery, collision, cache, retry,
  error-transport, lifetime, or authorization contract.

The frozen authorities therefore require exact identity checks for Record bytes
that a runtime obtains, but do not uniquely determine a general Record-storage or
Record-resolution mechanism.

### Competing outcomes

1. **Invent a persistent Record store contract:** select namespace ownership,
   discovery, caching, duplicate/collision handling, or authorization behavior.
   This would add unresolved runtime semantics and is rejected.
2. **Use an injected exact-byte supplier:** accept only caller-provided bytes for
   a requested exact Record ID, recompute that identity independently, and report
   missing, invalid, and identity-mismatched payloads separately. This is the
   bounded current composition seam.
3. **Freeze a resolver/store profile:** define the unresolved persistence and
   operational behavior in a future frozen successor, then implement that
   separately specified contract.

### Current bounded disposition

The current runtime may compose existing strict START and Receipt decoders through
an injected exact-byte supplier only. Missing bytes are unavailable evidence;
present malformed bytes are invalid evidence; bytes whose recomputed Record ID
differs from the requested exact ID are identity mismatches. None of those outcomes
establish authority, admission, Policy satisfaction, semantic-MANIFEST validity,
custody, durability, lifecycle truth, or external trust.

## SG-005 — Generic POLICY gate-SCOPE operation relation

**Affected lane:** determining whether a generic POLICY's required `gate_scope_ref`
contains, covers, or otherwise applies to a concrete operation, event, subject, or
object. This does not affect strict POLICY/SCOPE decoding, exact Record identity,
or structural resolution of the exact SCOPE bytes declared by POLICY key 16.

### Frozen authority and gap

- POLICY requires `gate_scope_ref` as an exact `RecordId SCOPE`
  (`docs/RECORD-SCHEMA-v0.3.md:1962-1988`), while the generic SCOPE schema
  defines only profile ID, profile version, opaque payload, and optional label
  (`docs/RECORD-SCHEMA-v0.3.md:2060-2074`). It provides no generic profile
  registry, subject/object binding grammar, or operation-to-SCOPE comparison.
- Policy applicability is defined from Policy field identity, an explicit supported
  context, the caller's current evaluation context, and where applicable consumed
  status-bearing Evidence (`docs/RECORD-SCHEMA-v0.3.md:1488-1506`). Its algorithm
  requires exact Policy loading/identity, any operation-required authority, explicit
  context support, structural obligations, and registered requirement evaluators
  (`docs/RECORD-SCHEMA-v0.3.md:2025-2056`), but does not define a universal generic
  gate-SCOPE containment predicate.
- The formal boundary makes Scope definition-relative, prohibits a universal Scope
  tuple, and requires every exact Definition to provide its own dimensions and
  coverage rules (`docs/FORMAL-VERIFICATION-IMPLEMENTATION-BOUNDARY-SPEC-v0.5.4.md:637-719`).
- One specialized Bootstrap Trust rule does require exact Scope-Record-ID equality
  and expressly prohibits subset/superset inference
  (`docs/EVIDENCE-REGISTRY-LIFECYCLE-SPEC-v0.10.2.md:4451-4485`). That rule does not
  define generic POLICY gate-SCOPE semantics for unrelated contexts or profiles.

The frozen authorities therefore do not uniquely determine how a generic
`gate_scope_ref` relates to a particular evaluated operation or subject.

### Competing outcomes

1. **Infer generic containment from profile IDs, labels, opaque payload bytes, or
   names:** this invents profile and coverage semantics and is rejected.
2. **Treat mere exact SCOPE resolution as operation coverage:** this promotes a
   resolved Record reference into subject/object or operation applicability and is
   rejected.
3. **Apply the specialized Bootstrap exact-ID equality rule universally:** this
   expands a named Bootstrap rule into unrelated POLICY contexts and is rejected.
4. **Fail closed for generic gate-SCOPE applicability:** retain exact structural
   Policy-to-SCOPE binding, but make no positive generic Policy-applicability,
   satisfaction, authority, or Admission conclusion until a frozen Scope profile and
   operation-binding rule exists.
5. **A future frozen Scope/profile successor:** define profile IDs, payload grammar,
   target bindings, and coverage/equality rules for the specific POLICY context.

### Current bounded disposition

The runtime may check only explicit caller-supplied Policy-context membership as a
separate prerequisite. A positive membership result does not resolve this gap and
must not be reported as Scope coverage, Policy applicability or satisfaction,
authority, Admission, lifecycle truth, custody, durability, or external trust.

## SG-006 — Fresh capability-observation replay authentication

**Affected lane:** authoritative replay of event 801
`ASSUMPTION_ESTABLISHMENT_RECORDED` when Record key 25
`capability_observation_provenance_ref` is absent. This does not affect strict
type-81 framing or cached observations carrying an exact type-63 provenance Record.

### Frozen authority and gap

- Record Schema v0.3 §87 makes key 25 context-required: cached capability Evidence
  requires it, while fresh noncached capability Evidence forbids it
  (`docs/RECORD-SCHEMA-v0.3.md:3274-3301`).
- The referenced type-63 Record explicitly carries `cache_reused` together with the
  storage and environment observation identities
  (`docs/RECORD-SCHEMA-v0.3.md:3016-3033`).
- Lifecycle v0.10.2 §112 defines the permitted cache scope and §112.1 bounds what
  cache reuse proves (`docs/EVIDENCE-REGISTRY-LIFECYCLE-SPEC-v0.10.2.md:3738-3789`).

The frozen replay inputs contain no independent field or retained authority that
authenticates the operation-time assertion that an omitted key 25 means a fresh,
noncached observation. Treating omission itself as proof of freshness is circular;
it verifies only the producer's encoding choice.

### Competing outcomes

1. **Infer freshness from key-25 absence:** this promotes structural omission into
   an authenticated operation-time fact and is rejected.
2. **Accept cached observations only:** require exact type-63 provenance with
   `cache_reused = true` and matching storage/environment identities; fail closed
   for the otherwise valid fresh-observation form at authoritative replay.
3. **Add an authenticated fresh-observation witness in a frozen successor:** define
   the authority, identity, and replay relation that distinguishes fresh from cached
   observation without circular inference.

### Current bounded disposition

Authoritative replay uses outcome 2. A type-81 Record with absent key 25 remains
structurally decodable but cannot enter the current authoritative replay lane.
This local fail-closed restriction is not claimed as a frozen format rule and does
not establish capability truth, continuous durability, Policy satisfaction,
Admission, or external trust.

## SG-007 — REVIEW_ADMISSION Policy outcome to terminal-event direction

**Affected lane:** authoritative recovery or publication of terminal events 302 and
303. This does not affect strict type-32 Record framing, Request/Result continuity,
or structural Journal dependency checks.

### Frozen authority and gap

- Lifecycle v0.10.2 §83 assigns accepted and rejected terminal events, but does not
  map the Record Schema Policy outcomes `SATISFIED`, `GATE_UNSATISFIED`, or
  `GATE_INDETERMINATE` to event 302 versus 303
  (`docs/EVIDENCE-REGISTRY-LIFECYCLE-SPEC-v0.10.2.md:2842-2884`).
- Record Schema v0.3 requires explicit context support, every applicable registered
  evaluator, and logical-AND composition
  (`docs/RECORD-SCHEMA-v0.3.md:1448-1468`,
  `docs/RECORD-SCHEMA-v0.3.md:2025-2056`), but its frozen evaluator registry does
  not include the prospective generic Review-Admission gate-Scope evaluator.
- SG-005 separately records why exact SCOPE resolution does not establish generic
  operation applicability.

The frozen baseline therefore does not uniquely authorize rederiving either
terminal direction from retained Request, Result, Policy, SCOPE, and event bytes.

### Current bounded disposition

Terminal Review-Admission history may pass every decidable local, identity,
dependency, chronology, and Request/Result continuity check, but authoritative
store-open still fails closed with semantic authority unavailable. It is neither
promoted to accepted/rejected authority nor reclassified as definitively invalid.
A future frozen successor must define the missing evaluator/applicability relation
and exact outcome-to-event mapping before positive terminal replay or publication.

## SG-008 — Capability-transition Record semantics

**Affected lane:** authoritative replay of event 600
`STORAGE_CAPABILITY_CHANGED`. This does not affect exact predecessor-hash continuity,
the prior/new IDs carried by the event Record, or the new IDs duplicated in the
Journal common fields.

### Frozen authority and gap

The frozen event/Record shape carries prior and new storage/environment capability
identities and material-change collections, but the baseline does not assign a
complete type/profile registry and evaluation relation that proves each listed
material transition as a PRESENT-to-ABSENT or ABSENT-to-PRESENT fact. Exact ID
continuity alone does not authenticate those capability semantics.

### Current bounded disposition

Replay first validates the exact immediate predecessor, previous-entry hash,
prior/new capability IDs, local Record grammar, and other decidable bindings. A
structurally complete event 600 then fails closed before positive authoritative
store-open. A future frozen successor must assign the referenced capability Record
types/profiles and the transition-evaluation rule before this lane can return
positive semantic authority.
