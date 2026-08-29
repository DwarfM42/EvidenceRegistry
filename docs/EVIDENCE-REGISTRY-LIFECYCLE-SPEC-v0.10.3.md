# EvidenceRegistry Evidence Lifecycle Specification v0.10.3
## Review Admission Policy-Satisfaction Legality Closure

**Status:** FREEZE CANDIDATE — NOT YET FROZEN
**Predecessor:** EvidenceRegistry Evidence Lifecycle Specification v0.10.2
**Project:** EvidenceRegistry
**Repository:** `DwarfM42/EvidenceRegistry`
**Implementation candidate:** Rust
**Formal-model candidate:** Dafny
**Primary deployment model:** local-first, cross-platform, filesystem-authoritative CLI/tooling
**Normative language:** MUST / MUST NOT / SHOULD / SHOULD NOT / MAY

---

# 0. Specification Position

This specification is the narrow semantic successor of EvidenceRegistry
Evidence Lifecycle Specification v0.10.2.

Except where this successor explicitly states otherwise, every normative rule
of v0.10.2 remains unchanged.

This successor closes one missing Review Admission semantic edge:

```text
PolicySatisfied(
    exact authoritative Policy,
    REVIEW_ADMISSION,
    exact authoritative evidence
)
        ↓
legal issuance of
REVIEW_ADMISSION_ACCEPTED
```

It additionally fixes the Review Admission disposition of completed
`GATE_UNSATISFIED` and `GATE_INDETERMINATE` Policy evaluations, preserves the
existing §83.1 pre-terminal abort boundary, and requires a faithful
Cross-Reference companion representation.

This successor does not:

* add a Record type;
* add a Record field;
* add a Journal field;
* add a Journal event;
* add an EventType ID;
* alter the v0.x 28-event domain;
* alter RecordId derivation;
* alter JournalReference derivation;
* alter deterministic CBOR encoding;
* alter Policy evaluator IDs;
* redefine existing Policy evaluator semantics;
* define generic `POLICY.gate_scope_ref` applicability;
* define Scope containment, inheritance, or Scope Algebra;
* define a general Policy implementation-capability profile;
* define a universal unavailable-evaluator outcome;
* require a persisted Policy-evaluation-result Record; or
* make Policy satisfaction reconstructible from Journal metadata alone.

The new Review Admission legality semantics defined by this successor are
owned by Lifecycle.

The Cross-Reference successor required by §20.1.1 is a mechanical companion
representation and MUST NOT be treated as the independent semantic source of
those rules.

---

## 0.1 Changes from v0.10.2

v0.10.3 introduces no new lifecycle object, Record type, Journal event, event
identifier, or identity format.

It adds only the following Review Admission semantics:

```text
event-302 PolicySatisfied legality guard                    §82.1
normative prerequisite ordering                             §82.1
mechanical applicable-requirement boundary                  §82.1.2
Policy-evaluation completion boundary                       §83.2
SATISFIED → accepted disposition                            §83.3.1
GATE_UNSATISFIED → rejected disposition                     §83.3.2
GATE_INDETERMINATE → rejected disposition                   §83.3.3
evaluation-result-undetermined → pre-threshold abort        §83.3.4
Cross-Reference companion requirement                       §20.1.1
explicit non-retroactivity                                  §0.2
```

The mapping:

```text
GATE_INDETERMINATE
→
REVIEW_ADMISSION_REJECTED
```

is a new explicit allocation made by v0.10.3.
It is not claimed to have been derivable from v0.10.2.

---

## 0.2 Review Admission non-retroactivity

The Review Admission legality and disposition rules introduced by v0.10.3 are
non-retroactive.

An event 302 recorded under Lifecycle authority predating v0.10.3 MUST NOT be
re-evaluated, reclassified, invalidated, or otherwise reinterpreted solely
because v0.10.3 introduces the Policy-satisfaction legality condition defined
below.

Historical Evidence remains governed by the Lifecycle authority and explicit
compatibility rules applicable when that Evidence was created.

No implementation MAY use v0.10.3, by itself, as authority to rewrite a prior
Review Admission disposition.

This rule specializes the general predecessor/successor compatibility
discipline of §0.

---

## 20.1.1 Event-specific legality-guard companion propagation

Lifecycle v0.10.2 §20.1 does not, by itself, require Cross-Reference
propagation of an event-specific legality guard.

v0.10.3 introduces that requirement.

