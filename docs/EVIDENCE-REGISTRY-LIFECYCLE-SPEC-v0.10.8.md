# EvidenceRegistry Evidence Lifecycle Specification v0.10.8
## Manifest Profile and Freeze Policy-Satisfaction Closure

**Status:** DRAFT SUCCESSOR CANDIDATE — NOT FROZEN, REVIEWED, ACCEPTED, OR IMPLEMENTATION AUTHORITY

**Frozen semantic predecessors:** EvidenceRegistry Evidence Lifecycle Specification v0.10.2; EvidenceRegistry Record Schema v0.3; EvidenceRegistry Identity Format v0.3; EvidenceRegistry Normative Event & Transition Cross-Reference v0.3.

**Adopted-contract inputs only:** Lifecycle v0.10.4 and Cross-Reference v0.5 remain adopted by their detached freeze record. They remain distinct from this candidate.

**Candidate package companions:** Record Schema v0.4, v0.5, and proposed v0.6; Lifecycle v0.10.6 and v0.10.7; Cross-Reference v0.6 and proposed v0.7; proposed Authoritative Store Profile v0.1.

## 0. Purpose, narrow scope, and Owner selections

This candidate closes only the prospective semantic edges necessary for a bounded positive Freeze authority lane:

```text
numeric Manifest path-profile selection
numeric Manifest digest-profile selection
selected-profile Manifest validation
exact authoritative Policy resolution for FREEZE_COMMIT
completed FREEZE_COMMIT Policy result as a FREEZE_COMMITTED legality guard
```

The following are new Owner-choice propositions, not deductions from frozen field names, current code, or candidate status:

1. `path_identity_profile_id = 1` selects `UTF8_STRICT_V1`.
2. `digest_profile_id = 1` selects an all-`SHA-256` Manifest digest profile.
3. Only the Record Schema v0.6 `FREEZE_COMMIT_MINIMAL_POLICY_MARKER` may support the bounded positive Freeze lane.
4. A prospective `FREEZE_COMMITTED` event is legal only after the exact Policy completes §46 with `SATISFIED` in `FREEZE_COMMIT` context.
5. A completed `GATE_UNSATISFIED` or `GATE_INDETERMINATE`, or any pre-completion failure, forbids `FREEZE_COMMITTED` but does not itself select a new Freeze terminal event.

Nothing here becomes authoritative unless the Owner separately approves an exact reviewed package.

## 1. Manifest profile registry

### 1.1 Path profile

For this candidate only:

| `path_identity_profile_id` | Profile |
| ---: | --- |
| `1` | `UTF8_STRICT_V1` as defined by Lifecycle v0.10.2 §40.1 |

A selected path component MUST be valid UTF-8 and MUST satisfy the pre-existing canonical-path prohibitions: nonempty; not `.`; not `..`; not an absolute root; and not a drive prefix as a relative component.

Manifest artifact paths MUST be strictly increasing under Lifecycle v0.10.2 §41's component-by-component lexicographic comparison of UTF-8 component bytes. Duplicate paths are invalid. The comparison MUST preserve component boundaries; concatenated-path comparison is forbidden.

No other numeric path profile is selected by this candidate. A Manifest carrying another ID is outside this candidate's positive Freeze lane and MUST NOT be treated as `UTF8_STRICT_V1`, `NATIVE_LOSSLESS_V1`, or an equivalent profile by inference.

### 1.2 Digest profile

For this candidate only:

| `digest_profile_id` | Profile |
| ---: | --- |
| `1` | `SHA256_ONLY_V1` |

`SHA256_ONLY_V1` requires every Manifest Artifact Entry to carry:

```text
digest_algorithm_id == 1  // Identity Format v0.3 §15 SHA-256
digest_bytes length  == 32
```

It introduces no cross-algorithm ordering, truncation, normalization, or digest conversion. Any other profile or algorithm is outside this positive Freeze lane.

### 1.3 Receipt and Manifest continuity

For a prospective positive Freeze under this candidate:

```text
FREEZE_RECEIPT.path_identity_profile_id == 1
MANIFEST.path_identity_profile_id       == 1
FREEZE_RECEIPT.path_identity_profile_id == MANIFEST.path_identity_profile_id
```

must hold in addition to every predecessor exact Subject, Manifest, START, Receipt, and Journal-reference binding. A frame-valid Manifest that fails §1 is not a semantically valid Manifest for this candidate.

## 2. Exact Policy authority and FREEZE_COMMIT evaluation

