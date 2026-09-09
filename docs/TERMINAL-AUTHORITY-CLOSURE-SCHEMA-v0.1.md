# EvidenceRegistry Terminal Authority Closure Schema v0.1

Status: CONDITIONAL SUCCESSOR DRAFT. NOT ADOPTED. NOT IMPLEMENTATION AUTHORITY.

## 1. Exact core binding and ownership

This structural/mechanical companion is conditional on adoption with TERMINAL-AUTHORITY-CLOSURE-CORE-v0.1.md and TERMINAL-AUTHORITY-CLOSURE-PLAN-v0.1.md. Core owns semantics and prospective dispatch. This companion owns the following assignments; no field-name inference or extension bag is allowed.

D = SHA256(raw bytes of docs/TERMINAL-AUTHORITY-CLOSURE-CORE-v0.1.md)

`cb7ee9e9e5f9d0229dba26012ecb5231c26771a6d54c3c95762c1094ec1fc16d`

D is encoded as canonical CBOR bstr(32), not 64-byte hex text. The displayed hexadecimal is a diagnostic projection of those exact bytes. Core byte changes require a new binding and fresh whole-package reviews; no digest normalization is permitted.

## 2. Exact grammar delta

All existing Record domain/framing, body keys 0=schema_version and 1=record_type, deterministic restricted CBOR, required/optional fields, sorted-set uniqueness and unknown-key rejection remain controlling. Enclosing frame/type/body versions remain 1 for these explicitly extended variants. No arbitrary version-2 inference exists.

| Record type | Body key | New field | Type/value | Presence |
|---|---:|---|---|---|
| 4 FREEZE_RECEIPT | 32 | terminal_authority_closure_sha256 | bstr(32), exactly D | required for new selected Freeze; absent retains predecessor form |
| 30 REVIEW_REQUEST | 26 | terminal_authority_closure_sha256 | bstr(32), exactly D | required for new selected Request; absent retains predecessor form |
| 32 REVIEW_ADMISSION | 25 | terminal_authority_closure_sha256 | bstr(32), exactly D | required for new terminal; absent selects only pre-existing forms |

Receipt keys 16..31 retain their types and presence rules; new selected Receipt key30 must be absent under Core capability restriction, and keys22/24 obey Core's typed profile relations. Request key24 remains operation_start_journal_ref; key25 remains review_package_anchor_binding_version=1. Result key29 remains that version=1, and key27 transports the Request key23 Anchor carrier exactly. Result gains NO new field. There is no collision between same numeric keys on different Record types.

New terminal requires BOTH type32 key24 with exact legacy literal `65813e356eaba89c638475a40c69f728123f4314040f075a09c6be34db14e114` AND key25=D. Type32 key24 alone preserves precisely its adopted v0.7/v0.10.9 lane. All original keys16..23 and accepted/rejected field combinations remain unchanged. Only completed new-lane outcomes are producible; required Request, Result and Policy authority references must therefore be present for both its accepted and rejected completions. Failed-candidate fields20/21 are absent in this completed-input lane; the predecessor's other valid representations are not converted or erased.

Closed grammar rejects duplicate keys, missing required keys, wrong Record type or version, unknown extra keys, noncanonical map ordering, indefinite-length encodings, nonshortest integers, wrong width/type/value, null/array/text aliases and trailing bytes. Decode/RecordId checking is structural only. Local constructors/decoders may represent these bytes for tests or untrusted ingress, but no public authoritative producer takes a caller selector or final conclusion; Store owns selected construction and all contextual validation.

## 3. New required creation parameter Record

Assign Record Type 64, symbolic name FREEZE_CREATION_PROFILE, first schema version 1, not retired. This assignment is limited to this adopted package and does not make unassigned IDs generally legal. No EventType maps to type64; it is a content-addressed static parameter, not an event or evidence of completed creation.

Its entire Record is deterministic CBOR of:

`["EvidenceRegistry.Record.v1", 64, 1, {0: 1, 1: 64, 16: 1}]`