The event-302 Policy-satisfaction legality rule defined by §82.1 MUST be
propagated into a successor of the normative Event & Transition
Cross-Reference.

The propagation requirement therefore derives from Lifecycle v0.10.3.
It MUST NOT be justified by reinterpreting the predecessor Cross-Reference
delegation as though it already contained this guard.

The Cross-Reference successor MUST preserve all of the following:

1. the positive Policy-satisfaction guard applies to event 302
   `REVIEW_ADMISSION_ACCEPTED` only;
2. event 303 `REVIEW_ADMISSION_REJECTED` does not acquire the same positive
   `PolicySatisfied` requirement;
3. unrelated events do not acquire the guard;
4. the rule MUST NOT be represented as a global:

   ```text
   LegalTransition
   AND
   PolicySatisfied
   ```

   condition applying to all events;
5. Policy identity, Policy RecordId validity, Policy authority, Policy
   authority dependency, or evidence that a Policy was evaluated MUST NOT be
   substituted for `PolicySatisfied`;
6. the existing 28-event domain remains unchanged;
7. all existing EventType identities remain unchanged;
8. existing lifecycle object kind, legal predecessor state, resulting state,
   phase classification, and terminality mappings remain unchanged; and
9. the Cross-Reference successor MUST identify Lifecycle v0.10.3 as the
   semantic source of the event-302 Policy-satisfaction rule.

A conforming implementation MUST NOT claim support for the v0.10.3
event-302 legality rule while relying only on a Cross-Reference representation
that omits the companion guard required by this section.

The authority boundary is intentionally stated in both Lifecycle and the
Cross-Reference successor.

Lifecycle states the propagation obligation.
The Cross-Reference states the corresponding self-limitation at the point
where the mechanical mapping is defined.

This duplication is intentional and MUST NOT be removed merely as redundant
wording.

---

# 82. Review Admission

The existing §82 text remains normative and unchanged.

Review existence is not Review Admission.

Admission MUST check:

* Request authority;
* exact Subject/Freeze binding;
* Manifest identity;
* Review role;
* Review Scope;
* Method;
* required Checks;
* Result Record integrity;
* Policy identity;
* returned Journal Anchor;
* method status requirements.

Missing required identity MUST NOT be reconstructed from prose.

---

## 82.1 Review Admission Policy-satisfaction legality condition

The existing Review Admission checks defined by §82 remain mandatory.
They are not replaced, weakened, subsumed, or made optional by this section.

The ordering below is normative.

The implementation MUST satisfy every existing Review Admission prerequisite
defined by §82 before evaluating the identified authoritative Policy.

Policy evaluation MUST NOT be performed first and MUST NOT independently
determine a Review Admission terminal disposition before the Admission
candidate has passed the existing §82 prerequisites.

This ordering is required so that a published event 303 records the rejection
of a candidate whose Admission prerequisites were themselves already
established, rather than the rejection of an unvalidated candidate.

After those prerequisite checks succeed, and before
`REVIEW_ADMISSION_ACCEPTED` is determined as the terminal disposition, the
implementation MUST evaluate the identified authoritative Policy in the
`REVIEW_ADMISSION` Policy evaluation context.

The Policy evaluation MUST use:

* the exact authoritative Policy identified for the Review Admission;
* the exact authoritative Review Request already validated for that attempt;
* the exact authoritative Review Result already validated for that attempt;
  and
* every additional authoritative Record payload or supporting Evidence
  required by every Policy requirement applicable to `REVIEW_ADMISSION`.

The normative Review Admission flow is therefore:

```text
existing §82 Admission prerequisites
        ↓ all succeed
§82.1.2 applicable Policy requirements determined
        ↓
Policy evaluation in REVIEW_ADMISSION context
        ↓
SATISFIED / GATE_UNSATISFIED / GATE_INDETERMINATE
or no completed result
        ↓
§83.3 terminal disposition or §83.3.4 pre-threshold abort
```

---

### 82.1.1 Meaning of exact authoritative evidence

For §82.1:

```text
exact authoritative evidence
```

means the exact identity-bound authoritative inputs required by the applicable
Policy evaluators after the existing §82 validation obligations have been
satisfied.

It does not define:

* a new Evidence class;
* a new Record type;
* a new identity type;
* a new serialization format;
* a new authority dependency type; or
* a new Journal field.

The phrase identifies the authoritative inputs already required by the
existing Policy and Lifecycle contracts.

---

### 82.1.2 Mechanical determination of applicable requirements

