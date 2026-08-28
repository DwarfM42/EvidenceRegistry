# EvidenceRegistry Identity Format Specification v0.3

**Status:** FINAL FROZEN
**Predecessor:** EvidenceRegistry Identity Format Specification v0.2
**Project:** EvidenceRegistry
**Repository:** `DwarfM42/EvidenceRegistry`
**Normative lifecycle dependency:** `EvidenceRegistry Evidence Lifecycle Specification v0.10.2`
**Normative formal dependency:** `EvidenceRegistry Formal Verification & Implementation Boundary Specification v0.5.4`
**Normative transition dependency:** `EvidenceRegistry Normative Event & Transition Cross-Reference Table v0.3`
**Purpose:** Stable byte-level identity, deterministic serialization, Journal framing, dependency encoding, and common identity primitives
**Normative language:** MUST / MUST NOT / SHOULD / SHOULD NOT / MAY

---

# 0. Specification Position

This specification defines the byte-level laws shared by EvidenceRegistry identity-bearing structures.

It answers:

```text
How is one semantic value represented by exactly one
authoritative byte sequence?

How is a Record ID computed?

How is a Journal Entry hash computed?

How is a Journal Reference encoded?

How are dependencies sorted?

How are duplicate or conflicting references rejected?

How are Registry, Attempt, Record, and derived identities
represented?

Which references live in Records and which live directly
in Journal Entries?

How can different implementations reproduce exactly the
same identity bytes without reading each other's source?
```

This specification does NOT define the complete semantic field set of every Record type.

That belongs to:

```text
EvidenceRegistry Normative Record Schema Specification
```

This specification does NOT redefine:

```text
Lifecycle authority

legal transitions

Policy meaning

formal theorem semantics

Review semantics

support-state semantics
```

Those remain governed by their upstream normative specifications.

---

## 0.1 Changes from v0.2

v0.3 incorporates the final non-blocking review observations before freeze.

It does not change the semantic or byte-level model established by v0.2.

Changes are limited to:

```text
authority dependency producer construction now explicitly
checks same-Registry and strictly-prior validity before
canonical sorting and encoding

FreezeRoot derivation now has an explicit positive
standalone identity vector requirement rather than relying
only on the broader Freeze START vector
```

All BLOCKING and MATERIAL findings through v0.2 review are closed.

No Lifecycle successor, Cross-Reference successor, or Formal Verification successor is required by this revision.

---

# 1. Core Identity Principle

For every authoritative identity-bearing structure:

```text
one semantic value
→
one canonical authoritative representation
→
one identity
```

EvidenceRegistry MUST NOT permit two semantically equivalent authoritative encodings where one canonical encoding can be defined.

Examples of prohibited ambiguity include:

```text
SHA-256 as lowercase hex text
vs
SHA-256 as uppercase hex text

dependency declaration order A,B
vs
dependency declaration order B,A

registry_id omitted from JournalReference
vs
registry_id included

null optional field
vs
field absent

same semantic map represented with different key order

non-minimal integer encoding
vs
minimal integer encoding
```

Human-readable projections MAY have multiple presentation forms.

Authoritative identity bytes MUST NOT.

---

# 2. Identity and Authority Are One-Way

The following dependency direction is normative:

```text
Record identity
    does not know
    its future authority Journal Entry

Journal authority
    knows
    the exact Record identity it authorizes
```

Conceptually:

```text
Record bytes
↓
record_id
↓
Journal Entry
  event_record_id = record_id
↓
Registry authority
```

The inverse dependency is forbidden.

A Record MUST NOT require its own future:

```text
authority entry_index
authority entry_hash
Journal filename
Journal slot
```

to compute its `record_id`.

---

## 2.1 No identity-authority cycle

The following construction is forbidden:

```text
record_id
depends on
authority Journal Entry

while

authority Journal Entry
depends on
record_id
```

An implementation MUST NOT solve such a cycle by:

```text
temporary fake IDs

zero placeholders later rewritten

mutable Record identity

two-pass authoritative Record rewriting
```

The correct architecture is one-directional.

---

# 3. Record-Contained Journal References

A Record MAY contain Journal References to authority that already existed before that Record was created.

Examples:

```text
observation_start_journal_ref

operation_start_journal_ref

predecessor authority references

other explicitly permitted prior-history references
```

These references do not violate §2 because they refer only backward.

---

# 4. Two-Stage Validation of Record-Contained Journal References

A Journal Reference embedded inside a Record has two distinct validation stages.

---

## 4.1 Record-local structural validation

This MAY be performed from the Record bytes alone.

It verifies:

```text
CBOR type

array length

field representation

registry_id length

entry_index range

entry_hash length

event_type_id range

event_record_id length
```

It establishes only:

```text
the embedded Journal Reference is structurally valid
```

It does NOT establish:

```text
the referenced Journal Entry exists

the referenced hash is historically correct

the Reference belongs to the same Registry authority
context

the Reference precedes this Record's authority event
```

---

## 4.2 Authority-context validation

This requires:

```text
the exact Record

+
the exact Journal Entry that made the Record authoritative

+
the retained authoritative Journal
```

Conceptually:

```text
ValidateAuthoritativeRecord(
    record,
    authority_entry,
    journal
)
```

For every Record-contained Journal Reference `R`:

```text
R.entry_index
<
authority_entry.entry_index
```

MUST hold.

The referenced Entry MUST also satisfy the common Journal Reference context validation defined later in this specification.

---

## 4.3 Record validation cannot establish strictly-prior alone

A Record validator operating without the Record's authority Journal Entry MUST NOT claim:

```text
chronology reference validated as strictly prior
```

It MAY claim only:

```text
structurally valid Journal Reference
```

This distinction is normative.

---

# 5. Missing Authoritative Record Payload

A Journal Entry MAY remain structurally valid while its authoritative `event_record_id` payload is unavailable.

In that case:

```text
Journal chain validation
```

and:

```text
Record semantic validation
```

MUST remain distinct.

Audit MUST report:

```text
AUTHORITATIVE_RECORD_PAYLOAD_MISSING
```

where applicable.

A missing Record payload MUST NOT be treated as though the Record's internal Journal References were successfully validated.

---

# 6. Validation Status Dimensions

Machine-readable inspection and verification SHOULD preserve at least four distinct dimensions:

```text
record_identity_status

authority_status

journal_reference_validation_status

semantic_validation_status
```

These dimensions MUST NOT be collapsed into one generic boolean.

---

## 6.1 record_identity_status

Candidate semantic values:

```text
NOT_PERFORMED

VALID

INVALID

PAYLOAD_UNAVAILABLE
```

---

## 6.2 authority_status

Candidate semantic values:

```text
NOT_PERFORMED

AUTHORITATIVE

NONAUTHORITATIVE

INVALID

UNKNOWN
```

A missing payload does not erase a surviving Journal authority event.

Therefore a state such as:

```text
authority_status = AUTHORITATIVE

record_identity_status = PAYLOAD_UNAVAILABLE
```

is valid.

---

## 6.3 journal_reference_validation_status

Candidate semantic values:

```text
NOT_APPLICABLE

NOT_PERFORMED

STRUCTURAL_ONLY

VALID

INVALID

UNAVAILABLE
```

---

## 6.4 semantic_validation_status

Candidate semantic values:

```text
NOT_PERFORMED

VALID

INVALID

INDETERMINATE
```

Exact machine-output encoding belongs to the output/schema specification.

The semantic distinction is frozen here.

---

# 7. Relation to Exit Code 6

Where a requested property cannot be determined because a required validation dimension is:

```text
NOT_PERFORMED

PAYLOAD_UNAVAILABLE

UNAVAILABLE

INDETERMINATE
```

the command MAY map to Lifecycle exit code:

```text
6
```

when inability to collect or validate sufficient Evidence is the controlling cause.

A missing validation MUST NOT be reported as successful validation.

---

# 8. Authoritative Serialization

EvidenceRegistry v0.x authoritative identity uses:

```text
RFC 8949 deterministic CBOR
```

under the stricter profile defined by this specification.

Every implementation producing authoritative bytes MUST produce byte-for-byte identical CBOR for the same semantic value.

---

# 9. Authoritative CBOR Profile

Unless a later Record Schema explicitly narrows a field further, authoritative structures MAY use:

```text
unsigned integers

byte strings

UTF-8 text strings

arrays

maps

booleans

null
```

The following are forbidden in v0.x foundational identity structures:

```text
floating-point values

indefinite-length byte strings

indefinite-length text strings

indefinite-length arrays

indefinite-length maps

CBOR tags unless this specification explicitly assigns one

negative integers unless an exact schema explicitly permits one

duplicate map keys
```

No foundational structure defined by this specification permits negative integers.

---

# 10. Minimal Integer Encoding

Every integer MUST use the shortest valid CBOR encoding.

Non-minimal integer encodings MUST be rejected as non-canonical.

A decoder MAY parse them diagnostically.

It MUST NOT accept them as authoritative canonical bytes.

---

# 11. Integer Range

Define:

```text
ER_UINT_MAX
=
2^53 - 1
=
9007199254740991
```

Unless a schema defines a smaller range, general EvidenceRegistry nonnegative counters and indexes MUST satisfy:

```text
0 <= value <= ER_UINT_MAX
```

---

## 11.1 Journal index

`entry_index` MUST satisfy:

```text
0 <= entry_index <= ER_UINT_MAX
```

GENESIS uses:

```text
entry_index = 0
```

---

## 11.2 Index exhaustion

If the current Journal head has:

```text
entry_index = ER_UINT_MAX
```

no successor Journal Entry can be created.

The Registry MUST fail closed with an explicit index-exhaustion result.

Unsigned integer wraparound is forbidden.

---

## 11.3 Rationale

The limit is intentionally below CBOR's native unsigned 64-bit maximum.

It provides:

```text
exact JSON-number interoperability

simpler cross-language implementations

no JavaScript integer precision ambiguity

simpler Dafny overflow reasoning

an operationally unreachable practical ceiling
```

This is a format rule, not a prediction that a Registry will approach the limit.

---

# 12. Small Numeric Identifier Types

The following identifier registries use unsigned integers in the range:

```text
1..65535
```

unless an upstream specification already fixes a smaller range:

```text
event_type_id

record_type_id

lifecycle_object_kind_id

digest_algorithm_id

artifact_kind_id

identity_dependency_kind_id
```

