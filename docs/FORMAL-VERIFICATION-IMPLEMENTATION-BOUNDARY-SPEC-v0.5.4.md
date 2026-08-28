# EvidenceRegistry Formal Verification & Implementation Boundary Specification v0.5.4

**Status:** FINAL FREEZE CANDIDATE
**Predecessor:** EvidenceRegistry Formal Verification & Implementation Boundary Specification v0.5.3
**Project:** EvidenceRegistry
**Repository:** `DwarfM42/EvidenceRegistry`
**Normative lifecycle dependency:** `EvidenceRegistry Evidence Lifecycle Specification v0.10.2`
**Formal-model candidate:** Dafny
**Production implementation candidate:** Rust
**Normative language:** MUST / MUST NOT / SHOULD / SHOULD NOT / MAY

---

# 0. Specification Position

This specification defines the boundary among:

```text
Evidence Lifecycle semantics

Formal model

Formal proof

Named assumptions

Assumption scope

Assumption establishment

Production implementation

Deterministic verification

Computational verification

Physical-environment verification

Human judgment

Bootstrap trust
```

It does not redefine the Evidence Lifecycle.

The normative Evidence Lifecycle Specification determines:

```text
what lifecycle states exist

what Registry authority means

what Journal chronology means

which Records may acquire authority

which Journal events provide that authority

how ONE_PHASE / TWO_PHASE semantics work

how succession works

how invalidation is recorded

how Closeout support is historically bound

how Policy requirements are represented

how source observations are named and interpreted
```

This specification determines:

```text
what Dafny proves

what Dafny assumes

what Rust must implement

what environmental Evidence establishes

how formal proof applies to a real environment

how assumptions lose applicability

how assumption compatibility is handled

how proof dependencies are extracted

how human interpretation is separated from solver output

how Bootstrap Trust terminates recursive qualification
```

The central rule is:

> A formal proof establishes a property only under the exact model, exact assumptions, exact scopes, exact authority bindings, and exact verification conditions declared by that proof.

---

## 0.1 Changes from v0.5.3

v0.5.4 introduces no new Formal Verification architecture.

It stabilizes three remaining semantic details:

```text
Formal Finding Classification impact is derived from
later authoritative Evidence rather than mutable Policy

SOURCE_SUBJECT_MATCH evaluator coverage now explicitly
includes PRE / POST Closeout bracket semantics

SupportImpact aggregation is defined by an explicit
normative total order and max-reduction algebra
```

Specifically:

```text
SupportImpact order:
  UNAFFECTED
    <
  AFFECTED_UNKNOWN
    <
  DEGRADED
    <
  AFFECTED

RawSupportImpact(S)
  =
  max(SupportImpactOf(e) for e in S)

Classification impact changes only when later
authoritative Evidence affects its support basis

SOURCE_SUBJECT_MATCH bracket tests include:
  target mismatch
  Source Binding mismatch
  Freeze mismatch
  wrong Closeout
  decisive first POST failure
  later successful POST non-rehabilitation
```

No predecessor Record changes meaning under v0.5.4.

Predecessor-format Records MUST NOT silently acquire v0.5.4 semantics.

---

## 0.2 Property Numbering Namespace

Properties prefixed:

```text
FV-P
```

belong to this Formal Verification & Implementation Boundary Specification.

Properties prefixed:

```text
ER-P
```

belong to the Evidence Lifecycle Specification.

The two namespaces are independent.

Cross-specification citations MUST use qualified form:

```text
Lifecycle ER-P31

FV FV-P21
```

Bare numeric property references SHOULD NOT be used across specifications.

---

# 1. Core Architecture

Preferred architecture:

```text
Frozen Evidence Lifecycle Specification
                 │
                 ▼
           Formal Model
              Dafny
                 │
                 ▼
             Theorems
                 │
                 ▼
        Named Assumptions
                 │
        ┌────────┴────────┐
        ▼                 ▼
 Assumption          Bootstrap
 Establishment       Trust Boundary
        │                 │
        └────────┬────────┘
                 ▼
        Applicability Model
                 │
                 ▼
        Rust Implementation
                 │
       ┌─────────┼──────────┐
       ▼         ▼          ▼
Deterministic Computational Physical
Verification   Verification Verification
       │         │          │
       └─────────┼──────────┘
                 ▼
          Conformance Evidence
                 │
                 ▼
              Closeout
```

No layer MUST silently claim authority belonging to another layer.

---

# 2. Lifecycle Authority Boundary

Formal-support Records are subject to the same authority rules as every other EvidenceRegistry Record.

The following distinction is normative:

```text
portable Record identity
!=
Registry authority
```

A formal-support Record existing in:

```text
records/
```

does not acquire authority merely because its deterministic Record identity is valid.

Authority requires the exact Lifecycle Journal event assigned to that Record type.

---

# 3. Lifecycle Journal Event Bindings

The following event identities are defined normatively by Evidence Lifecycle Specification v0.10.2:

```text
800  ASSUMPTION_DEFINITION_RECORDED

801  ASSUMPTION_ESTABLISHMENT_RECORDED

802  ASSUMPTION_INVALIDATION_RECORDED

803  FORMAL_VERIFICATION_RECORDED

804  ASSUMPTION_VERSION_COMPATIBILITY_RECORDED

805  ASSUMPTION_VERSION_COMPATIBILITY_INVALIDATED

806  BOOTSTRAP_TRUST_DECLARATION_RECORDED

807  BOOTSTRAP_TRUST_INVALIDATED

808  FORMAL_FINDING_CLASSIFICATION_RECORDED
```

The Lifecycle Specification is authoritative for these numeric assignments.

This specification defines their formal-support semantics but MUST NOT redefine their lifecycle authority semantics.

---

# 4. Formal-Support Operations Are ONE_PHASE

The formal-support authority operations defined by Lifecycle events 800–808 are ONE_PHASE operations.

Therefore:

```text
Record computation
validation
formal evaluation
human classification preparation
```

MAY occur before the terminal Journal append, but no Registry authority exists until that terminal append succeeds.

If the process crashes before the terminal Journal append:

```text
no authoritative lifecycle disposition exists
```

for that attempted ONE_PHASE operation.

The operation MAY be rerun.

This is not historical erasure because no authoritative prior disposition existed.

Formal Verification MUST NOT claim that an unpublished crash event is historical Evidence.

This rule is the Formal Verification application of Lifecycle ER-P31.

---

# 5. Context-Required Field Symmetry

Formal-support Record validation inherits the Lifecycle context-required field rule.

For every optional semantic field:

```text
context requires field
    → field MUST be present

context does not permit field
    → field MUST be absent
```

Examples:

```text
same Assumption Definition
    → compatibility_authority_ref MUST be absent

cross-Definition use
    → compatibility_authority_ref MUST be present

Policy does not require Bootstrap Trust
    → required_bootstrap_scope_ref MUST be absent

Policy requires Bootstrap Trust
    → required_bootstrap_scope_ref MUST be present
```

A meaningless-but-present field is a structural contradiction.

---

# 6. Responsibility Separation

## 6.1 PURE_FORMAL Dafny responsibility

Dafny SHOULD prove lifecycle and support properties that require no external physical assumption.

Examples:

```text
AbortedFreezeCannotCommit

NoForwardAuthorityDependency

FirstEligiblePostIsDecisive

BracketFailureIsTerminal

SupportStateIsOrderIndependent

SupportInvalidationIsAbsorbing

SupportDegradationIsNonReviving

IndeterminateMayResolveUnaffected

CloseoutSupportIsNonReviving

CompatibilityInvalidationDoesNotRewriteEstablishment

BootstrapInvalidationDoesNotRewriteDeclaration

FormalClassificationDoesNotRewriteVerification

SuccessorDoesNotRewritePredecessor
```

These properties depend only on the formal state model.

---

## 6.2 ASSUMPTION_CONDITIONAL Dafny responsibility

Other theorems MAY be formally proven only under explicit external assumptions.

Examples:

```text
FreshRootUniqueness

NoReplacePublicationSafety

WriterExclusion

DurableNamespacePublication

FilesystemObjectAliasSafety

SourceObservationSoundness
```

For these properties, Dafny proves:

> If the declared assumptions hold under the declared scopes, the modeled property follows.

Dafny does NOT prove that the physical assumptions actually hold.

---

## 6.3 Rust responsibility

Rust SHOULD implement physical mechanisms including:

```text
deterministic CBOR serialization

SHA-256

Journal persistence

Record persistence

filesystem traversal

path identity

exclusive create

atomic no-replace publication

filesystem locking

filesystem object identity

Source Binding observation

capability probes

flush operations

crash recovery

CLI / machine API

projection maintenance
```

Rust MUST NOT invent lifecycle semantics not defined by the frozen Lifecycle Specification.

---

# 7. Formal Property Classes

Every qualification-bearing formal property MUST be classified as:

```text
PURE_FORMAL

ASSUMPTION_CONDITIONAL
```

This classification SHOULD be machine-readable.

---

## 7.1 PURE_FORMAL

Consumes no external physical or platform assumption.

Proof is over the normative state machine.

---

## 7.2 ASSUMPTION_CONDITIONAL

Consumes one or more explicit:

```text
Assumption Definition
+
required Scope
```

pairs.

The theorem remains mathematically valid under those assumptions even when a particular environment does not establish them.

Failure of environmental applicability is NOT theorem falsification.

---

# 8. Named Assumptions

Every external fact consumed by a formal theorem MUST have an explicit stable Assumption identity.

Candidate assumptions include:

```text
HashFunctionDeterministic

HashCollisionResistance

ExclusiveCreateAtomic

NoReplacePublicationAtomic

WriterCoordinationExclusive

FileObjectIdentitySound

FileFlushPersistence

DurableNamespacePublication

SourceReadReturnsObservedBytes

JournalStoragePreservesCommittedBytes
```

An implicit environmental assumption is forbidden for qualification-bearing proof.

---

# 9. Assumption Categories

Assumptions SHOULD be classified at least into:

```text
COORDINATION_ASSUMPTION

PERSISTENCE_ASSUMPTION

OBSERVATION_ASSUMPTION

CRYPTOGRAPHIC_ASSUMPTION

COMPUTATIONAL_ASSUMPTION
```

Examples:

```text
ExclusiveCreateAtomic
    → COORDINATION_ASSUMPTION

DurableNamespacePublication
    → PERSISTENCE_ASSUMPTION

SourceReadReturnsObservedBytes
    → OBSERVATION_ASSUMPTION

HashCollisionResistance
    → CRYPTOGRAPHIC_ASSUMPTION

Z3SoundForDeclaredFragment
    → COMPUTATIONAL_ASSUMPTION
```

Assumption category affects which Scope dimensions are meaningful.

---

# 10. Assumption Definition

An Assumption Definition MUST contain at least:

```text
assumption_symbolic_name

assumption_version

assumption_category

semantic_property

required_scope_dimensions

scope_coverage_rules_ref

model_interpretation

allowed_establishment_methods[]
```

Registry authority requires:

```text
ASSUMPTION_DEFINITION_RECORDED
```

Portable Record existence alone does not establish Registry authority.

---

# 11. Assumption Versioning

Changing the semantic meaning of an Assumption requires:

```text
new assumption version
```

or:

```text
new Assumption Definition identity
```

Historical Establishments MUST NOT be silently reinterpreted under a later Definition.

For example:

```text
ExclusiveCreateAtomic/v1
```

and:

```text
ExclusiveCreateAtomic/v2
```

are distinct Definitions.

---

# 12. Scope Is Definition-Relative

A Scope has meaning only relative to one exact Assumption Definition.

There is no universal Scope tuple that applies to every Assumption.

Each Definition declares:

```text
required dimensions

dimension semantics

dimension coverage rules
```

Observation assumptions may need dimensions different from coordination assumptions.

---

# 13. Scope Is a Partial Order

Assumption Scope MUST NOT be modeled as one scalar strength.

Conceptually:

```text
Scope =
{
  writer_coverage,
  host_coverage,
  storage_coverage,
  transport_constraint,
  validity_epoch,
  observation_constraints,
  assumption_specific_dimensions...
}
```

Only dimensions required by the exact Assumption Definition participate.

---

# 14. Scope Coverage

For one exact Assumption Definition:

```text
ScopeCovers(A, B)
```

holds only if every required dimension covers the corresponding required target dimension.

Conceptually:

```text
ScopeCovers(A, B)
iff

for each required dimension d:

  Covers_d(A[d], B[d])
```

If any required dimension is:

```text
missing

unknown

incomparable
```

then automatic coverage fails.

Result:

```text
ASSUMPTION_SCOPE_INSUFFICIENT
```

No implementation-specific guessing is permitted.

---

# 15. Writer Coverage

Candidate writer selectors:

```text
SAME_PROCESS

SAME_HOST

ALL_OBSERVED_WRITERS
```

These MUST NOT be treated as a context-free total ordering.

Only after the concrete process / host / writer universe is established may implication be evaluated.

For one concrete universe:

```text
ALL_OBSERVED_WRITERS
```

may cover:

```text
SAME_HOST
```

and:

```text
SAME_HOST
```

may cover:

```text
SAME_PROCESS
```

only where the required containment is established.

Scopes referring to different host sets may be incomparable.

---

# 16. Storage and Transport Coverage

Candidate transport classes include:

```text
LOCAL

SMB

NFS

SYNC_MANAGED

UNKNOWN
```

These have no default strong-to-weak ordering.

Coverage requires:

```text
exact match
```

or an explicit Assumption-specific implication rule.

Implementations MUST NOT infer:

```text
LOCAL > SMB > NFS
```

or another intuitive ranking.

---

# 17. Observation Assumptions

Observation assumptions MAY use dimensions such as:

```text
storage_identity

hydration_state

placeholder_state

read_API

snapshot_mechanism

source_object_identity
```

