# AI Agent Evidence Binder v0 — implementation contract

Status: v0.3.0 implementation/design contract. It does not adopt a new Core successor or turn Binder-local observations into Core authority.

## Rebound baseline

Baseline commit `14aa2d2b4ef6342e53e1274acd393b87ab5ee6aa`, tree
`702e2a4a497fb605e7c2b7b63d6904e6112ba3bd`. Local canonical Windows checkout and
live GitHub main agree. The published source-only v0.2.0 release is ID 386254636;
annotated tag object `b24619e0d036884652cfd9101d116bf015e629b7` resolves to that
commit. It is unsigned and has no binary assets. Binder is not in v0.2.0.

C1–C3 were adopted together at Candidate02
`48cb437446419cc614af0ac84233d1a2b02b2062`, tree
`de653e61828319a7dc2881c2a670f94718d12c95`, through
[the detached adoption](FREEZE-RECORD-TERMINAL-AUTHORITY-CLOSURE-PACKAGE-v0.1.md).
Historical candidate headings do not revoke adoption. Core raw SHA-256 is
`cb7ee9e9e5f9d0229dba26012ecb5231c26771a6d54c3c95762c1094ec1fc16d`.
No adopted bytes or old release identities are changed by this work.

## Trust boundary

| Material | Authority by itself? | Input that can influence an authoritative decision? | What is actually established |
|---|---|---|---|
| Exact Store-resolved selected Review Request | Authoritative prerequisite in its selected context, not general permission | Yes: fixes Freeze, Manifest, role, Scope, Method, checks, Policy and Anchor | The retained request and its checked prerequisite chain |
| Agent CLEAN/PASS narrative or structured status | No | Yes: valid structured Result status/finding inputs can satisfy or fail existing Review Policy evaluators | A claim, not correct review reasoning; narrative is never translated by an LLM |
| Agent artifact list | No | Selection changes retained material and can prevent Binder ingestion | Untrusted candidates; not filesystem read authorization or proof of completeness |
| Binder-spawned process exit and owned pipe bytes | No | Only Binder-local completeness gates unless an adopted evaluator expressly says otherwise | Top-level process/pipe observation, not internal delegation completion |
| Store-captured bytes | Retained evidence, not independent truth/authorship | They establish their own selected Freeze; no automatic original Review Policy dependency | Bytes read and retained at capture, including current payload validation |
| Agent internal delegation logs | No | Any mapped semantic claim remains submitted input; opaque logs are not a new evaluator | Agent-reported internal work, not Binder observation of it |
| Binder executable/build identity | No | No automatic Policy effect | Provenance, not attestation or execution authentication |
| Selected Admission | Selected-lane authoritative disposition | Yes, only under applicable downstream semantics | Applicable Policy outcome, not semantic truth or external action permission |

Acceptance establishes the checked Request, target and submitted-input relationships
and satisfaction of the applicable Review Policy. It does not establish correct
reasoning, reviewer honesty, fresh thinking or discovery of all defects. A Request's
role/identity text does not authenticate the process owner or any internal subagent.

## Authority / public API conformance

References below bind to the baseline above; these are source findings, not fresh
execution qualification. P = existing authority/API; S = defined semantics but a
missing or defective public seam; L = nonauthoritative Binder-local contract;
N = new authority choice (not selected by this design).