Value:

```text
0
```

is reserved as:

```text
UNASSIGNED / INVALID
```

and MUST NOT identify an authoritative registered semantic value.

---

## 12.1 Identifier IDs vs field keys

The zero-reservation rule in §12 applies to semantic identifier registries.

It does NOT apply to field-key registries.

Field-key registries use numeric map keys as structural positions and MAY assign:

```text
key 0
```

normatively.

Examples include:

```text
Record body key 0 = schema_version

Journal Entry key 0 = schema_version
```

These are different registry families and MUST NOT be conflated.

---

# 13. Byte Strings and Digest Bytes

A SHA-256 digest in authoritative CBOR MUST be represented as:

```text
byte string length = 32
```

No prefix is embedded inside the authoritative digest byte string.

Therefore authoritative identity uses:

```text
32 raw digest bytes
```

not:

```text
"sha256:..."

hexadecimal text

base64 text
```

---

# 14. Human Digest Projection

Human-readable and JSON projection SHOULD represent SHA-256 as:

```text
sha256:<64 lowercase hexadecimal characters>
```

Uppercase hexadecimal MUST NOT be emitted by the canonical human projection.

Human projection does not participate in authoritative identity.

---

# 15. Digest Algorithm Registry

For v0.x:

```text
1 = SHA-256
```

Digest Algorithm Registry assignments are permanent.

Assigned identifiers MUST NOT be reused with new semantics.

v0.x Journal hashes, Record IDs, Journal Anchor IDs, and derived Freeze root IDs use SHA-256 and therefore have exactly:

```text
32 bytes
```

---

# 16. Digest Comparator

Where this specification requires lexicographic comparison of SHA-256 digest bytes:

```text
compare byte 0
then byte 1
...
then byte 31
```

using unsigned byte values:

```text
0x00 < 0x01 < ... < 0xff
```

All compared v0.x digest values have equal length.

---

## 16.1 Future digest algorithms

A future format permitting different digest algorithms in one comparator domain MUST explicitly define:

```text
algorithm-id ordering

digest-length handling

cross-algorithm comparison
```

v0.x MUST NOT invent such comparison because only SHA-256 is permitted in the relevant structures.

---

# 17. UTF-8 Text

Authoritative CBOR text strings MUST contain valid UTF-8.

Text values MUST NOT undergo implicit:

```text
Unicode normalization

case folding

whitespace normalization

line-ending normalization
```

Domain separator strings defined by this specification are ASCII subsets of UTF-8.

---

# 18. Integer Map Keys

Authoritative maps defined by EvidenceRegistry v0.x MUST use unsigned integer field keys unless a specific external utility format explicitly defines otherwise.

Text field names belong to:

```text
documentation

JSON projection

human-readable diagnostics
```

not authoritative map identity.

---

# 19. Map Ordering

For authoritative maps whose keys are unsigned integers:

```text
keys MUST be emitted in ascending numeric order
```

With minimal unsigned-integer CBOR encoding, ascending numeric order agrees with the deterministic CBOR ordering applicable to these keys.

A decoder MUST reject duplicate keys.

An encoder MUST never emit them.

---

# 20. Field Key Registry Architecture

Field-key registries are NOT global across every Record type.

Use:

```text
common reserved field range
+
independent Record-type-specific field-key registry
```

---

## 20.1 Common Record-body keys

Every Record-body map reserves:

```text
0 = schema_version

1 = record_type_id

2..15 = reserved for future common Record-body semantics
```

Keys:

```text
2..15
```

MUST be absent in v0.1 Record schemas unless this Identity Format specification is explicitly succeeded.

---

## 20.2 Record-type-local key space

For each `record_type_id`:

```text
16..65535
```

forms an independent field-key registry.

Therefore:

```text
Record Type A key 16
```

and:

```text
Record Type B key 16
```

MAY have unrelated meanings.

The `record_type_id` provides the namespace boundary.

---

## 20.3 Why key spaces are type-local

This permits:

```text
new Record types

new Record fields

independent Record-schema evolution
```

without moving the numeric field assignments of unrelated Record types.

Record-schema growth therefore does not require Identity Format renumbering merely because another Record type gains fields.

---

# 21. Field-Key Immutability

Within one `record_type_id` field-key registry:

```text
an assigned numeric key MUST NEVER be reused
for different semantics
```

If a field is retired:

```text
its key remains permanently reserved
```

Changing the semantic meaning of a field requires:

```text
a new field key

or

a new Record type

or

a schema successor
```

as appropriate.

---

# 22. Unknown Record Fields

For a known:

```text
record_type_id
+
schema_version
```

an authoritative validator MUST reject an unknown field key unless that exact schema explicitly defines an extension mechanism.

v0.x foundational schemas SHOULD NOT define invisible extension bags.

This prevents an implementation from silently ignoring identity-bearing semantics it does not understand.

---

# 23. Optional Field Representation

If an optional field is not semantically present:

```text
the field key MUST be absent
```

It MUST NOT be encoded as:

```text
null
```

unless the exact schema defines null as a distinct meaningful state.

This preserves Lifecycle context-required field symmetry.

The GENESIS `prev_entry_hash = null` rule in §73 is one explicit schema-defined exception.

---

# 24. Registry Governance

EvidenceRegistry maintains multiple independent numeric registries.

At minimum:

```text
Event Type Registry

Record Type Registry

Lifecycle Object Kind Registry

Digest Algorithm Registry

Artifact Kind Registry

Identity Dependency Kind Registry

Record-Type Field Key Registries

Journal Entry Field Key Registry
```

Numeric values have meaning only within their registry.

Identifier registries and field-key registries are distinct registry families as defined in §12.1.

---

# 25. Registry Assignment Rule

For every identifier or field-key registry:

```text
assigned number
→ permanent semantic assignment
```

An assigned number MUST NEVER be reused for different semantics within that registry.

Removed or obsolete assignments remain reserved.

---

# 26. Registry Extension Rule

Adding a new identifier MAY be backward-compatible only where the containing schema and reader rules explicitly permit unknown future values.

Otherwise a format/schema successor is required.

A reader MUST NOT guess unknown identifier semantics.

Unknown authoritative identifiers fail closed.

---

# 27. Registry Change Documentation

Every numeric registry SHOULD be maintained in one machine-readable source suitable for:

```text
code generation

documentation generation

test-vector generation

collision detection

CI reuse checks
```

Manual duplicate-number detection SHOULD NOT be the primary safeguard.

---

# 28. Record Type Registry Boundary

Concrete:

```text
record_type_id
```

assignments belong to the Normative Record Schema Specification.

This Identity Format specification freezes only:

```text
representation = unsigned integer

range = 1..65535

0 reserved

reuse forbidden

per-type field-key namespace semantics
```

Record Type Registry assignments MUST be machine-readable.

---

# 29. Lifecycle Object Kind Numeric Registry

The v0.x Lifecycle Object Kind Registry is:

```text
1  = REGISTRY

2  = FREEZE_ATTEMPT

3  = VERIFICATION

4  = REVIEW_REQUEST

5  = REVIEW_RESULT

6  = REVIEW_ADMISSION_ATTEMPT

7  = POLICY

8  = CLOSEOUT_ATTEMPT

9  = ARTIFACT_EVICTION_ATTEMPT

10 = ASSUMPTION_DEFINITION

11 = ASSUMPTION_ESTABLISHMENT

12 = ASSUMPTION_INVALIDATION

13 = FORMAL_VERIFICATION

14 = ASSUMPTION_VERSION_COMPATIBILITY

15 = ASSUMPTION_VERSION_COMPATIBILITY_INVALIDATION

16 = BOOTSTRAP_TRUST_DECLARATION

17 = BOOTSTRAP_TRUST_INVALIDATION

18 = FORMAL_FINDING_CLASSIFICATION
```

These assignments are permanent.

---

# 30. Event Type Registry

Event Type IDs are imported unchanged from Lifecycle v0.10.2.

This specification does not renumber them.

Representation is:

```text
unsigned integer
```

within the range defined by §12.

---

# 31. Artifact Kind Registry

Artifact Kind assignments remain governed by Lifecycle v0.10.2.

This specification does not renumber them.

The same no-reuse rule applies.

---

# 32. Identity Dependency Kind Registry

`identity_dependencies[]` requires explicit identity-domain separation.

The v0.x registry is:

```text
1 = RECORD_ID

2 = JOURNAL_ANCHOR_ID

3 = RESERVED_RAW_SHA256_DIGEST
```

---

## 32.1 RECORD_ID

Used for content-addressed EvidenceRegistry Records, including where applicable:

```text
Manifest

Policy

Scope

Method

Check

Source Binding

Storage Capability Class

Environment Observation

support Records
```

---

## 32.2 JOURNAL_ANCHOR_ID

Used for the content identity of a portable Journal Anchor.

---

## 32.3 RESERVED_RAW_SHA256_DIGEST

Identifier:

```text
3
```

is permanently reserved for a possible future raw SHA-256 identity dependency domain.

It MUST NOT appear in an authoritative v0.x:

```text
identity_dependencies[]
```

No current v0.x semantic field requires it.

A future specification that activates this identifier MUST define:

```text
permitted use sites

domain meaning

why RECORD_ID or another typed identity is insufficient
```

before it becomes legal.

This reservation prevents an untyped raw digest from becoming a convenience escape hatch around typed identity domains.

---

# 33. Authoritative Record Framing

An authoritative EvidenceRegistry Record uses the exact conceptual structure:

```text
[
  "EvidenceRegistry.Record.v1",
  record_type_id,
  schema_version,
  record_body
]
```

encoded with this specification's deterministic CBOR profile.

---

# 34. Record Domain Separator

The exact domain text is:

```text
EvidenceRegistry.Record.v1
```

UTF-8 payload bytes:

```text
45 76 69 64 65 6e 63 65 52 65 67 69 73 74 72 79
2e 52 65 63 6f 72 64 2e 76 31
```

Hexadecimal:

```text
45766964656e636552656769737472792e5265636f72642e7631
```

There is NO trailing:

```text
0x00
```

The domain is a CBOR text-string array element and is therefore already structurally delimited by CBOR.

---

# 35. Record Body Duplication

The `record_body` map MUST contain:

```text
key 0 = schema_version

key 1 = record_type_id
```

These values MUST exactly match the outer framing values.