Body key16 is UInt profile_id and has exactly one assigned value 1, denoting Core section4. Every body key is required; no optional label, extension or other value is allowed. Its RecordId is computed by unchanged Identity Format from those complete bytes. Receipt.creation_profile_ref must resolve those exact bytes from records/; a matching opaque digest without target bytes is not sufficient. Receipt.filesystem_profile_ref remains a RecordId and resolves the exact event101 type61 environment observation under Core section4; type60 or generic SCOPE substitution is invalid.

No new grammar is assigned to requested_commit_durability_ref. Its absence is mandatory only for this selected capability. Present values must not be silently parsed-and-ignored as supported.

### 3.1 Existing Subject and GENESIS carriers under the selected contract

MANIFEST.subject_id, START.subject_id and Receipt.subject_id remain OpaqueId32/bstr32 structurally. Core section3.1 assigns the selected content projection and equality/recomputation rules; none becomes a RecordId or a new independently supplied aggregate. Artifact rows remain exactly Schema v0.3 section53 and paths remain nonempty arrays of component bstr values (section14), not arrays of CBOR text. Selected source/Manifest counts and all scalar bounds remain frozen. No new START or Manifest marker is assigned.

GENESIS keys17 journal_format_version and18 record_identity_profile_id remain UInt. For the new selected Store profile only, both must equal1 with the exact algorithm meanings assigned by Core section6; this is not inferred from the distinct outer/body Record schema_version. The retained GENESIS value, not an ambient default, supplies the prefix-derived JournalAnchor's journal_format_version. Old generic scalar decoding and all unselected history are preserved; a numeric1 never implies D selection.

## 4. Policy schema applicability delta

Generic Policy registration remains Schema v0.3: sorted unique nonempty contexts and all existing requirement/dead-field rules, no self-evaluation at event400. Operation selection is separate.

New selected Freeze permits profile2/version1/empty gate Scope and exact contexts [1], with all optional requirement fields absent. This restricts the new operation's supported capability, not generic validity of the old minimum-durability profile.

New selected Review Request/Admission permits profile1/version1/empty gate Scope and exact contexts [2,5], required nonempty review_requirements and all ordinary applicable requirement classes/fields as defined by the adopted context matrix. Context5 gate_scope role is structural-only and evaluates ordinary selector1001 plus context coverage, NOT 1015. Context2 retains all applicable evaluators including 1015. No change to evaluator IDs, outcome composition, generic other-context applicability or the old exact [2] lane. Scope labels remain exact hashed body data with no independent scope-coverage interpretation as clarified by Core section5.

## 5. Mechanical event and dependency matrix

| Event | Event Record | New-lane selection | Required pre-publication relation |
|---|---|---|---|
| 1 GENESIS | type1 | Store initialization, no new marker | exact typed GENESIS duplicate, initial epoch/observations, slot0 and exact replay |
| 100 FREEZE_ATTEMPT_STARTED | type3 | unchanged START; no new marker | existing exact Attempt/FreezeRoot/Subject/Policy and OPEN transition before physical root |
| 101 FREEZE_COMMITTED | type4 | key32=D | exact retained START/dependencies, selected physical/profile/identity checks, completed Freeze SATISFIED; new positive replay |
| 400 POLICY_RECORDED | type40 | unchanged generic registration | existing structural/context/dead-field/authority rules; no dependence on future Request marker |
| 300 REVIEW_REQUEST_RECORDED | type30 | key26=D | selected positive Freeze; exact PR event400; context5; typed Anchor/L(P); event before transport |
| 301 REVIEW_RESULT_RECORDED | type31 | through exact type30 Request authority | valid independently replayable Request prefix; all existing redundant/Anchor bindings |
| 302 REVIEW_ADMISSION_ACCEPTED | type32 disposition1 | keys24=old literal AND25=D | new §82 prerequisites, completed §46 SATISFIED, exact §83 agreement |
| 303 REVIEW_ADMISSION_REJECTED | type32 disposition2 | keys24=old literal AND25=D | new §82 prerequisites, completed §46 non-success, exact §83 agreement |