| Responsibility | Class | Existing governing meaning and API | Work / limit |
|---|---|---|---|
| New selected Store | P | `initialize_selected_profile`; closure Core §6 | Absent root only; no repair-by-open |
| Typed parameters and Policy registration | S | Schema v0.3 §45.1, §§47–50; closure Core §5 generic event 400 | Baseline public Store producer absent; do not copy fixture direct writes or caller-selected Journal slots |
| Target Freeze | P | `prepare_selected_embedded_freeze`, `commit_prepared_selected_embedded_freeze`; Core §§3–4 | Complete selected EMBEDDED inventory; preparation alone is not committed authority |
| Request and predispatch binding | P/S | `record_selected_review_request`, `validate_selected_review_request`; Core §5; Lifecycle v0.10.7 | Revalidate exact event before intent/spawn; chronology and typed-selector implementation require conformance repair/check |
| All dispatch lifecycle observations | L | No Core event assigned; external directive permits local ledger | Intent before spawn; no implicit retry; preserve every incomplete attempt |
| Structured submission | P/L | `SelectedReviewResultInput`; Schema v0.3 §66 | Strict deterministic grammar; Request-derived redundancy only; actual statuses, not invented CLEAN wire enum |
| Postdispatch output capture | P/S | Separate selected output Freeze using same Store intake | Keep original target Freeze/Manifest/Anchor; root-bound source-confinement seam needs repair/qualification |
| Output capture ↔ attempt ↔ Result link | L | Local exact-reference ledger | Not a Core supporting-evidence dependency; do not misuse findings or reviewer metadata |
| New Core-enforced supporting-capture prerequisite | N | No such existing relation/evaluator | Not required or implemented in this design; needs separate adoption before such a claim |
| Policy / Admission | P | `complete_selected_review_admission`; Core §5, retained §82/§46/§83 | Store chooses accepted/rejected; malformed/capture failure stays preterminal |
| Build identity / observation / Agent reports | L/P | Local observations retained as opaque output-Freeze files | Hashes are provenance, not execution signatures |
| Cold replay / known exact references | P | `open_selected_profile`, Request/Result/Freeze validators | Current retained consistency is not historical flush receipt |
| Enumerate all results / reconcile uncertain publication | S/L | Bounded retained history exists privately; latest-only `inspect_selected_terminal` is insufficient | Add public read-only captured-view enumeration; preserve all matches, never pick latest silently |
| Typed Result material readback | S | Existing private findings and metadata | Public immutable getters, no new wire fields |
| Crash / duplicate / explicit retry | L | No invented Core retry semantics | Do not recreate live witnesses; conservative uncertainty and full history |

The baseline selected test writes Policy prerequisites directly and uses Scope-shaped
Method/CheckSet fixtures (`tests/freeze_committed_binding.rs:1075–1134`). This is not a
public-only fresh-Store example. Baseline producer/replay incorrectly constrains
Policy/Request/Result operation-start relations to Freeze START. This is a defined
semantics repair, not an adopted contradiction or a new Owner choice. Missing
methods alone are implementation gaps and do not reopen C1–C3.

The table classifies the initial baseline, not the current implementation's
availability. The current public seams include typed Scope/Method/Check/CheckSet
construction, Store-owned typed staging and selected Policy registration,
`RetainedJournal::references()` (all exact retained event instances, including
GENESIS), and immutable Result findings/metadata getters. Bounded selected output
preparation and same-snapshot payload readback are described below. The companion
provides live `bridge::run_to_admission` and read-only `inspection::inspect`;
implementation/failure-matrix and native qualification are separate closure gates.
No new authority choice is selected: the output/Result association remains local,
and existing Policy, chronology and terminal semantics remain controlling.

### Chronology conformance resolution

