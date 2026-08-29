# EvidenceRegistry Normative Event & Transition Cross-Reference v0.4
## Event-302 Policy-Satisfaction Guard Companion

**Status:** FREEZE CANDIDATE — NOT YET FROZEN
**Predecessor:** EvidenceRegistry Normative Event & Transition Cross-Reference v0.3
**Normative semantic source:** EvidenceRegistry Evidence Lifecycle Specification v0.10.3
**Purpose:** Mechanical propagation of the v0.10.3 event-302 Policy-satisfaction legality guard
**Normative language:** MUST / MUST NOT / SHOULD / SHOULD NOT / MAY

---

# 0. Companion Position

v0.4 does not create the event-302 Policy-satisfaction rule.

Lifecycle v0.10.3 is the normative semantic source of that rule.

v0.4 exists because Lifecycle v0.10.3 §20.1.1 explicitly requires
Cross-Reference propagation of the event-302 Policy-satisfaction legality
condition.

This companion is therefore derivative from Lifecycle v0.10.3.

It is not an independent Cross-Reference augmentation of the semantic
authority delegated by Lifecycle v0.10.2 §20.1.

If v0.4 is interpreted in a way that creates a Lifecycle legality rule not
present in Lifecycle v0.10.3, that interpretation is invalid.

The authority boundary is intentionally repeated here and in Lifecycle
v0.10.3 §20.1.1.

Lifecycle records the propagation obligation.
This companion records the corresponding self-limitation at the mechanical
mapping boundary.

---

## 0.1 Successor scope

v0.4 changes only the mechanical representation of the event-302 legality
guard required by Lifecycle v0.10.3.

It does not:

* create a new Lifecycle rule;
* add or remove an EventType;
* change EventType identity;
* change operation identity;
* change phase classification;
* change lifecycle object kind;
* change legal predecessor state;
* change resulting state;
* change terminality;
* change authority dependency identity;
* change identity dependency identity;
* change Record schema;
* change Record identity;
* change JournalReference identity;
* change canonical bytes;
* change Policy evaluator IDs; or
* create a persisted Policy-evaluation result.

---

# §23.1 Event-specific Policy-satisfaction guard mapping

This subsection is a normative addition under §23.

This mapping exists because Lifecycle v0.10.3 §20.1.1 explicitly requires it.

It is not an independent Cross-Reference extension of the table dimensions
delegated by Lifecycle v0.10.2 §20.1.

Its semantic authority is derivative from Lifecycle v0.10.3.

The Cross-Reference adds the mechanical dimension:

```text
required_policy_satisfaction_guard
```

The value domain for this v0.4 mapping is:

```text
NONE
REVIEW_ADMISSION_POLICY_SATISFIED
```

The event mapping is:

| event_type_id | event_type_name | required_policy_satisfaction_guard |
| ---: | --- | --- |
| `302` | `REVIEW_ADMISSION_ACCEPTED` | `REVIEW_ADMISSION_POLICY_SATISFIED` |
| every other registered event | unchanged existing event identity | `NONE` |

`REVIEW_ADMISSION_POLICY_SATISFIED` means exactly the Lifecycle v0.10.3
§82.1.3 condition:

```text
PolicySatisfied(
    identified authoritative Policy,
    REVIEW_ADMISSION,
    exact authoritative evidence
)
```

It MUST NOT be given an independent broader or narrower Cross-Reference
meaning.

In particular:

```text
303 REVIEW_ADMISSION_REJECTED
→ required_policy_satisfaction_guard = NONE
```

This mapping does not alter:

```text
event_type_id
event_type_name
operation_name
phase_classification
lifecycle_object_kind
legal_predecessor_state
resulting_state
terminal
authority dependencies
identity dependencies
```

for any registered event.

---

# §70.1 LegalTransition consumption of the Policy-satisfaction guard

This subsection is a normative addition under §70.

Where a mechanical `LegalTransition` model consumes this Cross-Reference, it
MUST consume:

```text
required_policy_satisfaction_guard
```

event-specifically.

Conceptually:

```text
if event_type_id == 302:
    existing LegalTransition conditions
    AND
    PolicySatisfied(
        identified authoritative Policy,
        REVIEW_ADMISSION,
        exact authoritative evidence
    )
else:
    existing LegalTransition conditions
```

The following construction is prohibited:

```text
LegalTransition(all events)
AND
PolicySatisfied(...)
```

