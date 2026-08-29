# EvidenceRegistry Record Schema Successor v0.4 — Review Admission Exact Gate-Scope Binding

**Status:** Draft successor candidate — not frozen, reviewed, accepted, or implementation authority
**Predecessor:** EvidenceRegistry Record Schema v0.3
**Semantic scope:** `POLICY.gate_scope_ref` applicability for `REVIEW_ADMISSION` only
**Normative language:** MUST / MUST NOT / SHOULD / SHOULD NOT / MAY

---

## 1. Purpose and successor boundary

This successor defines one bounded, profile-specific meaning for the required
`POLICY.gate_scope_ref` field when, and only when, a Policy is evaluated in
`REVIEW_ADMISSION` context.

It supplies the previously missing applicability contract for one exact Review
Admission use case:

```text
POLICY.gate_scope_ref
    ==
validated ReviewRequest.review_scope_ref
    ==
validated ReviewResult.review_scope_ref
```

The equality relation is exact `RecordId SCOPE` identity. It is not semantic
Scope coverage.

This successor is a Record Schema successor because Record Schema owns Policy
applicability, the evaluator registry, evaluator coverage, Policy
field-to-evaluator mapping, and SCOPE Record structure. Lifecycle v0.10.4
continues to own Review Admission prerequisites, completion boundaries, and
terminal disposition. Cross-Reference v0.5 remains a mechanical companion and
must not create this meaning independently.

---

## 2. Explicit exclusions

This successor does **not** define or authorize:

- generic `POLICY.gate_scope_ref` applicability;
- any context other than `REVIEW_ADMISSION`;
- Scope containment, inheritance, coverage, subset/superset, overlap, or
  compatibility;
- Scope Algebra, ACL, query, glob, path, subject, or target language;
- interpretation of nonempty `scope_payload` bytes;
- a semantic meaning for `scope_label`;
- a new Journal field, Record field, Record type, EventType, Journal event,
  canonical-byte rule, RecordId rule, or JournalReference rule;
- a new Lifecycle disposition or a change to Lifecycle v0.10.4;
- a new Cross-Reference mapping or a change to Cross-Reference v0.5;
- any claim of Policy satisfaction, authority, Admission, lifecycle legality,
  runtime qualification, or publication absent all separately required
  authority and implementation evidence.

All predecessor Scope behavior outside this exact profile/context remains
unresolved rather than becoming implied by this successor.

---

## 3. Bounded Scope Profile Registry entry

The following profile is assigned by this successor.

| `scope_profile_id` | `scope_profile_version` | Stable name |
| ---: | ---: | --- |
| `1` | `1` | `REVIEW_ADMISSION_EXACT_REVIEW_SCOPE_BINDING` |

A SCOPE Record under this profile is structurally valid for this successor
only if:

```text
scope_profile_id      == 1
scope_profile_version == 1
scope_payload         == h''
```

`scope_label`, when present, is descriptive only. It MUST NOT affect profile
validation, target extraction, equality, evaluator invocation, evaluator
outcome, Policy satisfaction, authority, Admission, or a Lifecycle
consequence.

The empty payload is intentional. This profile identifies an exact
identity-binding regime; it has no payload target grammar and no hidden
payload interpretation.

A Policy using this profile MUST declare exactly one supported evaluation
context:

```text
supported_context_ids == { REVIEW_ADMISSION }
```

A Policy that names this profile and declares any additional context is outside
this bounded profile. This successor does not supply a generic gate-Scope rule
for that Policy in another context.

---

## 4. Review Admission target extraction

Before the Policy applicability algorithm evaluates this profile, the existing
Lifecycle v0.10.4 §82 Review Admission prerequisites remain mandatory.

In particular, Request/Result Review Scope consistency is established by the
existing Review Admission prerequisite path, not by this successor. The
comparison target exists only after that path has established one exact common
Review Scope:

```text
ReviewRequest.review_scope_ref
    ==
ReviewResult.review_scope_ref
    ==
review_admission_common_scope_ref
```

A Request/Result Review Scope mismatch remains an existing §82 prerequisite
failure. It is not a result of evaluator `1015`, it MUST NOT be reclassified as
`GATE_UNSATISFIED` or `GATE_INDETERMINATE`, and Lifecycle v0.10.4's existing
pre-terminal no-event boundary remains controlling.

---