The set of Policy requirements applicable to `REVIEW_ADMISSION` MUST be
determined mechanically from the normative context-applicability rules in
Record Schema §30 and every directly governing field-presence/applicability
rule referenced by that matrix.

An implementation MUST NOT narrow the applicable requirement set by:

* implementation preference;
* unsupported local code paths;
* absence of convenient supporting Evidence;
* expected gate outcome;
* desired Admission disposition; or
* an attempt to avoid invoking an applicable evaluator.

A Policy requirement that is applicable under the normative matrix and
applicable-field rules MUST remain applicable.

A Policy requirement that is not applicable under those rules MUST NOT become
a hidden gate.

Field presence, absence, and context validity remain governed by Record Schema.
This section does not alter those rules.

---

### 82.1.3 Accepted-event legality

Event 302:

```text
REVIEW_ADMISSION_ACCEPTED
```

is legal only if:

```text
PolicySatisfied(
    identified authoritative Policy,
    REVIEW_ADMISSION,
    exact authoritative evidence
)
```

is established.

A legal event 302 therefore requires:

```text
all existing §82 Review Admission prerequisites
+
PolicySatisfied(
    exact Policy,
    REVIEW_ADMISSION,
    exact authoritative evidence
)
```

evaluated in the order required by §82.1.

The following propositions are insufficient individually or collectively as a
substitute for `PolicySatisfied`:

* the Policy exists;
* the Policy is structurally valid;
* the Policy RecordId is valid;
* the Policy has Registry authority;
* the Policy appears as an authority dependency;
* the Policy was selected;
* the Policy evaluator was invoked; or
* the Policy was actually evaluated.

Policy authority and Policy satisfaction remain distinct propositions.

---

### 82.1.4 Event-303 asymmetry

The positive `PolicySatisfied` legality condition in §82.1.3 applies only to
event 302.

Event 303:

```text
REVIEW_ADMISSION_REJECTED
```

MUST NOT be made conditional on positive `PolicySatisfied`.

The fact that events 302 and 303 belong to the same `REVIEW_ADMISSION`
operation does not transfer the accepted-event guard to the rejected terminal
event.

The circumstances under which a completed Policy evaluation selects event 303
are defined by §83.3.

---

# 83. Admission Outcomes

The existing §83 text remains normative except as explicitly specialized by
§§83.2 through 83.6.

Every completed authoritative Admission operation produces:

```text
REVIEW_ADMISSION_ACCEPTED
```

or:

```text
REVIEW_ADMISSION_REJECTED
```

Both remain history.

Repeated Admission attempts remain separately visible.

Admission is ONE_PHASE.

A crash before terminal publication creates no authoritative Admission
disposition, and the Result MAY be submitted again.

That is not silent rewriting, because no authoritative prior disposition
existed.

---

## 83.1 Failure-recording boundary

The existing §83.1 remains normative.

The historical threshold for Review Admission is:

```text
supported Review Result envelope parsed
+
candidate Review Result record_id successfully recomputed
+
terminal disposition determined
```

Once a terminal disposition is determined, it MUST be published as an
immutable Admission Record with its Journal event.

Failures before that point MAY remain ordinary command/input errors as already
defined by §83.1.

Implementations MUST NOT dynamically move this threshold to avoid preserving
inconvenient failure history.

Sections 83.2 through 83.4 make the Policy-evaluation side of that boundary
mechanically explicit.

---

## 83.2 Policy-evaluation completion boundary

For Review Admission v0.10.3, an ordinary Policy evaluation is complete when
the normative Record Schema §46 `EvaluatePolicy` control flow reaches and
returns one of the following completed Policy results:

```text
SATISFIED
GATE_UNSATISFIED
GATE_INDETERMINATE
```

Once one of those results has been returned, an implementation MUST NOT
reclassify the same evaluation as incomplete merely to select a different
Review Admission outcome.

Conversely, where the evaluation cannot reach one of those completed results
because a required authoritative input cannot be obtained, reconstructed, or
consumed as required, the implementation MUST NOT pretend that §46 returned a
completed result.

In particular:

```text
§46 returns SATISFIED / GATE_UNSATISFIED / GATE_INDETERMINATE
    → Policy evaluation completed

required authoritative input prevents §46
from returning any of those three results
    → Policy evaluation not completed
```

The implementation MUST NOT move this boundary according to:

* desired Admission outcome;
* desired exit code;
* whether event 303 would be inconvenient;
* whether retaining a failed disposition would be inconvenient;
* whether an evaluator is expensive;
* whether supporting Evidence is locally available by an easier path; or
* whether a retry might later obtain a more favorable result.

