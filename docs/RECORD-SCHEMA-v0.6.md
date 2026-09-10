# EvidenceRegistry Record Schema Successor v0.6
## Freeze Commit Minimal Policy-Scope Profile

**Status:** DRAFT SUCCESSOR CANDIDATE — NOT FROZEN, REVIEWED, ACCEPTED, OR IMPLEMENTATION AUTHORITY

**Frozen semantic predecessors:** EvidenceRegistry Record Schema v0.3; EvidenceRegistry Evidence Lifecycle Specification v0.10.2.

**Candidate package companions:** Record Schema v0.4 and v0.5; Lifecycle v0.10.4, v0.10.6, v0.10.7, and proposed v0.10.8; Cross-Reference v0.5, v0.6, and proposed v0.7; proposed Authoritative Store Profile v0.1.

**Scope:** One prospective `POLICY.gate_scope_ref` profile for the `FREEZE_COMMIT` context only. This document does not adopt, modify, or make authoritative any listed candidate.

## 1. Purpose and ownership

Record Schema v0.3 requires `POLICY.gate_scope_ref` but supplies no generic Scope-profile registry, target grammar, or `FREEZE_COMMIT` applicability rule. It also permits an explicitly supported `FREEZE_COMMIT` context with no additional optional requirement field.

This candidate supplies the smallest proposed semantic selection required for a positive Freeze authority lane without generalizing Scope semantics:

```text
an exact, payload-free FREEZE_COMMIT marker profile
+
FREEZE_COMMIT-only Policy support
+
no implicit target, containment, coverage, or evaluator result
```

This is a new Owner-choice rule. It is not a logical consequence of a required `gate_scope_ref` field, a zero-optional-requirement context, or a similarly named Scope rule elsewhere.

## 2. Proposed profile registry entry

The proposed SCOPE profile registry gains exactly:

| `scope_profile_id` | `scope_profile_version` | Stable name |
| ---: | ---: | --- |
| `2` | `1` | `FREEZE_COMMIT_MINIMAL_POLICY_MARKER` |

A SCOPE Record selects this profile only when:

```text
scope_profile_id      == 2
scope_profile_version == 1
scope_payload         == h''
```

`scope_label`, when present, is descriptive only. It MUST NOT affect profile selection, Policy applicability, evaluator dispatch, Policy completion, Freeze authority, Record identity, or Journal authority.

This profile deliberately has no Subject, Manifest, path, filesystem, ACL, containment, inheritance, coverage, selector, or target grammar. It MUST NOT be used to infer any such relation.

## 3. Prospective FREEZE_COMMIT Policy profile

A Policy selects the prospective `FREEZE_COMMIT` minimal profile only if all of the following hold:

```text
P.supported_context_ids == { FREEZE_COMMIT }
P.gate_scope_ref resolves to one exact SCOPE Record under profile 2/version 1
P.review_requirements absent
P.required_method_status absent
P.allowed_finding_states absent
P.verification_requirements absent
P.intervening_event_constraints absent
P.max_journal_distance_to_closeout absent
P.journal_anchor_requirements absent
P.required_bootstrap_scope_ref absent
P.required_formal_finding_classification absent
P.required_establishment_diversity absent
P.closeout_postconditions absent
```

`minimum_durability` remains optional and, if present, retains Record Schema v0.3 evaluator `1008` semantics. No other optional Policy requirement is permitted in this restricted profile.

The exact SCOPE profile validation is a context-specific structural obligation before the §46 evaluator loop. If it fails because the Scope bytes are missing, malformed, wrong-identity, wrong profile/version, or nonempty, the attempted `FREEZE_COMMIT` evaluation is not completed. The implementation MUST NOT manufacture `SATISFIED`, `GATE_UNSATISFIED`, or `GATE_INDETERMINATE` from that failure.

## 4. Proposed applicability and completion rule

For a Policy that selects §3:

```text
FREEZE_COMMIT_MINIMAL_POLICY_MARKER
is not an evaluator requirement.
```

It establishes only that the Policy explicitly chooses the bounded marker profile for this one context. It does not establish Scope coverage or a positive authority conclusion by itself.

After §3 succeeds, the ordinary Record Schema v0.3 §46 loop evaluates every remaining applicable requirement. In this narrow profile that set is:

```text
{}                              when minimum_durability is absent
{ minimum_durability / 1008 }   when minimum_durability is present
```

Therefore, subject to the proposed Lifecycle v0.10.8 Freeze guard and all predecessor structural requirements:

```text
empty applicable set                 -> SATISFIED
1008 passes                           -> SATISFIED
1008 fails                            -> GATE_UNSATISFIED
1008 is indeterminate under its rule  -> GATE_INDETERMINATE
```

This is an explicit prospective selection. It does not resolve generic `POLICY.gate_scope_ref` semantics for any other Scope profile or evaluation context.

## 5. Required conformance vectors

The successor package MUST add vectors proving at least:

```text
POL-FREEZE-MINIMAL-SCOPE-VALID-001
POL-FREEZE-MINIMAL-SCOPE-WRONG-PROFILE-PRECOMPLETE-001
POL-FREEZE-MINIMAL-SCOPE-NONEMPTY-PRECOMPLETE-001
POL-FREEZE-MINIMAL-SCOPE-UNAVAILABLE-PRECOMPLETE-001
POL-FREEZE-MINIMAL-ZERO-REQUIREMENT-SATISFIED-001
POL-FREEZE-MINIMUM-DURABILITY-FAIL-001
POL-FREEZE-MULTICONTEXT-REJECT-001
POL-FREEZE-EXTRA-OPTIONAL-REQUIREMENT-REJECT-001
```

The vectors must show causally that profile/precondition failures reach neither a completed Policy result nor `FREEZE_COMMITTED`; a completed evaluator failure must remain distinct from a pre-completion failure.

## 6. Non-retroactivity and non-claims

This candidate does not change Record framing, Record keys, deterministic CBOR, RecordId derivation, JournalReference derivation, event IDs, the generic Scope model, or any predecessor Record's meaning.

It does not define a generic Policy implementation-capability profile. It does not authorize a Runtime, Freeze, Admission, Journal publication, persistent store, implementation, qualification, release, or adoption. Those claims require the entire candidate package to be reviewed, explicitly adopted, implemented, and independently verified.