## 5. Policy gate-Scope applicability guard

### 5.1 Preconditions

For a Policy evaluated under `REVIEW_ADMISSION`, this bounded contract can
enter the ordinary Policy evaluator loop only when all of the following hold:

1. the exact authoritative Policy is available and valid for the operation;
2. its `gate_scope_ref` resolves to the exact typed SCOPE Record described in
   §3;
3. `supported_context_ids` is exactly `{ REVIEW_ADMISSION }`;
4. the existing §82 prerequisite path has established
   `review_admission_common_scope_ref`; and
5. the Scope Record identified by `review_admission_common_scope_ref` resolves
   to the exact typed SCOPE Record described in §3.

Items 2–5 are context-specific structural obligations preceding ordinary
requirement evaluation under the Record Schema §46 algorithm.

If a required Scope or Review target Record cannot be obtained, structurally
consumed, or compared as required, or if profile ID/version/payload does not
select the profile in §3, evaluation MUST stop before evaluator `1015` returns
a result. The implementation MUST NOT fabricate `SATISFIED`,
`GATE_UNSATISFIED`, or `GATE_INDETERMINATE` from that condition.

For Review Admission, Lifecycle v0.10.4's existing non-completed-evaluation
boundary then applies. This successor does not itself select a terminal
Admission disposition for unsupported profile/version, malformed profile
representation, unavailable Scope, or unavailable Review target.

### 5.2 Registered evaluator

The Policy Evaluator Registry gains the following immutable assignment:

```text
1015 = POLICY_REVIEW_ADMISSION_GATE_SCOPE_EXACT_BINDING
```

Evaluator `1015` is applicable only when all §5.1 preconditions have been
established and only in `REVIEW_ADMISSION` context. Existing evaluator IDs,
including evaluator `1001`, retain their predecessor semantics and MUST NOT be
repurposed for this contract.

Evaluator `1015` evaluates:

```text
Policy.gate_scope_ref == review_admission_common_scope_ref
```

using exact `RecordId` byte identity only.

| Condition after §5.1 | Evaluator `1015` result |
| --- | --- |
| exact identities equal | normal success; §46 evaluation continues |
| exact identities differ | `FAIL` |

An exact identity difference is a completed evaluator `FAIL`. Under the
existing Record Schema §46 loop, it yields `GATE_UNSATISFIED`. Under the
existing Lifecycle v0.10.4 Review Admission mapping, that completed result
selects `REVIEW_ADMISSION_REJECTED` and event `303`.

No label comparison, payload decoding, profile substitution, semantic
similarity, Scope coverage, containment, inheritance, or fallback comparison
is permitted.

---

## 6. Applicability and coverage changes

The v0.3 Policy Applicability Matrix gains one bounded row:

| Policy requirement | REVIEW_REQUEST_CREATION | FREEZE_COMMIT | REVIEW_ADMISSION | CLOSEOUT_CREATION | CLOSEOUT_POSTCONDITION |
| --- | ---: | ---: | ---: | ---: | ---: |
| `gate_scope_ref` under profile `1` / version `1` | no | no | yes | no | no |

The v0.3 Policy Field to Evaluator Mapping gains:

| Policy field | Evaluator ID | Positive vector | Negative vector |
| --- | ---: | --- | --- |
| `gate_scope_ref` under profile `1` / version `1` | `1015` | `POL-GATE-SCOPE-RA-EXACT-VALID-001` | `POL-GATE-SCOPE-RA-EXACT-MISMATCH-001` |

The following additional non-success vectors are required to prove the
pre-completion boundary rather than a fabricated evaluator result:

```text
POL-GATE-SCOPE-RA-PROFILE-UNSUPPORTED-PRECOMPLETE-001
POL-GATE-SCOPE-RA-PROFILE-MALFORMED-PRECOMPLETE-001
POL-GATE-SCOPE-RA-TARGET-UNAVAILABLE-PRECOMPLETE-001
POL-GATE-SCOPE-RA-REQUEST-RESULT-SCOPE-PREREQUISITE-FAIL-001
```

Evaluator coverage must show that the first two table outcomes are causal and
that each pre-completion vector reaches neither `SATISFIED`,
`GATE_UNSATISFIED`, nor `GATE_INDETERMINATE`.

### 6.1 Policy Machine Registry and §46 integration

The Policy Machine Registry entry for this bounded Policy field MUST contain:

```text
policy_field_identity       = POLICY.gate_scope_ref/profile-1-version-1
applicable_contexts         = { REVIEW_ADMISSION }
applicable_subfield_rules   = §3 profile validity and §5.1 preconditions
evaluator_id                = 1015
positive_vector_ids         = { POL-GATE-SCOPE-RA-EXACT-VALID-001 }
negative_vector_ids         = { POL-GATE-SCOPE-RA-EXACT-MISMATCH-001 }
material_failure_dimensions = { exact-common-Review-Scope-mismatch }
```

This is a profile-specific semantic use of the existing required
`POLICY.gate_scope_ref` field; it does not add a POLICY Record key or create a
second generic gate-Scope mapping layer.

For the §46 `EvaluatePolicy(P, Context, Evidence)` algorithm, after
`validate context-specific structural obligations` and before its ordinary
applicable-requirement loop, an invocation in `REVIEW_ADMISSION` that selects
the §3 profile MUST apply §5.1. Once §5.1 succeeds, the profile-specific
`gate_scope_ref` requirement is an applicable requirement and the ordinary
loop invokes evaluator `1015`.

If §5.1 cannot succeed, that execution does not enter the evaluator-`1015`
branch. It does not silently omit an applicable requirement: it has failed the
preceding profile/context structural obligation and remains on the
pre-completion side defined in §5.1.

### 6.2 Coverage and vector-family amendments

The §43 evaluator-coverage CI rule, §111 Policy Evaluator Coverage Invariant,
and §112 Positive/Negative Vector Coverage apply to evaluator `1015`.
Accordingly, the §114 Policy Vector Families list gains exactly:

```text
POL-GATE-SCOPE-RA-EXACT-VALID-001
POL-GATE-SCOPE-RA-EXACT-MISMATCH-001
POL-GATE-SCOPE-RA-PROFILE-UNSUPPORTED-PRECOMPLETE-001
POL-GATE-SCOPE-RA-PROFILE-MALFORMED-PRECOMPLETE-001
POL-GATE-SCOPE-RA-TARGET-UNAVAILABLE-PRECOMPLETE-001
POL-GATE-SCOPE-RA-REQUEST-RESULT-SCOPE-PREREQUISITE-FAIL-001
```

The first is the required positive vector. The second is the dedicated
negative vector for evaluator `1015`'s sole material evaluator failure
dimension. The remaining vectors demonstrate that prerequisites outside the
completed evaluator-result grammar cannot be recast as its negative result.

### 6.3 Successor impact and predecessor vectors

```text
VECTORS_INVALIDATED = false
```

This successor adds prospective profile-specific obligations and new vectors;
it does not change the authoritative bytes or the stated valid/invalid result
of a predecessor v0.3 vector. Predecessor vectors remain intact and MUST NOT
be overwritten. The new §6.2 vectors are additional successor obligations.

---

## 7. Lifecycle and Cross-Reference preservation

This successor makes no Lifecycle change.

- Existing Lifecycle §82 continues to validate Review Request/Result Scope
  consistency before Policy evaluation.
- Existing Lifecycle §83.2 continues to distinguish a completed Policy result
  from an evaluation that cannot complete because a required input cannot be
  consumed.
- Existing Lifecycle §83.3.2 continues to map a completed
  `GATE_UNSATISFIED` Review Admission evaluation to event `303`.
- Existing Lifecycle §83.3.3 continues to map a completed
  `GATE_INDETERMINATE` result as already specified; this successor does not
  manufacture that result for unsupported or unavailable Scope conditions.

This successor makes no Cross-Reference change. The existing event-303
mechanical mapping already follows the Lifecycle result for completed
`GATE_UNSATISFIED`; no new event, guard, payload, dependency identity, or table
meaning is introduced here.

---

## 8. Non-retroactivity and qualification boundary

This successor applies only where its exact profile/version and all stated
preconditions are explicitly selected by the governing successor authority.
It MUST NOT be used to reinterpret historical Scope Records, Policies, Review
Requests, Review Results, Admissions, Journal events, or prior frozen
specifications.

A draft or frozen specification alone does not implement evaluator `1015` or
qualify any runtime. Any later implementation must receive independent exact
candidate review and evidence. Until the selected successor is frozen and the
required implementation authority exists, the parent mainline predicate
remains unresolved.