rather than writer count.

Example:

```text
SourceReadReturnsObservedBytes
```

is an OBSERVATION_ASSUMPTION.

It MUST NOT automatically inherit coordination Scope semantics.

---

# 18. Source Observation Vocabulary

The current Lifecycle verification target for point-in-time source comparison is:

```text
SOURCE_SUBJECT_MATCH
```

Formal Verification and implementation-boundary documents MUST use that vocabulary when referring to current Lifecycle source-match Evidence.

The retired historical target:

```text
SOURCE_SUBJECT_NODRIFT
```

MUST NOT be interpreted as establishing continuous immutability.

It also MUST NOT be silently treated as:

```text
SOURCE_SUBJECT_MATCH
```

for compatibility purposes.

A predecessor-format Verification Record using the retired name cannot satisfy a v0.10.2 Policy requiring `SOURCE_SUBJECT_MATCH` unless an explicit compatibility specification authorizes that mapping.

Absent such compatibility:

```text
UNSUPPORTED_SCHEMA
```

or another fail-closed result is required.

---

# 19. Observation Interval Constraints

A recorded Verification has the Lifecycle-defined Registry-history interval:

```text
[
  observation_start_journal_ref,
  VERIFICATION_RECORDED Journal Reference
]
```

This interval establishes Registry chronology only.

It does NOT establish:

```text
wall-clock recency

continuous source immutability

short verification duration
```

Where a formal or environmental support Policy requires that no relevant Registry event occur during such an interval, the requirement MUST be expressed through the Lifecycle:

```text
intervening-event constraint
```

Policy mechanism.

Formal Verification MUST NOT infer this property from Journal-entry adjacency.

Formal Verification MUST NOT infer wall-clock freshness from Journal distance.

---

# 20. Same-Definition Scope Requirement

Direct:

```text
ScopeCovers(...)
```

comparison is defined only when the Establishment and theorem requirement refer to the same exact:

```text
assumption_definition_authority_ref
```

Symbolic-name equality is insufficient.

Version-number similarity is insufficient.

---

# 21. Cross-Version Compatibility

If Assumption Definitions differ, Establishment reuse requires an exact authoritative:

```text
ASSUMPTION_VERSION_COMPATIBILITY
```

Record.

Registry authority requires:

```text
ASSUMPTION_VERSION_COMPATIBILITY_RECORDED
```

Compatibility is directional.

```text
v1 → v2
```

does not imply:

```text
v2 → v1
```

---

# 22. Compatibility Record

A Compatibility Record SHOULD contain at least:

```text
source_assumption_definition_authority_ref

target_assumption_definition_authority_ref

compatibility_direction

scope_mapping_ref

compatibility_result

limitations[]

method_ref

method_status

finding_state
```

No implicit cross-version compatibility exists.

---

# 23. Compatibility Scope Mapping

Where cross-version reuse is authorized:

```text
source Scope
↓
authoritative scope mapping
↓
target-definition Scope
↓
ScopeCovers(...)
```

Implicit:

```text
dimension dropping

dimension invention

defaulting
```

are forbidden.

Missing required target Scope information yields:

```text
ASSUMPTION_SCOPE_INSUFFICIENT
```

or:

```text
UNKNOWN
```

according to the exact evaluation rule.

---

# 24. Compatibility Binding Consistency

For an exact Assumption support binding:

```text
if establishment Definition
==
required Definition

then:
  compatibility_authority_ref MUST be absent
```

and:

```text
if establishment Definition
!=
required Definition

then:
  compatibility_authority_ref MUST be present
```

and authoritative.

A redundant Compatibility reference where no compatibility is required is structurally invalid.

---

# 25. Compatibility Is a Support Edge

Cross-version support is modeled as:

```text
Establishment
    Definition v1
        ↓
Compatibility K
        ↓
Requirement
    Definition v2
```

The Establishment itself does not become a v2 Establishment.

Compatibility authorizes the application edge.

---

# 26. Compatibility Invalidation

A Compatibility Record may later lose current applicability.

Registry authority for invalidation requires:

```text
ASSUMPTION_VERSION_COMPATIBILITY_INVALIDATED
```

The historical Compatibility Record remains unchanged.

The source Establishment also remains unchanged.

The invalidated object is:

```text
the cross-version application edge
```

---

## 26.1 Compatibility invalidation impact

If a Closeout used Compatibility K and K becomes:

```text
AFFECTED
```

then that support edge becomes affected.

The Closeout support aggregator processes the resulting impact.

A later replacement Compatibility MUST NOT revive the old Closeout.

It may support a successor Closeout.

---

# 27. Assumption Establishment

An Assumption Establishment records Evidence supporting applicability of one exact Assumption Definition under one exact Scope and environment.

Minimum fields SHOULD include:

```text
assumption_definition_authority_ref

scope

observation_start_journal_ref

storage_capability_class_id

environment_observation_id

capability_epoch_ref

capability_observation_provenance_ref
  conditionally required as defined in §27.1

establishment_method_ref

method_status

finding_state

evidence_refs[]

proof_of_scope_inputs[]

coverage_limitations[]

implementation_identity

harness_identity

host_identity

operator_identity

review_procedure_identity

shared_dependency_manifest_ref
```

Not every diversity field is mandatory for every Establishment.

A Policy that requires a field makes that field required for participating Establishments.

---

## 27.1 Capability Observation Provenance

If an Establishment relies on cached capability Evidence rather than a fresh capability probe performed for that Establishment, it MUST contain:

```text
capability_observation_provenance_ref
```

referencing an exact deterministic provenance object.

That provenance MUST bind sufficient information to identify at least:

```text
capability observation identity

whether cache reuse occurred

declared cache scope

mount identity where available

volume identity where available

resolved Registry storage identity where available

environment_observation_id

storage_capability_class_id
```

It MAY additionally bind:

```text
process-instance provenance

probe Method identity

probe tool identity

cache invalidation observations
```

The provenance object need not independently acquire Registry authority if its exact identity is committed inside the authoritative Assumption Establishment.

The authoritative Establishment thereby binds the provenance bytes.

If cached capability Evidence was used and the required provenance cannot be retained:

```text
the Establishment MUST NOT claim that capability as established
```

for qualification-bearing use.

For a fresh non-cached capability observation, `capability_observation_provenance_ref` MAY be omitted where the equivalent provenance is already fully represented by other exact Establishment fields.

---

# 28. Establishment Authority

Registry authority requires:

```text
ASSUMPTION_ESTABLISHMENT_RECORDED
```

An Establishment existing only as portable bytes has no Registry authority.

---

# 29. Establishment Capability State

For a given evaluated Scope:

```text
PRESENT

ABSENT

NOT_PROBED
```

retain the Lifecycle meanings.

A narrower PRESENT result MUST NOT establish a broader Scope.

---

# 30. Establishment Reuse

An Establishment may support a new application only if all relevant conditions remain satisfied.

At minimum:

```text
Definition is exact or authoritative Compatibility exists

Scope covers theorem requirement

Method remains acceptable

Finding supports the required property

capability epoch remains compatible

Establishment has not expired for reuse

no applicable invalidation removes support
```

---

# 31. Establishment Applicability Epoch

An Establishment becomes reusable from:

```text
ASSUMPTION_ESTABLISHMENT_RECORDED
```

onward.

It is not universally reusable forever.

---

# 32. Capability Epoch

Storage-sensitive Establishments SHOULD bind:

```text
capability_epoch_ref
```

to the exact authoritative Journal Reference representing the applicable storage-capability epoch.

For Lifecycle v0.10.2 Registries, the first epoch MAY be the:

```text
GENESIS Journal Reference
```

because Lifecycle v0.10.2 requires:

```text
GENESIS Record:
  initial storage_capability_class_id
  initial environment_observation_id

GENESIS Journal Entry:
  storage_capability_class_id
  environment_observation_id
```

and requires both copies to refer to the same observation.

A mismatch MUST produce:

```text
INVALID_RECORD_FRAMING
```

as specified by Lifecycle §11.1.

Therefore:

```text
capability_epoch_ref = GENESIS Journal Reference
```

is valid before the first material:

```text
STORAGE_CAPABILITY_CHANGED
```

event.

Predecessor Lifecycle formats MUST NOT be assumed to provide this binding unless explicit compatibility establishes it.

---

# 33. Establishment Expiration

If a later incompatible capability epoch applies:

```text
ESTABLISHMENT_EXPIRED_FOR_REUSE
```

unless Policy explicitly establishes compatibility.

Expiration means:

> This historical observation is insufficient for a new use.

It does NOT mean:

> The historical Establishment was false when created.

---

# 34. Expiration Is Not Invalidation

There is no separate normative lifecycle or impact state named:

```text
ESTABLISHMENT_INVALIDATED
```

in this specification.

Instead, distinguish:

```text
ESTABLISHMENT_EXPIRED_FOR_REUSE
```

from the support impact:

```text
AFFECTED
```

derived when authoritative Assumption Invalidation Evidence establishes that an Establishment's support basis is affected.

Expiration prevents future reuse.

`AFFECTED` contributes to current Closeout support invalidation.

Ordinary OS or environment evolution MUST NOT automatically make a historical Establishment `AFFECTED`.

Ordinary expiration therefore MUST NOT automatically invalidate historical Closeouts.

---

# 35. No Implicit Wall-Clock Expiration

Without a trusted time profile:

```text
older than 30 days
```

is not mechanically enforceable.

Permitted freshness mechanisms include:

```text
Journal distance

capability epoch

environment identity

explicit future trusted-time profile
```

Journal distance MUST NOT be described as wall-clock freshness.

---

# 36. Probe Cache and Establishment Semantics

Lifecycle v0.10.2 permits capability-probe results to be reused only within their declared cache scope, such as:

```text
same process

same mount / volume identity

same relevant Registry storage context
```

A cached probe result means only:

> A capability observation considered valid under the declared cache scope was reused.

Formal Verification MUST NOT interpret cached reuse as:

```text
the capability was continuously re-proven

the capability was freshly probed before every Journal append

the physical environment could not have changed between uses
```

If a Policy requires stronger observation freshness, that requirement MUST be explicit.

An Assumption Establishment relying on cached capability Evidence MUST preserve the provenance required by §27.1.

The provenance requirement is therefore a schema-level obligation, not merely documentation guidance.

---

# 37. Assumption Invalidation

Later Evidence may invalidate Assumption Establishment support.

Registry authority requires:

```text
ASSUMPTION_INVALIDATION_RECORDED
```

Invalidation MUST NOT mutate Establishment bytes.

---

# 38. Assumption Invalidation Target Modes

Supported target modes:

```text
EXPLICIT_ESTABLISHMENT_SET

ATTRIBUTE_PREDICATE
```

---

## 38.1 Explicit set

Exact authoritative Establishment references are listed.

---

## 38.2 Attribute predicate

A versioned mechanically evaluable predicate selects potentially affected Establishments.

Free-form prose MUST NOT determine mechanical affectedness.

---

# 39. Invalidation Predicate Horizon

A predicate invalidation applies only to Establishments authoritative at or before that invalidation's Journal position.

It MUST NOT silently govern future Establishments.

Later affected Establishments require:

```text
successor invalidation
```

or another authoritative invalidation.

---

# 40. Invalidation Metadata Stability

If an invalidation predicate references metadata absent from an older Establishment schema:

```text
AFFECTED_UNKNOWN
```

MUST be produced.

Missing metadata MUST NOT silently become:

```text
UNAFFECTED
```

---

# 41. Invalidation Coverage

For each Establishment evaluated against an invalidation:

```text
AFFECTED

UNAFFECTED

AFFECTED_UNKNOWN
```

are distinct.

An invalidation Record MUST expose:

```text
selector identity

evaluated coverage

coverage limitations

unknown applicability
```

Negative conclusions remain coverage-bound.

---

# 42. Formal Verification Subject

Formal proof artifacts MAY themselves be EvidenceRegistry Subjects.

Candidate contents:

```text
Dafny source

formal model

theorem declarations

Assumption Definitions

verification configuration

dependency manifest

toolchain lock

solver configuration

verification report
```

These may be:

```text
freeze
↓
verify
↓
review
↓
admit
↓
closeout
```

like other Subjects.

---

# 43. Formal Proof Subject Directory Limitation

Lifecycle v0.10.2 does not include directories as Manifest Artifacts.

Therefore, when a Formal Verification Subject is frozen:

```text
empty directories
```

do not participate in Subject content identity.

If empty-directory existence is semantically material to a formal build or proof harness, the Subject MUST:

```text
include a sentinel Artifact
```

or use a future Lifecycle Artifact Kind that explicitly models directories.

Formal qualification MUST NOT silently assume that empty directory structure was bound by the Manifest when it was not.

---

# 44. Formal Verification Record

A Formal Verification Record MUST bind at least:

```text
formal_subject_id

formal_model_id

theorem_id

proof_dependency_manifest_id

Dafny identity

Dafny version

Dafny configuration

SMT solver identity

solver version

solver arguments

random seed where material

thread configuration

timeout configuration

resource limits

memory limit where configured

runtime architecture

OS provenance

environment_observation_id

declared Assumption requirements

verification outcome
```

Registry authority requires:

```text
FORMAL_VERIFICATION_RECORDED
```

---

# 45. Formal Verification Outcome

Normative outcome classes SHOULD include:

```text
VERIFIED

COUNTEREXAMPLE

TIMEOUT

RESOURCE_EXHAUSTED

TOOL_ERROR

INVALID_CONFIGURATION

UNSUPPORTED
```

---

## 45.1 VERIFIED

Typical mapping:

```text
method_status = VALID

finding_state = NO_BLOCKING
```

within the declared theorem-verification Scope.

---

## 45.2 COUNTEREXAMPLE