Mismatch:

```text
INVALID_RECORD_FRAMING
```

---

# 36. Record ID

Define:

```text
record_bytes
=
deterministic_cbor(
  [
    "EvidenceRegistry.Record.v1",
    record_type_id,
    schema_version,
    record_body
  ]
)
```

Then:

```text
record_id
=
SHA256(record_bytes)
```

`record_id` is represented authoritatively as:

```text
32-byte bstr
```

---

# 37. Record ID Is Not Stored Inside Its Own Body

A Record MUST NOT contain its own `record_id` as an identity-bearing field required to compute that same ID.

A human or storage envelope MAY expose the computed ID externally.

The authoritative Record frame itself is self-hashed.

---

# 38. Authoritative Record File Bytes

Where Records are stored under:

```text
records/<record-id>.cbor
```

the authoritative file contents MUST be exactly:

```text
record_bytes
```

from §36.

A semantically equivalent but non-canonical re-encoding is not the same authoritative Record bytes.

---

# 39. Record Load Validation

A conforming Record loader MUST:

```text
read exact bytes

parse under strict CBOR profile

verify canonical encoding

verify frame length

verify domain separator

verify record_type_id

verify schema_version

verify body duplication

recompute SHA-256

compare recomputed record_id to expected identity
```

Failure at any step prevents:

```text
record_identity_status = VALID
```

---

# 40. Record Authority Validation

After Record identity validation, authority validation MAY determine whether an exact Journal Entry authorizes the `record_id`.

The authority Journal Entry MUST contain:

```text
event_record_id = record_id
```

The Record itself does not contain that future authority Entry identity.

---

# 41. Registry ID Authoritative Representation

A v0.x `registry_id` is:

```text
32 random bytes
```

generated according to Lifecycle CSPRNG requirements.

Authoritative CBOR representation:

```text
byte string length = 32
```

---

## 41.1 Registry ID human projection

Canonical human projection:

```text
er-registry:<64 lowercase hexadecimal characters>
```

The prefix is not present in authoritative CBOR.

---

# 42. Freeze Attempt ID

A v0.x `freeze_attempt_id` is:

```text
32 random bytes
```

Authoritative CBOR representation:

```text
bstr(32)
```

Canonical human projection:

```text
er-freeze-attempt:<64 lowercase hexadecimal characters>
```

---

# 43. Artifact Eviction Attempt ID

A v0.x `eviction_attempt_id` is:

```text
32 random bytes
```

Authoritative CBOR representation:

```text
bstr(32)
```

Canonical human projection:

```text
er-eviction-attempt:<64 lowercase hexadecimal characters>
```

---

# 44. Lifecycle Object ID Representation

All v0.x lifecycle object IDs resolve to one of:

```text
registry_id

freeze_attempt_id

eviction_attempt_id

event_record_id
```

Each is exactly:

```text
32 bytes
```

Therefore Journal Entry field:

```text
lifecycle_object_id
```

is encoded as:

```text
bstr(32)
```

Its semantic domain is determined by:

```text
lifecycle_object_kind_id
```

The raw 32 bytes MUST NOT be interpreted without the kind.

---

# 45. Lifecycle Object ID Derivation

`lifecycle_object_id` is not caller-selectable.

It MUST be derived from `lifecycle_object_kind_id` and the other authoritative Journal Entry fields according to this table.

| lifecycle_object_kind                           | lifecycle_object_id derivation                   |
| ----------------------------------------------- | ------------------------------------------------ |
| `REGISTRY`                                      | `journal_entry_body[1]` = `registry_id`          |
| `FREEZE_ATTEMPT`                                | `journal_entry_body[16]` = `freeze_attempt_id`   |
| `ARTIFACT_EVICTION_ATTEMPT`                     | `journal_entry_body[21]` = `eviction_attempt_id` |
| `VERIFICATION`                                  | `journal_entry_body[5]` = `event_record_id`      |
| `REVIEW_REQUEST`                                | `journal_entry_body[5]`                          |
| `REVIEW_RESULT`                                 | `journal_entry_body[5]`                          |
| `REVIEW_ADMISSION_ATTEMPT`                      | `journal_entry_body[5]`                          |
| `POLICY`                                        | `journal_entry_body[5]`                          |
| `CLOSEOUT_ATTEMPT`                              | `journal_entry_body[5]`                          |
| `ASSUMPTION_DEFINITION`                         | `journal_entry_body[5]`                          |
| `ASSUMPTION_ESTABLISHMENT`                      | `journal_entry_body[5]`                          |
| `ASSUMPTION_INVALIDATION`                       | `journal_entry_body[5]`                          |
| `FORMAL_VERIFICATION`                           | `journal_entry_body[5]`                          |
| `ASSUMPTION_VERSION_COMPATIBILITY`              | `journal_entry_body[5]`                          |
| `ASSUMPTION_VERSION_COMPATIBILITY_INVALIDATION` | `journal_entry_body[5]`                          |
| `BOOTSTRAP_TRUST_DECLARATION`                   | `journal_entry_body[5]`                          |
| `BOOTSTRAP_TRUST_INVALIDATION`                  | `journal_entry_body[5]`                          |
| `FORMAL_FINDING_CLASSIFICATION`                 | `journal_entry_body[5]`                          |

There are:

```text
18
```

normatively covered object kinds.

The one-shot set contains:

```text
15
```

object kinds.

---

## 45.1 Required equality check

For every Journal Entry:

```text
journal_entry_body[9]
==
DerivedLifecycleObjectId(
    journal_entry_body
)
```

MUST hold.

Mismatch is:

```text
INVALID_LIFECYCLE_OBJECT_ID
```

An implementation MUST NOT accept an arbitrary 32-byte value in key 9.

---

# 46. Derived Freeze Root ID

Define:

```text
freeze_root_identity_bytes
=
deterministic_cbor(
  [
    "EvidenceRegistry.FreezeRoot.v1",
    registry_id,
    freeze_attempt_id
  ]
)
```

Then:

```text
intended_root_id
=
SHA256(freeze_root_identity_bytes)
```

Authoritative representation:

```text
bstr(32)
```

---

# 47. Freeze Root Domain Separator

Exact text:

```text
EvidenceRegistry.FreezeRoot.v1
```

UTF-8 payload hex:

```text
45766964656e636552656769737472792e467265657a65526f6f742e7631
```

No trailing zero byte is present.

The derived root ID identifies the Registry-local logical intended root identity.

It does NOT replace the deterministic on-disk root naming rule.

---

# 48. Why intended_root_id Is Retained

`intended_root_id` is intentionally redundant.

It is a deterministic function of:

```text
registry_id

freeze_attempt_id
```

and therefore contributes no independent entropy.

Its purpose is not to introduce new information.

Its purpose is to provide an explicit role-bearing Journal index for:

```text
the logical root intended by this Freeze START
```

as required by Cross-Reference v0.3.

The redundancy permits a strict producer/consumer conformance check:

```text
encoded intended_root_id
==
DerivedIntendedRootId(
    registry_id,
    freeze_attempt_id
)
```

before the START is accepted.

---

## 48.1 Not a physical root locator

`intended_root_id` MUST NOT be interpreted as:

```text
the physical directory name

the filesystem path

proof that the root exists

proof that root creation completed
```

The physical root naming rule remains governed by Lifecycle storage semantics.

---

## 48.2 Removal requires coordinated succession

Because Cross-Reference v0.3 explicitly requires `intended_root_id` as a Freeze START Journal field, v0.x Identity Format MUST retain it.

Removing the field requires at minimum:

```text
Cross-Reference successor

+
Identity Format successor
```

It MUST NOT be silently optimized away by an implementation merely because it is derivable.

---

# 49. JournalReference Semantic Structure

A Journal Reference has exactly five fields:

```text
registry_id

entry_index

entry_hash

event_type_id

event_record_id
```

All five fields are retained.

`registry_id` MUST NOT be omitted merely because Profile L requires same-Registry references.

---

# 50. JournalReference Authoritative Encoding

A JournalReference is encoded as a fixed-length CBOR array:

```text
[
  registry_id,
  entry_index,
  entry_hash,
  event_type_id,
  event_record_id
]
```

with:

```text
array length = 5

registry_id     = bstr(32)

entry_index     = uint <= ER_UINT_MAX

entry_hash      = bstr(32)

event_type_id   = registered uint

event_record_id = bstr(32)
```

No map representation is authoritative for JournalReference v0.x.

---

# 51. Why registry_id Is Retained

Although every valid Profile L Reference used by one Registry repeats the same `registry_id`, the value remains encoded because a JournalReference may be:

```text
exported

embedded in a portable Record

inspected independently

compared outside its original in-memory context
```

Self-contained references are preferred over context-dependent byte savings.

---

# 52. JournalReference Structural Validation

A structurally valid JournalReference MUST satisfy:

```text
array length = 5

registry_id length = 32

entry_index within range

entry_hash length = 32

event_type_id registered or otherwise explicitly
permitted by the current format

event_record_id length = 32
```

This alone does not prove historical validity.

---

# 53. JournalReference Context Validation

Given a retained Journal, a Reference MUST additionally satisfy:

```text
referenced entry exists

referenced entry bytes hash to entry_hash

referenced entry registry_id equals ref.registry_id

referenced entry entry_index equals ref.entry_index

referenced entry event_type_id equals ref.event_type_id

referenced entry event_record_id equals ref.event_record_id
```

Where the Reference is used inside Registry `R`:

```text
ref.registry_id == R.registry_id
```

MUST hold in Profile L.

---

## 53.1 Journal Entry-contained Reference strictly-prior rule

Where a JournalReference is embedded directly inside Journal Entry `E`, including as:

```text
authority dependency

attempt_start_journal_ref

conflicting_terminal_journal_ref

brackets_closeout_authority_ref

freeze_authority_ref

eviction_start_journal_ref
```

the following MUST additionally hold:

```text
ref.entry_index
<
E.entry_index
```

A forward Reference is invalid even if every other field resolves correctly.

Expected classification:

```text
FORWARD_JOURNAL_REFERENCE
```

---

## 53.2 Record-contained Reference strictly-prior rule

For a JournalReference embedded in a Record, the containing Record alone does not provide the comparison index.

Its strictly-prior rule is:

```text
ref.entry_index
<
authority_entry.entry_index
```

where `authority_entry` is the exact Journal Entry that made that Record authoritative.