[Cross-Reference v0.3 §30](CROSS-REFERENCE-v0.3.md#30-review-result-rules)
enumerates the Request-defined redundant Result fields, then separately requires
the Result operation-start field. [§48](CROSS-REFERENCE-v0.3.md#48-required-journal-chronology-fields)
defines each ordinary operation's start as its observed Registry head. Lifecycle
v0.10.7's package Anchor selection does not replace that chronology; the closure
Core preserves those existing relations. Policy registration, Request and Result
therefore each retain their own operation start. The Result still copies the exact
Request's Freeze, Manifest, role, Scope, Method and package Anchor.

The bounded producer holds one publication lock from authoritative reload through
append and captures its head immediately after that reload, before selection and
validation. This is a serialized implementation choice, not a new universal
acceptance boundary or a replay requirement that every historical start be the
immediate predecessor. Exact retained-reference resolution and strict prior-event
chronology remain enforced. Old retained bytes are not rewritten or backfilled;
historical chronology does not become proof of an observed in-memory execution.

Portable Scope (20), Method (21), Check (22), and nonempty sorted unique CheckSet
(23) require their own typed codecs. Store staging and replay must resolve each
CheckSet member as Check, and validate parameter types on both Policy/Request
producer and prefix-replay paths. No portable-parameter Journal event is invented.
Method/Check definitions identify exact bytes, not proof that an Agent performed
them. Policy event 400 registration remains generic and does not self-evaluate;
its start is Store-derived, and it may precede any Freeze or Request.

## Separate target and output evidence

### Binder-local submission grammar v1

The independent `ai-agent-evidence-binder` workspace member contains the JSON
parser; its serde dependencies do not enter Core. Parsing alone cannot publish,
spawn or manufacture a live dispatch witness. JSON has required `schema: 1`,
`request`, `method_status`, `finding_state`, `reason_codes`, `findings`, `artifacts`,
and optional `reviewer_metadata` text/null. Unknown and duplicate object keys,
trailing JSON, missing required fields, and mistyped values are rejected.
`request` has exactly `registry_id`, `entry_index`, `entry_hash`, `event_type_id`,
and `event_record_id`; identities use literal lowercase 64-character hex, event
300, and the exact managed Request reference. This is untrusted redundancy,
not ingestion permission. The parser does not accept role/Scope/Policy selectors.

Status/state IDs remain Core's local 1..=3 values; reason codes retain strict
byte ordering and findings retain submitted order/duplicates. No free text becomes
a status and no output Manifest is invented as a finding. Artifact names are
candidates only. v0 deliberately limits names to relative slash-separated ASCII
alphanumeric/`._-` components, excluding empty/dot/dotdot, trailing dots and reserved
Windows device stems. This permission syntax is not filesystem validation; Store
alone enumerates, reads and retains payloads. Mandatory-set comparison follows
Store capture and cannot depend on the Agent listing a required file.

Parser input is capped before deserialization at 262144 bytes; each list is capped
at 1024 members and each artifact name at 1024 bytes. Input boundedness also bounds
parser allocation; it is not a process-output bound or a Store-capture bound.
Those remain separately enforced lifecycle limits. The current parser tests are
local grammar tests, not a working Binder dispatch or Result-registration example.

Chronology:

`target Freeze F → Request Q / fixed Anchor A → persisted intent → dispatch → output capture O → Result R for Q → Admission`

O is a distinct Freeze with its own attempt, Subject and Manifest. Q/R's target fields
remain F and A. The local ledger binds exact Q/O/R/event references. That local join
is **not** a new Record type or Core-validated supporting-capture relation. `findings`
contains genuine finding Record references, not unrelated output Manifest IDs used to
simulate such a relation. `reviewer_metadata` remains reviewer text, not a hidden
Binder authority envelope. Additional history is evaluated by the preselected Policy;
never refresh A or weaken Policy to manufacture acceptance.

## All-attempt retention contract

The companion owns a separate, explicitly nonauthoritative ledger outside the strict
Store namespace. One live writer holds an OS-managed exclusive ledger lock. Bounded
append-only frames carry version, monotonic sequence, previous-frame digest, exact
Request reference, explicit attempt identifier, event kind and bounded payload.
Length and digest checks detect malformed/torn frames; this is mutation detection
within retained history, not tamper-proof custody or tail-deletion detection.

Ordering and recovery rules:

1. Validate exact retained Q; persist predeclared capture root/set, mandatory names,
   process command, limits, Binder identity, optional predecessor-attempt link and
   dispatch intent. Flush required ledger persistence before any spawn. Any failure
   means no spawn.
2. Spawn exactly once from that in-memory intent; keep the actual child handle and
   owned pipes. Append observations separately from Agent claims. A crash between
   intent and spawn-record publication is `launch unresolved`, never `not launched`.
3. Observe process exit and each pipe's EOF independently. Timeout/cancel/overflow,
   inherited open pipes, truncated stream, spawn failure and observation-write
   failure remain distinct. No unbounded pipe buffer or indefinitely waiting join.
4. Capture a predeclared complete output-root regular-file set through Store intake.
   Agent lists cannot expand roots; missing mandatory files/external paths reject.
   No subset mode is silently inferred. Capture records exactly its verified set,
   not 'all evidence needed for review' or all hidden Agent artifacts.
5. Parse exact captured structured submission with unknown/duplicate/missing keys
   rejected. Only a complete managed attempt can ingest R for its in-memory Q.
   No arbitrary historical-log import, externally supplied ID token, or replayed
   ledger record can recreate the live dispatch witness.
6. Persist publication intent before each Store mutation and its returned reference
   and receipt after it. If the return or ledger append is lost, mark publication
   unresolved and use read-only exact retained history to expose candidates. Never
   append a replacement or upgrade uncertain historical flushes to PUBLISHED.
7. Call Admission only after required capture and strict Result ingestion. Store,
   not Agent/Binder, derives the outcome. Completed rejection differs from invalid,
   unsupported and unavailable preterminal states.
8. Reopen never spawns or resumes. Enumerate every local attempt in sequence and
   validate all known Q/O/R/Admission references/current payloads via public Store
   APIs. Preserve multiple candidates as ambiguity. Display prior failures and
   unresolved Requests alongside successful attempts, not latest Admission only.
9. A new attempt must be explicitly requested and link to its predecessor; no
   CLEAN-seeking retries. One submission delivery cannot count as independent
   reviews. Prior malformed, missing or failed material stays retained.
10. Corrupt/torn ledger tails remain visible as a read-only incomplete history;
    block further dispatch/mutation, do not truncate or silently repair. Earlier
    validated frames may be reported only with that corruption boundary explicit.

Resource limits cover frame size/count, total ledger bytes, stdout/stderr bytes,
output files/bytes, runtime, post-exit pipe drain and open descriptors. Concrete
values belong to the implemented typed configuration and tests, not guessed CLI
flags. Same-principal hostile writers, Binder-external executions, deleted whole
stores/ledgers and hidden Agent-side retries are outside complete-history claims.
Capture establishes capture-time bytes, not agent authorship or equality with bytes
at process exit. Times and PIDs are diagnostics, not attribution authority.

### Implemented local ledger v1 integration boundary

`ai_agent_evidence_binder::ledger` implements this local history without assigning
Core authority to its frames. `LedgerWriter::open` holds the actual file's OS lock;
the caller must place it outside both Store and capture roots and retain it through
managed execution. `append_intent` returns a non-Clone `SpawnPermit` only after file
and containing-directory sync. Consuming that permit is not proof of execution.
The process route must still validate Q and bind the actual child and owned pipes.

`append_observation` retains typed caller observations, not authenticated Agent
activity. `append_publication` requires independent exit-zero and both EOF
observations with no fault. Separate Store prepare and commit calls require separate
`OutputPreparation` and `OutputCapture` intents, followed by `ReviewResult` and
`Admission`. A returned-reference frame follows its intent; explicit uncertainty
cannot later be rewritten as returned success. Receipt text is opaque diagnostic
material, not a ledger-created Core receipt. Failures after publication begins must
be integrated without inventing a successful returned reference or losing the pending
operation; the managed process/Store integration remains a separate required slice.

`read_ledger` reports the validated prefix and exact corrupt/torn boundary.
`ReadReport::attempts()` enumerates every attempt with unresolved launch/publication
projections and `live_witness=false`; reopened writers cannot append observations or
publications to historical attempts. The caller must resolve known references and
uncertain candidates through Core, without resuming execution or registering again.

Concrete ledger limits: 65,536-byte frame payload, 16,384 frames, 1,024 attempts,
67,108,864 file bytes. Command declarations default to and cannot exceed 8 MiB per
pipe, 64 MiB/256 output files, 300,000 ms runtime, 5,000 ms post-exit drain and 64
open descriptors. Smaller positive declarations are allowed; the process and capture
routes must enforce them rather than treating declaration validation as enforcement.

The native Windows focused ledger tests cover actual directory-flush failure,
competing processes, creation races, links, torn/corrupt boundaries and cold reopen.
This is not Linux/macOS qualification, hardware power-loss proof, predispatch root
custody or a working Agent-to-Admission result. Cross-platform archival replay of
native output-root spellings is not claimed by ledger v1.

## Store-owned capture traversal contract

Capture binds one real source-root handle and keeps it for the complete preparation.
Every descendant enumeration, no-follow open, identity recheck and final reread must
consume retained parent handles, not reconstructed ambient paths. The retained
payload rescan must likewise consume its existing payload hold. Portable names are
validated as individual components; inaccessible, ambiguous, non-regular, reparse,
multiply linked or resource-exhausting entries fail rather than disappear from the
inventory. A bounded depth/descriptor budget supplements the existing file, entry,
path and aggregate-byte limits.

This repair does not authenticate initial pathname ancestors or a directory's
predispatch identity. A path-only entry point selects the object successfully bound
at capture start. Binder must not present that as a predispatch retained-root
capability. Any stronger predispatch object binding must be supplied by a narrow
Store-owned live handle acquired before dispatch, not caller-provided identity text.
No check/canonicalize/open sequence establishes that stronger boundary.

On POSIX, a directory descriptor survives rename but does not pin current ancestry.
Membership/identity rechecks detect observed replacement, not adversarial
write-and-restore or an atomic multi-file snapshot. Privileged mount/namespace
changes are not covered. Native enumeration metadata never authorizes payload reads;
the actually opened, validated regular-file handle does. Capture still means the
bytes read and retained, not all outputs at one instant or bytes at process exit.

Native tests must exercise root/intermediate/leaf replacement, no-follow before
payload read, hardlinks, membership changes, resource limits and retained-payload
corruption through the real capture route. Initial rejection leaves no START;
failure after START preserves the incomplete attempt without a successful commit.
This is an implementation requirement, not a claim that these gates already pass.

`AuthoritativeRegistryStore::read_selected_embedded_payload` now supplies the
consumer-facing readback seam. It resolves an exact committed event, validates
selected authority, and returns the complete Manifest with same-index artifact
bytes from the validating Store-owned traversal. Binder must parse those returned
bytes, not validate a digest and then reopen a diagnostic payload pathname.
The returned snapshot is not a new publication receipt or a live execution witness;
subsequent reads revalidate the current payload and may fail after mutation.

Nested publication exposed a Windows read-only-directory flush failure during
public-only readback setup. Nested destinations now acquire identity-matched
publication handles and synchronize the containing directory after creation;
ordinary read holds remain read-only. The exact failed preparation and Windows
access-denied negative control are retained in test evidence, not omitted because
the repaired nested example passes. This is not general destination-race or native
Linux/macOS qualification.

### Managed process and declared capture limits

The implemented `process::run_managed` appends and flushes dispatch intent before
its sole spawn, retains the ledger writer and owned pipes, and returns a private,
non-Clone `ManagedCompletion` only after exit zero, both EOFs and persisted
observations. A Store bridge must consume that witness by value. Historical
observations never reconstruct it. Windows requires an absolute `.exe`; a declared
Python executable with explicit Hermes arguments is distinct from an implicit
batch-file shell. Child-process-tree containment is not provided.

Only the supervisor's conservative Binder-owned descriptor budget of 64 is
implemented. This does not limit child/descendant descriptors or represent an
end-to-end Store handle cap. Runtime/output/drain/termination bounds and errors
remain explicit; synchronous filesystem/spawn calls are not hard-real-time on a
hung filesystem. The direct Windows supervisor tests do not qualify Unix runtime.

`prepare_selected_embedded_freeze_with_limits(input, SelectedEmbeddedCaptureLimits)`
adds narrower file-count and aggregate-content limits to all preparation scans.
The Store checks the actually opened file's observed size before content allocation
or reading; an extra bounded byte remains available to detect growth. Zero limits
reject, and larger limits cannot expand Core's fixed bounds. Metadata, path,
per-file and traversal-depth bounds remain enforced independently. Initial limit
rejection emits no START; later-pass failure preserves the already visible START.
These are local intake controls, not new authority fields. Other authority checks
and cold payload readback still use Core's fixed bounds, not a smaller per-dispatch
memory ceiling. Binder must also check mandatory names and submission candidates
against the complete captured Manifest, without another filesystem inventory.

### Repeated-Admission replay work bounds

Selected terminal replay shares one operation-local 64 MiB work allowance across
the complete terminal history. Each payload traversal, including a binding-cache
hit, still checks current retained bytes. The allowance charges bounded file reads
with their extra-byte growth sentinel, retained path copies, enumeration names and
metadata/container overhead, including empty directories and verification passes.
It does not reserve the entire allowance once per Admission or skip older payloads
to make repeated Admissions fit.

These are work-accounting units, not exact allocator memory, syscall traffic or an
end-to-end process-memory cap. Native enumeration keeps its independent bounds;
one bounded enumeration scratch batch can be allocated before its aggregate charge
is rejected. Per-object, content, entry, name, path and depth limits remain separate.
A large or sufficiently repeated history may still fail closed with a resource
limit. Cold open is not a guarantee of unlimited-history liveness. Public repeated
Admission and exact-budget regressions preserve earlier event bytes and detect
current corruption even on cache hits; final-candidate native gates remain separate.

## Closure gates

### Windows access transitions and current observation limits

The managed supervisor opens the output-root hold read-only. After flushing its
owned stdout/stderr spools, it uses same-object `ReOpenFile` access transitions
before handing a live completion to Store intake. Original writable spool handles
remain held until both reduced-access replacements validate. The final retained
spool handles deny write/delete sharing; this does not prove that capture-time
bytes equal process-exit bytes.

Reusing an existing content-addressed Core Record still needs a fresh operation-local
flush. A read-only Windows guard cannot perform that flush. The reuse path retains
the same object through a read-only custody hold, a writable flush handle, and a
restored read-only seal. DELETE sharing is never granted by this handoff, but WRITE
sharing is briefly necessary between seals. Exact object identity remains held;
uninterrupted content-write exclusion does not. Observable content, identity or
link drift, failed flush, and failed resealing reject the operation. A transition
failure poisons the live Store instance instead of minting durability facts or
silently retrying. An explicit cold open performs new validation, not reconstruction
of a lost successful publication receipt.

A competing Store reader can prevent the writable handoff, so concurrent-reader
publication liveness is not guaranteed. Hostile change-and-restore inside a sharing
window, mapped-file writes, ACL/kernel adversaries, and physical power-cut persistence
remain unestablished. Successful syscalls establish the reported actions, not
strongest-available storage capability. Native Windows regressions exercise these
paths; they do not substitute for final-tree Linux/macOS qualification.

This design did not itself authorize or complete the release. v0.3.0 closure is
recorded separately through exact Git objects, retained native qualification,
historical Hermes Request→dispatch→capture→Admission→cold inspection evidence,
independent review, protected integration, release readback, canonical-main
synchronization, and a sender receipt. Post-release Codex CLI and Claude Code
observations use the same literal-argv route but do not retroactively alter release
closure or native qualification. v0.2.0 remains unchanged and Core-only.
