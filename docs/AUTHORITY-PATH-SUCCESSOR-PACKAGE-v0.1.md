# EvidenceRegistry Authority-Path Successor Package v0.1

**Status:** DRAFT PACKAGE FOR EXPLICIT OWNER REVIEW — NOT ADOPTED, FROZEN, IMPLEMENTATION AUTHORITY, OR PUBLIC RUNTIME AUTHORITY

## 1. Purpose

This package is the proposed semantic and Store-profile closure needed before implementing the advertised end-to-end authority path:

```text
authoritative Store inputs
-> strict decode and retained replay
-> §82 prerequisites / §46 Policy evaluation
-> terminal authority result
-> canonical Record and Journal publication
-> retained replay and inspection
```

It is a review manifest, not a detached adoption record. No runtime code may use any document listed below as authority until an explicit Owner adoption binds the exact reviewed package bytes.

## 2. Bound frozen predecessors

The current adopted/frozen foundation remains unchanged:

| Path | Git blob | SHA-256 | Bytes | Role |
| --- | --- | --- | ---: | --- |
| `docs/FREEZE-RECORD-LIFECYCLE-v0.10.4-CROSS-REFERENCE-v0.5.md` | `f76b3497ed2ad2f554c0c2e60085a6d029e99e84` | `c712c54e4a67d0a454d78bb38db96ece980ba8ff079e7a7936756c333e467040` | 5300 | Detached adoption record for Lifecycle v0.10.4 and Cross-Reference v0.5 only |
| `docs/EVIDENCE-REGISTRY-LIFECYCLE-SPEC-v0.10.4.md` | `2bce98105b057551412a5f19fec7c88ee3ad9d15` | `5ff7bef38f7652df904c308b3ab69bc81efcefd37cf451a175e99462b0f7d781` | 29864 | Adopted Review Admission semantic successor |
| `docs/CROSS-REFERENCE-v0.5.md` | `5443c7119ce8679eb19c81d9689f854a4b2379a3` | `9254d9406557163239552899bddcf5daa811d3a942e22fcc4a4411de131b6cf0` | 11984 | Adopted Review Admission mechanical companion |
| `docs/EVIDENCE-REGISTRY-LIFECYCLE-SPEC-v0.10.2.md` | `b629e1e964678d6306502cd5f0d7ede901da7269` | `4778e07f27e125fd204f999c6934cfaf90bb87b47f01a47e015b9e709ca7c5f9` | 122171 | Frozen Lifecycle baseline |
| `docs/CROSS-REFERENCE-v0.3.md` | `bff7a1ed75aec750d47e29024abb618e51d38473` | `e9179edcda588d317cebf07d984462a0051bf1a4e165d57f46f30d6e6d098bca` | 87533 | Frozen transition/dependency baseline |
| `docs/IDENTITY-FORMAT-v0.3.md` | `7195b901faf2faca3f23b2fe76859cae04399cef` | `317f257139dca32e1d0a9f8aebd38eb4f831a8a069bdea95cea2d244947f4769` | 93419 | Frozen identity baseline |
| `docs/RECORD-SCHEMA-v0.3.md` | `ec6bb9a5423fca18fc91113942fd4de19531f4e3` | `f12dda3c72763474c2675b2b1d4c427ddda35bfb00fa68eafd5a054ef22051b9` | 124142 | Frozen Record Schema baseline |

The existing adoption record does **not** adopt the additional candidates in §3. Later filenames, Git ancestry, or a candidate's reference to an adopted input are not adoption evidence.

## 3. Candidate package members

Every member is required for the claimed bounded path; partial adoption is not sufficient.