This is the two-stage rule defined by §4.

---

# 54. Authority Dependency Collection

`authority_dependencies[]` is semantically:

```text
a set of JournalReference values
```

Declaration order has no semantic meaning.

An implementation MUST NOT use input declaration order as authoritative ordering.

---

# 55. Authority Dependency Canonical Comparator

Canonical sorting key:

```text
(
  entry_index ascending,
  entry_hash lexicographic ascending
)
```

The secondary comparator is the unsigned-byte comparator defined in §16.

---

# 56. Same-Index Dependency Rule

Within one `authority_dependencies[]` collection:

```text
at most one dependency may use a given entry_index
```

Therefore:

```text
same entry_index + same entry_hash
    → duplicate → reject

same entry_index + different entry_hash
    → conflicting same-index reference → reject
```

A conforming implementation MUST NOT:

```text
pick one

prefer the first

prefer the last

prefer the lower hash

merge silently
```

---

# 57. Authority Dependency Canonical Order

After validating uniqueness:

```text
sort by entry_index ASC

if a comparison reaches equal entry_index,
sort by entry_hash lexicographically
```

In a valid dependency set the secondary key will not be needed after same-index rejection.

It remains part of the comparator definition so malformed inputs have deterministic comparison semantics before rejection.

---

# 58. Authority Dependency Encoding

Authoritative encoding:

```text
[
  JournalReference,
  JournalReference,
  ...
]
```

as a definite-length CBOR array sorted by §55.

The field itself is always present in every Journal Entry.

If there are no authority dependencies:

```text
[]
```

is the required canonical value.

The key MUST NOT be omitted.

It MUST NOT be encoded as null.

---

# 59. IdentityDependency Semantic Structure

An identity dependency is:

```text
(
  identity_dependency_kind_id,
  identity_bytes
)
```

v0.x usable identity bytes are exactly:

```text
32 bytes
```

for the active kinds defined in §32.

---

# 60. IdentityDependency Authoritative Encoding

Fixed-length array:

```text
[
  identity_dependency_kind_id,
  identity_bytes
]
```

where:

```text
array length = 2

identity_dependency_kind_id = registered and active uint

identity_bytes = bstr(32)
```

Reserved-but-inactive identity kinds are not valid elements.

---

# 61. Identity Dependency Collection

`identity_dependencies[]` is semantically a set.

Declaration order has no meaning.

Canonical comparator:

```text
(
  identity_dependency_kind_id ascending,
  identity_bytes lexicographic ascending
)
```

---

# 62. Identity Dependency Duplicate Rule

Within one collection:

```text
same identity_dependency_kind_id
+
same identity_bytes
```

is a duplicate and MUST be rejected.

The same 32 raw bytes MAY appear under different active identity kinds because domain meaning differs.

---

# 63. Identity Dependency Encoding

Authoritative encoding:

```text
[
  IdentityDependency,
  IdentityDependency,
  ...
]
```

sorted by §61.

The field itself is always present in every Journal Entry.

If there are no identity dependencies:

```text
[]
```

is the required canonical value.

The key MUST NOT be omitted.

It MUST NOT be encoded as null.

---

# 64. Ordered Semantic Lists Must Be Separate

Because dependency collections are sets:

```text
their serialized order carries no domain meaning
```

If a future Record requires an ordered semantic sequence such as:

```text
first reviewer
second reviewer
third reviewer
```

that sequence MUST use a separate explicitly ordered field.

It MUST NOT overload:

```text
authority_dependencies[]

identity_dependencies[]
```

with hidden sequence semantics.

---

# 65. Named Reference and Dependency Collection Dual Binding

Some Journal Entry fields intentionally encode the same underlying identity or Journal Reference in two locations.

Examples include:

```text
attempt_start_journal_ref
+
authority_dependencies[]

conflicting_terminal_journal_ref
+
authority_dependencies[]

brackets_closeout_authority_ref
+
authority_dependencies[]

freeze_authority_ref
+
authority_dependencies[]

eviction_start_journal_ref
+
authority_dependencies[]

pre_eviction_manifest_id
+
identity_dependencies[]

eviction_scope_identity
+
identity_dependencies[]
```

This duplication is intentional.

---

## 65.1 Different semantic roles

The named field supplies:

```text
role
```

For example:

```text
this dependency is the Freeze START

this dependency is the conflicting terminal event

this dependency is the POST-bracket Closeout

this identity is the pre-eviction Manifest
```

The dependency collection supplies:

```text
uniform dependency enumeration

canonical ordering

graph construction

common validation

generic audit
```

A set element alone does not identify which semantic role that dependency plays.

---

## 65.2 Exact-copy requirement

Where this specification requires dual binding:

```text
named field value
```

MUST exactly equal the matching element in the corresponding dependency collection.

One MUST NOT be:

```text
derived approximately

matched by Record ID only

matched by entry_index only

silently substituted
```

---

## 65.3 Neither copy may be optimized away

An implementation MUST NOT remove:

```text
the named field
```

because the dependency collection contains the same value.

It likewise MUST NOT omit:

```text
the dependency collection member
```

because the named field exists.

The two encodings serve different normative purposes.

---

# 66. Journal Entry Framing

Every authoritative Journal Entry uses exact framing:

```text
[
  "EvidenceRegistry.JournalEntry.v1",
  journal_entry_body
]
```

encoded with deterministic CBOR.

---

# 67. Journal Entry Domain Separator

Exact text:

```text
EvidenceRegistry.JournalEntry.v1
```

UTF-8 payload hex:

```text
45766964656e636552656769737472792e4a6f75726e616c456e7472792e7631
```

No trailing:

```text
0x00
```

is present.

---

# 68. Journal Entry Hash

Define:

```text
journal_entry_bytes
=
deterministic_cbor(
  [
    "EvidenceRegistry.JournalEntry.v1",
    journal_entry_body
  ]
)
```

Then:

```text
entry_hash
=
SHA256(journal_entry_bytes)
```

Authoritative representation of an Entry hash is:

```text
bstr(32)
```

The Entry MUST NOT contain its own `entry_hash`.

---

# 69. Journal Entry Common Field Key Registry

`journal_entry_body` is a CBOR map.

The v0.x common key registry is:

```text
0  = schema_version

1  = registry_id

2  = entry_index

3  = prev_entry_hash

4  = event_type_id

5  = event_record_id

6  = identity_dependencies

7  = authority_dependencies

8  = lifecycle_object_kind_id

9  = lifecycle_object_id

10 = storage_capability_class_id

11 = environment_observation_id

12..15 = reserved
```

Keys 12 through 15 MUST be absent in v0.1 Journal Entry schema.

---

# 70. Journal Entry Common-Key Presence Rule

Every authoritative Journal Entry, including GENESIS, MUST contain all keys:

```text
0
1
2
3
4
5
6
7
8
9
10
11
```

No common key in this range is optional.

Thus every Entry contains explicit:

```text
identity_dependencies

authority_dependencies
```

arrays.

When either semantic set is empty:

```text
[]
```

MUST be encoded.

---

## 70.1 Common-key absence

Missing any key 0–11 is:

```text
REQUIRED_FIELD_MISSING
```

The implementation MUST NOT synthesize an omitted empty dependency collection while validating authoritative bytes.

---

## 70.2 Reserved common keys

Keys:

```text
12
13
14
15
```

MUST be absent.

Presence is:

```text
FORBIDDEN_FIELD_PRESENT
```

or equivalent fail-closed validation.

---

# 71. Journal Entry Event-Specific Key Registry

The v0.x event-specific key registry is:

```text
16 = freeze_attempt_id

17 = intended_root_id

18 = attempt_start_journal_ref

19 = conflicting_terminal_journal_ref

20 = brackets_closeout_authority_ref

21 = eviction_attempt_id

22 = freeze_authority_ref

23 = pre_eviction_manifest_id

24 = eviction_scope_identity

25 = eviction_start_journal_ref
```

These assignments are permanent.

A retired field key MUST NOT be reused.

---

# 72. Journal Entry Common Field Types

```text
schema_version
    = uint
    = 1 for JournalEntry.v1

registry_id
    = bstr(32)

entry_index
    = uint <= ER_UINT_MAX

prev_entry_hash
    = null for GENESIS
      otherwise bstr(32)

event_type_id
    = registered uint

event_record_id
    = bstr(32)

identity_dependencies
    = always-present canonical array defined by §63

authority_dependencies
    = always-present canonical array defined by §58

lifecycle_object_kind_id
    = registered uint

lifecycle_object_id
    = bstr(32)
    = exact derived value from §45

storage_capability_class_id
    = bstr(32)

environment_observation_id
    = bstr(32)
```

---

# 73. GENESIS prev_entry_hash

For:

```text
entry_index = 0
```

the event MUST be:

```text
GENESIS
```

and:

```text
prev_entry_hash = null
```

This is the explicit schema-defined null exception permitted by §23.

For every:

```text
entry_index > 0
```

`prev_entry_hash` MUST be:

```text
bstr(32)
```

and MUST equal the SHA-256 hash of the exact preceding Journal Entry bytes.

---

# 74. Journal Entry Presence Matrix

The presence rules consist of two layers.

---

## 74.1 Common keys

For every event ID:

```text
required:
  keys 0–11

forbidden:
  keys 12–15
```

Keys 6 and 7 remain present even when empty.

---

## 74.2 Event-specific keys

|         Event ID | Required event-specific keys | Forbidden event-specific keys |
| ---------------: | ---------------------------- | ----------------------------- |
|                1 | none                         | 16–25                         |
|              100 | 16, 17                       | 18–25                         |
|              101 | 16, 18                       | 17, 19–25                     |
|              102 | 16, 18                       | 17, 19–25                     |
|              103 | 16, 18                       | 17, 19–25                     |
|              104 | 16, 18, 19                   | 17, 20–25                     |
|     200 ordinary | none                         | 16–25                         |
| 200 POST bracket | 20                           | 16–19, 21–25                  |
|              300 | none                         | 16–25                         |
|              301 | none                         | 16–25                         |
|              302 | none                         | 16–25                         |
|              303 | none                         | 16–25                         |
|              400 | none                         | 16–25                         |
|              500 | none                         | 16–25                         |
|              501 | none                         | 16–25                         |
|              600 | none                         | 16–25                         |
|              700 | 21, 22, 23, 24               | 16–20, 25                     |
|              701 | 21, 25                       | 16–20, 22–24                  |
|              702 | 21, 25                       | 16–20, 22–24                  |
|              703 | 21, 25                       | 16–20, 22–24                  |
|          800–808 | none                         | 16–25                         |

