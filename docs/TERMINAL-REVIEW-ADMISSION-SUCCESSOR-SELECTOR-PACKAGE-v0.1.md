# EvidenceRegistry Terminal Review Admission Successor-Selector Package v0.1

**Status:** DRAFT PACKAGE FOR EXPLICIT OWNER REVIEW — NOT ADOPTED, FROZEN, IMPLEMENTATION AUTHORITY, OR PUBLIC RUNTIME AUTHORITY

## 1. Purpose and exact existing authority binding

This is the smallest proposed package that closes only the retained
successor-selection gap for terminal `REVIEW_ADMISSION` records. It distinguishes
a successor-governed terminal event `302` or `303` from a historical,
discriminator-less predecessor terminal event without deriving applicability
from any existing field or ambient fact.

Its sole adopted-package input is the exact detached adoption record:

```text
docs/FREEZE-RECORD-AUTHORITY-PATH-SUCCESSOR-PACKAGE-v0.1.md
raw-byte SHA-256
= 65813e356eaba89c638475a40c69f728123f4314040f075a09c6be34db14e114
```

That adopted record binds the exact original authority-path successor closure,
including Record Schema v0.4's profile-1/version-1/empty-Scope Review Admission
rule and evaluator `1015`. This package does not rewrite, extend by inference,
or adopt any other candidate document.

## 2. Exact draft package closure

All three members are required. Partial review, partial adoption, Git ancestry,
filename order, source-tree presence, or a matching selector field is not an
authority basis.

| Path | Git blob | Raw-byte SHA-256 | Bytes | Package role |
| --- | --- | --- | ---: | --- |
| `docs/RECORD-SCHEMA-v0.7.md` | `230c0f589b1e471970d54470897168be9714f8fd` | `bc56fc67ad084dc9bddc50e7c97d7068becd10dfbef1339a8de8fbd060a694df` | 6230 | one key-24 selector grammar, fixed adopted-package value, and predecessor structural preservation |
| `docs/EVIDENCE-REGISTRY-LIFECYCLE-SPEC-v0.10.9.md` | `40f22f9edfae8fd363b87b5475168d180315a891` | `3b509cc086565853dcc49b8789966e1f9f1d68f2f94f8c0ac8c93f569c25e4a6` | 7317 | sole semantic producer, replay, event relation, and non-retroactivity rule |
| `docs/CROSS-REFERENCE-v0.8.md` | `9bda50c7c9ff41ff1d4006b8f11bc2a060d72048` | `79d801ce3d51cac5b71ab62bf62c7570a104149a1c467344e0bcd98527103b83` | 3756 | mechanical event and existing Journal RecordId commitment propagation only |

The hashes in this table describe the exact present candidate objects. Any
change to a member invalidates this draft closure and requires new hashes and a
new independent review before an Owner decision.

## 3. Derivation ledger

| ID | Proposition | Classification | Source / rationale |
| --- | --- | --- | --- |
| M-1 | Existing terminal `REVIEW_ADMISSION` Record bytes already determine RecordId, and the existing terminal Journal `event_record_id` already commits that RecordId. | Mechanically implied | Existing Record/Journal identity and terminal-entry grammar. No new Journal selector field is needed. |
| M-2 | Existing direct authority dependencies, event IDs `302`/`303`, event/disposition relation, §82-before-§46 ordering, and §83 mapping remain required. | Mechanically implied / preserved | Frozen Lifecycle v0.10.2 and adopted Lifecycle v0.10.4. |
| M-3 | A discriminator-less historical type-32 Record must keep its predecessor form; absent new bytes cannot be retrospectively supplied. | Mechanically implied preservation | The adopted package is prospective and does not change historical bytes. |
| C-1 | Type-32 key `24` is a canonical `bstr(32)` terminal selector with one value: the raw-byte SHA-256 of the already adopted Authority-Path package adoption record. | Proposed new semantic choice | An existing Record has no retained selector; this draft proposes the smallest explicit retained discriminator for future Owner adoption. |
| C-2 | Exact key-24 value alone is the exclusive successor-selection source. A malformed or other value fails closed without predecessor fallback. | New semantic choice needed to prevent inference | Prevents Policy/context/timing/configuration/Scope/reference resemblance from creating successor applicability. |
| C-3 | Exact key-24 presence requires replay of the adopted exact profile and result/event relation; key-24 absence stops successor replay and preserves predecessor treatment. | New semantic choice needed for non-retroactivity | Connects explicit selection to the already adopted successor without turning the selector into a claimed conclusion. |

No remaining semantic choice is delegated to runtime implementation. The exact
selector representation, its literal binding, producer source, terminal rule,
replay ordering, malformed-value behavior, and discriminator-less predecessor
preservation are all stated by the package members.

## 4. Deliberate exclusions

This package does **not** add or modify:

```text
Scope grammar, applicability, containment, coverage, labels, payloads, or profiles
Policy-context or Review-request-creation authority
Record types, EventTypes, Journal fields, Journal entries, dependencies, or states
Identity Format / RecordId / JournalReference algorithms
Store-profile namespaces or resolution mechanisms
runtime code, tests, CLI behavior, publication, platform qualification, README claims
historical record conversion, marker backfill, or compatibility inference
```

The absence of these additions is intentional. They are unnecessary to identify
an explicitly selected future terminal record, and would broaden this successor
beyond the authorized gap.

## 5. Required independent review and adoption sequence

1. Freeze the candidate as one exact Git commit/tree containing only this
   manifest and the three package members.
2. Independently review the exact candidate bytes, checking both semantic
   completeness and minimality against the adopted package and predecessor
   preservation. Any candidate-byte change makes that review stale.
3. If the review has no blocking finding, prepare a separate detached Owner
   adoption record that names this manifest, every member, exact blob IDs,
   SHA-256 values, sizes, candidate commit/tree, review artifact(s), and an
   explicit disposition.
4. Only an explicit Owner adoption of those exact reviewed bytes authorizes
   implementation. This draft and any review of it do not authorize runtime
   modification or terminal publication.