A valid solver counterexample typically maps to:

```text
method_status = VALID

finding_state = BLOCKING
```

for the formal property itself.

It does NOT automatically mean the product implementation is defective.

---

## 45.3 TIMEOUT

Typical mapping:

```text
method_status = ENVIRONMENTALLY_DEGRADED

finding_state = INDETERMINATE
```

A timeout is not proof that the theorem is false.

---

## 45.4 RESOURCE_EXHAUSTED

Typical mapping:

```text
method_status = ENVIRONMENTALLY_DEGRADED

finding_state = INDETERMINATE
```

---

## 45.5 TOOL_ERROR / INVALID_CONFIGURATION

May map to:

```text
method_status = INVALID
```

according to the Method definition.

---

# 46. Formal Verification Is Computationally Environment-Dependent

Verification engineering is divided into:

```text
DETERMINISTIC_VERIFICATION

COMPUTATIONAL_ENVIRONMENT_VERIFICATION

PHYSICAL_ENVIRONMENT_VERIFICATION
```

Dafny / SMT verification belongs to:

```text
COMPUTATIONAL_ENVIRONMENT_VERIFICATION
```

because outcome may depend on:

* solver version;
* configuration;
* timeout;
* memory;
* thread configuration;
* resource constraints.

---

# 47. Deterministic Verification

Examples:

```text
CBOR vectors

Record identity vectors

Manifest vectors

Journal framing vectors

finite state transition tests

malformed input vectors

pure serialization properties
```

These SHOULD reproduce without special physical environments.

---

# 48. Computational Environment Verification

Examples:

```text
Dafny theorem verification

SMT solving

large property campaigns

bounded model checking
```

These require exact computational environment provenance.

---

# 49. Physical Environment Verification

Examples:

```text
filesystem concurrency

NFS behavior

SMB behavior

hardlinks

reparse points

sync clients

crash recovery

fault injection

durability

power-loss behavior
```

Each physical Verification MUST state its required environment.

---

# 50. Verification Cost

Verification class and cost MUST remain distinct.

Recommended cost bands:

```text
LOW

MODERATE

HIGH

SPECIALIZED
```

Examples:

```text
CBOR vector
  → DETERMINISTIC / LOW

Dafny theorem suite
  → COMPUTATIONAL / MODERATE

filesystem hostile matrix
  → PHYSICAL / HIGH

power-loss harness
  → PHYSICAL / SPECIALIZED
```

Cost is not assurance strength.

---

# 51. Formal Counterexample Interpretation

A formal counterexample establishes:

> The declared theorem does not hold under the exact formal model and assumptions used.

It does NOT by itself establish:

> The production implementation is defective.

Possible later classifications include:

```text
IMPLEMENTATION_PROPERTY_VIOLATION

FORMAL_MODEL_DEFECT

PROPERTY_UNDERSPECIFIED

ASSUMPTION_MODELING_DEFECT

UNRESOLVED
```

These are human semantic classifications.

---

# 52. Formal Finding Classification

Human interpretation MUST be represented separately from raw Formal Verification.

Registry authority requires:

```text
FORMAL_FINDING_CLASSIFICATION_RECORDED
```

Minimum semantics include:

```text
formal_verification_authority_ref

classification

classifier_procedure_identity

classifier_role

judgment_basis = HUMAN_JUDGMENT

rationale_ref

supporting_evidence_refs[]

known_uncertainties[]
```

---

# 53. Classification Is Human Judgment

An authoritative Classification establishes:

> This exact judgment was authoritatively recorded under the declared procedure.

It does NOT establish:

> The judgment is mathematically proven correct.

This distinction is normative.

---

# 54. Raw Result and Classification History

Example:

```text
Formal Verification:
  COUNTEREXAMPLE

later Classification:
  FORMAL_MODEL_DEFECT
```

Correct history is:

```text
counterexample occurred
+
later classified as model defect
```

The raw result MUST NOT be rewritten away.

---

# 55. Policy Requirement for Formal Finding Classification

Lifecycle v0.10.2 defines the Policy field:

```text
required_formal_finding_classification
```

Where the field is present with:

```text
required: true
```

the applicable Formal Verification support MUST include an authoritative Formal Finding Classification satisfying:

```text
acceptable_classifications

classifier_procedure_identity_allowed
```

or other mechanically specified Policy conditions.

If the required Classification is absent or unacceptable:

```text
INDETERMINATE
```

and:

```text
Policy gate not satisfied
```

are required.

The Formal Finding Classification requirement is additive to all other applicable Lifecycle Policy requirements.

In particular:

```text
required_method_status
allowed finding state
required_formal_finding_classification
```

MUST each be evaluated independently.

Satisfying an acceptable Formal Finding Classification MUST NOT override failure of:

```text
required_method_status
```

or:

```text
allowed finding state
```

For example:

```text
TIMEOUT
→ method_status = ENVIRONMENTALLY_DEGRADED
→ finding_state = INDETERMINATE
```

does not satisfy a default Lifecycle Policy requiring:

```text
required_method_status:
  - VALID
```

even if a later Formal Finding Classification exists.

No single satisfied Policy dimension compensates for another unsatisfied required dimension.

---

# 56. Default Unclassified Formal Finding Rule

A raw formal counterexample MUST NOT automatically become:

```text
IMPLEMENTATION_PROPERTY_VIOLATION
```

The default is fail-closed.

Even where Policy does NOT explicitly contain:

```text
required_formal_finding_classification
```

EvidenceRegistry MUST NOT derive a product-level defect conclusion from an unclassified raw formal result.

A Policy MAY define a mechanically unambiguous raw-outcome-to-product mapping, but only through an explicit frozen Policy rule.

Thus:

```text
raw formal BLOCKING
!=
automatic product defect
```

unless exact Policy semantics establish that mapping.

---

# 57. Proof Dependency Manifest

Every theorem consuming external Assumptions MUST expose its exact dependency set.

A manually maintained prose list is not sufficient authoritative Evidence.

---

# 58. Constrained Assumption Consumption

External Assumption use in qualification-bearing model code MUST occur through one machine-recognizable mechanism.

Conceptually:

```text
RequiresAssumption(
  assumption_definition_id,
  required_scope
)
```

Equivalent implementation syntax MAY differ.

Free-form comments do not create dependencies.

Raw untracked:

```text
assume
```

statements MUST NOT bypass the dependency mechanism.

---

# 59. Proof Dependency Extraction

The build MUST produce deterministic:

```text
PROOF_DEPENDENCY_MANIFEST
```

from the parsed or otherwise machine-recognized formal representation.

The Formal Verification Record MUST bind that Manifest.

---

# 60. Dependency Mismatch

If machine-recognized Assumption consumption disagrees with extracted dependencies:

```text
FORMAL_DEPENDENCY_MISMATCH
```

MUST be produced.

The Formal Verification MUST NOT support downstream applicability.

---

# 61. Dependency Count Reconciliation

Where possible, the system MUST reconcile:

```text
recognized Assumption-consumption occurrence count
```

against:

```text
extracted dependency occurrence count
```

under a frozen normalization rule.

Unexplained deficit is a qualification defect.

---

# 62. Proof Dependency Extractor Is a Subject

The extractor itself MUST be treated as a qualification-critical Subject.

Candidate contents:

```text
extractor source

extractor binary

Dafny integration logic

AST / parser integration

dependency schema

normalization rules

test corpus

toolchain identity
```

The extractor MUST NOT qualify itself merely by producing internally consistent output.

---

# 63. Dependency Extractor Test Corpus

Minimum controls:

```text
zero assumptions

one assumption

multiple assumptions

duplicate assumption use

different scopes

nested helper dependency

cross-module dependency

trait/module boundary

re-exported dependency

import through another verified module

dead/unreachable dependency declaration

malformed dependency declaration

forbidden raw assume
```

---

# 64. Extractor Negative Controls

Qualification MUST include intentionally difficult-to-detect dependencies.

Examples:

```text
dependency only in imported module

dependency only through generic abstraction

dependency only through nested helper

dependency behind alternative verified path
```

The extractor MUST demonstrate detection.

---

# 65. Assumption Producer Qualification

The Verification Matrix MUST include both:

```text
Assumption consumers
```

and:

```text
Assumption producers
```

A theorem that consumes:

```text
ExclusiveCreateAtomic(scope)
```

is not fully documented unless the mechanism establishing that Assumption is also identified.

---

# 66. Capability Probes Are Subjects

Qualification-critical capability probes SHOULD themselves be EvidenceRegistry Subjects.

Examples:

```text
ExclusiveCreate probe

NoReplacePublication probe

WriterLock probe

FileObjectIdentity probe

Durability probe
```

---

# 67. Probe Qualification

Probe qualification SHOULD bind:

```text
probe source identity

probe binary identity

Method definition

positive controls

negative controls

fault-injection controls

real hostile environment tests

known blind spots

false-positive criteria

false-negative criteria
```

---

# 68. Probe Negative Controls

Where practical, a probe that distinguishes PRESENT from ABSENT SHOULD be tested against a known-failing environment or synthetic failure mechanism.

Examples:

```text
non-atomic filesystem shim

lock-ignoring harness

forced rename race

aliasing fixture

fault-injected flush path
```

Real NFS / SMB / sync-client tests SHOULD supplement synthetic controls where relevant.

---

# 69. Probe Invalidation Propagation

An Establishment MUST bind the exact:

```text
probe / Method identity
```

used.

If the Method later becomes invalid:

```text
affected Establishments
↓
Assumption invalidation evaluation
↓
dependent applicability
↓
Closeout support
```

No silent Method substitution is permitted.

---

# 70. Diversity Metadata

Assumption Establishments MAY carry mechanically evaluated diversity identities:

```text
implementation_identity

harness_identity

host_identity

operator_identity

review_procedure_identity

storage_identity
```

Distinctness of these identities does not automatically prove causal or organizational independence.

---

# 71. Implementation Identity

`implementation_identity` SHOULD bind the exact implementation Subject used.

Preferred identity:

```text
implementation Subject manifest_id
```

or exact Freeze authority.

Two builds with different identities are merely distinct builds.

They are not automatically independent implementations.

---

# 72. Shared Dependency Manifest

Establishments using implementation diversity SHOULD bind:

```text
shared_dependency_manifest_ref
```

The Record describes dependencies relevant to independence analysis.

Candidate fields:

```text
dependency_identity

dependency_class

source / binary identity

declared common infrastructure
```

---

# 73. Dependency Overlap Policy

Policy MAY require constraints such as:

```yaml
required_establishment_diversity:
  - dimension: implementation_identity
    distinct_count: 2

    dependency_overlap:
      mode: NONE_EXCEPT_ALLOWED

      allowed_shared_dependency_classes:
        - operating_system
        - language_runtime
```

This proves compliance with the declared overlap rule only.

It MUST NOT be described as universal implementation independence.

---

# 74. Shared Dependency Exceptions Are Human Judgment

Decisions such as:

```text
operating system sharing is acceptable

language runtime sharing is acceptable
```

are Policy judgments.

Where material to bootstrap qualification, rationale SHOULD be represented through:

```text
Bootstrap Trust Declaration
```

or an exact governing human-judgment procedure.

---

# 75. Harness Identity

`harness_identity` SHOULD bind the exact harness Subject.

Where material:

```text
source

configuration

fault-injection logic

fixtures

runner
```

belong to the harness identity.

Two runs of the same exact harness are not two harness identities.

---

# 76. Host Identity

`host_identity` is a procedural opaque identity.

It MUST NOT automatically mean:

```text
unique physical machine

cryptographically attested host

unique motherboard

independent administrator
```

Profile L SHOULD use a locally generated opaque stable identity with documented:

```text
generation procedure

stability scope

cloning limitations
```

VM clones may duplicate host identity.

Policy MUST NOT overclaim beyond the procedure.

---

# 77. Operator Identity

`operator_identity` SHOULD be Registry-local and opaque.

It SHOULD NOT expose:

```text
personal name

email address

employee number

external account ID
```

unless external Policy explicitly requires such disclosure.

It SHOULD be random, stable within declared Scope, and non-meaningful externally.

Distinct operator identities do not prove distinct human beings.

---

# 78. Review Procedure Identity

`review_procedure_identity` SHOULD be a content-addressed identity over one exact versioned procedure.

Examples:

```text
formal-counterexample-triage-v1

filesystem-specialist-review-v2

independent-adoption-review-v1
```

Human-readable labels are metadata.

---

# 79. Storage Identity

`storage_identity` MUST use platform-observed storage identity.

It MUST NOT be inferred from pathname alone.

---

# 80. Diversity Policy

Lifecycle Policy MAY mechanically require diversity through:

```text
required_establishment_diversity
```

Example:

```yaml
required_establishment_diversity:
  - dimension: implementation_identity
    distinct_count: 2

  - dimension: host_identity
    distinct_count: 2
```

Only exact registered identity dimensions may be used.

Missing values do not count as distinct.

---

# 81. Diversity Claim Discipline

EvidenceRegistry MAY claim:

```text
two distinct implementation identities

two distinct harness identities

two distinct opaque operator identities

declared dependency overlap requirement satisfied
```

It MUST NOT silently claim:

```text
two independent organizations

two independent humans

two independently developed implementations
```

unless stronger Evidence establishes that claim.

---

# 82. Environmental Bootstrap Diversity

Environment-dependent bootstrap SHOULD use diverse observations where materially required.

Possible Policy requirement:

```text
candidate probe

+
separate harness

+
negative control

+
second host

+
specialist Review
```

No universal fixed diversity combination is required.

The exact combination belongs to Policy.

---

# 83. Diversity Is Machine-Enforced When Claimed

A bootstrap procedure MUST NOT claim:

```text
two independent implementations were required
```

unless the applicable Policy contains a mechanically evaluable diversity requirement corresponding to that claim.