A field that is forbidden for the event MUST be absent.

It MUST NOT be encoded as null.

---

# 75. Lifecycle Object ID Validation

After parsing:

```text
lifecycle_object_kind_id
```

the validator MUST use §45 to derive the only legal:

```text
lifecycle_object_id
```

and compare it byte-for-byte to key 9.

This validation applies to every one of the 18 v0.x object kinds.

---

## 75.1 REGISTRY

For:

```text
lifecycle_object_kind_id = REGISTRY
```

required:

```text
body[9] == body[1]
```

---

## 75.2 FREEZE_ATTEMPT

For:

```text
lifecycle_object_kind_id = FREEZE_ATTEMPT
```

required:

```text
body[16] present

body[9] == body[16]
```

---

## 75.3 ARTIFACT_EVICTION_ATTEMPT

For:

```text
lifecycle_object_kind_id
=
ARTIFACT_EVICTION_ATTEMPT
```

required:

```text
body[21] present

body[9] == body[21]
```

---

## 75.4 Record-backed one-shot objects

For each of the 15 one-shot object kinds:

```text
body[9] == body[5]
```

MUST hold.

---

# 76. Freeze Attempt Redundancy Rules

For events 100–104:

```text
journal_entry_body[16]
=
freeze_attempt_id
=
journal_entry_body[9]
```

Mismatch MUST be rejected.

This is the `FREEZE_ATTEMPT` specialization of §75.

---

# 77. Intended Root Validation

For event 100:

```text
journal_entry_body[17]
=
intended_root_id
```

MUST equal the derived value from §§46–47 using:

```text
registry_id

freeze_attempt_id
```

A producer MUST NOT supply an arbitrary intended root ID.

This validation is an intentional redundant conformance check as defined by §48.

---

# 78. Freeze START Reference

For events:

```text
101
102
103
104
```

key:

```text
18 = attempt_start_journal_ref
```

MUST encode a JournalReference to the exact event 100 for the same:

```text
registry_id

freeze_attempt_id
```

That exact Reference MUST also appear in:

```text
authority_dependencies[]
```

The dual binding is intentional under §65.

---

# 79. Conflicting Terminal Reference

For event:

```text
104 FREEZE_COMMIT_REJECTED
```

key:

```text
19 = conflicting_terminal_journal_ref
```

MUST refer to one of:

```text
101 FREEZE_COMMITTED

102 FREEZE_ABORTED_RECOVERY

103 FREEZE_ABORTED_BY_OPERATOR_ASSERTION
```

for the same Freeze Attempt.

The exact Reference MUST appear in:

```text
authority_dependencies[]
```

---

# 80. POST Bracket Reference

For a POST-bracket event 200:

```text
20 = brackets_closeout_authority_ref
```

MUST be present.

It MUST refer to exact event:

```text
500 CLOSEOUT_COMMITTED
```

The exact same JournalReference MUST appear in:

```text
authority_dependencies[]
```

An ordinary Verification MUST NOT contain key 20.

POST intent MUST NOT be inferred from unrelated dependencies.

---

# 81. Eviction Attempt Redundancy

For events 700–703:

```text
journal_entry_body[21]
=
eviction_attempt_id
=
journal_entry_body[9]
```

Mismatch MUST fail closed.

This is the `ARTIFACT_EVICTION_ATTEMPT` specialization of §75.

---

# 82. Eviction START Freeze Reference

For event 700:

```text
22 = freeze_authority_ref
```

MUST be an exact JournalReference to the authoritative Freeze whose embedded bytes are targeted.

That exact Reference MUST appear in:

```text
authority_dependencies[]
```

The named field gives the dependency its semantic role.

The dependency collection exposes the same Reference to generic authority validation.

---

# 83. Eviction Manifest Identity

For event 700:

```text
23 = pre_eviction_manifest_id
```

is encoded as:

```text
bstr(32)
```

and is a `RECORD_ID` identity dependency.

The exact matching:

```text
IdentityDependency(
  RECORD_ID,
  pre_eviction_manifest_id
)
```

MUST appear in:

```text
identity_dependencies[]
```

---

# 84. Eviction Scope Identity

For event 700:

```text
24 = eviction_scope_identity
```

is encoded as:

```text
bstr(32)
```

and MUST identify an immutable content-addressed eviction-scope Record under the future Record Schema.

The matching:

```text
RECORD_ID
```

IdentityDependency MUST appear in:

```text
identity_dependencies[]
```

---

# 85. Eviction START Reference

For events 701–703:

```text
25 = eviction_start_journal_ref
```

MUST refer to the exact event 700 with the same:

```text
registry_id

eviction_attempt_id
```

The exact Reference MUST appear in:

```text
authority_dependencies[]
```

---

# 86. Event Record Identity

Every Journal Entry contains:

```text
event_record_id = bstr(32)
```

The Journal Entry may be structurally valid even if the corresponding Record payload is unavailable.

Full semantic event validation requires loading the exact Record where the event semantics depend on its payload.

---

# 87. Journal Entry Map Validation

A strict Journal Entry validator MUST reject:

```text
missing common key 0–11

unknown common key

reserved key 12–15

unknown event-specific key

field forbidden for event type

missing required event-specific field

wrong field type

non-canonical dependency ordering

duplicate dependency

same-index dependency conflict

non-minimal integer

non-canonical CBOR

wrong lifecycle_object_kind_id

wrong lifecycle_object_id relation

event-specific redundancy mismatch
```

No missing common dependency key may be interpreted as an empty collection.

---

# 88. Journal Entry Filename Is Not Entry Identity

The on-disk Journal filename MAY encode:

```text
entry_index
```

according to the storage layout.

The filename does not participate in:

```text
entry_hash
```

The Journal Entry body contains the authoritative `entry_index`.

A loader SHOULD verify that the storage locator agrees with the body where the canonical filesystem layout requires it.

Locator mismatch is not resolved by rewriting the Entry body.

---

# 89. JournalReference in Record Payloads

The exact JournalReference encoding from §50 MUST also be used when a Record body contains:

```text
observation_start_journal_ref

operation_start_journal_ref

other prior-history Journal References
```

There MUST NOT be a second Record-specific JournalReference encoding.

---

# 90. Chronology Reference Placement

The following references live in Record payloads:

```text
observation_start_journal_ref

operation_start_journal_ref
```

They do NOT live directly in the Journal Entry common/event-specific map in v0.x.

Their structural validation can occur Record-locally.

Their historical and strictly-prior validation requires the Record's exact authority Journal Entry.

---

# 91. Replay-Critical Reference Placement

The following replay-critical references live directly in Journal Entry bodies:

```text
attempt_start_journal_ref

eviction_start_journal_ref

conflicting_terminal_journal_ref

brackets_closeout_authority_ref
```

`freeze_authority_ref` for Eviction START is also directly Journal-indexed because the Cross-Reference requires it for the exact targeted Freeze relation.

They may additionally be represented in event Record payloads where the Record Schema intentionally requires redundant binding.

If both copies exist:

```text
they MUST match exactly
```

---

# 92. POST Intent Representation

POST-bracket intent is represented explicitly by:

```text
brackets_closeout_authority_ref
```

It MUST NOT be inferred from:

```text
dependency count

presence of any arbitrary Closeout dependency

event proximity

Record filename

human prose
```

---

# 93. Journal Anchor Format

A portable Journal Anchor uses exact framing:

```text
[
  "EvidenceRegistry.JournalAnchor.v1",
  schema_version,
  anchor_body
]
```

where:

```text
schema_version = 1
```

---

# 94. Journal Anchor Domain Separator

Exact text:

```text
EvidenceRegistry.JournalAnchor.v1
```

UTF-8 payload hex:

```text
45766964656e636552656769737472792e4a6f75726e616c416e63686f722e7631
```

No trailing zero byte is present.

---

# 95. Journal Anchor Body

`anchor_body` is a deterministic CBOR map:

```text
0 = registry_id

1 = journal_head_index

2 = journal_head_hash

3 = journal_format_version
```

Types:

```text
registry_id
    = bstr(32)

journal_head_index
    = uint <= ER_UINT_MAX

journal_head_hash
    = bstr(32)

journal_format_version
    = uint
```

---

## 95.1 Journal Anchor schema_version placement

Lifecycle's Journal Anchor minimum semantic field set includes:

```text
schema_version
```

Identity Format v0.x places that value in the Anchor framing array:

```text
[
  domain,
  schema_version,
  anchor_body
]
```

rather than duplicating it inside `anchor_body`.

Therefore:

```text
schema_version
```

is present in the authoritative Anchor structure even though it is not an `anchor_body` map key.

This placement is intentional and normative.

---

# 96. Journal Anchor ID

Define:

```text
anchor_bytes
=
deterministic_cbor(
  [
    "EvidenceRegistry.JournalAnchor.v1",
    1,
    anchor_body
  ]
)
```

Then:

```text
anchor_id
=
SHA256(anchor_bytes)
```

Authoritative identity representation:

```text
bstr(32)
```

Identity dependency kind:

```text
JOURNAL_ANCHOR_ID
```

---

# 97. Domain Separator Registry

The current exact domain separators are:

```text
EvidenceRegistry.Record.v1

EvidenceRegistry.JournalEntry.v1

EvidenceRegistry.JournalAnchor.v1

EvidenceRegistry.FreezeRoot.v1
```

These strings are permanent for the formats defined here.

Changing one creates a new identity domain.

---

# 98. Domain Separator UTF-8 Bytes

Exact payload bytes are:

```text
EvidenceRegistry.Record.v1

45766964656e636552656769737472792e5265636f72642e7631
```

```text
EvidenceRegistry.JournalEntry.v1

45766964656e636552656769737472792e4a6f75726e616c456e7472792e7631
```

```text
EvidenceRegistry.JournalAnchor.v1

45766964656e636552656769737472792e4a6f75726e616c416e63686f722e7631
```

```text
EvidenceRegistry.FreezeRoot.v1

45766964656e636552656769737472792e467265657a65526f6f742e7631
```

No listed domain separator includes a trailing NUL byte.

---