Where the normative contract of an applicable evaluator itself defines a
result for the observed condition, the implementation MUST execute that
contract.

It MUST NOT bypass a defined evaluator result merely in order to classify the
evaluation as incomplete.

---

### 83.2.1 Precondition failure is not a completed Policy result

`POLICY_CONTEXT_UNSUPPORTED` is a precondition failure occurring before the
Record Schema §46 applicable-requirement evaluation loop.

It is therefore not one of the three completed Policy results enumerated in
§83.2.

For Review Admission, a `POLICY_CONTEXT_UNSUPPORTED` disposition means that no
Review Admission terminal disposition has been determined by the Policy
evaluation. It falls under §83.3.4 and MUST NOT be treated as
`GATE_UNSATISFIED` or `GATE_INDETERMINATE`.

This section does not redefine `POLICY_CONTEXT_UNSUPPORTED` or any other
predecessor context-validation rule. Those remain governed by Record Schema.

Any other predecessor-defined disposition that occurs before §46 reaches one
of the three completed results is likewise not a completed Policy result for
the purposes of §83.3.

---

## 83.3 Policy evaluation result to Review Admission disposition

For Review Admission, the implementation MUST distinguish at least:

1. completed Policy evaluation returning `SATISFIED`;
2. completed Policy evaluation returning `GATE_UNSATISFIED`;
3. completed Policy evaluation returning `GATE_INDETERMINATE`; and
4. an attempted evaluation for which no completed Policy result can be
   determined under §83.2.

These cases MUST NOT be collapsed into one generic failure state.

---

### 83.3.1 SATISFIED

In accordance with the prerequisite ordering required by §82.1, if the
identified authoritative Policy completes evaluation with:

```text
SATISFIED
```

in `REVIEW_ADMISSION` context, then the Policy-satisfaction legality guard of
§82.1.3 is satisfied.

Subject to every other applicable Lifecycle requirement:

```text
REVIEW_ADMISSION_ACCEPTED
```

may be determined and published as event 302.

`SATISFIED` does not bypass or replace any other Review Admission prerequisite.

---

### 83.3.2 GATE_UNSATISFIED

If Policy evaluation completes and returns:

```text
GATE_UNSATISFIED
```

then:

```text
PolicySatisfied
```

is not established and event 302 is illegal.

For Review Admission v0.10.3, this completed non-success Policy result
determines the terminal disposition:

```text
REVIEW_ADMISSION_REJECTED
```

The disposition MUST be published as event 303 in accordance with §83 and
§83.1.

Once `GATE_UNSATISFIED` has been determined, an implementation MUST NOT
reclassify the attempt as a pre-threshold silent abort.

---

### 83.3.3 GATE_INDETERMINATE

If Policy evaluation completes and returns:

```text
GATE_INDETERMINATE
```

then:

```text
PolicySatisfied
```

is not established and event 302 is illegal.

For Review Admission v0.10.3, a completed `GATE_INDETERMINATE` result
determines the fail-closed terminal disposition:

```text
REVIEW_ADMISSION_REJECTED
```

The disposition MUST be published as event 303 in accordance with §83 and
§83.1.

This mapping is not derived from Lifecycle v0.10.2 or Record Schema v0.3.

Those predecessor specifications define `GATE_INDETERMINATE` as a Policy
evaluation result but do not select a Review Admission terminal disposition
for that result.

v0.10.3 explicitly allocates a completed `GATE_INDETERMINATE` result to:

```text
REVIEW_ADMISSION_REJECTED
```

The allocation is fail-closed and treats a completed Policy evaluation as a
determined terminal disposition for purposes of §83.1.

The allocation does not redefine the meaning of `GATE_INDETERMINATE` inside
the Policy evaluation model.

It is specific to Review Admission disposition.

An alternative allocation was available:

```text
completed GATE_INDETERMINATE
→ treat terminal disposition as not determined
→ permit pre-threshold abort
```

v0.10.3 does not select that alternative.

Under v0.10.3, once Policy evaluation completes and returns
`GATE_INDETERMINATE`, the Review Admission terminal disposition has been
determined and MUST be preserved as event 303.

---

### 83.3.4 Policy evaluation result not determined

If the Policy evaluation cannot reach a completed result under §83.2, then no
Review Admission terminal disposition has been determined solely from that
attempted Policy evaluation.

This case is distinct from:

```text
GATE_INDETERMINATE
```

