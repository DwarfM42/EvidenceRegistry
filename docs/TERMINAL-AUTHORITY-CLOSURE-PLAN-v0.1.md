# EvidenceRegistry Terminal Authority Architecture Review and Implementation Plan v0.1

Status: NONNORMATIVE CONDITIONAL DECISION PACKAGE. No adoption, implementation or qualification result is asserted.

## 1. Rebound state and authority inventory

Baseline HEAD 7854fe96753c714e3e043c418f425b2cd713ba91; tree 8e4411936c20a85a8ffc8388d0356a01943ed091; canonical repository D:/Desktop/Sandbox/hermes/EvidenceRegistry; branch integration/authority-path-runtime, clean at discovery. The candidate is documentation-only and is retained through an additive local review ref without moving the active HEAD, ordinary index or worktree. No second worktree is used. Final candidate commit/tree and review dispositions belong in the detached final handoff, not self-referential candidate bytes.

Prior Session 20260909_191727_e8bef7 and C:/Users/sngme/AppData/Local/hermes/cache/handoffs/EvidenceRegistry-20260909_191727_e8bef7-stop-handoff.md were read as continuation evidence. Adopted authority was independently rebound from Git objects. The stopped Freeze selector choice is still open, not silently resolved by this draft.

Read-only remote observation 2026-09-09T17:04:54.403402+00:00: PR #6 https://github.com/DwarfM42/EvidenceRegistry/pull/6 is open, non-draft, head dfb6fc98207e0af50e8f94666ba3f517bba3c898 against main fa7753a7175d64ee4c87453a7a710e6dc33913cd. API identity was explicitly DwarfM42. Local main is 3f724caf33cafcccd8bb64745ff3725629125c09; local integration HEAD differs from the remote PR. Mergeability was clean at observation, not authorization or proof of current CI. No PR, remote branch, About, protection or qualification environment was changed. Existing stash 16177affd29db4a0937e67e81bbf5b2327437a83 is preserved; do not apply it as authority.

The package consists of four added docs: Core, Schema, this Plan, and TERMINAL-AUTHORITY-CLOSURE-INPUTS-v0.1.json. Inputs records all 31 baseline documentation Git blobs/raw SHA256/byte sizes/introduction commits and manifest checks. Inventory is a derivation/evidence index, not semantic authority. The parent audit checked 40 manifest rows with zero mismatches; rows are not distinct documents. Detached adoption controls membership despite embedded DRAFT headings.

Authority layers:
- Frozen baseline: Identity Format v0.3, Record Schema v0.3, Lifecycle v0.10.2, Cross-Reference v0.3, Formal Boundary v0.5.4, exact FROZEN-BASELINE-SHA256.txt entries.
- Lifecycle v0.10.3/Cross-Reference v0.4: historical blocked candidate, not adopted active semantics.
- Lifecycle v0.10.4/Cross-Reference v0.5: adopted candidate b974f01a033df6673507933e86196b286c221120; external freeze record is adoption evidence. It closes §46/§82/§83, not every input producer.
- Authority-Path package: adopted candidate 9e674925deecceaaeab4272f960df759b30e4512, detached adoption introduced at local main 3f724caf33cafcccd8bb64745ff3725629125c09. Schema v0.4/5/6 and Lifecycle v0.10.6/7/8, Cross-Reference v0.6/7, Store v0.1 and package membership remain exact inputs. These are adopted-but-often-unimplemented, not generic drafts just because their headers say so. Lifecycle v0.10.5 is rejected historical/non-authoritative remediation context, explicitly excluded from adopted membership; its inventory hash preserves provenance only.
- Terminal selector package: candidate f53decc6c2fe89523c5463643fd09ac986732eb0, adopted by 7854fe96753c714e3e043c418f425b2cd713ba91; Schema v0.7, Lifecycle v0.10.9, Cross-Reference v0.8 and manifest. It does not change Request context5 or add Freeze selection.
- SPECIFICATION-GAP-CANDIDATES.md and all architecture audit reports: nonnormative diagnostics. This closure package is prospective draft only. Nothing is globally superseded by filename order.