# 99. ArtifactSet v2 Domain Difference

The existing derived utility format:

```text
EvidenceRegistry.ArtifactSet.v2
```

may use an explicit trailing:

```text
0x00
```

in its own concatenation framing.

That difference is intentional.

ArtifactSet v2 uses a raw concatenation/domain-separation construction distinct from the CBOR-array framing used by:

```text
Record

JournalEntry

JournalAnchor

FreezeRoot
```

EvidenceRegistry MUST NOT "harmonize" these formats merely for visual consistency.

Changing an existing domain framing would change identity.

---

# 100. No Domain Separator Normalization

Domain separator bytes MUST NOT be:

```text
case folded

trimmed

NUL-terminated unless explicitly specified

Unicode normalized

converted to another encoding
```

The exact bytes define the identity domain.

---

# 101. Framing Shapes Are Intentionally Different

EvidenceRegistry v0.x uses several framing shapes:

```text
Record:
[
  domain,
  record_type_id,
  schema_version,
  body
]

JournalEntry:
[
  domain,
  body
]

JournalAnchor:
[
  domain,
  schema_version,
  body
]

FreezeRoot:
[
  domain,
  registry_id,
  freeze_attempt_id
]
```

These shapes MUST NOT be normalized into one generic envelope.

---

## 101.1 Record framing rationale

Record framing carries:

```text
record_type_id
schema_version
```

outside the body so a reader can identify and dispatch the Record schema at the framing boundary.

The body redundantly repeats both values to detect framing/body mismatch.

---

## 101.2 Journal Entry framing rationale

Journal Entry has one foundational body schema in v0.x.

Its format family/version is domain-separated by:

```text
EvidenceRegistry.JournalEntry.v1
```

while:

```text
body[0] = schema_version
```

is retained as explicit in-body schema data for strict body validation and future controlled evolution.

This redundancy is intentional.

---

## 101.3 Journal Anchor framing rationale

Journal Anchor is a small portable structure whose:

```text
schema_version
```

is placed directly in its frame.

It does not use Record framing and is not a Record type.

---

## 101.4 FreezeRoot framing rationale

FreezeRoot is a deterministic derived identity tuple.

It has no independently extensible body map.

Its version is therefore represented entirely by:

```text
EvidenceRegistry.FreezeRoot.v1
```

Changing its tuple semantics requires a new domain/version.

---

# 102. JSON Projection

JSON is nonauthoritative.

The canonical JSON projection SHOULD use:

```text
named text fields

lowercase hexadecimal human IDs

JSON numbers for values <= ER_UINT_MAX

arrays preserving semantic list order

dependency arrays emitted in canonical dependency order
```

JSON MUST NOT be hashed as a substitute for authoritative CBOR.

---

# 103. JSON JournalReference Projection

Recommended projection:

```json
{
  "registry_id": "er-registry:<hex>",
  "entry_index": 123,
  "entry_hash": "sha256:<hex>",
  "event_type_id": 500,
  "event_record_id": "sha256:<hex>"
}
```

This projection is informational.

The authoritative JournalReference is the fixed CBOR array in §50.

---

# 104. JSON Record ID Projection

Recommended:

```text
sha256:<64 lowercase hex>
```

A JSON consumer MUST NOT assume that a textual `sha256:` identity can be inserted directly into authoritative CBOR without hex decoding to exactly 32 bytes.

---

# 105. Canonical Decoder Requirement

An authoritative decoder MUST reject data that is syntactically valid CBOR but violates this canonical profile.

Examples:

```text
non-minimal integers

indefinite containers

wrong map ordering

duplicate keys

unknown keys for known schema

forbidden null

wrong byte-string length

unexpected tag

floating point in foundational format
```

"Decodable" is not equivalent to "canonical".

---

# 106. Canonical Re-encode Check

An implementation MAY enforce canonicality by:

```text
decode strictly

re-encode canonically

compare exact bytes
```

where safe and unambiguous.

A mismatch means the original bytes are not the canonical authoritative representation.

An implementation MAY instead validate canonicality directly during parsing.

---

# 107. Unknown Schema Version

If an implementation encounters an authoritative structure with an unsupported:

```text
schema_version
```

it MUST NOT silently interpret it using the nearest known schema.

Preferred result:

```text
UNSUPPORTED_SCHEMA
```

or another explicit fail-closed disposition.

---

# 108. Unknown lifecycle_object_kind_id

Unknown authoritative kind ID:

```text
fail closed
```

No heuristic mapping from textual metadata is permitted.

---

# 109. Unknown event_type_id

Unknown authoritative event type:

```text
fail closed
```

Journal replay MUST NOT skip an unknown authoritative event and continue claiming complete lifecycle reconstruction.

---

# 110. Unknown Identity Dependency Kind

Unknown:

```text
identity_dependency_kind_id
```

prevents semantic interpretation of that dependency.

A validator MUST NOT discard the unknown dependency and continue as though the dependency did not exist.

Reserved-but-inactive ID 3 is likewise invalid in v0.x authoritative dependency collections.

---

# 111. Record Key Registry Handoff

The Normative Record Schema Specification MUST assign:

```text
record_type_id

schema_version

type-specific field keys >= 16

field types

required / optional conditions

context symmetry

JournalReference fields

identity dependencies

authority dependencies where relevant
```

for every Record type it freezes.

---

# 112. Policy Schema Handoff

The Normative Record Schema Specification MUST define Policy v0.x requirement composition as:

```text
logical AND
```

Conceptually:

```text
PolicySatisfied(P, E)
iff

every applicable requirement in P
is satisfied
```

There is no implicit OR in Policy v0.x.

---

# 113. No Implicit Policy OR

The following behavior is forbidden unless a future explicit requirement-expression algebra is introduced:

```text
requirement A failed

requirement B passed

therefore gate passed because A OR B was inferred
```

If future Policy needs:

```text
A OR B
```

that requires an explicit normative extension.

It MUST NOT be introduced by implementation convenience.

---

# 114. Policy Field Evaluation Contract

Every normative Policy requirement field defined by the Record Schema MUST have:

```text
schema field identity

evaluator_id

positive vector IDs

negative vector IDs
```

The mapping MUST be machine-readable.

---

# 115. Policy Evaluator Coverage CI

The repository SHOULD mechanically verify:

```text
every normative Policy field
has exactly one declared evaluator contract

every evaluator_id exists

every evaluator has at least one positive vector

every evaluator has at least one negative vector

no vector references an unknown evaluator

no normative field is parsed but unevaluated
```

This coverage checker is itself a qualification-relevant component.

---

# 116. Requirement Evaluator Registry

Evaluator IDs SHOULD belong to their own stable machine-readable registry.

Assigned evaluator IDs MUST NOT be silently reused for different semantics.

The exact evaluator-ID encoding belongs to the Record Schema / implementation qualification layer rather than authoritative identity bytes unless a Record explicitly binds it.

---

# 117. Identity Format Does Not Define Policy Meaning

Sections 112–116 are normative handoff requirements.

The complete Policy field vocabulary and evaluator semantics belong to the Record Schema and governing semantic specifications.

Identity Format MUST NOT become a second Policy specification.

---

# 118. Byte-Level Error Classes

Implementations SHOULD distinguish failures including:

```text
NONCANONICAL_CBOR

DUPLICATE_MAP_KEY

UNKNOWN_FIELD_KEY

FORBIDDEN_FIELD_PRESENT

REQUIRED_FIELD_MISSING

INTEGER_OUT_OF_RANGE

INVALID_BYTE_STRING_LENGTH

INVALID_DOMAIN_SEPARATOR

INVALID_RECORD_FRAMING

RECORD_ID_MISMATCH

INVALID_JOURNAL_ENTRY_FRAMING

JOURNAL_ENTRY_HASH_MISMATCH

INVALID_JOURNAL_REFERENCE

CROSS_REGISTRY_REFERENCE

FORWARD_JOURNAL_REFERENCE

DUPLICATE_AUTHORITY_DEPENDENCY

DUPLICATE_IDENTITY_DEPENDENCY

CONFLICTING_SAME_INDEX_REFERENCE

NONCANONICAL_DEPENDENCY_ORDER

INVALID_LIFECYCLE_OBJECT_KIND

INVALID_LIFECYCLE_OBJECT_ID

EVENT_SPECIFIC_FIELD_MISMATCH

AUTHORITATIVE_RECORD_PAYLOAD_MISSING

UNSUPPORTED_SCHEMA
```

Exact numeric error-code assignments MAY be defined by the implementation/output specification.

---

# 119. Dependency Construction Order

A producer constructing `authority_dependencies[]` for containing Journal Entry `E` MUST:

```text
1. gather semantic dependency References

2. structurally validate each Reference

3. validate each Reference against the retained Registry
   context and require:
     ref.registry_id == E.registry_id
     ref.entry_index < E.entry_index

4. reject duplicate entry_index

5. reject conflicting same-index hash

6. sort by canonical comparator

7. encode definite-length array
```

It MUST NOT sort first and silently deduplicate afterward.

It MUST NOT emit a forward or cross-Registry authority dependency and rely on the consumer to reject it later.

The producer-side check and consumer-side historical validation are independent defenses.

---

# 120. Identity Dependency Construction Order

A producer constructing `identity_dependencies[]` MUST:

```text
1. gather semantic identity dependencies

2. validate identity kind and byte length

3. reject inactive/reserved kinds

4. reject exact duplicates

5. sort by canonical comparator

6. encode definite-length array
```

---

# 121. JournalReference Construction

A JournalReference MUST be constructed from an already-existing exact Journal Entry.

The producer SHOULD derive:

```text
registry_id

entry_index

entry_hash

event_type_id

event_record_id
```

from the authoritative Entry rather than accepting five unrelated caller-supplied values.

This reduces internally inconsistent Reference construction.

---

# 122. JournalReference Equality

Two JournalReferences are exactly equal iff all five fields are equal.

However within one valid same-Registry dependency collection, equal:

```text
entry_index
```

already implies there may be only one reference.

A second Reference at the same index is rejected even before semantic equality is used for set membership.

---

# 123. JournalReference Standalone Portability

Because all five fields are retained, a JournalReference can be displayed or exported without requiring its containing Journal Entry to supply omitted context.

This does not give the Reference authority outside its Registry.

It only preserves self-description.

---

# 124. Prev Hash Is Not JournalReference