---

# 84. Environment Identity for Applicability

The conceptual environment argument is explicit.

Define:

```text
FormalEnvironment =
{
  storage_capability_class_id,

  environment_observation_id,

  other Assumption-specific environmental identities
}
```

Then conceptually:

```text
Applicable(
  Theorem T,
  FormalEnvironment E,
  JournalState J
)
```

is evaluated against explicit environment identity.

No unnamed ambient machine state is permitted.

---

# 85. Theorem Applicability

A theorem is applicable only if every external dependency has appropriate support.

For each Assumption dependency:

```text
authoritative Establishment exists

Definition matches
or valid authoritative Compatibility exists

Scope covers theorem requirement

Method is acceptable

Finding establishes required property

Establishment is not expired for reuse

no applicable invalidation removes support

capability epoch remains applicable
```

A theorem may be applicable in one environment and not another.

Its historical Formal Verification Record remains unchanged.

---

# 86. Journal Replay Is Not Enough for Applicability

Formal applicability MUST NOT be derived from Journal metadata alone.

Lifecycle v0.10.2 explicitly states that Journal-only replay does not reconstruct:

```text
Policy satisfaction

Review Scope contents

Review Method semantics

Verification target semantics

Method status

Finding state

Source Binding contents

Assumption semantics

Assumption Version Compatibility contents

Invalidation predicate contents

Bootstrap Trust contents

Formal Finding Classification contents

formal theorem contents
```

Formal applicability does not necessarily consume every item in that list in every workflow.

The list is intentionally synchronized with Lifecycle §22 to define what Journal replay alone cannot establish.

In particular, `Source Binding contents` are required wherever:

```text
SOURCE_SUBJECT_MATCH
```

Evidence participates in applicability or qualification.

Therefore applicability evaluation MUST load and validate every semantic Record payload required by the exact Policy and support graph.

If a required payload is unavailable:

```text
UNKNOWN
MISSING_EVIDENCE
```

or another fail-closed result is required.

Journal metadata alone MUST NOT satisfy a formal-support Policy gate.

---

# 87. Applicability Is Derived State

Theorem applicability is normally derived from authoritative:

```text
Formal Verification

Assumption Definitions

Establishments

Compatibility Records

Invalidations

environment state
```

Applicability does not automatically create its own immutable Record.

---

# 88. Closeout Support Snapshot

A Closeout's exact support dependencies form an immutable snapshot of the support basis accepted at creation.

The support snapshot SHOULD include:

```text
formal_verification_authority_refs[]

assumption_bindings[]

bootstrap_trust_declaration_authority_refs[]
```

and, where Policy requires human classification of a formal result:

```text
formal_finding_classification_authority_refs[]
```

Each Assumption binding contains:

```text
required_assumption_definition_authority_ref

establishment_authority_ref

compatibility_authority_ref
  iff cross-Definition reuse was required
```

---

# 89. Bootstrap Scope Requirement Binding

Where Lifecycle Policy requires Bootstrap Trust, the Policy contains:

```text
required_bootstrap_scope_ref
```

and the Closeout records the same exact value in its historical support snapshot.

The Formal Verification evaluator MUST verify:

```text
Policy.required_bootstrap_scope_ref
==
Closeout.required_bootstrap_scope_ref
==
BootstrapTrustDeclaration.declaration_scope_ref
```

v0.x uses exact identity equality.

No semantic subset/superset inference is permitted.

If Lifecycle Policy does not require Bootstrap Trust:

```text
Closeout.required_bootstrap_scope_ref
```

MUST be absent.

This is a direct application of Lifecycle field symmetry.

---

# 90. Historical Versus Current Support

The Closeout support snapshot is historical.

Current support is derived.

Thus:

```text
historical support graph
```

does not change when later Evidence appears.

But:

```text
what the graph supports now
```

may change.

---

# 91. Support Elements

A Closeout support graph MAY contain:

```text
Assumption Establishments

Compatibility edges

Bootstrap Trust Declarations

Formal Verification Records

Formal Finding Classifications
  where required by Policy

other explicitly required support Records
```

Every support-element class participating in aggregation MUST have a defined current-impact derivation.

A support-element type lacking such a derivation MUST NOT be silently treated as `UNAFFECTED`.

Instead:

```text
AFFECTED_UNKNOWN
```

or another fail-closed result is required until its impact semantics are defined.

---

# 92. Support Element Impact

For every exact support element:

```text
UNAFFECTED

AFFECTED_UNKNOWN

DEGRADED

AFFECTED
```

are distinct.

The current impact of a support element is derived from authoritative Evidence according to that support-element type.

The support aggregator MUST NOT invent impact values.

It consumes impacts produced by the type-specific derivation rules below.

---

## 92.1 Normative SupportImpact Order

For support aggregation purposes, `SupportImpact` has the following normative total order:

```text
UNAFFECTED
    <
AFFECTED_UNKNOWN
    <
DEGRADED
    <
AFFECTED
```

Equivalently:

```text
rank(UNAFFECTED)        = 0
rank(AFFECTED_UNKNOWN)  = 1
rank(DEGRADED)          = 2
rank(AFFECTED)          = 3
```

This order expresses effect on qualification support.

It MUST NOT be interpreted as:

```text
probability

severity of product defect

moral importance

certainty of the underlying finding
```

It exists only to define deterministic support aggregation.

---

## 92.2 Assumption Establishment Impact

An Assumption Establishment's current impact is derived from applicable Assumption Invalidation Evidence.

Conceptually:

```text
no applicable invalidation establishes impact
    → UNAFFECTED

applicable selector result = AFFECTED_UNKNOWN
    → AFFECTED_UNKNOWN

applicable selector result = AFFECTED
    → AFFECTED
```

A conclusive, explicitly defined weaker-support condition MAY produce:

```text
DEGRADED
```

where the governing Method or formal-support semantics define such a state.

`ESTABLISHMENT_EXPIRED_FOR_REUSE` is NOT a historical Closeout impact.

Expiration affects future reuse only.

---

## 92.3 Compatibility Edge Impact

A Compatibility edge's current impact is derived from authoritative Compatibility Invalidation Evidence.

Conceptually:

```text
no applicable Compatibility invalidation
    → UNAFFECTED

invalidation applicability unknown
    → AFFECTED_UNKNOWN

Compatibility invalidated for this support path
    → AFFECTED
```

A conclusive partial-loss condition MAY yield:

```text
DEGRADED
```

only if exact Compatibility semantics explicitly define such a result.

---

## 92.4 Bootstrap Trust Declaration Impact

A Bootstrap Trust Declaration's current impact is derived from authoritative Bootstrap Trust Invalidation Evidence.

Conceptually:

```text
no applicable Bootstrap invalidation
    → UNAFFECTED

invalidation applicability unknown
    → AFFECTED_UNKNOWN

trust basis conclusively weakened
    → DEGRADED

required trust basis invalidated
    → AFFECTED
```

A successor Declaration by itself does NOT affect the predecessor.

Succession is not revocation.

---

## 92.5 Formal Verification Record Impact

A Formal Verification Record is an immutable historical verification event.

v0.x defines no standalone:

```text
FORMAL_VERIFICATION_INVALIDATED
```

lifecycle event.

Therefore the Formal Verification Record itself is normally:

```text
UNAFFECTED
```

provided:

```text
its authoritative Record payload remains valid

its required Method / Finding conditions were satisfied
at the Closeout's creation-time support evaluation

any Policy-required Formal Finding Classification was
present and acceptable
```

Later problems in the surrounding support basis MUST be represented through the corresponding explicit support element rather than by rewriting the Formal Verification Record.

Examples:

```text
Assumption later invalidated
    → Establishment impact changes

cross-version mapping later invalidated
    → Compatibility impact changes

solver/verifier trust later challenged
    → applicable Bootstrap Trust impact changes

counterexample later judged a model defect
    → Formal Finding Classification governs the
      higher-level interpretation
```

If the Formal Verification Record payload itself becomes unavailable or cannot be authoritatively validated:

```text
AFFECTED_UNKNOWN
```

or:

```text
MISSING_EVIDENCE
```

is required.

The Formal Verification Record MUST NOT silently default to UNAFFECTED when its required payload cannot be inspected.

---

## 92.6 Formal Finding Classification Impact

Where Policy requires a Formal Finding Classification, that exact Classification becomes an explicit support element.

Its current impact is derived from authoritative Evidence, not from mutation or reinterpretation of Policy.

Policy remains immutable and defines only:

```text
whether Classification is required

which Classification values are acceptable

which classifier procedures are acceptable

other creation-time qualification requirements
```

Later changes to Classification support MUST arise from later authoritative Evidence.

### 92.6.1 UNAFFECTED

A Formal Finding Classification is:

```text
UNAFFECTED
```

when:

```text
its authoritative Record remains valid

its payload is available

its classifier_procedure_identity satisfies
the exact Policy bound to the Closeout

its Classification value satisfies the exact
Policy requirement

and

no later authoritative Evidence establishes
loss of the trust basis materially supporting
that Classification
```

### 92.6.2 AFFECTED_UNKNOWN

A Formal Finding Classification becomes:

```text
AFFECTED_UNKNOWN
```

when later authoritative Evidence makes its support materially uncertain but insufficient to establish definite loss.

Examples include:

```text
required Classification payload becomes unavailable

supporting Evidence needed to interpret the Classification
is missing

applicable Bootstrap Trust invalidation has
AFFECTED_UNKNOWN impact on a human-judgment point
required by that Classification

the authoritative status of a materially required
classification-support Record cannot be established
```

Missing information MUST NOT silently produce `UNAFFECTED`.

### 92.6.3 DEGRADED

A Formal Finding Classification may become:

```text
DEGRADED
```

only when later authoritative Evidence conclusively establishes that the Classification's support basis remains meaningful but no longer provides the full support originally required.

This result MUST be traceable to exact later authoritative Evidence.

Policy itself MUST NOT change a Classification from `UNAFFECTED` to `DEGRADED`.

### 92.6.4 AFFECTED

A Formal Finding Classification becomes:

```text
AFFECTED
```

when later authoritative Evidence conclusively invalidates a materially required support basis for that exact Classification as used by the Closeout.

Example:

```text
Closeout C
  ↓ support
Formal Finding Classification F
  ↓ relies on accepted human-judgment trust point
Bootstrap Trust Declaration B

later:
B → AFFECTED
```

where that trust point was materially required for F's accepted use.

In that case F may derive:

```text
AFFECTED
```

for support aggregation.

The exact propagation conditions MUST be mechanically traceable through the historical support graph.

### 92.6.5 Policy Immutability

The following is forbidden:

```text
Policy P unchanged
+
no new authoritative Evidence
+
Classification impact changes
```

A content-addressed immutable Policy cannot itself later decide that an already-bound Classification has lost support.

Current impact changes only because authoritative Evidence grows.

### 92.6.6 Successor Classification

A later successor Classification does not silently replace the Classification historically bound to an existing Closeout.

Where a different Classification is needed for a new qualification result:

```text
successor Closeout
```

or another explicit successor qualification path is required.

---

## 92.7 Other Support Element Types

A future support-element type MUST define its impact derivation before it may participate in:

```text
RawSupportImpact
```

aggregation.

Undefined impact semantics are fail-closed.

---

## 92.8 Impact Evaluation Actor

Current support impact is a derived-state computation performed by the EvidenceRegistry formal-support evaluator over:

```text
authoritative Journal history

required authoritative Record payloads

applicable invalidation Records

exact Closeout support snapshot

exact Policy
```

The evaluator MUST NOT infer an impact solely from filenames, timestamps, prose, or projection state.

A projection MAY cache the result, but authority remains with Journal + Records.

---

# 93. Raw Support Aggregation

For a complete support-element set `S`, define:

```text
RawSupportImpact(S)
=
max(
  SupportImpactOf(e)
  for every e in S
)
```

under the normative order:

```text
UNAFFECTED
<
AFFECTED_UNKNOWN
<
DEGRADED
<
AFFECTED
```

For an empty support set where Policy permits such a set:

```text
RawSupportImpact(∅)
=
UNAFFECTED
```

unless the applicable Policy or support model requires at least one support element.

A missing required support element fails separately and MUST NOT be converted into a successful empty-set aggregation.

Map aggregate impact to support state:

```text
RawSupportImpact = UNAFFECTED
    → SUPPORT_ACTIVE

RawSupportImpact = AFFECTED_UNKNOWN
    → SUPPORT_INDETERMINATE

RawSupportImpact = DEGRADED
    → SUPPORT_DEGRADED

RawSupportImpact = AFFECTED
    → SUPPORT_INVALIDATED
```

This definition is normative.

---

# 94. Support States

Use:

```text
SUPPORT_ACTIVE

SUPPORT_INDETERMINATE

SUPPORT_DEGRADED

SUPPORT_INVALIDATED

REQUALIFICATION_REQUIRED
```

---

# 95. SUPPORT_INDETERMINATE

Means:

> Available Evidence cannot yet establish whether the original support remains unaffected.

It is provisional.

It MAY later resolve to:

```text
SUPPORT_ACTIVE
```

if the uncertainty is resolved as UNAFFECTED.

Or:

```text
SUPPORT_INVALIDATED
```

if affectedness is established.

This is clarification, not revival.

---

# 96. SUPPORT_DEGRADED

Means:

> Later Evidence conclusively establishes that support is weaker than originally required.

It is non-reviving for one exact Closeout.

A successor Closeout is required for restored full qualification.

---

# 97. SUPPORT_INVALIDATED

Means:

> Later authoritative Evidence invalidates a required support basis.

It is terminal for the exact Closeout.

A later favorable Establishment, Compatibility, Classification, or Bootstrap Trust Declaration MUST NOT restore the old Closeout.

---

# 98. REQUALIFICATION_REQUIRED