Before a prospective `FREEZE_COMMITTED` terminal disposition can be determined, the implementation MUST resolve the Policy Record named by the exact retained `FREEZE_ATTEMPT_START.policy_record_id` and `FREEZE_RECEIPT.policy_record_id` as follows:

```text
1. retain the predecessor equality of the two policy_record_id values;
2. resolve exact bytes only through the selected authoritative Store profile's
   RecordId-addressed Record namespace;
3. strictly decode the POLICY Record and recompute the exact matching RecordId.
```

A missing, malformed, or wrong-identity candidate does not establish Policy input. It is a pre-completion failure and MUST NOT be relabeled as `SATISFIED`, `GATE_UNSATISFIED`, or `GATE_INDETERMINATE`.

This candidate deliberately adds no `POLICY_RECORDED` event-400 authority or ordering requirement to Freeze. Under the predecessor schema, Freeze START and Receipt carry a prospective `RecordId POLICY`, while event 101's direct authority dependency remains the exact START. A later event-400 relation, cross-Registry test, or prior-order requirement would be a distinct semantic choice and is outside this candidate.

After §§1–2 and all existing Freeze START/Receipt/Journal structural prerequisites succeed, the implementation MUST evaluate the exact Policy under:

```text
Context = FREEZE_COMMIT
Evidence = exact retained FREEZE_ATTEMPT_START + FREEZE_RECEIPT + semantically valid MANIFEST
```

The Policy must select the prospective Record Schema v0.6 minimal Freeze profile. Every applicable evaluator selected by that successor and Record Schema v0.3 §46 remains mandatory.

## 3. Prospective Freeze terminal legality

For this candidate only, event 101 `FREEZE_COMMITTED` is legal only when all of the following hold:

```text
all pre-existing FREEZE_COMMITTED and exact START/Receipt/Manifest continuity rules
+
Manifest validates under §1
+
exact Policy authority resolves under §2
+
EvaluatePolicy(P, FREEZE_COMMIT, Evidence) returns SATISFIED
```

The policy-satisfaction condition is a Lifecycle legality guard, not a replacement for any exact START dependency, Receipt field, policy identity continuity rule, Manifest identity, Journal ordering rule, or durability requirement.

If the evaluation completes as `GATE_UNSATISFIED` or `GATE_INDETERMINATE`, `FREEZE_COMMITTED` MUST NOT be determined or published. This candidate intentionally does not reinterpret either result as an existing abort or `FREEZE_COMMIT_REJECTED`; those event meanings remain governed by predecessors. A caller may return a fail-closed uncommitted result and existing lifecycle rules continue to govern any later recovery or operator-abort operation.

If evaluation cannot complete, no Freeze terminal disposition is selected by this candidate. The implementation MUST NOT manufacture a committed result, a policy result, or a terminal event.

Once event 101 is legally published through the proposed Store profile and passes retained replay, the predecessor Lifecycle v0.10.2 §55 authority rule applies to the resulting Freeze.

## 4. Required vectors

The candidate package MUST include additive vectors for:

```text
MANIFEST-UTF8-PROFILE-VALID-001
MANIFEST-UTF8-INVALID-COMPONENT-REJECT-001
MANIFEST-UTF8-NONCANONICAL-ORDER-REJECT-001
MANIFEST-UTF8-DUPLICATE-PATH-REJECT-001
MANIFEST-DIGEST-SHA256-VALID-001
MANIFEST-DIGEST-PROFILE-UNSUPPORTED-PRECOMPLETE-001
FREEZE-POLICY-AUTHORITY-EXACT-VALID-001
FREEZE-POLICY-AUTHORITY-MISSING-PRECOMPLETE-001
FREEZE-POLICY-AUTHORITY-WRONG-IDENTITY-PRECOMPLETE-001
FREEZE-POLICY-SATISFIED-COMMITTED-001
FREEZE-POLICY-GATE-UNSATISFIED-NO-COMMIT-001
FREEZE-POLICY-GATE-INDETERMINATE-NO-COMMIT-001
FREEZE-POLICY-PRECOMPLETE-NO-COMMIT-001
```

Each no-commit vector must prove that no event 101 Record or Journal entry is constructed or published.

## 5. Non-retroactivity and non-claims

This candidate adds no Record key, Record type, Journal field, event ID, canonical encoding, RecordId derivation, JournalReference derivation, generic Scope algebra, generic Manifest profile, generic Policy capability profile, or historical reinterpretation.

It does not define a universal manifest profile or a broad artifact-ingestion system. It does not authorize external custody, filesystem observations, universal durability, a runtime implementation, terminal Review Admission, release, publication permission, or Owner adoption.