| Path | Git blob | SHA-256 | Bytes | Package role |
| --- | --- | --- | ---: | --- |
| `docs/RECORD-SCHEMA-v0.4.md` | `24ef8e58b675bbaec8016a5c45f026420bd38c36` | `828430df8e99869021f9931ecde387c71cdb3d4e9228b6ab8e9ce8a699f32ebc` | 12649 | Existing candidate: exact Review Admission gate-Scope profile and evaluator 1015 |
| `docs/RECORD-SCHEMA-v0.5.md` | `99531e473583ce473eec7e8944b096817e4175e0` | `a821ead608c1847f721ea88a33df40c713a862d18f95f8bf04e79c955138ee99` | 9046 | Existing candidate: typed Review Package Anchor carriers |
| `docs/RECORD-SCHEMA-v0.6.md` | `88cadfb2f52c68b4b049d10f2439c0a26497eb3c` | `84c11122805026829a6df5614f67c0ab3eb6e428c5f4efe4f93f06372f920951` | 5989 | New candidate: bounded FREEZE_COMMIT Policy marker profile |
| `docs/EVIDENCE-REGISTRY-LIFECYCLE-SPEC-v0.10.6.md` | `e06ec3c514b5c57557c9edf35673f53e46eba857` | `bcb7cfc7b655c2a38a3e5d1e9e3581d7ae8d7d2462dc1a27c76e7ef23ec1b3fe` | 15031 | Existing candidate: atomic Review Admission operation-start binding |
| `docs/EVIDENCE-REGISTRY-LIFECYCLE-SPEC-v0.10.7.md` | `82967e5ca271aeca7d3bb0963b01f11d32b415c2` | `979db8b45286ba89fa4aa4929fa23da215f355064715dc0cb87998eb0dc3367c` | 16410 | Existing candidate: authoritative Review Package Anchor binding and resolution |
| `docs/EVIDENCE-REGISTRY-LIFECYCLE-SPEC-v0.10.8.md` | `64fd8d7c45f8e1e6af0db83143668cc3c3350a41` | `d401db64b4fcad9b04fb2b2f8027ee2a2aa1d861cf60d6e76b5885409051cc5e` | 8388 | New candidate: Manifest profiles, Freeze Policy authority, and event-101 guard |
| `docs/CROSS-REFERENCE-v0.6.md` | `1fbf288d264e313d8d3c48c54a4495288e7366e6` | `b1b071218b6c7d9411a29ce45fe614bdf78bbaa179377e8aadc7ad2017d1bdcc` | 7705 | Existing candidate: Review Package Anchor mechanical companion |
| `docs/CROSS-REFERENCE-v0.7.md` | `3809b13f486f50813c3c249b01b5a3221c40a00c` | `545e5ae047c2da45055bca91fbe8b69c668f371d4789a4f4445431a3e72f64cb` | 3087 | New candidate: event-101 policy-satisfaction mechanical companion |
| `docs/AUTHORITATIVE-STORE-PROFILE-v0.1.md` | `6d2d3ffe03f5fec63e4293fb4d94abedb3cd2983` | `5636a533debc35a2f193317413029d7116d32f8e2087a63d0ebfb51d5eebce02` | 12513 | New candidate: public local Store resolver, prerequisite producer, and publication profile |

The four new document hashes are exact only for the current candidate bytes. Any edit invalidates this table and requires recomputation before review.

## 4. Authority derivation and Owner-choice ledger