## 2. Root causes, not isolated bugs

### R1: Operation-specific profiles were reviewed without satisfiability of their composition

Schema v0.3:2537-2578 requires Request's event400 Policy to support context5. Schema v0.4:89-98 and Lifecycle v0.10.9:38-49 require the selected Admission profile's exact context set [2]. Current source derives Admission Policy from Request (authoritative_store.rs:860-866,946-952) and enforces equality on replay (:5754-5756). One set cannot be both exact [2] and contain5. This is a concrete unreachable composition, not merely a missing evaluator.

The source equality is not itself frozen authority. Separate Request and Admission fields exist; the authority does not unambiguously select their cross-stage relation. Nor was a Freeze-to-Review Policy equality found: the explicit Freeze equality is START==Receipt. Thus a global Policy redesign is unnecessary. A two-Policy new lane is proposed because it keeps Review Policy committed before transport using the existing Request carrier. A three-role design is also representable but requires a precise pretransport selection relation if that commitment is desired. The earlier audit's phrase 'Owner-steered' refers to a parent-agent hypothesis, NOT an actual Owner choice; its preference is nonauthoritative.

### R2: Prospective semantic rules lack a consistent record-local selector model

The adopted terminal key24 fixes only type32/events302-303. Freeze's new identity/positive-replay meaning cannot be inferred from that downstream marker. A new context5 rule also first matters at event300, not at a future terminal. Prefixes ending at300 and301 must stand alone. One universal new epoch or replacing the old key24 meaning would silently reinterpret historical inputs. Proposed closure uses three local carriers of one acyclic contract digest, with Result deriving its route through its exact Request authority.

### R3: Required references were mistaken for complete semantic contracts

Receipt.freeze_id is OpaqueId32 with no baseline derivation; FreezeRoot, Attempt, Subject content, Manifest and Record identities have distinct roles (Identity v0.3:1576-1672; Schema v0.3:2263-2278,3633-3661). The baseline also carries Subject as OpaqueId32 without a complete source/inventory-to-Subject function, and GENESIS's required format/profile UInt fields lack exact value-to-algorithm mappings. A copy-derived Subject cannot be implemented from cross-record equality alone; hashing the complete Manifest would be self-dependent because it contains subject_id. Required creation_profile_ref/filesystem_profile_ref are RecordIds without assigned referent grammar or producing/consuming relations. Embedded custody specifies copy/hash/verify ordering but not a canonical local payload base. Field names and frame-valid fixtures cannot complete these relations. These are finite required identity/profile choices; Core section3.1 explicitly proposes a non-self-referential Subject projection while leaving all existing Record/Journal identity algorithms unchanged.

### R4: Live production, retained assertions, semantic replay and durability were conflated

A selector commits a declared semantic lane, not proof that a producer binary ran or a filesystem flush happened. A Receipt cannot attest its own later final-directory flush (Lifecycle v0.10.2:2214-2222). Store requires independent content/parent flush and exact readback (Store v0.1:79-112), even without optional minimum_durability. General evaluator1008 target/aggregation/post-publication evidence remains under-specified. Proposed bounded lane explicitly declines 1008 and requested durability, while fully retaining mandatory physical Store operations and precisely scoping Receipt payload facts. Stronger attestation or hostile-owner resistance is outside Profile L, not a blocker to local integrity.

### R5: Mechanism tests and dormant composition were mistaken for a working Store product

Open (:508-665) implements three namespaces, synthesizes a missing GENESIS resolver copy (:562-573), and omits roots/coordination/profile conformance. Source has no full Store-owned initializer, START/root/copy producer, typed prerequisite event producer or review-pack workflow. Positive Freeze always returns unavailable (:679-695); a second generic hold blocks completion (:916-921). Whole-history gates reject Review prefixes (:6151-6185). Existing local constructors/primitives do not satisfy the public authority path.