The positive Policy guard MUST NOT be generalized to:

* event 303;
* another Review event;
* Closeout;
* Verification;
* Freeze;
* or any unrelated lifecycle transition.

A direct Policy authority dependency is not sufficient to satisfy this
predicate.

The following remain distinct:

```text
Policy identity
Policy authority
Policy authority dependency
Policy was evaluated
PolicySatisfied
```

Only the last proposition satisfies the event-302 guard.

---

## §70.1.1 Completion-boundary correspondence

Where the mechanical model represents Review Admission disposition, it MUST
preserve the Lifecycle v0.10.3 §83.2 distinction between:

```text
Policy evaluation completed with a normative result
```

and:

```text
Policy evaluation did not reach a normative completed result
```

The model MUST NOT permit an implementation to classify:

```text
GATE_INDETERMINATE
```

as an incomplete evaluation after the evaluator has returned that result.

The model MUST NOT require event 302 or 303 where Lifecycle v0.10.3 §83.3.4
permits pre-threshold abort because no terminal disposition was determined.

This subsection does not create new Policy-result semantics.
It faithfully propagates the Lifecycle boundary.

---

## §70.1.2 Admission-disposition mapping

For the Review Admission operation, the mechanical companion relation is:

```text
Policy evaluation result cannot be determined
    → no terminal Admission event required by this mapping

GATE_UNSATISFIED
    → event 303

GATE_INDETERMINATE
    → event 303

SATISFIED
+ all other legal Review Admission conditions
    → event 302 may be legal
```

The mapping is derivative from Lifecycle v0.10.3 §§83.2 through 83.4.

It MUST NOT be generalized into Policy disposition semantics for other
Lifecycle operations.

---

# §80.1 Replay and payload boundary

This subsection is a normative addition under §80.

The event-302 Policy-satisfaction guard does not become Journal-only
replayable merely because it is represented in this Cross-Reference.

A:

```text
policy_ref
```

authority dependency, event mapping, or Journal event identity identifies the
relevant authority and transition relation.

It does not establish:

```text
PolicySatisfied
```

Semantic Policy satisfaction still requires the exact authoritative Record
payloads and supporting Evidence required by Lifecycle v0.10.3 and the
applicable Policy evaluators.

Missing required authoritative payloads remain governed by the existing
fail-closed missing-payload rules.

The Cross-Reference MUST NOT derive a successful event-302 Policy guard from
Journal metadata alone.

---

# v0.4 Event-Domain Preservation

v0.4 adds no registered event.

The v0.x event domain remains exactly 28 events.

All existing EventType IDs and EventType names remain unchanged.

In particular:

```text
302 = REVIEW_ADMISSION_ACCEPTED
303 = REVIEW_ADMISSION_REJECTED
```

retain their existing identities.

---

# v0.4 Semantic-Source Rule

If a future Lifecycle successor changes or removes the event-302
Policy-satisfaction legality condition, this Cross-Reference has no
independent authority to preserve an incompatible older semantic rule.

A future Cross-Reference successor MUST follow the then-governing Lifecycle
semantic source.

Cross-Reference representation follows Lifecycle meaning.
It does not own that meaning.

---

# v0.4 Impact Boundary

v0.4 changes only the mechanical event-to-guard and disposition mapping
required by Lifecycle v0.10.3.

It does not change:

```text
EventType domain
EventType identity
Record schema
Record identity
Journal schema
JournalReference identity
canonical bytes
Policy evaluator IDs
Policy evaluator semantics
authority dependency identity
identity dependency identity
historical Evidence
```

No identity test vector changes solely because of v0.4 unless a future
implementation-specific conformance artifact chooses to encode this
Cross-Reference representation into a separately versioned machine-readable
format.

Such an encoding would require its own authority and is not silently created
by this specification.

---

# Joint Lifecycle v0.10.3 / Cross-Reference v0.4 Claim Boundary

Until both:

```text
Lifecycle v0.10.3
```

and its required:

```text
Cross-Reference v0.4 companion
```

are frozen, implemented, and qualified to the applicable review strength, the
runtime MUST NOT claim conformance to the v0.10.3 event-302
Policy-satisfaction legality rule.

In particular, structural validity of an event 302 does not establish:

```text
PolicySatisfied
```

and Policy authority or authority dependency does not substitute for that
predicate.

The new rule affects prospective v0.10.3 semantics only.
It does not retroactively reinterpret predecessor Lifecycle history.