`prev_entry_hash` is:

```text
bstr(32)
```

not a JournalReference.

Its semantics are fixed by Journal sequence position:

```text
entry N.prev_entry_hash
=
hash(entry N-1)
```

No event metadata is duplicated inside `prev_entry_hash`.

---

# 125. Journal Chain Validation

For Entry index `N > 0`:

```text
entry_index == previous.entry_index + 1

prev_entry_hash == SHA256(previous canonical bytes)

registry_id == previous.registry_id
```

MUST hold.

Integer overflow MUST be checked before:

```text
previous.entry_index + 1
```

If previous index equals `ER_UINT_MAX`, no successor is legal.

---

# 126. Journal Fork Input

If two candidate Journal Entry files claim the same:

```text
registry_id
+
entry_index
```

with different canonical bytes or different hashes:

```text
JOURNAL_FORK
```

or equivalent fail-closed detection is required.

A loader MUST NOT choose one branch by:

```text
filesystem order

mtime

filename order

larger hash

smaller hash
```

unless operating under an explicitly stronger branch-selection specification.

Profile L v0.x has no such rule.

---

# 127. Same-Index Reference Conflict

The dependency rule in §56 is deliberately stricter than ordinary set duplicate detection.

It exists so corrupted history cannot cause an implementation to decide:

```text
which version of entry index N should this dependency mean?
```

The correct answer is:

```text
reject the dependency collection
```

---

# 128. Record Schema Cannot Know Future Authority Entry

A Record Schema MUST NOT define any required field equivalent to:

```text
this_record_authority_entry_index

this_record_authority_entry_hash
```

where the value would have to refer to the Journal Entry that will later authorize that same Record.

Such a field would create an identity-authority cycle.

---

# 129. Prior Authority References Are Allowed

A Record MAY bind:

```text
previous authority

operation-start head

observation-start head

predecessor Closeout

previous Bootstrap Declaration
```

because those authorities already exist before Record identity is computed.

Backward references are valid.

Self-future references are not.

---

# 130. Journal Entry Can Know Record Identity

A Journal Entry MUST know:

```text
event_record_id
```

before it is serialized.

The corresponding Record therefore exists as an exact identity before Journal authority is granted.

This is expected.

---

# 131. Nonauthoritative Record Before Journal Commit

A valid Record may exist physically before its Journal event commits.

During that interval:

```text
record_identity_status = VALID

authority_status = NONAUTHORITATIVE
```

is valid.

A crash before the ONE_PHASE terminal append does not transform that Record into authority.

---

# 132. Record Reuse After Unpublished Attempt

A content-addressed nonauthoritative Record produced during a crashed ONE_PHASE operation MAY physically remain.

A later operation MAY reuse the exact same immutable Record bytes as input if lifecycle semantics permit.

Authority still arises only through the later legal Journal event.

Physical byte reuse is not historical authority reuse.

---

# 133. Canonical Empty Collections

Where a schema permits an empty collection:

```text
[]
```

is the only canonical empty-array representation.

For Journal Entry common dependency fields:

```text
identity_dependencies

authority_dependencies
```

the field is always present, and `[]` is required when empty.

For Record fields, the Record Schema determines whether:

```text
field absent
```

or:

```text
field present as []
```

is the permitted semantic state.

---

# 134. Empty vs Absent

Identity Format distinguishes:

```text
field absent

field present with empty array

field present with null
```

These are different semantic values.

Journal Entry dependency fields have already been fixed by §70:

```text
present + []
```

when empty.

For Record fields, Record Schema MUST choose exactly which state is permitted.

Implementations MUST NOT normalize one into another during identity computation.

---

# 135. Boolean Values

Boolean fields MUST use CBOR:

```text
false
true
```

They MUST NOT use:

```text
0 / 1 integers

"true" / "false" strings
```

as authoritative substitutes.

---

# 136. Text Enumerations

Foundational authoritative enumerations SHOULD use registered integer IDs rather than text strings.

Where Record Schema uses text by necessity, exact case-sensitive UTF-8 text is identity-bearing.

Implementations MUST NOT normalize it.

---

# 137. Canonical Arrays

Arrays preserve order by definition.

Where an array is semantically a set, this specification or the Record Schema MUST define its canonical ordering.

Where an array is semantically ordered, the declared semantic order is identity-bearing.

No implementation may decide this implicitly.

---

# 138. Canonical Maps

Map key order does not carry domain semantics.

Field identity comes from the integer key assignment.

Canonical serialization emits the map in ascending numeric key order.

---

# 139. Version Fields

Every framed structure has an explicit format version through its domain separator and/or a structural version field.

Current versions:

```text
Record framing:
  EvidenceRegistry.Record.v1
  +
  schema_version

Journal Entry framing:
  EvidenceRegistry.JournalEntry.v1
  +
  journal_entry_body.schema_version

Journal Anchor framing:
  EvidenceRegistry.JournalAnchor.v1
  +
  frame schema_version

Freeze root identity:
  EvidenceRegistry.FreezeRoot.v1
```

The placement differences are intentional as defined by §101.

---

# 140. Version Is Not Mutable Interpretation

A reader MUST NOT interpret:

```text
Record.v1 bytes
```

using:

```text
Record.v2 semantics
```

merely because v2 is newer.

Compatibility requires explicit specification.

---

# 141. Domain Separator Addition

A new domain separator MUST be:

```text
globally distinct within EvidenceRegistry identity formats
```

and documented with exact UTF-8 bytes.

A previously assigned domain separator MUST NOT be reused.

---

# 142. Stable Format Freeze Inputs

Before declaring Identity Format stable, publish machine-readable registries for:

```text
Lifecycle Object Kind IDs

Identity Dependency Kind IDs

Journal Entry field keys

Domain separators

Digest Algorithm IDs

imported Event Type IDs
```

and integrate imported registries for:

```text
Artifact Kind IDs

future Record Type IDs
```

---

# 143. Record Schema Freeze Inputs

The subsequent Record Schema work MUST freeze:

```text
Record Type Registry

per-Record-type key registries

Policy schema

Scope schemas

Method / Check schemas

GENESIS Record

Manifest Record

Freeze Receipt

Source Binding

Verification Record / Observation

Review Request / Result / Admission

Policy Record

Closeout Record

Storage Capability Class

Environment Observation

Eviction Records

Assumption Definition

Assumption Establishment

Assumption Invalidation

Formal Verification

Compatibility

Compatibility Invalidation

Bootstrap Trust Declaration

Bootstrap Trust Invalidation

Formal Finding Classification

Proof Dependency Manifest

Shared Dependency Manifest

Capability Observation Provenance

Eviction Scope Record
```

as applicable to the implementation milestone.

---

# 144. Policy Record First Rule

Before individual Policy fields are frozen, the Record Schema Specification MUST state:

```text
all applicable Policy requirements compose by AND
```

No implicit OR exists in v0.x.

This rule is a semantic handoff and MUST NOT be omitted from the Policy schema.

---

# 145. Policy Field Coverage Metadata

Each normative Policy field definition SHOULD carry machine-readable metadata equivalent to:

```text
field_key

field_name

evaluator_id

positive_vector_ids[]

negative_vector_ids[]
```

A Policy field with no evaluator contract MUST NOT be declared normatively enforceable.

---

# 146. Identity Vector Case Design

Identity-vector case design MAY proceed before this specification freezes.

After this specification is frozen, expected bytes and hashes MUST match it exactly.

At minimum cases SHOULD cover:

```text
minimal Record frame

Record framing mismatch

integer key ordering

unknown Record key

duplicate map key

non-minimal integer

maximum ER_UINT_MAX

ER_UINT_MAX + 1 rejected

registry_id encoding

attempt ID encoding

FreezeRoot derivation

JournalReference encoding

wrong JournalReference length

dependency canonical ordering

empty dependency arrays present

missing identity_dependencies rejected

missing authority_dependencies rejected

duplicate dependency

same-index conflicting hash

identity dependency ordering

inactive RAW_SHA256_DIGEST rejected

Journal GENESIS

Journal ordinary entry

REGISTRY lifecycle_object_id derivation

one-shot lifecycle_object_id derivation

Freeze lifecycle_object_id derivation

Eviction lifecycle_object_id derivation

wrong lifecycle_object_id rejected

Freeze START

Freeze intended_root_id mismatch

Freeze terminal

Freeze rejected commit

POST Verification

Eviction START

Eviction terminal

named/reference dependency mismatch

Journal-side forward reference

Record-local chronology structural validation

authority-context chronology validation

Journal Anchor

authoritative payload missing
```

---

## 146.1 FreezeRoot derivation positive vector

A standalone positive vector MUST exercise only the derived FreezeRoot identity function.

Conceptually:

```text
input:
  registry_id
  freeze_attempt_id

canonical bytes:
  deterministic_cbor(
    [
      "EvidenceRegistry.FreezeRoot.v1",
      registry_id,
      freeze_attempt_id
    ]
  )

expected:
  exact canonical CBOR bytes
  exact SHA-256 intended_root_id
```

This vector is distinct from the full:

```text
Freeze START
```

Journal Entry vector.

Its purpose is to independently verify:

```text
FreezeRoot domain separator bytes

CBOR tuple framing

registry_id placement

freeze_attempt_id placement

SHA-256 derivation
```

before that result is consumed as Journal Entry key 17.

---

# 147. Expected-Byte Vector Freeze

After this Identity Format specification is accepted:

```text
test vectors MUST publish exact canonical CBOR bytes
```

for representative structures.

Expected hashes MUST be computed from those exact bytes.

A test vector MUST NOT provide only:

```text
expected JSON

expected semantic object

expected hash
```

without also publishing the canonical bytes whose hash is expected.

---

# 148. Cross-Implementation Reproduction

At least two independent implementations SHOULD reproduce foundational vectors without sharing serialization code.

Success criterion:

```text
same semantic vector input

→ same canonical bytes

→ same SHA-256
```

A shared bug in one common serializer does not establish independent reproduction.

---

# 149. Vector File Structure

A machine-readable test-vector entry SHOULD contain:

```text
vector_id

format_name

format_version

semantic_input

expected_cbor_hex

expected_sha256_hex
  where the structure is hashed

expected_validation_status

expected_error
  where negative
```

The exact vector container format belongs to the Test Vector Specification.

---

# 150. Negative Vector Principle

Negative vectors are first-class.