because `GATE_INDETERMINATE` is itself a completed Policy evaluation result.

Under the conjunctive historical threshold of §83.1:

```text
supported Review Result envelope parsed
+
candidate Review Result record_id successfully recomputed
+
terminal disposition determined
```

absence of the third condition means the threshold has not been crossed.

The implementation MAY therefore fail closed and terminate the command or
Admission attempt without publishing event 302 or event 303.

Such a pre-threshold abort:

* MUST NOT claim `PolicySatisfied`;
* MUST NOT claim `REVIEW_ADMISSION_ACCEPTED`;
* MUST NOT fabricate `SATISFIED`;
* MUST NOT fabricate `GATE_UNSATISFIED`;
* MUST NOT fabricate `GATE_INDETERMINATE`;
* MUST NOT fabricate event 303 merely to force terminalization; and
* MUST NOT create an authoritative Review Admission disposition.

This section does not define a universal unavailable-Evidence result for other
Policy contexts.

---

## 83.4 Review Admission disposition summary

The decisive distinction is whether Policy evaluation reached a normative
completed result under §83.2.

```text
existing §82 prerequisite failure
before terminal disposition is determined
    → existing fail-closed behavior

Policy evaluation result cannot be determined
    → evaluation did not complete with a normative Policy result
    → includes POLICY_CONTEXT_UNSUPPORTED per §83.2.1
    → §83.1 historical threshold not crossed
    → fail-closed abort permitted
    → no event 302 or 303

--- completed Policy evaluation results below ---

Policy evaluation = GATE_UNSATISFIED
    → PolicySatisfied not established
    → REVIEW_ADMISSION_REJECTED
    → event 303

Policy evaluation = GATE_INDETERMINATE
    → PolicySatisfied not established
    → REVIEW_ADMISSION_REJECTED
    → event 303

Policy evaluation = SATISFIED
+
all other Review Admission requirements satisfied
    → event 302 may be legal
```

An implementation MUST NOT move an evaluation between the incomplete and
completed sides of this boundary according to desired outcome.

Nothing in this summary permits Policy identity, Policy authority, Policy
dependency, or mere Policy evaluation to stand in for Policy satisfaction.

---

## 83.5 Historical re-verification

v0.10.3 does not require a persisted Policy-evaluation-result Record.

Where every authoritative Record payload required by the applicable Policy
evaluation remains available, a verifier MAY re-execute the applicable Policy
evaluation against those exact retained authoritative inputs when performing
later semantic verification of an event-302 legality claim.

If a required authoritative payload is unavailable, the verifier MUST follow
the existing frozen missing-payload and fail-closed rules.

Payload unavailability MUST NOT, by itself:

* retroactively erase a historical Journal event;
* rewrite its original disposition;
* retroactively classify it under v0.10.3 if it predates v0.10.3;
* convert the event into a newly invented state; or
* establish successful fresh semantic re-verification.

v0.10.3 does not create a perpetual independent-reverification guarantee.

Journal metadata alone remains insufficient to establish Policy satisfaction.

---

## 83.6 Event-302 claim boundary

Until the requirements of v0.10.3 are implemented and qualified, an
implementation MUST NOT claim that an authoritative event 302 is semantically
correct merely because:

* its Record structure is valid;
* its Journal structure is valid;
* the Policy RecordId is valid;
* the referenced Policy has authority;
* the Policy appears as an authority dependency; or
* evidence exists that the Policy was evaluated.

A v0.10.3 event-302 semantic-correctness claim requires evidence that:

```text
all applicable Review Admission prerequisites held
+
the exact identified authoritative Policy completed
with SATISFIED in REVIEW_ADMISSION context
```

in addition to every other applicable Lifecycle requirement.

---

# v0.10.3 Successor Impact Boundary

The semantic changes in v0.10.3 require:

```text
Lifecycle semantic successor                      YES
Cross-Reference faithful companion successor       YES
Record Schema successor                            NO
Identity Format successor                          NO
Formal Boundary semantic successor                 NO
new Record field                                   NO
new Record type                                    NO
new evaluator ID                                   NO
new evaluator semantics                            NO
new EventType ID                                   NO
new Journal event                                  NO
new Journal field                                  NO
canonical byte-format change                       NO
RecordId derivation change                         NO
JournalReference derivation change                 NO
persisted Policy-evaluation result                 NO
```

Any additional semantic, identity, schema, or byte-format change requires
independent authority and is outside the scope of v0.10.3.