Dormant terminal publication evaluates before acquiring serialization (:921-965), retries stale results (:972-1018), and only runs full typed/contextual validation after Journal visibility (:1162-1168). Reused Record content flush is not established (:1727-1784). These are implementation/ordering defects once the route is enabled, not excuses to change Policy semantics. Many STOPs came from repeatedly exposing the next missing layer instead of qualifying the whole prefix and producer/replay graph.

### R6: One identity/label conflict class in two adopted profiles

Schema v0.6:44 excludes scope_label from Record identity while its own no-identity-change clause and frozen whole-body hashing retain that field. Schema v0.4:80-83 also prohibits label effects on equality/evaluator outcome while its section5 requires full RecordId equality for1015. Both manifestations conflict with retained whole-body identity. The minimum new-route consistency correction explicitly replaces both prohibitions, keeps full-byte identity and its indirect evaluator consequences, removes independent label coverage semantics, and never strips labels or rewrites history. The old key24-only lane retains its original constraints; where those do not uniquely determine a labeled case, it cannot borrow this new correction or claim positive authority. New-path draft resolves this class only conditionally; an unadopted correction has not repaired the old contract.

Conclusion: the foundations are usable. The repeated STOPs are a combination of missing finite semantic closure, selector layering, ambiguous Policy-role composition and substantial authority-defined-but-unimplemented production. Not every STOP is a new Owner choice; not every available helper is a supported authority path.

## 3. Consolidated Owner decision boundary

Categories below are authority classifications, not the old A/B/C selector option labels.

### Category A: existing implications

Preserve exact whole-body identity, retained Store resolution and exact event references; no caller conclusions; §82 before §46 before §83; Request/Result Scope mismatch preterminal; unsupported observation is not success; no post-action self-attestation; no silent reinterpretation. Do not enforce an unsupported global PF==PR merely because fixtures share a value. API/implementation fixes strictly implied by these can be planned without new semantic choices, but implementation remains unauthorized this session.

### Category B: consistency consequences of adopted choices

Implement the selected Store namespace/duplicate/root relation, Manifest/profile resolution, atomic L(P)/L(A), selected evaluator1015 and old selector codec/replay, strict publication/reopen and truthful claims. Keep Scope label bytes in Record identity; add only the minimum selected clarification to contradictory prose. These are not invitations to change adopted bytes or broaden unrelated profiles.

### Category C: genuinely new, coupled decisions — NONE ADOPTED

**C1 Retained positive Freeze architecture:**
- Recommend old Choice A: Store-only Receipt key32 canonical bstr32, bound to Core D; prospective START-derived freeze_id, absent=predecessor, malformed/wrong=fail-closed.
- Old Choice B remains possible, but must name exact retained carrier/grammar/identity/chronology/cardinality. It would need a different coherent candidate and reviews. No arbitrary option-B bytes are drafted or silently selected here.
- Old Choice C (live-only) intentionally forfeits replayable positive Freeze and therefore cannot enable the claimed reopened Freeze→Review terminal lane. It requires truthful unavailable claims rather than terminal completion.

**C2 Review Policy and prefix topology:**
- Recommend separate PF [1]/profile2 and shared PR [2,5]/profile1, exact Request/Admission JournalReference equality, structural-only context5 gate_scope, ordinary1001/context guard at creation, unchanged context2 evaluators. New Request key26 and terminal key25 select this lane; old key24 remains unchanged and required alongside25.
- Alternative: distinct Request-creation and Admission Policies. Must define context5 profile and exact selection/acceptance/pretransport-binding relation; existing fields alone do not choose them. Global one-Policy redesign is larger and not recommended.