A Policy MAY derive:

```text
REQUALIFICATION_REQUIRED
```

from DEGRADED or INVALIDATED support.

For one exact Closeout, a state requiring requalification MUST NOT return to ACTIVE by later substitution of support.

A successor Closeout is required.

---

# 99. Terminal Support Floor

To preserve non-revival across later Evidence, define conceptually:

```text
TerminalSupportFloor(C, J)
```

for Closeout `C` at Journal state `J`.

Priority:

```text
SUPPORT_INVALIDATED
    >
SUPPORT_DEGRADED
```

Final support:

```text
if historical terminal floor = INVALIDATED
    → INVALIDATED

else if historical terminal floor = DEGRADED
    → DEGRADED

else
    → current RawSupportState
```

---

# 100. SupportImpact Algebra

Under the normative total order, define binary combination:

```text
CombineImpact(a, b)
=
max(a, b)
```

The formal model SHOULD prove:

```text
commutativity:
  CombineImpact(a, b)
  ==
  CombineImpact(b, a)

associativity:
  CombineImpact(
    CombineImpact(a, b),
    c
  )
  ==
  CombineImpact(
    a,
    CombineImpact(b, c)
  )

idempotence:
  CombineImpact(a, a)
  ==
  a

identity:
  CombineImpact(
    UNAFFECTED,
    a
  )
  ==
  a
```

These properties make aggregation independent of support-element enumeration order.

---

# 101. Support Order Independence

For one exact authoritative Evidence set:

```text
SupportStateIsOrderIndependent
```

SHOULD be PURE_FORMAL.

The proof SHOULD reduce to the algebraic properties of:

```text
CombineImpact = max
```

under the normative `SupportImpact` total order.

No iteration order, container order, filesystem enumeration order, or Record presentation order may affect:

```text
RawSupportImpact
```

or the resulting raw support state.

---

# 102. Support Non-Revival

The formal model SHOULD prove:

```text
SupportInvalidationIsAbsorbing

SupportDegradationIsNonReviving

CloseoutSupportIsNonReviving
```

These are distinct from bracket semantics.

---

# 103. Indeterminate Resolution

The formal model SHOULD prove:

```text
IndeterminateMayResolveUnaffected
```

An INDETERMINATE support state may return to ACTIVE only when the unresolved question is later established UNAFFECTED and no terminal support loss already exists.

---

# 104. Multiple Invalidation Effects

Support aggregation is over all exact support elements using the normative `max` reduction.

Example:

```text
A → AFFECTED_UNKNOWN
B → AFFECTED
C → UNAFFECTED
```

then:

```text
max(
  AFFECTED_UNKNOWN,
  AFFECTED,
  UNAFFECTED
)
=
AFFECTED
```

therefore:

```text
SUPPORT_INVALIDATED
```

If A later resolves to:

```text
UNAFFECTED
```

the current raw support impact remains:

```text
AFFECTED
```

because B remains `AFFECTED`.

The historical terminal floor independently ensures that a previously terminal Closeout support loss does not revive even if later current-element inputs change.

---

# 105. Bracket Failure Is Separate

Lifecycle bracket state and later formal-support invalidation are distinct mechanisms.

Formal theorem names SHOULD distinguish:

```text
BracketFailureIsTerminal
```

from:

```text
CloseoutSupportIsNonReviving
```

---

# 106. Compatibility Invalidation Does Not Rewrite Establishment

The formal model SHOULD prove as a frame/forbidden-transition theorem:

```text
CompatibilityInvalidationDoesNotRewriteEstablishment
```

The Establishment remains unchanged.

Only the cross-version support edge loses applicability.

---

# 107. Bootstrap Trust

Recursive qualification MUST terminate at explicit authoritative:

```text
BOOTSTRAP_TRUST_DECLARATION
```

where qualification depends on accepted but unqualified root premises.

---

# 108. Bootstrap Trust Categories

At minimum:

```text
MATHEMATICAL_TRUST

PHYSICAL_TRUST

HUMAN_JUDGMENT_TRUST
```

Deployment MAY also use:

```text
COMPUTATIONAL_TOOLCHAIN_TRUST

EXTERNAL_GOVERNANCE_TRUST
```

---

# 109. Mathematical Trust

Examples:

```text
SHA-256 collision resistance

soundness of mathematical foundations outside project Scope

external theorem-system assumptions
```

A Bootstrap Declaration records acceptance of these premises.

It does not prove them.

---

# 110. Computational Trust

Examples:

```text
Dafny verifier soundness

Z3 solver soundness

compiler correctness below evaluated layer

CPU arithmetic correctness
```

External Evidence MAY reduce uncertainty.

The Declaration identifies where recursive qualification stops.

---

# 111. Physical Trust

Examples:

```text
storage hardware behaves sufficiently according to documented semantics

CPU executes relevant operations as observed

fault-injection harness represents declared fault class
```

Observation can strengthen confidence.

It does not make physical assumptions mathematical facts.

---

# 112. Human Judgment Trust

Examples:

```text
hostile-review Scope judged sufficient

negative controls judged representative

Policy diversity judged adequate

remaining UNKNOWN accepted

counterexample classification accepted
```

Human judgment MUST remain explicitly typed as judgment.

---

# 113. Bootstrap Trust Declaration

Minimum semantics:

```text
declaration_scope_ref

trusted_mathematical_assumptions[]

trusted_physical_assumptions[]

trusted_computational_assumptions[]

human_judgment_points[]

externally_governed_trust_points[]

rationale_refs[]

known_limitations[]

bootstrap_evidence_refs[]

predecessor_declaration_ref
```

Registry authority requires Lifecycle authority.

---

# 114. Bootstrap Trust Scope

Bootstrap Trust scope MUST be an exact content-addressed Scope Record.

v0.x uses exact matching:

```text
declaration_scope_ref
==
required_bootstrap_scope_ref
```

No semantic subset/superset inference is permitted.

---

# 115. Bootstrap Trust Is a Support Element

Where Closeout support terminates at Bootstrap Trust, the exact Declaration MUST appear in its support graph.

Ambient:

```text
current Bootstrap Trust
```

is insufficient.

The exact historical Declaration must be bound.

---

# 116. Bootstrap Trust Succession

A successor Declaration does NOT revoke the predecessor.

Therefore:

```text
B1
↓ successor
B2
```

does not automatically degrade Closeouts depending on B1.

Trust-boundary strengthening MUST NOT punish historical qualification merely because the boundary improved.

---

# 117. Bootstrap Trust Invalidation

Later Evidence may invalidate a Bootstrap Trust Declaration.

The Declaration itself remains historical.

Its current impact may become:

```text
AFFECTED

AFFECTED_UNKNOWN

DEGRADED
```

according to the invalidation Evidence.

The Closeout support aggregator processes that impact.

---

# 118. Bootstrap Trust Non-Revival

If Closeout C becomes SUPPORT_INVALIDATED because Bootstrap Declaration B was invalidated:

```text
later Declaration B2
```

MUST NOT restore C.

B2 may support successor Closeout C2.

---

# 119. Bootstrap Trust Is Not Self-Proving

An authoritative Declaration establishes:

> This exact trust boundary was accepted.

It does not establish:

> The premises inside the Declaration are objectively true.

This distinction MUST appear both in lifecycle authority and formal-support semantics.

---

# 120. Bootstrap Trust Does Not Recurse Forever

EvidenceRegistry MUST NOT require:

```text
Bootstrap Declaration
qualified by Bootstrap Declaration
qualified by Bootstrap Declaration
...
```

indefinitely.

The Declaration is explicitly the Record of:

```text
qualification stops here
```

Its authority establishes the declaration, not the universal truth of its premises.

---

# 121. Bootstrap Trust Invalidation Is Not Infinite Requalification

An invalidation of a Bootstrap Declaration may itself have:

```text
Evidence

Method

judgment

limitations
```

without requiring an infinitely deeper trust chain.

Where the invalidation basis is uncertain:

```text
SUPPORT_INDETERMINATE
```

is permitted.

UNKNOWN MUST NOT be hidden by creating fictional deeper proof.

---

# 122. Environmental Bootstrap

Deterministic bootstrap and environmental bootstrap are different.

---

## 122.1 Deterministic bootstrap

May use:

```text
published vectors

independent recomputation

cross-implementation identity reproduction

static review
```

---

## 122.2 Environmental bootstrap

Should rely on diverse observations when exact reproduction is impossible.

Possible ingredients:

```text
candidate probe

separate harness

different host

negative control

hostile environment

independent specialist review
```

The exact requirement belongs to Policy.

---

# 123. Bootstrap Diversity Claim

Multiple observations MUST NOT be described as independent merely because there is more than one.

Relevant diversity dimensions may include:

```text
implementation

harness

host

Method

storage

operator

review procedure
```

Achieved diversity MUST be explicit.

---

# 124. Bootstrap Trust Scope Is Bounded

A Declaration for:

```text
Milestone 1 local filesystem semantics
```

MUST NOT automatically support:

```text
remote attestation

Profile H

cross-Registry federation

distributed consensus
```

Bootstrap Trust is Scope-bounded support.

---

# 125. Formal Support Binding

The exact Closeout support binding SHOULD contain:

```text
formal_verification_authority_refs[]

assumption_bindings[]

bootstrap_trust_declaration_authority_refs[]

formal_finding_classification_authority_refs[]
  where Policy requires Classification
```

Each Assumption binding records exact:

```text
required Definition

Establishment

Compatibility if needed
```

This is the immutable historical support snapshot.

---

# 126. Applicability Projection

Current applicability MAY be cached in a nonauthoritative projection.

Candidate indexes:

```text
current capability epoch

Assumption → Establishment

Establishment → invalidation state

Compatibility state

Bootstrap Declaration state

Formal Finding Classification state

Closeout support state
```

Projection is acceleration only.

---

# 127. Projection Disagreement

If projection disagrees with fresh derivation:

```text
authoritative Journal + Records win
```

Projection MUST be:

```text
rebuilt
repaired
or discarded
```

It MUST NOT redefine support.

---

# 128. Capability Epoch Optimization

An Establishment SHOULD retain:

```text
capability_epoch_ref
```

so applicability can determine whether a later material capability transition exists without scanning unrelated full history repeatedly.

An implementation MAY maintain indexed epoch projections.

Semantics remain Journal-derived.

---

# 129. Formal Model Assumptions Are Named Objects

The formal model MUST avoid anonymous ambient assumptions.

Conceptually:

```text
RequiresAssumption(
  Definition,
  Scope
)
```

must appear wherever environmental properties are consumed.

This enables:

```text
Proof
↓ depends on
Assumption
↓ established by
Evidence
```

to remain mechanically traceable.

---

# 130. Assumption Applicability Failure

If a theorem requires:

```text
ExclusiveCreateAtomic(required_scope)
```

but the environment establishes only an incomparable or insufficient Scope:

```text
ASSUMPTION_SCOPE_INSUFFICIENT
```

is the correct result.

The theorem is not false.

It is not applicable to that claimed environment.

---

# 131. Formal Assumption Invalidation

Suppose:

```text
T1 Establishment A authoritative

T2 Formal Verification V authoritative

T3 Closeout C authoritative using A and V

T4 later invalidation affects A
```

Correct history:

```text
A existed
V existed
C existed
later A's support basis changed
```

Incorrect history:

```text
A never existed
V was never valid
C never happened
```

Later knowledge changes present support, not historical bytes.

---

# 132. Invalid Method Does Not Prove Opposite

If an Establishment Method later becomes INVALID:

```text
historical observation
```

does not automatically become false.

Current support may become:

```text
UNKNOWN
AFFECTED_UNKNOWN
SUPPORT_INDETERMINATE
```

unless separate Evidence establishes falsity.

Loss of justification is not proof of the opposite.

---

# 133. Formal Verification Toolchain Identity

Formal Verification MUST bind exact toolchain identity where material.

At minimum:

```text
Dafny

SMT solver

configuration

resource limits
```

A result under one toolchain MUST NOT be silently interpreted as having been reproduced under another.

---

# 134. Toolchain Timeout Semantics

Timeout is a computational-environment result.

It MUST NOT become:

```text
theorem false
```

or:

```text
product defect
```

without additional Evidence.

---

# 135. Physical Assumption Establishment

Formal proofs depending on physical properties MUST consume explicit Establishment Evidence.

Example:

```text
FreshRootUniqueness
```

may require:

```text
ExclusiveCreateAtomic(required Scope)
```

The proof is conditional.

The platform Evidence establishes whether the proof may be applied.

---

# 136. Durable Publication

A high-level formal assumption SHOULD express the required property:

```text
DurableNamespacePublication(scope)
```

rather than one platform-specific mechanism such as:

```text
parent directory fsync exists
```

Different platforms may establish different strengths through different mechanisms.

Materially weaker semantics require:

```text
different Assumption
or
different Scope
```

not a false PRESENT value.

---

# 137. Platform Assumption Honesty

If:

```text
Linux/ext4
```

establishes one Scope of durable publication and:

```text
Windows/NTFS
```

establishes a different Scope, they MUST NOT be represented as one identical unqualified Assumption unless that equivalence is separately justified.

---

# 138. Verification Matrix

The canonical implementation-boundary matrix SHOULD include:

| Column                     |
| -------------------------- |
| Property / Mechanism       |
| Formal Class               |
| Dafny Responsibility       |
| Required Assumption        |
| Required Scope             |
| Assumption Producer        |
| Rust / Tool Responsibility |
| Deterministic Verification |
| Computational Verification |
| Physical Verification      |
| Required Environment       |
| Negative Control           |
| Bootstrap Trust            |
| Cost Band                  |

---

# 139. Required Verification Matrix Rows

At minimum include:

```text
Aborted Freeze cannot Commit

ONE_PHASE crash leaves no authority

No Forward Authority Dependency

First Eligible POST Is Decisive

Support State Aggregation

Fresh Root Uniqueness

No-Replace Publication

Journal Writer Exclusion

Filesystem Object Identity

Durable Publication

Exclusive Create Probe

Writer Lock Probe

File Object Identity Probe

Durability Probe

Proof Dependency Extractor

Compatibility Scope Mapper

Invalidation Predicate Evaluator

Support-State Aggregator

Diversity Evaluator

Bootstrap Trust Scope Evaluator

Formal Finding Classification Policy Evaluator

SOURCE_SUBJECT_MATCH Policy Evaluator
```

The required:

```text
SOURCE_SUBJECT_MATCH Policy Evaluator
```

coverage MUST include both:

```text
ordinary source-match qualification
```

and:

```text
PRE / POST bracket source-match qualification
```

under Lifecycle §§90–95.

A separate bracket-specific engineering row MAY additionally be used.

Required coverage MUST NOT disappear between rows.

---

# 140. Example Verification Matrix

| Property / Mechanism                           | Formal Class                             | Dafny                                      | Assumption                                | Scope                              | Producer                     | Rust / Tool                               | Deterministic                                                                                                                                                   | Computational             | Physical                   | Environment                                 | Negative Control                                                                                | Cost        |
| ---------------------------------------------- | ---------------------------------------- | ------------------------------------------ | ----------------------------------------- | ---------------------------------- | ---------------------------- | ----------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------- | -------------------------- | ------------------------------------------- | ----------------------------------------------------------------------------------------------- | ----------- |
| Aborted Freeze cannot Commit                   | PURE_FORMAL                              | prove                                      | none                                      | none                               | none                         | transition guard                          | property                                                                                                                                                        | Dafny                     | none                       | none                                        | none                                                                                            | LOW         |
| ONE_PHASE crash leaves no authority            | PURE_FORMAL                              | prove publication boundary                 | none                                      | none                               | none                         | authority guard                           | property                                                                                                                                                        | Dafny                     | fault seam optional        | none                                        | unpublished result fixture                                                                      | LOW         |
| First POST decisive                            | PURE_FORMAL                              | prove                                      | none                                      | none                               | none                         | derived-state evaluator                   | unit/property                                                                                                                                                   | Dafny                     | none                       | none                                        | failure POST fixture                                                                            | LOW         |
| Fresh root uniqueness                          | ASSUMPTION_CONDITIONAL                   | prove conditionally                        | ExclusiveCreateAtomic                     | required writer/storage Scope      | exclusive-create probe       | create-new implementation                 | wrapper test                                                                                                                                                    | Dafny                     | concurrency hostile        | target FS                                   | known non-atomic shim                                                                           | HIGH        |
| File alias correctness                         | ASSUMPTION_CONDITIONAL                   | prove use of observation                   | FileObjectIdentitySound                   | platform Scope                     | object-ID probe              | platform ID API                           | fixtures                                                                                                                                                        | limited                   | hardlink/alias hostile     | Win/Linux/macOS                             | alias fixture                                                                                   | HIGH        |
| Durable publication                            | ASSUMPTION_CONDITIONAL                   | prove conditional state                    | DurableNamespacePublication               | platform/FS Scope                  | durability Evidence          | flush path                                | limited                                                                                                                                                         | Dafny                     | crash/fault                | OS/FS-specific                              | fault harness                                                                                   | SPECIALIZED |
| Dependency extractor completeness              | qualification mechanism                  | controlled dependencies                    | none                                      | none                               | extractor qualification      | extractor                                 | synthetic corpus                                                                                                                                                | formal AST reconciliation | none                       | build env                                   | hidden dependency fixtures                                                                      | MODERATE    |
| Bootstrap Trust Scope Evaluator                | qualification mechanism                  | exact-scope relation may be modeled        | none                                      | exact Scope IDs                    | Bootstrap Scope evaluator    | Policy / Closeout evaluator               | exact-match vectors                                                                                                                                             | optional Dafny relation   | none                       | none                                        | same human name, different Scope ID                                                             | LOW         |
| Formal Finding Classification Policy Evaluator | qualification mechanism                  | Policy conjunction may be modeled          | none                                      | Policy Scope                       | Classification evaluator     | Policy evaluator                          | classification / method / finding vectors                                                                                                                       | optional Dafny relation   | none                       | none                                        | acceptable Classification with invalid method_status                                            | LOW         |
| SOURCE_SUBJECT_MATCH / bracket evaluator       | ASSUMPTION_CONDITIONAL support evaluator | prove binding and bracket structural rules | SourceObservationSoundness where required | Source Binding / observation Scope | source Verification producer | Verification / Policy / bracket evaluator | current target accepted; retired target rejected; wrong Source Binding; PRE/POST target mismatch; PRE/POST Source Binding mismatch; decisive-first-POST vectors | optional Dafny relation   | source-match hostile tests | platform where source identity is evaluated | predecessor-format target with same bytes; failed first POST followed by successful second POST | MODERATE    |

The example table is illustrative rather than exhaustive.

The required-row list in §139 is normative for planning coverage.

---

# 141. Verification Class Does Not Equal Cost

A deterministic vector may cost seconds.

A physical crash harness may cost days.

Both may occupy one matrix row.

Engineering planning MUST preserve separate:

```text
verification class

cost band
```

---

# 142. Named Model Assumptions

Dafny model assumptions SHOULD have stable identifiers.

Examples:

```text
ExclusiveCreateAtomic

NoReplacePublicationAtomic

WriterCoordinationExclusive

FileObjectIdentitySound

DurableNamespacePublication

SourceObservationSound
```

This enables theorem dependency inspection.

---

# 143. Assumption Definition and Environment Failure

If a required Assumption cannot be established in a deployment environment:

```text
UNSUPPORTED_ENVIRONMENT
```

or:

```text
ASSUMPTION_SCOPE_INSUFFICIENT
```

may be appropriate.

A successful theorem proof MUST NOT override missing environmental applicability.

---

# 144. Formal Proof Does Not Prove External Physics

Dafny MUST NOT be presented as proving:

* NTFS implementation correctness;
* ext4 durability;
* APFS semantics;
* NFS lock behavior;
* SMB cache behavior;
* CPU correctness;
* storage honesty;
* real power-loss behavior.

Those are external Assumptions or verification targets.

---

# 145. Formal Proof Does Not Prove Solver Soundness

A Dafny verification result depends on the accepted computational trust boundary.

Where solver/verifier soundness is not recursively qualified, it SHOULD appear in the applicable Bootstrap Trust Declaration.

---

# 146. Proof Supply Chain

The complete proof supply chain may conceptually be:

```text
Lifecycle Specification
↓
Dafny Model
↓
Theorem
↓
RequiresAssumption
↓
Dependency Extractor
↓
Proof Dependency Manifest
↓
Formal Verification
↓
Assumption Establishment
↓
Probe / Environment Evidence
↓
Bootstrap Trust
↓
Closeout
```

Each edge SHOULD be explicit where material.

---

# 147. Supply-Chain Component Qualification

Qualification-critical components include:

```text
formal parser / extractor

scope mapper

invalidation evaluator

capability probe

support aggregator

diversity evaluator

Bootstrap Trust Scope evaluator

Formal Finding Classification Policy evaluator

SOURCE_SUBJECT_MATCH Policy evaluator
```

These components MUST NOT be assumed correct merely because they are infrastructure.

They require verification appropriate to their failure impact.

---

# 148. Scope Mapper Qualification

Cross-version Compatibility Scope mapping is qualification-critical.

A mapper bug can silently widen Assumption applicability.

Therefore the mapper SHOULD have:

```text
positive vectors

negative vectors

missing-dimension vectors

incomparable Scope vectors

version mismatch vectors
```

---

# 149. Invalidation Predicate Evaluator Qualification

The evaluator MUST demonstrate correct handling of:

```text
AFFECTED

UNAFFECTED

AFFECTED_UNKNOWN

missing historical metadata

predicate version mismatch

Schema version mismatch

coverage limitation
```

Unknown historical fields MUST fail toward UNKNOWN, not UNAFFECTED.

---

# 150. Support Aggregator Qualification

The support aggregator SHOULD be backed by both:

```text
Dafny PURE_FORMAL proof

+
Rust property / conformance tests
```

because it is both a semantic state machine and a production implementation.

---

# 151. Support Aggregator Properties

At minimum test and/or prove:

```text
SupportImpact total order is complete

CombineImpact is commutative

CombineImpact is associative

CombineImpact is idempotent

UNAFFECTED is the identity element

AFFECTED is the maximal element

RawSupportImpact is independent of enumeration order

AFFECTED maps to SUPPORT_INVALIDATED

DEGRADED maps to SUPPORT_DEGRADED

AFFECTED_UNKNOWN maps to SUPPORT_INDETERMINATE

UNAFFECTED maps to SUPPORT_ACTIVE

terminal INVALIDATED never revives

terminal DEGRADED never revives

INDETERMINATE may resolve ACTIVE

multiple simultaneous support elements produce
deterministic result

each supported support-element type has a total
impact derivation or fails closed as AFFECTED_UNKNOWN
```

---

# 152. Diversity Evaluator Qualification

Diversity evaluation MUST verify:

```text
exact identity comparison

distinct_count

missing identity handling

shared dependency overlap

Policy dimension matching

Policy unsupported dimension rejection
```

Missing values MUST NOT be fabricated into diversity.

---

# 153. Bootstrap Scope Evaluator Qualification

Bootstrap Trust exact Scope matching SHOULD have deterministic vectors including:

```text
exact match

different Scope Record ID

same human name but different identity

successor Scope

similar but non-equal Scope

missing required Scope

Policy requires Bootstrap Trust but Closeout field absent

Policy does not require Bootstrap Trust but Closeout field present
```

Only exact match passes in v0.x.

---

# 154. Formal Finding Classification Evaluator Qualification

The Policy evaluator for formal Classification SHOULD include vectors for:

```text
Policy requires Classification and valid Classification exists

Policy requires Classification and Classification missing

Policy requires Classification and unacceptable classification exists

classifier procedure identity allowed

classifier procedure identity not allowed

Policy omits classification requirement and raw result is counterexample

explicit frozen raw-outcome mapping exists

raw-outcome mapping absent

Classification acceptable but method_status requirement fails

Classification acceptable but finding_state requirement fails

Classification acceptable and every other required Policy dimension passes
```

Default product-level interpretation without an explicit acceptable Classification or frozen mechanical mapping MUST fail closed.

Classification MUST NOT override failure of another required Policy dimension.

---

# 155. SOURCE_SUBJECT_MATCH Evaluator Qualification

The Policy evaluator for source-match Verification SHOULD include deterministic and environmental tests for:

```text
current SOURCE_SUBJECT_MATCH accepted where exact
Source Binding and required authority are valid

EMBEDDED_COPY_INTEGRITY rejected where Policy requires
SOURCE_SUBJECT_MATCH

predecessor SOURCE_SUBJECT_NODRIFT rejected without an
explicit compatibility specification

correct bytes but wrong Source Binding rejected

missing Source Binding fails closed

source object identity unavailable produces the exact
Policy-defined indeterminate behavior

intervening-event Policy constraint enforced where present

Journal-distance constraint enforced where present

Journal distance not presented as wall-clock freshness
```

Where `SOURCE_SUBJECT_MATCH` participates in a Lifecycle PRE / POST Closeout bracket, tests MUST additionally include:

```text
PRE target = SOURCE_SUBJECT_MATCH
POST target = SOURCE_SUBJECT_MATCH
same Source Binding
same Freeze authority
    → structurally eligible

PRE target != POST target
    → bracket MUST NOT satisfy

PRE Source Binding != POST Source Binding
    → bracket MUST NOT satisfy

PRE Freeze authority != POST Freeze authority
    → bracket MUST NOT satisfy

POST points to wrong Closeout
    → ineligible POST

POST is structurally eligible but finding fails Policy
    → decisive POST produces bracket failure

first eligible POST fails,
later eligible POST succeeds
    → original Closeout remains bracket FAILED

unrelated Journal events between PRE / Closeout / POST
    → permitted unless Policy explicitly forbids them
```

These tests MUST preserve the Lifecycle distinction between:

```text
POST structural eligibility
```

and:

```text
POST Policy success
```

An unfavorable POST MUST NOT be made ineligible merely to allow a later favorable POST to become decisive.

---

# 156. Self-Hosting

EvidenceRegistry MAY use itself to manage formal Verification Evidence after an external bootstrap exists.

Conceptually:

```text
external bootstrap
↓
qualified EvidenceRegistry N
↓
N records proof Evidence for N+1
↓
successor chain
```

Self-hosting MUST NOT erase the external bootstrap path.

---

# 157. Bootstrap Environment Evidence

For environmental properties, exact deterministic external reproduction may be impossible.

Bootstrap MAY therefore rely on:

```text
diverse observations

negative controls

independent harness

multiple hosts

specialist review
```

instead of pretending physical environment Evidence is deterministic.

---

# 158. Bootstrap Trust Declaration Is the Recursion Stop

The project MUST explicitly identify:

```text
what was evaluated

what was not evaluated

what was accepted as root trust
```

A hidden trust root is a specification defect.

An explicit trust root is an auditable boundary.

---

# 159. Human Judgment Must Remain Visible

Where final qualification depends on judgments such as:

```text
this negative control is sufficient

this Scope is appropriate

this reviewer procedure is adequate

this remaining UNKNOWN is acceptable
```

the dependency SHOULD be represented through:

```text
Policy

review procedure

Bootstrap Trust Declaration

Formal Finding Classification
```

as applicable.

It MUST NOT be converted into a fake mechanically proven predicate.

---

# 160. Current Applicability Versus Historical Existence

For every support Record:

```text
historical existence
```

and:

```text
current applicability
```

MUST remain separate.

Example:

```text
Establishment A
was authoritative at T1

later:
A becomes unusable for current qualification
```

Correct:

```text
A remains historical

current applicability changes
```

Incorrect:

```text
A is deleted or rewritten
```

---

# 161. Applicability and Closeout Are Not the Same State

A theorem's applicability to a new environment MAY change repeatedly as environments change.

An already-created Closeout's terminal support loss MUST NOT repeatedly revive.

Thus:

```text
Theorem Applicability
    = re-evaluable

Closeout terminal support loss
    = non-reviving
```

This asymmetry is normative.

---

# 162. New Establishment After Invalidation

A later new Establishment MAY make a theorem applicable again in a current environment.

It may support:

```text
new Closeout
```

It MUST NOT substitute for the old Establishment inside an already-invalidated Closeout.

---

# 163. New Compatibility After Invalidation

A new Compatibility edge MAY enable future cross-version use.

It does not rehabilitate old Closeouts that depended on an invalidated Compatibility edge.

---

# 164. New Bootstrap Declaration After Invalidation

A successor Bootstrap Declaration MAY support future qualification.

It does not rehabilitate an old Closeout whose exact Bootstrap basis was invalidated.

---

# 165. New Formal Finding Classification

A later Formal Finding Classification MAY provide additional human interpretation of a Formal Verification Record.

It MUST NOT silently replace the exact Classification historically bound to an existing Closeout.

Where a different Classification is required to support a different qualification result:

```text
successor Closeout
```

or another explicit successor qualification path is required.

---

# 166. No Semantic Laundering Through Successor Evidence

Later stronger Evidence MUST NOT be used to rewrite:

```text
which Establishment was used

which Compatibility was used

which Formal Verification was used

which Formal Finding Classification was used

which Bootstrap Trust Declaration was used
```

in an old Closeout.

Successor Evidence supports successor qualification.

---

# 167. Formal Model Boundary

The normative Dafny model SHOULD represent at least:

```text
Registry history abstraction

JournalRef abstraction

operation authority publication boundary

FreezeState

EvictionState

Closeout bracket state

Support element impact

Closeout support state

Assumption Definition

Scope

Compatibility edge

Establishment

Invalidation

Bootstrap Trust support

Formal Verification dependencies

Formal Finding Classification support
```

---

# 168. Candidate Dafny Types

Conceptually:

```text
datatype SupportImpact =
    Unaffected
  | AffectedUnknown
  | Degraded
  | Affected

datatype CloseoutSupportState =
    SupportActive
  | SupportIndeterminate
  | SupportDegraded
  | SupportInvalidated
  | RequalificationRequired
```

The constructor declaration order MUST NOT be relied upon implicitly for semantic comparison.

The formal model MUST define an explicit rank or comparison function equivalent to:

```text
rank(Unaffected)       = 0

rank(AffectedUnknown)  = 1

rank(Degraded)         = 2

rank(Affected)         = 3
```

Conceptually:

```text
function ImpactRank(
  i: SupportImpact
): nat
```

and:

```text
function CombineImpact(
  a: SupportImpact,
  b: SupportImpact
): SupportImpact
```

returning the impact having the greater normative rank.

This prevents implementation-language constructor ordering from silently becoming specification semantics.

Exact syntax is an implementation detail.

The semantics are normative.

---

# 169. Candidate Formal Relations

Conceptually:

```text
ScopeCovers(...)

CanUseEstablishment(...)

Applicable(...)

SupportImpactOf(...)

ImpactRank(...)

CombineImpact(...)

RawSupportImpact(...)

RawSupportState(...)

TerminalSupportFloor(...)

SupportState(...)

EligiblePost(...)

FirstEligiblePost(...)

BracketSatisfied(...)

BracketFailed(...)

AuthorityExistsAfterTerminalAppend(...)

BootstrapScopeMatches(...)

FormalClassificationSatisfiesPolicy(...)
```

Their exact definitions MUST conform to this specification and Lifecycle v0.10.2.

---

# 170. Formal Frame Theorems

Frame / preservation theorems SHOULD explicitly prove that unrelated historical Records remain unchanged.

Examples:

```text
CompatibilityInvalidationDoesNotRewriteEstablishment

BootstrapInvalidationDoesNotRewriteDeclaration

SuccessorDoesNotRewritePredecessor

FormalClassificationDoesNotRewriteVerification
```

---

# 171. Formal Transition Theorems

Transition/invariant theorem candidates SHOULD include:

```text
AbortedFreezeCannotCommit

OnePhaseCrashLeavesNoAuthority

NoForwardAuthorityDependency

FirstEligiblePostIsDecisive

BracketFailureIsTerminal

SupportImpactOrderIsTotal

CombineImpactIsCommutative

CombineImpactIsAssociative

CombineImpactIsIdempotent

SupportImpactIsDefinedOrFailsClosed

SupportStateIsOrderIndependent

SupportInvalidationIsAbsorbing

SupportDegradationIsNonReviving

IndeterminateMayResolveUnaffected

BootstrapScopeMismatchFailsClosed
```

The implementation MAY combine these into fewer Dafny lemmas if convenient.

Their semantics MUST remain equivalent.

---

# 172. Assumption-Conditional Theorems

Candidate conditional properties:

```text
FreshRootUniqueness

NoReplacePublicationSafety

WriterExclusion

FilesystemAliasSafety

DurableNamespacePublication

SourceObservationSoundness
```

Each MUST declare exact external dependency.

---

# 173. Rust / Dafny Conformance

Where a Dafny model exists, Rust semantic operations SHOULD map to named model transitions or relations.

Examples:

```text
Rust:
  commit_freeze()

Model:
  CommitFreeze()

Rust:
  record_operator_abort()

Model:
  AbortFreezeByOperator()

Rust:
  derive_closeout_support()

Model:
  SupportState()

Rust:
  evaluate_scope()

Model:
  ScopeCovers()

Rust:
  evaluate_bootstrap_scope()

Model:
  BootstrapScopeMatches()

Rust:
  derive_support_impact()

Model:
  SupportImpactOf()
```

A Rust success path SHOULD be covered by conformance tests against legal model behavior.

---

# 174. Conformance Does Not Prove Platform Assumptions

Rust/model conformance proves:

> The implementation follows modeled transition semantics for supplied facts.

It does not prove:

> The filesystem facts supplied to the transition were physically true.

That requires environmental Evidence.

---

# 175. Platform Assumption Mapping

The implementation architecture SHOULD maintain:

```text
Rust mechanism
↓ establishes or fails to establish
Named Assumption + Scope
```

Example:

```text
Windows exclusive create implementation
↓
ExclusiveCreateAtomic(SAME_HOST, storage X)

NFS test result
↓
ExclusiveCreateAtomic(ALL_OBSERVED_WRITERS, storage Y)
  = NOT_ESTABLISHED
```

---

# 176. Assumption Scope Insufficiency Is First-Class

If a platform establishes:

```text
ExclusiveCreateAtomic(SAME_HOST)
```

but the theorem requires:

```text
ExclusiveCreateAtomic(ALL_OBSERVED_WRITERS)
```

result:

```text
ASSUMPTION_SCOPE_INSUFFICIENT
```

MUST be visible.

Do not silently narrow the theorem Claim.

---

# 177. Environmental Evidence Strength

Environmental Evidence may establish:

```text
PRESENT

ABSENT

NOT_PROBED
```

for the declared Assumption Scope.

A test failing to run is not ABSENT.

A test succeeding in narrower Scope is not PRESENT in broader Scope.

---

# 178. Observation Evidence Limit

A finite experiment cannot prove all possible behavior of a physical platform.

Environmental Establishment MUST retain:

```text
Method

Scope

coverage

known limitations

Bootstrap Trust
```

where material.

---

# 179. Negative Control Importance

Negative controls are particularly important for mechanisms that can fail silently by returning false PRESENT.

Examples:

```text
exclusive-create probe

writer-lock probe

dependency extractor

scope mapper

invalidation evaluator
```

A mechanism tested only on expected-good inputs may be insufficiently qualified.

---

# 180. Test Vector Role

Identity-layer deterministic vectors are foundational.

They SHOULD be completed before platform implementation is treated as stable.

Candidate vector classes:

```text
CBOR

Record ID

Manifest ID

Journal Entry

Journal Reference

Journal Anchor

Scope Record

Assumption Definition

Compatibility Record

Bootstrap Trust Declaration

Formal Finding Classification

formal-support binding

capability observation provenance
```

---

# 181. Environment Test Role

Physical-hostile testing SHOULD cover at least:

```text
Windows

Linux

macOS

NTFS

ext4

APFS

hardlinks

reparse points

symlink behavior

NFS

SMB

sync clients

crash injection
```

where the claimed platform Scope requires them.

---

# 182. Crash Harness Boundary

Crash testing is environment-dependent verification.

The implementation SHOULD expose deterministic fault-injection seams at lifecycle-relevant points.

Examples:

```text
after START

before root creation

after root creation

after payload N

after Manifest

after Receipt

before COMMIT

after Eviction START

during partial eviction
```

Crash-harness design does not alter lifecycle semantics.

---

# 183. Crash Harness and ONE_PHASE Operations

Fault injection MAY also exercise ONE_PHASE operations.

The expected invariant is exactly the ONE_PHASE rule defined in §4.

For a crash before a ONE_PHASE terminal Journal append:

```text
no authoritative disposition exists
```

and the operation may be rerun under §4.

The harness MUST NOT expect a historical crash Record merely because execution had begun.

This distinction SHOULD have permanent conformance tests.

---

# 184. Qualification of Crash Harness

The crash harness itself is a test mechanism.

Where it materially supports a high-strength claim, its:

```text
fault injection semantics

coverage

known limitations
```

SHOULD be reviewed or otherwise qualified.

A simulated crash MUST NOT be described as identical to every physical power-loss mode unless established.

---

# 185. Human Reviewer Boundary

EvidenceRegistry may preserve:

```text
review role

procedure identity

Scope

finding

rationale
```

It cannot mechanically prove:

```text
reviewer sincerity

creativity

completeness of adversarial imagination
```

These remain human judgment / Bootstrap Trust boundaries where material.

---

# 186. Policy Boundary

The Project Policy defines acceptable support strength.

Formal Verification MUST NOT decide:

```text
whether SUPPORT_DEGRADED is acceptable for release

whether one hostile Review is enough

whether two implementations count as sufficient diversity

whether a remaining UNKNOWN is acceptable
```

EvidenceRegistry evaluates the mechanics of the Policy.

The governing authority defines the Policy.

---

# 187. Lifecycle Policy Fields Consumed by Formal Support

Lifecycle §85 defines the full normative Policy-field vocabulary.

The list below is not a complete reproduction of Lifecycle §85.

It contains only fields that the formal-support evaluator may need to consume directly when formal or environmental Evidence participates in qualification.

The complete Lifecycle §85 vocabulary additionally includes fields such as:

```text
Review role

Review Scope

Review Method

Review count

Journal Anchor requirements
```

Those remain governed by the Lifecycle layer and other relevant evaluators unless the formal-support workflow explicitly consumes them.

Formal-support evaluation MAY consume fields including:

```text
required_method_status

allowed finding state

Verification target

Verification authority

intervening-event constraint

max_journal_distance_to_closeout

minimum durability

required_bootstrap_scope_ref

required_formal_finding_classification

required_establishment_diversity

Closeout postconditions
```

The formal-support layer MUST NOT enforce a requirement that cannot be expressed in the normative Lifecycle Policy schema.

A Policy field parsed by the formal-support layer MUST actually be enforced before the formal-support layer claims that field satisfied.

Nothing in this section removes or narrows Policy requirements evaluated by the Lifecycle Review, Admission, Anchor, or Closeout machinery.

---

# 188. Claim Boundary

Formal proof existence MUST NOT automatically authorize product wording.

Conceptually:

```text
Formal Verification
↓
Applicability
↓
Support State
↓
Policy
↓
Claim authorization
```

The Claim layer remains separate.

---

# 189. Formal Verification Claim Discipline

EvidenceRegistry MAY claim:

```text
this theorem was verified under model M

these exact assumptions were declared

these exact Establishments were authoritative

this Scope covered that requirement

this compatibility edge was used

this bootstrap trust boundary was used

this Formal Finding Classification was used where required

this Closeout currently has support state X
```

It MUST NOT automatically claim:

```text
the real world universally satisfies the theorem

the solver is infallible

the physical platform can never violate the assumption

two implementations are causally independent

a human classification is objectively correct
```

---

# 190. Bootstrap Claim Discipline

EvidenceRegistry MAY claim:

> This qualification accepted exact premise X at the Bootstrap Trust boundary.

It MUST NOT claim:

> X was proven by EvidenceRegistry.

---

# 191. Cross-Registry Federation

Cross-Registry trust federation is OUT OF SCOPE for v0.x.

This specification does not define:

```text
foreign Establishment acceptance

foreign Compatibility recognition

foreign Bootstrap Trust recognition

foreign Scope equivalence

mutual Registry authority

cross-Registry chronology
```

These belong to a future Profile H / federation specification.

---

# 192. Profile H Boundary

Future Profile H MAY introduce:

```text
signed Records

external authority

remote attestation

TPM / TEE

separately administered reviewer identity

cross-Registry trust

external witness
```

Profile L assumptions and Evidence MUST NOT silently acquire Profile H meaning.

---

# 193. Self-Hosted Formal Evidence

EvidenceRegistry MAY record formal Verification Evidence about EvidenceRegistry itself.

This produces:

```text
EvidenceRegistry source
↓
formal Subject
↓
formal Verification
↓
EvidenceRegistry lifecycle
```

Bootstrap requirements remain.

Self-hosting is allowed.