| ID | Proposition | Classification | Candidate owner | Why it is needed |
| --- | --- | --- | --- | --- |
| I-1 | Existing §82 prerequisites precede Review Policy evaluation; §46 completes as `SATISFIED`, `GATE_UNSATISFIED`, or `GATE_INDETERMINATE`; v0.10.4 maps completed outcomes and requires terminal publication. | Explicit frozen proposition | No new choice | Review terminal behavior already selected after inputs/evaluation are complete. |
| I-2 | Review Package Anchor must be authoritative rather than caller supplied; operation-start must be atomically selected and cannot be caller selected. | Existing reviewed candidate, not frozen | Lifecycle v0.10.6/v0.10.7 + Record Schema v0.5 + Cross-Reference v0.6 | Required to make the present runtime's Review input route authoritative rather than synthetic. |
| C-1 | Review `gate_scope_ref` uses exact identity equality with the established common Request/Result Review Scope under profile 1/version 1; evaluator 1015 mismatch is `FAIL`. | New Owner choice already drafted, not frozen | Record Schema v0.4 | Frozen v0.3 requires the field but assigns no generic applicability/evaluator semantics. |
| C-2 | `path_identity_profile_id = 1` is `UTF8_STRICT_V1`; `digest_profile_id = 1` is `SHA256_ONLY_V1`. | New Owner choice | Lifecycle v0.10.8 | Frozen profile names and digest algorithm ID exist, but no numeric Profile mapping exists. |
| C-3 | The profile-2 empty Scope marker selects a bounded FREEZE_COMMIT policy with no target/coverage relation and only optional minimum-durability evaluation. | New Owner choice | Record Schema v0.6 | This creates a minimal positive Freeze lane without claiming generic Scope applicability. |
| C-4 | The exact Policy named by Freeze START/Receipt must resolve through the selected Store's exact RecordId-addressed namespace and §46 must return `SATISFIED` before event 101. | New Owner choice | Lifecycle v0.10.8 + Cross-Reference v0.7 | Existing exact policy ID continuity does not independently establish authoritative Policy evaluation or event-101 legality. This candidate intentionally does not add a `POLICY_RECORDED` event-400 prerequisite. |
| C-5 | `GATE_UNSATISFIED`, `GATE_INDETERMINATE`, and pre-completion failures forbid event 101 but do not select a new Freeze terminal event. | New Owner choice, fail-closed minimum | Lifecycle v0.10.8 + Cross-Reference v0.7 | Existing Freeze terminal vocabulary does not map those policy outcomes. |
| C-6 | The fixed local Store namespace, cooperative serialization, no-replace publication, and mandatory replay readback are the supported public Store profile. | New Owner choice | Authoritative Store Profile v0.1 | Frozen exact-byte requirements do not select a public persistence/resolution contract. |

## 5. Prospective implementation mapping

No runtime implementation is authorized by this table yet. After explicit adoption, the intended RED-to-GREEN work is:

| Adopted requirement | Production locus | Required proof |
| --- | --- | --- |
| Review Scope profile/evaluator | `src/lib.rs` Policy decode/dispatch and `src/authoritative_store.rs` Review route | Positive, mismatch, malformed, unsupported, and no-publication tests |
| Review start and Anchor input chain | `src/authoritative_store.rs` acceptance and §82 input resolution | Caller-injection, concurrency, retained-replay, and restart vectors |
| Freeze source, custody, durability, and current-capability workflow | new Store-owned Freeze producer layered on existing START/Receipt/Manifest types | exclusive-create ordering, retained-copy/reference verification, observed-durability, stale-observation, and capability-change vectors |
| Manifest profile validation | typed Manifest decoding/semantic validator | UTF-8, ordering, duplicate, profile, and digest vectors |
| Freeze Policy authority and event-101 guard | authoritative Store Freeze validation | positive Freeze, completed gate non-success, and pre-completion no-commit tests |
| Store profile | public Store API and future CLI | exact-root, publication, conflict, uncertain-receipt, replay/readback, and platform tests |
| Public CLI | `src/main.rs` | real root-backed positive, non-success, inspection, and replay integration tests |

## 6. Required review and adoption sequence

1. Create a normal candidate commit containing only the package members and this manifest; recompute every blob ID, raw-byte SHA-256, size, commit, and tree from Git objects.
2. Perform specification/hostile review against the exact candidate commit and classify each C-* choice explicitly as accepted, rejected, or amended.
3. If approved, create a detached Owner adoption record that names every package member, candidate commit/tree, all hashes/sizes, review artifacts, and exact disposition. Do not amend the existing v0.10.4/v0.5 freeze record.
4. Re-bind the adopted package from immutable Git blobs, then begin runtime implementation with behavior-level RED tests. No candidate itself is implementation authority.

## 7. Explicit non-claims

This manifest does not adopt any rule, prove that a Store operation is correct, qualify a platform, authorize publication, create a production API, or make current README limitations false. The existing public runtime remains structural/read-only until the Owner completes §6 and the later runtime work is independently verified.