**C3 Source identity, initialization assignments and minimum truthful physical capability:**
- Recommend the exact selected Subject projection in Core section3.1: SHA256 of deterministic CBOR `["EvidenceRegistry.Subject.RegularFiles.v1", 1, 1, N, A]` over the complete sorted existing Artifact Entry array, excluding self-dependent Record/Subject metadata. This is a new semantic Owner choice, not implied by an opaque field or ArtifactSet utility. START-source, retained-copy and replay derivations must agree. Also explicitly map GENESIS journal_format_version=1 and record_identity_profile_id=1 to the unchanged frozen algorithms; propagate the validated GENESIS format to Anchors. These assignments were identified by hostile review and are consolidated here rather than hidden as implementation defaults.
- Recommend Core's explicit bounded EMBEDDED regular-file lane, type64 creation profile1, filesystem_profile_ref→exact event101 type61 observation, payload/ mapping, payload-only Receipt action facts, no optional1008/requested-durability support, exact operational lock/temp/probe namespace and explicit pre-GENESIS bootstrap exception. New marker gates all changed Receipt/Subject interpretation. Store's mandatory flush remains unconditional.
- Alternative: broader custody/profile/1008/strongest-available or later observation support. That requires exact target/aggregation/record/provenance grammar and potentially a larger package; it is not implemented by accepting this bounded draft.

These three choices are the smallest consolidated decision surface identified, not five unrelated wakeups. Core/Schema materialize the recommended combination ONLY AS A CONDITIONAL PROPOSAL. The Owner may adopt that exact combination after clean reviews, reject it, or choose alternatives requiring a revised reviewed candidate. No automatic adoption or implementation follows from recommendation. This session ends at that boundary and notifies Discord.

## 4. End-to-end dependency graph and classification

Legend: AUTH=sufficient adopted authority, IMP=bounded implementation exists, UNIMP=unimplemented, BLOCK=semantic/choice blocked, X=intentionally unsupported. Core citations are conditional draft, not AUTH until adopted.

| Edge | Current contract classification | Runtime status / selected successor consequence |
|---|---|---|
| Store locator→exact generation/namespaces | adopted sufficient Store§2-3 | IMP partial/UNIMP selected profile; strict duplicate, roots, coordination needed |
| capability/environment→GENESIS/epoch | frozen observation rules sufficient; format/profile scalar assignments semantically incomplete | UNIMP/BLOCK C3 exact mappings and bootstrap sequence; event600 numeric change mapping BLOCK/X in proposed subset |
| GENESIS→retained Journal/Records | frozen/adopted sufficient | IMP structural; synthesis is not selected profile conformance |
| Freeze intent→Attempt/START | frozen/adopted sufficient | UNIMP Store-owned producer; exact Subject/Policy and START readback |
| START→FreezeRoot→physical root/coordination | adopted sufficient | UNIMP; root exclusive after START, unknown liveness nonauthoritative |
| root/source→retained payload→Manifest | adopted custody/profile core sufficient; Subject derivation and exact local mapping incomplete | UNIMP/BLOCK until C3; profile1/digest1 existing, Subject projection and payload base new |
| supporting profile references→Receipt | semantically incomplete | BLOCK C3 typed64/environment relation; not arbitrary frame-valid refs |
| START→Receipt.freeze_id | semantically incomplete/prospective-only | BLOCK C1 exact copy, not inferred baseline meaning |
| Receipt→selected Freeze replay | replay-incomplete/prospective-only | BLOCK C1; old terminal selector cannot supply it |
| PF/Scope→Freeze§46 | adopted sufficient bounded profiles; optional1008 evidence incomplete | UNIMP; C3 declares absent-only subset, present unsupported |
| completed Freeze→101→immutable Record/Journal | adopted sufficient guard/publication mechanism | UNIMP/BLOCK on upstream; never invented rejection event |
| positive Freeze→review-pack/Request | adopted timing/Anchor sufficient; context5 applicability incomplete | BLOCK C2; independently valid event300 prefix required |
| Request→Result301 | adopted exact transport/Anchor sufficient | IMP decoders partial/UNIMP authoritative producer and selected prefix |
| Request/Result/PR→§82 | adopted ordering sufficient; cross-stage Policy relation ambiguous | IMP checks/UNIMP positive; C2 fixes new exact reference topology |
| common Scope/PR→§46/1015 | adopted old singleton sufficient, new multi-context successor-only | UNIMP; valid gate mismatch reaches completed FAIL, not prerequisite failure |
| §46→§83→terminal Record | adopted sufficient | IMP local predecessor constructor only; new/old selector codecs UNIMP |
| terminal Record→immutable publication | adopted sufficient | IMP primitives/publication-incomplete composition; preflight/lock/flush fixes |
| Record→Journal→flush→reopen | adopted sufficient | IMP dormant/UNIMP lawful E2E; uncertain visibility never success |
| reopen→per-prefix replay→inspection | predecessor-only and adopted selected terminal rules sufficient locally; whole chain replay-incomplete | IMP structural, UNIMP selected full route; D cannot authenticate past syscalls |
| README/About→public product | claims truth maintenance, not normative authority | README currently states positive unavailable; About high-level wording must not be read as reachable positive support; later align only to verified capability |