Existing direct/identity dependencies, exact JournalReference authority instances and chronology are preserved from Cross-Reference v0.3/v0.5/v0.6/v0.7/v0.8 wherever unchanged. Markers and type64 introduce no new Journal dependency: the event_record_id commits its complete Record, whose typed RecordId references commit its parameter inputs. Every required referenced Record must still resolve. No event104 or other Freeze rejection event is invented for a failed Policy; no preterminal failure is event303.

## 6. Required design/implementation vectors

Each case is a future causal test obligation, not executed evidence:
- Receipt absent/exact/invalid selector, independent freeze_id mutation after rehashing dependent objects, wrong START/domain/Registry; absent remains predecessor.
- Request absent/exact/invalid selector at an event300-only prefix; generic PR [2,5] registration before Request; no terminal needed to validate its prefix.
- Result event301-only prefix derives route from its exact Request; missing/old/wrong Anchor version cannot be upgraded.
- Full terminal selector cross-product: absent/absent; old-only; both exact; new-only; either wrong, wrong width/type, unknown, duplicate or noncanonical; different-D selected upstream. Assert which handler ran and no fallback.
- Old discriminator-less 302 and303 and old-key24-only histories retain bytes and their own dispatch, even with new surrounding Records.
- Type64 exact bytes round-trip and identity; wrong type/profile/key/version, missing target, generic frame-valid substitute fail.
- Filesystem reference/event environment inequality, missing observation, stale context or unsupported material-change transition fail before completion.
- Same PR exact event reference across Request/Admission; same RecordId with different event instance rejects; unrelated PF not forced equal.
- Request/Result Scope mismatch: no evaluator/no terminal construction or mutation. Valid common Scope with PR gate mismatch: context5 success then actual1015 FAIL and303. All-match path302. Missing/malformed profile is not completed Policy failure.
- Scope label substitution changes RecordId and exact equality while not independently defining coverage.
- Present minimum_durability all-false/true or requested durability, custody2, unsupported object/profile: explicit unsupported, no1008 fake result/no101.
- Real nonempty EMBEDDED Store production, reopen and positive selected full chain; empty payload action-state treatment; full no-follow inventory, source/copy mismatch, extra/missing/changed payload and same-byte generation replacement.
- Whole semantic candidate-prefix preflight before Record staging, fixed L(A) under racing append, no stale-result retry, mandatory reused-Record flush, Record-before-Journal ordering and before/after-visibility faults.
- Retained semantic inspection after uncertain publication must not upgrade the prior live publication receipt or claim historical flush attestation.

Additional required causal vectors: coherent substitution of Subject in START/Manifest/Receipt with recomputed dependent Record/Journal hashes; wrong/unknown GENESIS format or Record identity profile despite valid framing/duplicates; Anchor format derived from a different source; initializer with no pre-existing root/GENESIS; actual probes only in assigned operational namespace; partial initialization remains non-success and ordinary open never repairs it.

### 6.1 Selected Subject known-answer vectors

These exact deterministic-CBOR projection bytes and SHA256 values are fixed design vectors, not claims of product execution. They use the Core section3.1 formula. Path component `a` below is the single bstr byte61, not CBOR text.

Empty inventory, N=0, A=[]:

- Projection hex: `85782845766964656e636552656769737472792e5375626a6563742e526567756c617246696c65732e763101010080`
- Subject SHA256: `a2162a87b04ff2db6bd00561019575bfa9a7aa88c4a787fa537babd6c00732b4`

One regular file at component `a`, exact content bytes616263 (`abc`), N=1, size3, digest_algorithm_id=1:

- Projection hex: `85782845766964656e636552656769737472792e5375626a6563742e526567756c617246696c65732e763101010181850181416103015820ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad`
- Subject SHA256: `2a04582a51c7b892add0a891e209a30bc171ddb983dc4076ba52a19ecb145adb`

Reordered rows, duplicate paths, false count, wrong size/digest/profile/kind, or text instead of component bstr must not be normalized into these vectors. Validate grammar/profiles first and recompute from actual admitted bytes at production/current payload verification. A correct vector hash alone is not Freeze authority.

These supplement, not replace, every applicable adopted Store/profile/Anchor/selector vector. Exact qualification plan and slice ordering are in the nonnormative companion plan.