They MUST include cases where a permissive implementation might otherwise "repair" malformed input.

Examples:

```text
unsorted dependency list

missing always-present dependency key

duplicate dependency

same-index conflicting hash

unknown field

future Journal Reference

cross-Registry Reference

noncanonical integer

null where absent required

wrong domain separator

wrong object kind

wrong lifecycle_object_id

event-specific forbidden field

named dependency role mismatch

unexpected POST bracket field
```

The expected outcome is rejection, not normalization.

---

# 151. No Repair Before Identity Validation

A validator MUST NOT transform malformed authoritative input into canonical form and then claim the original input was valid.

It MAY produce a diagnostic canonicalized representation separately.

Validation applies to the bytes actually presented.

---

# 152. Canonical Producer vs Strict Consumer

A conforming producer MUST emit only canonical bytes.

A conforming authoritative consumer MUST reject non-canonical bytes.

This symmetry prevents:

```text
producer A emits form X

producer B emits form Y

both readers accept both

but hashes differ
```

---

# 153. Record Identity Status Before Authority

Record identity can be verified without Registry authority.

Therefore:

```text
identity verification
```

SHOULD be available independently of:

```text
authority verification
```

This supports portable nonauthoritative Records.

---

# 154. Authority Requires Journal Context

Registry authority cannot be established from Record bytes alone.

A caller presenting:

```text
Record bytes
+
record_id
```

without the Registry Journal can establish identity only.

It cannot establish Registry authority.

---

# 155. Journal Entry Identity Without Record Payload

Journal Entry hash can be verified from Journal Entry bytes alone.

Therefore:

```text
entry_hash validity
```

does not imply:

```text
event Record payload availability
```

or:

```text
event semantic validity
```

These states MUST remain separate in output.

---

# 156. Anchor Identity Without Registry Availability

A Journal Anchor ID can be recomputed from Anchor bytes alone.

Whether that Anchor relates to a current Registry requires the Registry Journal.

Again:

```text
portable identity
!=
historical relation
```

---

# 157. Exact Byte Preservation

Where EvidenceRegistry exports an authoritative Record or Journal Entry for independent recomputation:

```text
the exact canonical bytes SHOULD be exportable
```

A JSON-only export is insufficient for byte-level independent identity reproduction.

---

# 158. No Host-Endianness Dependence

All integer and byte representations are defined by CBOR or exact byte-string semantics.

No authoritative value may depend on:

```text
CPU endianness

native integer memory layout

Rust struct memory layout

C ABI padding
```

---

# 159. No Language-Enum Ordinal Dependence

Numeric registries in this specification are explicit.

An implementation MUST NOT derive authoritative IDs from:

```text
Rust enum declaration order

Dafny constructor order

C enum compiler defaults

Python Enum auto()

database row order
```

---

# 160. No HashMap Iteration Dependence

Authoritative map or dependency serialization MUST NOT depend on host-language hash-map iteration order.

All authoritative ordering is defined normatively.

---

# 161. No Filesystem Enumeration Dependence

Nothing in this Identity Format permits filesystem enumeration order to determine:

```text
Journal dependency order

Record map order

Manifest order

identity dependency order
```

Manifest path ordering remains governed by Lifecycle/path-profile rules.

---

# 162. Algorithm Agility Boundary

v0.x foundational IDs use SHA-256.

A future successor MAY add algorithms.

It MUST NOT reinterpret existing 32-byte SHA-256 identities under another algorithm.

Algorithm agility is succession, not reinterpretation.

---

# 163. Human Prefixes Are Projections

Prefixes such as:

```text
er-registry:

er-freeze-attempt:

er-eviction-attempt:

sha256:
```

are presentation-level domain hints.

They are not included inside the authoritative bstr values unless an exact enclosing format explicitly says otherwise.

---

# 164. Identity Equality

Two authoritative Record IDs are equal iff their 32 bytes are equal under the same identity domain.

Two raw 32-byte values from different identity domains MUST NOT automatically be treated as semantically interchangeable.

Domain type remains part of semantic identity.

---

# 165. Typed Identity APIs

Production APIs SHOULD use distinct types for:

```text
RegistryId

RecordId

JournalEntryHash

FreezeAttemptId

EvictionAttemptId

JournalAnchorId

RawSha256Digest
```

rather than one generic `[u8; 32]` throughout the program.

This reduces domain-confusion bugs.

---

# 166. Generic Byte Type Is Not Semantic Type

The fact that all current types contain 32 bytes does not mean they share semantics.

The implementation SHOULD require explicit conversion or construction at domain boundaries.

---

# 167. JournalReference Type Safety

A JournalReference SHOULD use typed members conceptually equivalent to:

```text
RegistryId

JournalIndex

JournalEntryHash

EventTypeId

RecordId
```

not untyped integers and byte arrays.

---

# 168. IdentityDependency Type Safety

An IdentityDependency SHOULD preserve:

```text
IdentityDependencyKind

Typed 32-byte identity
```

and reject impossible or inactive kind/value combinations where the schema can determine them.

---

# 169. Common Implementation Validation Order

Recommended validation pipeline:

```text
1. read exact bytes

2. strict CBOR parse

3. canonical encoding validation

4. framing validation

5. primitive range/type validation

6. registry identifier validation

7. common-key and field-presence validation

8. dependency canonicality validation

9. lifecycle_object_id derivation validation

10. identity/hash recomputation

11. authority-context lookup

12. Journal Reference historical validation

13. semantic Record validation

14. Policy/support evaluation
```

For Journal Entry-contained References, stage 12 includes:

```text
strictly-prior validation
```

against the containing Journal Entry.

For Record-contained References, stage 12 requires the exact authority Journal Entry.

Later stages MUST NOT make an earlier invalid stage disappear.

---

# 170. Error Preservation

If multiple validation failures exist, tooling MAY report more than one.

It MUST NOT classify an earlier structural failure as a stronger semantic conclusion.

Example:

```text
Record payload non-canonical
```

does not justify:

```text
product property BLOCKING
```

unless separate valid Evidence establishes that conclusion.

---

# 171. Format Version Evolution

A byte-level change affecting any of the following requires Identity Format successor treatment:

```text
Record framing

Journal Entry framing

JournalReference encoding

dependency collection comparator

dependency element encoding

dependency empty/presence semantics

digest byte representation

Journal Entry field-key meaning

Journal Entry common-key presence

Lifecycle Object Kind numeric assignment

lifecycle_object_id derivation

intended_root_id encoding/removal

domain separator

integer bound semantics
```

---

# 172. Record Schema Evolution Without Identity Format Change

The following MAY evolve through Record Schema succession without changing Identity Format, provided all Identity Format rules remain unchanged:

```text
new Record type assignment

new type-local field key

new Policy field

new Scope Record field

new formal-support Record field
```

The Record's own:

```text
record_type_id

schema_version

field-key registry
```

must represent the change explicitly.

---

# 173. New Record Type Isolation

Adding a new Record type does not alter:

```text
existing Record IDs

existing field-key meanings

JournalReference bytes

Journal Entry framing
```

provided the Identity Format itself remains unchanged.

This separation is a design goal of the two-specification architecture.

---

# 174. Stable Identity Format Criterion

Identity Format v0.3 is frozen because review established:

```text
all authoritative primitives have one canonical encoding

Record framing is cycle-free

Journal Entry framing is exact

JournalReference has one exact five-field encoding

dependency collections are canonical sets

dependency common fields have an exact empty representation

every Journal Entry common key has a defined
presence rule including the empty case

same-index conflicting dependencies fail closed

all common Journal keys are assigned exactly once

all event-specific Journal keys are assigned exactly once

field presence matrix covers all 28 event IDs

all replay-critical references have one exact placement

Record chronology references have one exact placement

every Journal Entry-contained Journal Reference is
strictly prior to its containing Entry

every Record-contained Journal Reference has explicit
two-stage validation

named role references and generic dependency collections
have explicitly distinct purposes and exact-copy rules

lifecycle_object_id derivation is normatively
defined for every lifecycle_object_kind

all 18 lifecycle object kinds are covered by that
derivation

intended_root_id redundancy is intentional,
normatively derived, and not confused with physical path

domain separators have exact bytes

digest representation is unique

integer bounds are explicit

identifier registries prohibit reuse

Record-type field-key namespaces are unambiguous

missing authoritative payload cannot appear validated

Record-local and authority-context Reference validation
are distinct

reserved identity dependency kinds cannot be used
before activation

producer construction rejects forward and cross-Registry
authority dependencies before encoding

FreezeRoot derivation has an independent positive
expected-byte/hash vector requirement
```

No known BLOCKING or MATERIAL finding remains open against this frozen revision.

---

# 175. Next Specification Boundary

After Identity Format freeze, the next normative document is:

```text
EvidenceRegistry Normative Record Schema Specification v0.1
```

Its first tasks are:

```text
Record Type Registry

Policy AND composition

Policy field → evaluator → vector mapping

GENESIS schema

Policy schema

Review / Verification / Closeout schemas

800–808 formal-support schemas

Scope / Method / Check schemas

support auxiliary schemas
```

---

# 176. Next Test Vector Boundary

Identity-vector case design may continue in parallel.

Because this specification is now frozen:

```text
expected CBOR bytes

expected Record IDs

expected Journal Entry hashes

expected JournalReference bytes

expected Journal Anchor IDs

expected FreezeRoot derivation bytes and hash
```

may now be frozen against Identity Format v0.3.

Any later change that modifies those authoritative bytes requires the appropriate explicit successor rather than silent vector regeneration.

---

# 177. Final Identity Principle

> Record identity is established before authority and does not depend on the Journal Entry that will later grant that authority.

> Journal authority points to exact Record identity and exact prior history.

> References to earlier history may live inside Records, but their chronology relative to the Record becomes provable only when the Record is viewed together with the exact Journal Entry that made it authoritative.

> Dependency collections are sets, not accidental sequences. Named fields assign semantic roles to selected dependencies; generic dependency collections provide uniform graph and validation semantics. When both are required, both are authoritative and must agree exactly.

> Derived redundancy is permitted only when its role is explicit. `intended_root_id` does not add entropy and does not name a physical directory; it is an explicit, deterministically checkable Journal binding of the Freeze START's intended logical root.

> If two implementations agree on semantics but produce different authoritative bytes, at least one implementation is wrong.