Adopted-but-internally-conflicting: the Scope label identity/equality/outcome clauses in Schema v0.4 and v0.6. Superseded only in proposed selected lane: both conflicting clauses and singleton Review applicability for the new marker combination; old selected history is not relabeled superseded. Intentionally unsupported in bounded plan: optional1008/requested durability, referenced custody, generic Scope, capability-change event600 mapping and other unavailable families. Implementing this subset must advertise those bounds.

## 5. Mechanical RED→GREEN implementation slices (future authorization only)

For every slice: begin with a behavior-level RED at the named production seam, prove earlier prerequisites valid and exact objects present, then implement narrowly; retain raw command/results and exact source/test identity. No-write assertions are phase-specific: pre-staging prerequisite/semantic rejection leaves the terminal Record namespace and next Journal slot unchanged; failure after successful Record staging but before Journal visibility permits the exact identified nonauthoritative Record residue and requires no authoritative terminal event; after possible Journal visibility require an uncertain receipt, preserved visible bytes and no rollback or retry-to-success. Preserve legitimate earlier START/prerequisite appends when testing a later failure; do not compare that whole workflow with a pre-START empty namespace and demand rollback. Review checkpoint means code/authority review at that boundary, not automatic repeated governance generations. Full independent review occurs before exposing terminal publication.