Circular self-proof is not.

---

# 194. Historical Integrity

Later:

```text
Assumption invalidation

Compatibility invalidation

Bootstrap Trust invalidation

Formal Finding Classification

new Establishment

new Formal Verification
```

MUST NOT rewrite predecessor Records.

Current derived support may change.

History does not.

---

# 195. Unknown Is First-Class

If the system cannot establish:

```text
Assumption applicability

Compatibility applicability

invalidation affectedness

Scope coverage

human classification

Bootstrap premise validity

support-element impact
```

it MUST preserve:

```text
UNKNOWN
INDETERMINATE
AFFECTED_UNKNOWN
```

as appropriate.

It MUST NOT fabricate certainty to complete a qualification graph.

---

# 196. Bootstrap Trust and UNKNOWN

Bootstrap Trust Declaration MAY explicitly accept a known remaining UNKNOWN for a particular qualification Scope.

The Declaration MUST make that acceptance visible.

It MUST NOT change UNKNOWN into mechanically established PRESENT.

---

# 197. Specification Freeze Gate

Before declaring this Formal Verification specification frozen, the following semantics MUST be treated as stable:

```text
PURE_FORMAL vs ASSUMPTION_CONDITIONAL

Formal-support ONE_PHASE authority publication

Assumption Definition

Scope partial order

same-Definition Scope comparison

cross-version Compatibility

Compatibility invalidation

Assumption Establishment

capability observation provenance

Establishment expiration

Assumption invalidation

AFFECTED / AFFECTED_UNKNOWN / UNAFFECTED vocabulary

Formal Verification Record

Formal Verification outcome classes

SOURCE_SUBJECT_MATCH vocabulary

Proof Dependency Manifest

dependency extractor qualification

diversity identity semantics

SupportImpact total order

SupportImpact per-element derivation

Formal Finding Classification impact derivation

RawSupportImpact max-reduction semantics

support aggregation

terminal support floor

Bootstrap Trust support

Bootstrap Trust exact Scope matching

Bootstrap Trust invalidation

Formal Finding Classification

required_formal_finding_classification Policy semantics

human judgment boundary

projection independence

GENESIS capability epoch interpretation
```

---

# 198. Implementation Freeze Inputs

Before implementation Records are treated as stable, freeze exact schemas for:

```text
Assumption Definition

Assumption Establishment

capability observation provenance object

Assumption Invalidation

Assumption Version Compatibility

Compatibility Invalidation

Proof Dependency Manifest

Formal Verification Record

Formal Finding Classification

Shared Dependency Manifest

Bootstrap Trust Declaration

Bootstrap Trust Invalidation

Formal support binding

Scope Records used by Assumptions

Scope Records used by Bootstrap Trust
```

---

# 199. Formal Test Plan

The initial Dafny suite SHOULD begin with PURE_FORMAL properties.

Recommended order:

```text
AbortedFreezeCannotCommit

OnePhaseCrashLeavesNoAuthority

NoForwardAuthorityDependency

FirstEligiblePostIsDecisive

BracketFailureIsTerminal

SupportImpactOrderIsTotal

CombineImpactIsCommutative

CombineImpactIsAssociative

CombineImpactIsIdempotent

SupportImpactIsDefinedOrFailsClosed

SupportStateIsOrderIndependent

SupportInvalidationIsAbsorbing

SupportDegradationIsNonReviving

IndeterminateMayResolveUnaffected

CompatibilityInvalidationDoesNotRewriteEstablishment

BootstrapInvalidationDoesNotRewriteDeclaration

FormalClassificationDoesNotRewriteVerification
```

Then proceed to ASSUMPTION_CONDITIONAL properties.

---

# 200. Implementation Order

Recommended high-level implementation sequence:

```text
Normative transition / cross-reference table

↓

Identity test vectors

↓

Dafny PURE_FORMAL core

↓

Rust Record / Journal core

↓

Rust transition guards

↓

Rust/model conformance

↓

filesystem abstraction

↓

Assumption producers / probes

↓

fault-injection seams

↓

crash harness

↓

physical-environment qualification

↓

ASSUMPTION_CONDITIONAL theorem application

↓

bootstrap qualification
```

---

# 201. Verification Matrix Is a Living Engineering Artifact

The matrix MAY evolve as implementation mechanisms change.

However, changes to:

```text
formal property meaning

required Assumption meaning

Scope semantics

SupportImpact total order

support-impact semantics

support aggregation

authority semantics

Bootstrap Trust semantics

Formal Finding Classification semantics
```

require specification successor treatment rather than ordinary matrix editing.

---

# 202. Formal Verification Development Governance

Material changes to:

```text
Assumption semantics

Scope ordering

Compatibility semantics

SupportImpact ordering

support-impact derivation

support aggregation

Bootstrap Trust semantics

proof dependency semantics

human judgment boundary

Lifecycle authority binding
```

SHOULD require:

```text
design
↓
frozen specification candidate
↓
hostile review
↓
independent review
↓
accepted successor
```

Reversible implementation details MAY use lighter verification.

---

# 203. FV Core Principles

## FV-P1: Conditional Formal Proof

**Formal proof is conditional on exact named Assumptions and exact Scope.**

---

## FV-P2: Partial-Order Scope

**Assumption Scope is a partial order, not scalar strength.**

---

## FV-P3: Authority Is Explicit

**Portable Assumption or Formal Records do not acquire Registry authority merely by existing.**

---

## FV-P4: Invalidation Preserves History

**Assumption invalidation moves history forward; it does not rewrite Establishment bytes.**

---

## FV-P5: Applicability Versus Closeout Support

**Theorem applicability may vary by environment, but terminal Closeout support loss does not revive.**

---

## FV-P6: Expiration Is Not Invalidation

**Expiration for future reuse is not retrospective invalidation or AFFECTED support impact.**

---

## FV-P7: Unknown Invalidation Coverage

**Incomplete invalidation coverage yields AFFECTED_UNKNOWN, not silent UNAFFECTED.**

---

## FV-P8: Computational Verification Identity

**Formal Verification binds exact solver and computational environment sufficient to describe the event.**

---

## FV-P9: Probe Qualification

**Qualification-critical probes are themselves qualification Subjects.**

---

## FV-P10: Machine-Derived Proof Dependencies

**Proof dependency manifests are machine-derived, not maintained only by prose.**

---

## FV-P11: Verification-Class Separation

**Deterministic, computational, and physical Verification are distinct engineering classes.**

---

## FV-P12: Scope-Bounded Applicability

**A theorem consuming an external Assumption is inapplicable where required Scope is not established.**

---

## FV-P13: Support-State Semantics

**SUPPORT_INDETERMINATE may resolve; SUPPORT_DEGRADED and SUPPORT_INVALIDATED do not revive for one exact Closeout.**

---

## FV-P14: Definition-Relative Scope

**Scope comparison is relative to one exact Assumption Definition.**

---

## FV-P15: Explicit Cross-Version Compatibility

**Cross-version Establishment reuse requires explicit authoritative directional Compatibility.**

---

## FV-P16: Extractor Qualification

**The Proof Dependency Extractor is part of the proof supply chain and must itself be qualified.**

---

## FV-P17: Machine-Enforced Bootstrap Diversity

**Bootstrap diversity must be mechanically enforceable if Policy claims to require it.**

---

## FV-P18: Projection Independence

**Current applicability projections are disposable acceleration structures, not authority.**

---

## FV-P19: Capability Epoch Reuse

**Capability-epoch expiration prevents future reuse without retroactively invalidating historical qualification.**

---

## FV-P20: Missing Historical Metadata

**Invalidation selectors lacking required historical metadata produce AFFECTED_UNKNOWN.**

---

## FV-P21: Order-Independent Support Aggregation

**Closeout support is aggregated over all exact support elements through an order-independent reduction.**

---

## FV-P22: Compatibility Is an Explicit Edge

**Compatibility is a support edge and may itself be invalidated.**

---

## FV-P23: Compatibility Invalidation Is Edge-Local

**Invalidating Compatibility removes the application edge, not the historical Establishment.**

---

## FV-P24: Diversity Claim Bound

**Diversity identity proves only the exact distinctness property encoded by that identity.**

---

## FV-P25: Shared Dependencies Remain Visible

**Shared dependencies must remain visible when Policy claims implementation diversity.**

---

## FV-P26: Counterexample Claim Discipline

**Counterexample to the formal model is not automatically a product defect.**

---

## FV-P27: Explicit Bootstrap Boundary

**Bootstrap Trust is an explicit boundary declaration, not infinite regress disguised as Evidence.**

---

## FV-P28: Bootstrap Authority Is Declaration Authority

**Authority of a Bootstrap Trust Declaration establishes what was accepted, not universal truth of the accepted premise.**

---

## FV-P29: Human Judgment Is Typed

**Human judgment is typed and preserved separately from machine proof.**

---

## FV-P30: Succession Over Revision

**Later knowledge changes present applicability and support by succession, never historical Evidence by revision.**

---

## FV-P31: ONE_PHASE Authority Boundary

**A formal-support ONE_PHASE operation acquires no Registry authority before its terminal Journal append, and a crash before that append leaves no authoritative disposition.**

---

## FV-P32: Current Source-Match Vocabulary

**Current formal-support evaluation uses SOURCE_SUBJECT_MATCH semantics and does not silently reinterpret predecessor SOURCE_SUBJECT_NODRIFT Records.**

---

## FV-P33: Journal Replay Is Not Semantic Evaluation

**Formal applicability cannot be established from Journal metadata alone when required semantic Record payloads are unavailable.**

---

## FV-P34: Bootstrap Scope Field Consistency

**Policy, Closeout, and Bootstrap Trust Declaration must bind the same exact required Bootstrap Scope where Bootstrap Trust is required.**

---

## FV-P35: Formal Classification Defaults Fail Closed

**An unclassified raw formal finding does not automatically authorize a product-level conclusion.**

---

## FV-P36: Probe Cache Is Not Continuous Proof

**Reuse of a capability probe result within an allowed cache scope does not establish continuous re-observation of the underlying physical capability.**

---

## FV-P37: Cached Capability Provenance Is Bound

**An Establishment using cached capability Evidence must bind exact provenance sufficient to identify the reused observation and its declared cache scope.**

---

## FV-P38: Support Impact Must Be Defined

**Every support-element type participating in Closeout support aggregation must have an explicit impact derivation, or fail closed rather than defaulting to UNAFFECTED.**

---

## FV-P39: Classification Is Additive to Method and Finding Requirements

**Satisfying a Formal Finding Classification requirement does not override an unsatisfied required_method_status or finding-state requirement.**

---

## FV-P40: SupportImpact Is Ordered Explicitly

**SupportImpact aggregation uses the explicit normative order `UNAFFECTED < AFFECTED_UNKNOWN < DEGRADED < AFFECTED`; implementation constructor order has no authority.**

---

## FV-P41: Policy Does Not Mutate Classification Impact

**An immutable Policy defines Classification requirements but does not later change a Classification's current impact; impact changes only through later authoritative Evidence.**

---

## FV-P42: Bracket Source Binding Symmetry

**A SOURCE_SUBJECT_MATCH bracket requires structurally compatible PRE and POST target type, Freeze authority, and Source Binding; mismatch cannot satisfy the bracket.**

---

# 204. Formal Boundary Claim

The Formal Verification layer may establish:

```text
the theorem follows from the model under assumptions

the exact assumptions were declared

the exact scopes were required

the exact Establishments were used

the exact Compatibility edge was used

the exact Bootstrap Trust Declaration was used

the exact Formal Finding Classification was used where required

the exact support-element impacts are derivable from
retained authoritative Evidence

the exact support state is derivable from those impacts
```

It does not establish:

```text
all physical assumptions are universally true

the formal model perfectly represents reality

the theorem applies to every environment

the verifier / solver is infallible

human judgment is mathematically proven

a predecessor SOURCE_SUBJECT_NODRIFT Record is equivalent
to SOURCE_SUBJECT_MATCH

a cached capability probe is continuous physical proof

an undefined support-element impact may safely default
to UNAFFECTED

cross-Registry trust is valid
```

---

# 205. Freeze Interpretation

Freezing this specification means:

> The semantic boundary among theorem, assumption, Scope, Establishment, Compatibility, invalidation, environment applicability, support-element impact, support aggregation, human judgment, Bootstrap Trust, production implementation, and Lifecycle authority is sufficiently defined to begin formal-model and implementation work without inventing local semantic rules.

If downstream implementation exposes a genuine contradiction:

```text
successor specification required
```

rather than silent reinterpretation.

At this freeze candidate:

```text
Known BLOCKING findings:
    NONE

Known MATERIAL findings from the current review:
    NONE outstanding
```

This does not mean:

```text
Dafny implementation complete

Rust implementation qualified

physical assumptions established

environmental qualification complete

production ready
```

It means the semantic boundary is a suitable candidate for formalization and implementation.

---

# 206. Core Historical Principle

> A theorem may remain mathematically verified while becoming unusable in a particular environment.

> An Establishment may remain historically real while losing future applicability.

> A Closeout may remain authoritative history while losing present support.

> A Bootstrap Trust Declaration may remain historical while later Evidence shows that its accepted premise should no longer support new qualification.

> A raw formal result may remain historically BLOCKING while later human judgment changes what higher-level conclusion is justified.

> A support element may remain historically present while later authoritative Evidence changes its current SupportImpact.

None of these cases requires rewriting the past.

---

# 207. One-Line Principle

> Dafny proves the law under exact named and scoped assumptions; Rust implements the physical mechanisms intended to establish those assumptions; deterministic, computational, physical, and human Evidence establish how far the proof may actually be applied; Lifecycle authority records exactly when those facts became authoritative; every support element has an explicit current-impact rule; later knowledge changes current support by succession, never historical fact by convenience.