| Slice | Source / production seam | Required RED behavior | Forbidden shortcut / failure & publication boundary | Review checkpoint |
|---|---|---|---|---|
| S0 adoption binding | detached adoption + all four candidate docs | external exact-object/hash/role check rejects wrong candidate/missing adoption | no cargo or edits until authority/implementation scope exists; no stash reuse | exact byte authority gate |
| S1 strict selected Store | Store§2-3, Core§6; AuthoritativeRegistryStore::open/loaders | required roots/coordination, physical GENESIS duplicate, extras, alias/generation replacement | no synthesis/repair; retain legacy bounded opener explicitly if needed; read-only no writes | source-vs-profile matrix |
| S2 typed codecs and dispatch | Schema§2-5, Core§3.1/6; lib.rs typed codecs + store typed validators | every selector combination/version,64 exact grammar, historical dispatch, hash-sensitive labels, Subject empty/nonempty known answers and coherent-substitution rejection, exact GENESIS mappings | no global relaxed unknown-key or generic Scope gate; no self-hash/fixture-derived Subject or profile default; codecs confer no authority | exact vectors plus identity/branch causality |
| S3 real observations/init/staging | Store§4-5, Core§6; existing publish_record_bytes/OS adapters plus new Store facade | exclusive init, exact duplicates/slot0, absent/unsupported/stale probes, reused-record flush | no head-ID copy as fresh observation; no repair partial init; no invented600 mapping | actual capability/initialization provenance |
| S4 Freeze START/root/payload | Lifecycle§§49-68, Core§3-4; new Store-owned Freeze producer | source-derived Subject, START readback first, root/first payload create-new, nonempty real source copy/hash/verify, conflicts/crash/UNKNOWN | no direct fixture write as producer; no silent retries/skips; START may publish, failed work never101 | physical producer and namespace review |
| S5 selected Freeze decision/replay | Lifecycle v0.10.8/Schema v0.6 plus Core§3-4; validate_freeze_committed_authority/replay | exact selector/id/profiles/payload complete; optional1008 and key30 reject precompletion; empty-set positive only after guards | no arbitrary profile Records, no assertion bool from caller; no101 on failure; full preflight before Receipt staging | first real101 plus reopen witness |
| S6 Policy and Request prefix | Schema v0.3§§24-44,65, v0.5, Lifecycle v0.10.7, Core§5 | PR[2,5] generic400 then selected300; typed Anchor L(P); context5 1001+coverage; gate-Scope mismatch survives creation | no future terminal-dependent validity; no1015 at creation; Request event before transport | prefix ending at300 independently valid |
| S7 Result ingestion/prefix | Schema v0.3§66/v0.5, Core§5; new Store Result ingress / existing consumers | actual returned typed Result→exact Request; every redundancy and Anchor mutation; prefix ends301 | untrusted result not Registry conclusion; no marker backfill or shifted Policy/Anchor; no terminal writes | prefix ending301 independently valid |
| S8 accepted §82/§46 nonpublishing | Lifecycle v0.10.4/6, Core§5; accept/complete_section_82 and new selected evaluator path | exact reference topology, fixed L(A), Request/Result mismatch no evaluator; valid gate mismatch actual1015 FAIL; all-match SATISFIED | no generic hold removal; no treating unavailable as completed; no terminal construction/staging here | all applicable evaluators & scope causal proof |
| S9 terminal lawful construction/publication | Store§4-5, Lifecycle v0.10.9, Core§6, Schema§2; complete_authoritative_review_admission | selected302 and303 real Store route; full dry-run semantic preflight; lock through history-sensitive eval+append; conflicts and no stale retry | no caller marker/disposition; Record before slot; mandatory flush on reuse; before visibility non-success, after visibility uncertain | independent full implementation review before enabling positive publication |
| S10 reopen/inspection/truth | Core§7, Store§4; selected replay + separate inspection facade | producer exit, reopen full chain302/303, corrupt/missing/extra payload, historical mixed selectors, uncertain-vs-retained distinction | no current view claim from stale captured head; no past-syscall attestation; CLI unchanged unless separately authorized | full native qualification and claims audit |

S2 is syntactic preparation, not authority implementation approval by itself. S3-S7 introduce lawful producing edges that fixtures currently bypass. At S4 source/copy inventories must prove the target failure rather than an incidental stale hash. Initial prerequisite appends must each satisfy their own readback; a completed plan or presence of all Record files cannot substitute for a valid independently replayable prefix.

Terminal publication becomes lawful only after Owner adopts exact coherent semantics, separate implementation permission exists, S1-S8 prerequisites and full selected candidate-prefix semantic replay are complete, S9's serialized construction/no-replace/mandatory flush/readback are implemented and independently reviewed, and the exact adapter/root satisfies required qualification. During development fault tests may exercise authorized disposable roots, but cannot be promoted to public runtime qualification. A design PASS alone is never a publication gate opening.

## 6. Final qualification and integration plan

No command in this section was run in this review session. No qualification environment was modified.

Native targets from current workflows: Windows-2022 x86_64, Ubuntu-22.04 x86_64, macOS-14 arm64; Rust/Cargo1.97.1. Bind OS/arch/filesystem/toolchain, exact committed source/tree and built binary hashes for each actual run. Do not emulate a missing native result or report a skip as support. Required Store vectors must run against every qualified adapter and a real full-chain producer/reopen302/303, not only helper tests.

Existing workflow commands, after the README/workflow's prescribed per-platform setup:
- cargo build --release --locked
- cargo fmt --check
- cargo test --all-targets --locked
- cargo clippy --all-targets --locked -- -D warnings
- cargo test --release --test review_admission_runtime --locked
- cargo test --release --test freeze_committed_binding --locked
- cargo test --doc --locked
- git diff --check
- git diff --exit-code

Linux all-target tests supervise six ignored contender workers by exact name and deadline; verify python3 and keep that supervision. Never invoke ignored workers unbounded or accept zero matching tests. Add/exercise applicable vectors through canonical suites; record counts, names, raw stdout/stderr and actual exit status. Predecessor tests expecting unavailable remain valid for their old lanes; separate new selected tests from rewritten history.

Qualification must cover cold producer exit/reopen, same-generation versus same-byte replacements, current capability failure/change, process-cache invalidation, START/root/first-payload interruptions, no-follow filesystem hazards, Record reuse flush, exact append conflict, semantic preflight rejection before staging, every post-visibility fault/uncertain receipt, historical selector matrix and causal Request/Result-versus-gate-Scope mismatch. No stronger storage claim than observed adapter operations.

Then exact implementation candidate hostile and independent review, protected PR/merge gate, and final exact merge-result qualification across Windows/Linux/macOS and applicable formal-provenance checks. The read-only ruleset observation requires only windows-ci at GitHub's strict protected-branch gate; broader project qualification is a separate requirement and is not currently enforced merely by that ruleset. Re-read ruleset22505650 and actual required check names then; historical mergeable=true is not CI. PR#6 updates, push, merge, remote About edit and release all require separately authorized scopes. Preserve existing remote state in this session. Integrate only reviewed bytes in the canonical worktree, no second worktree without Owner approval, no force push/history rewrite. A changed source or merged tree needs correctly rebound evidence; prior SHA reviews do not qualify it automatically.

README/API/About truth alignment is a later controlled change: enumerate precisely the positive selected Store lane, supported adapters/custody/profiles and unsupported1008/generic families; keep Journal-only CLI distinct from Store authority; expose unknown/unsupported vs invalid vs completed results. Update adoption discoverability without rewriting adopted documents. No claim of production, universal durability or trusted producer attestation from local tests. After authorized protected merge, keep local sandbox main current with reviewed integration, verify local/remote exact identities, clean the approved temporary branch state only; preserve stash and review/evidence refs until explicitly authorized cleanup. Never delete adverse review evidence to make history look clean.

## 7. Known traps and first implementation action

- Draft or review PASS is not adoption; filename order and source ancestry are not semantic selectors.
- Never pop the selected-route stash and presume its inputs were authoritative.
- Never replace the global fail-closed Scope gate, force intentionally RED predecessor tests GREEN, or use a downstream terminal marker to legalize an upstream prefix.
- Do not infer identity equalities, replace exact event authority with a RecordId, strip labels, or derive the new core digest from text-normalized bytes.
- Omitted minimum_durability is not omitted physical work. Marker equality cannot reconstruct an earlier exclusive create/flush. Type64 encodes a procedure, not proof of performance.
- Do not accept structurally valid caller bytes as Store-owned evidence or permit helper publication before semantic preflight.
- Do not refresh L(A) or carry a completed Anchor-sensitive evaluation across a changed publication head.
- Interrupted visible publication is not a safe automatic retry; old uncertain receipts stay uncertain even if later semantic inspection succeeds.
- Three passing platform helper suites are not a lawful full production/replay route; final README claims wait for actual evidence.

First action in the separate implementation session: read the new supplemental handoff and exact Owner adoption/implementation authorization, then rebind the canonical HEAD/tree, candidate commit/tree, four package blobs and every adopted input from Git. If adoption or any C choice is unresolved, emit OWNER_DECISION_REQUIRED/WORK_STOPPED and do not edit or run Cargo. Otherwise start S1 with a real selected-profile Store-boundary RED, not terminal publication.

Review outputs and final handoff remain detached from this immutable candidate. Both exact-byte hostile semantic and independent consistency/provenance/minimality reviews must bind the same candidate commit/tree and raw package bytes; any repair creates a new candidate and makes prior review outcomes stale for it. Completion notification must name actual final HEAD/tree, candidate status, unresolved choices, handoff path and paused implementation. After review/specification completion, remain idle.
