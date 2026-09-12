![EvidenceRegistry — Decide with evidence. Replay with authority.](docs/assets/evidence-registry-banner.png)

# EvidenceRegistry

Use EvidenceRegistry to check a review against a specified request, target, submitted inputs, and Policy, then retain the inputs and decision for later rechecking—instead of adopting an AI agent's answer unconditionally. It does **not** automatically establish that the review is correct.

The Rust Core provides exact Record handling, retained Journal replay, and one bounded, Store-owned Review Admission path. The **AI Agent Evidence Binder** is a separate v0.3.0 companion that connects a managed agent execution to that path through a generic literal-argv process route. Hermes was its first real-agent integration; bounded post-release Windows observations also exercised that route with Codex CLI and Claude Code. Core and its read-only CLI do not require any agent runtime.

> **Claim boundary:** `STRUCTURALLY_VALID != AUTHORITATIVELY_VALID`. **Policy acceptance != semantic truth.** These are explanations, not new wire enums. A successful decode, replay, inspection, or command establishes only its reported facts. Acceptance in the selected lane means the checked Request/target/submitted-input relationships satisfy the applicable Policy—not that the reviewer is honest, the reasoning is correct, every defect was found, or external action is authorized.

> **v0.3.0 source release:** v0.2.0 remains the immutable Core-only release and does not contain Binder. The Binder implementation and public examples are present in v0.3.0. Its exact implementation tree completed native qualification on Windows x86_64, Linux x86_64, and macOS arm64; the records are identified in the [README verification ledger](docs/README-VERIFICATION-LEDGER.md). This documentation revision does not claim a new native implementation run. Verify the annotated tag, its resolved commit/tree, and matching GitHub Release as the public release locator before building. The retained [Hermes dogfood record](docs/HERMES-BINDER-DOGFOOD-1bbaeea.md) is a historical Windows observation; [post-release Codex CLI and Claude Code observations](docs/BINDER-REAL-AGENT-OBSERVATIONS-v0.3.0.md) are separately bounded Windows records. None establishes semantic correctness, reviewer identity/correctness, independence, formal adapter support, cross-platform runtime qualification, sandboxing, or external authorization.

## Choose a surface

| Surface | Available scope | Boundary |
|---|---|---|
| Rust Core library | Strict Record framing/typed decoding, retained replay, selected Store-owned Freeze/Request/Result/Policy/Admission APIs | Not a general authority engine; no agent runtime dependency. |
| `evidence-registry` CLI | Read-only `journal verify` over explicitly ordered byte files | Journal-only structural/state replay; authority and Admission remain unavailable. |
| AI Agent Evidence Binder | Separate v0.3.0 workspace component; public [`binder` example](agent-evidence-binder/examples/binder.rs) provides `init`, `run`, and read-only all-attempt `inspect`; [design and trust contract](docs/AGENT-BINDER-DESIGN.md) | Native implementation qualification completed on Windows x86_64, Linux x86_64, and macOS arm64. Verify the annotated tag and matching GitHub Release before building. Binder is not in v0.2.0, and its local ledger is not a new Core authority Record. |
| Observed real-agent runtimes | Binder's generic literal-argv process route can invoke an approved configured executable. Hermes is the first real-agent integration; Codex CLI and Claude Code have later bounded Windows observations. | These observations do not make any runtime a built-in/formally qualified adapter or establish other versions/configurations/platforms, reviewer correctness, or independent-review quorum. Native implementation qualification used deterministic fixture routes; Core requires no agent runtime. |

EvidenceRegistry fits evidence-sensitive review workflows where you need to distinguish retained bytes, a reviewer's assertions, and a policy-derived local decision. Use the CLI when the question is only whether the supplied Journal replays structurally. Use the library when you need the selected Store path and can obey its exact storage and authority contract. Do not use either as an automatic correctness oracle, a substitute for independent review, a general evidence database, or deployment authorization. For signing, transparency, supply-chain attestation, metadata discovery, or deployment enforcement, compare the [neighboring tools](#related-tools-and-standards).

### What the terms mean

- **Record:** exact bytes with strict framing and content identity. A typed decoder checks a supported Record-local schema; reference resolution and authority are additional steps.
- **Journal:** retained, ordered entries linking Records and supported state transitions. A valid hash chain is not by itself a trusted history.
- **Store:** the bounded local namespace that retains and validates Records, Journal entries, and selected payloads. Selected producers derive references and publication state from retained Store state.
- **Freeze / Request / Result:** respectively the fixed target evidence, the retained review request for that target, and a bound structured review submission. A later capture of review outputs is not the original target Freeze.
- **Policy / Admission:** the applicable evaluation rules and the selected-lane disposition derived from them. Neither a process exit nor an agent's word `CLEAN` is an Admission.

## Core CLI and Rust library

### Read-only CLI

The [CLI](src/main.rs) exposes exactly one command (no separate `--help` or `--version` command):

```text
evidence-registry journal verify --genesis <path> [--entry <path> ...]
```

The caller supplies the exact Genesis **Journal Entry**, not the Genesis Record, and each later Journal Entry byte file in order using repeated `--entry`. The command does not scan a directory, discover missing entries, open a Store, resolve Record payloads, inspect selected terminals, mutate state, or infer authority. Do not let an agent infer missing entries or reorder the history.

It emits JSON with `output_schema_version: 1` and operation `journal verify`.

| Exit | Meaning |
|---:|---|
| `0` | Supported Journal-only structural/state replay completed (`JOURNAL_ONLY_REPLAY`). |
| `1` | Exact input was structurally rejected (`STRUCTURAL_REPLAY_REJECTED`). |
| `2` | Command-line usage was invalid. |
| `3` | A retained entry is outside the supported replay subset. |
| `6` | A caller-selected input file was unavailable. |

Interpret the JSON outcome, `error_class` when present, and actual process exit together. Successful output has no `error_class`. Even on exit `0`, `authority_status` and `admission_status` remain `UNAVAILABLE`: this surface cannot establish them, rather than having evaluated them as pass or fail. Preserve the binary identity, input bytes, sizes/hashes, order, arguments, stdout/stderr, and process exit together; a narrative about that output is not evidence or authority.

### Rust library

The library provides strict Record framing/identity, retained Journal replay, and a Store-owned selected terminal-closure profile. The selected profile exposes:

- `initialize_selected_profile`
- `prepare_selected_embedded_freeze`
- `commit_prepared_selected_embedded_freeze`
- `record_selected_review_request`
- `record_selected_review_result`
- `derive_selected_review_admission_section_82`
- `complete_selected_review_admission`
- `inspect_selected_terminal`

`derive_selected_review_admission_section_82` is nonpublishing; `inspect_selected_terminal` is read-only. The remaining selected producer APIs can mutate only the selected Store namespace. Their inputs select narrow documented choices; Records, references, anchors, Journal positions, and publication state are derived from retained Store state rather than supplied as caller assertions.

The selected lane is not a general authority engine. It is unavailable to legacy/generic Store profiles and does not infer generic Scope coverage, generic evaluator capability, authority from structural validity, external admission authority, or a trusted producer. The Journal-only CLI is not upgraded into this lane.

## Installation and release identity

EvidenceRegistry is distributed as tagged source from the [official GitHub repository](https://github.com/DwarfM42/EvidenceRegistry) and [GitHub Releases](https://github.com/DwarfM42/EvidenceRegistry/releases), not crates.io. `publish = false` is intentional. Prebuilt binaries are outside the v0.3.0 source release's distribution scope. See the [v0.3.0 release notes](docs/RELEASE-NOTES-v0.3.0.md) and the [v0.2.0 release notes](docs/RELEASE-NOTES-v0.2.0.md) for the historical Core-only release.

Prerequisites are Git with public HTTPS access, Rustup/Cargo and the platform linker. Use the pinned toolchain and components described in [Build and verify](#build-and-verify); Cargo may download the locked dependencies through your normal approved configuration. No private SSH key, author-specific directory, private dispatcher, or Hermes installation is needed for Core.

**Fresh consumer environment:** choose a new, empty, user-authorized parent directory and obtain the official repository over public HTTPS into a new child checkout. v0.2.0 is the Core-only release; pin the annotated v0.3.0 tag and verify its matching GitHub Release before building Binder. The empty-consumer shell recipes remain individually qualified only where the [README verification ledger](docs/README-VERIFICATION-LEDGER.md) records raw evidence; do not infer a shell/platform result from another platform or use the existing-checkout recipe in an unrelated or dirty development checkout.

**Existing checkout:** stop if it is dirty or in use; check ownership of files and running work before changing its revision. Obtain permission for network/ref changes and build-output writes. Never discard changes, force-replace a tag, or reset a checkout to make onboarding pass. The following preserved v0.2.0 recipe is **not re-executed in this restoration draft**; run commands separately and stop on failure:

```sh
git status --short
git fetch https://github.com/DwarfM42/EvidenceRegistry.git tag v0.2.0
git checkout --detach v0.2.0
git rev-parse "v0.2.0^{tag}" "v0.2.0^{commit}" HEAD
```

For a release, retain the annotated tag object, resolved commit, source tree, build commands, and binary hash. Check the resolved commit against the corresponding GitHub Release before building; a moving branch is not a release identity. The published v0.2.0 tag object is `b24619e0d036884652cfd9101d116bf015e629b7`, resolving to `14aa2d2b4ef6342e53e1274acd393b87ab5ee6aa`, tree `702e2a4a497fb605e7c2b7b63d6904e6112ba3bd`. Source identity is provenance, not a trusted-producer attestation.

**Binder installation identity:** do not check out v0.2.0 and try `selected_review_demo` or the Binder example. Use the annotated v0.3.0 tag and verify its tag object, resolved commit, source tree, lockfile, and release page before building. A Cargo version string alone cannot identify changed source. The commands below are not a substitute for platform/shell evidence not recorded in the ledger.

```sh
# Fetch the annotated v0.3.0 tag and verify its matching GitHub Release:
git fetch https://github.com/DwarfM42/EvidenceRegistry.git tag v0.3.0
git checkout --detach v0.3.0
git rev-parse "v0.3.0^{tag}" "v0.3.0^{commit}" HEAD 'HEAD^{tree}'
git status --short
```

Stop if the tag object, resolved commit, `HEAD`, or tree does not match the GitHub
Release record, or if the checkout is dirty/in use. Do not force-move a tag,
reset an existing checkout, or substitute moving `main` for the annotated release.

### Local path dependency

Use a local path dependency when consuming source directly:

```toml
[dependencies]
evidence-registry = { path = "../EvidenceRegistry" }
```

Cargo resolves the path relative to the consuming manifest; adjust it to the actual checkout location. `StrictRecordFrame::decode_authoritative` checks framing/identity, **not** the type-local body schema. Use a corresponding strict typed decoder such as `GenesisRecord::decode_authoritative` where implemented; neither decoding layer resolves references or establishes semantic authority by itself.

Begin with [the executable structural example](examples/inspect_demo.rs), [crate-root APIs](src/lib.rs), and the focused tests. Tests may include controlled or negative fixtures; their direct fixture construction is not automatically a consumer-facing Store producer recipe:

- [`tests/retained_journal_replay.rs`](tests/retained_journal_replay.rs)
- [`tests/freeze_committed_binding.rs`](tests/freeze_committed_binding.rs)
- [`tests/review_admission_runtime.rs`](tests/review_admission_runtime.rs)
- [`tests/journal_verify_cli.rs`](tests/journal_verify_cli.rs)

### Quick start setup — Bash

The following recipe is intended for **Windows Git Bash, Linux Bash and macOS Bash**, not PowerShell or generic `sh`. Start at an authorized v0.3.0 repository root containing the examples. The implementation tree completed native qualification on Windows x86_64, Linux x86_64, and macOS arm64; the [README verification ledger](docs/README-VERIFICATION-LEDGER.md) identifies those records. These observations do not turn the Bash snippets into tested PowerShell recipes; PowerShell execution remains unverified rather than inferred from Bash.

Approve writes under the new `target/readme-public-examples-v1` leaf, including build/temp files, private logs and disposable Stores, before running. Change `RUN_NAME` for a later authorized run; `mkdir` refuses an existing leaf, including partial work. No cleanup or resume is attempted. Check that the checkout is owned, not dirty/in use by other work, and has the intended source identity as described above. This shared setup only creates destinations and a raw-stream/exit recorder; it does not launch an agent.

```bash
case "$(uname -s)" in
  MINGW*|MSYS*) REPO="$(pwd -W)"; EXE=.exe
    export RUSTUP_TOOLCHAIN=1.97.1-x86_64-pc-windows-msvc ;;
  Linux*|Darwin*) REPO="$PWD"; EXE=
    export RUSTUP_TOOLCHAIN=1.97.1 ;;
  *) printf 'Unsupported recipe shell/host\n' >&2; exit 1 ;;
esac
export RUSTUP_AUTO_INSTALL=0
RUN_NAME=readme-public-examples-v1
EXAMPLE_ROOT="$REPO/target/$RUN_NAME"
mkdir -p "$REPO/target" || exit 1
mkdir "$EXAMPLE_ROOT" || exit 1
LOGS="$EXAMPLE_ROOT/logs"
export CARGO_TARGET_DIR="$EXAMPLE_ROOT/build"
export TMPDIR="$EXAMPLE_ROOT/tmp" TEMP="$EXAMPLE_ROOT/tmp" TMP="$EXAMPLE_ROOT/tmp"
mkdir "$LOGS" "$TMPDIR" || exit 1
record() {
  local label="$1" status=0
  shift
  [ ! -e "$LOGS/$label.exit" ] && [ ! -e "$LOGS/$label.stdout" ] &&
    [ ! -e "$LOGS/$label.stderr" ] && [ ! -e "$LOGS/$label.argv" ] || return 2
  printf '%s\0' "$@" > "$LOGS/$label.argv" || return 2
  "$@" > "$LOGS/$label.stdout" 2> "$LOGS/$label.stderr" || status=$?
  printf '%s\n' "$status" > "$LOGS/$label.exit" || return 2
  printf '%s exit=%s\n' "$label" "$status"
  return "$status"
}
record source-identity git rev-parse HEAD 'HEAD^{tree}' || exit 1
record source-state git status --porcelain=v1 --untracked-files=all || exit 1
[ ! -s "$LOGS/source-state.stdout" ] || { printf 'Dirty checkout: stop and preserve work\n' >&2; exit 1; }
record rustc rustc --version || exit 1
record cargo cargo --version || exit 1
record components rustup component list --installed || exit 1
```

Read the source-state and toolchain records before proceeding; a successful `git status` process does not mean a clean checkout. These are local nonauthoritative logs, not a hostile-writer-safe recorder. Keep the raw logs private and retain binary digests with the source/build records; a hash alone is not build attestation. Fresh-consumer builds below use `--locked` and may fetch locked dependencies with approved network access. Retained development gates used `--locked --offline` against an existing cache; add `--offline` only when the needed dependencies are already available. Missing tools/cache are a blocker, not permission to install or change global configuration.

### Bounded positive Core example

[`selected_review_demo`](examples/selected_review_demo.rs) is a public-API-only Core consumer, with no Binder/Hermes/network runtime dependency. It initializes a selected Store; stages typed Scope/Method/Check/CheckSet parameters and registers Policies; captures a synthetic target through Store-owned intake; records a Request and controlled Result; and completes selected Admission. It never writes positive Record/Journal fixtures directly. Its only child process is itself for cold inspection. The label `CONTROLLED_FAKE_NOT_AI` means exactly that: no genuine AI review was performed.

After the shared setup, run the four deliberately different controls, each in its own **absent** root. They do not change Policy to obtain acceptance and are not retries of a failed Request:

```bash
record core-build cargo build --locked -p evidence-registry --example selected_review_demo || exit 1
CORE_DEMO="$CARGO_TARGET_DIR/debug/examples/selected_review_demo$EXE"
if command -v sha256sum >/dev/null 2>&1; then
  record core-binary-sha256 sha256sum "$CORE_DEMO" || exit 1
else
  record core-binary-sha256 shasum -a 256 "$CORE_DEMO" || exit 1
fi
record core-accepted "$CORE_DEMO" run accepted "$EXAMPLE_ROOT/core-accepted" || exit 1
record core-rejected "$CORE_DEMO" run rejected "$EXAMPLE_ROOT/core-rejected" || exit 1
record core-invalid "$CORE_DEMO" run invalid "$EXAMPLE_ROOT/core-invalid" || exit 1
record core-unsupported "$CORE_DEMO" run unsupported "$EXAMPLE_ROOT/core-unsupported" || exit 1
record core-inspect "$CORE_DEMO" inspect "$EXAMPLE_ROOT/core-accepted" || exit 1
```

| Control | Actual source behavior / how to read the retained stdout |
|---|---|
| `accepted` | Controlled Method status `1`, finding state `1`; completed `Satisfied` → `ReviewAdmissionAccepted`. |
| `rejected` | Only the controlled Method status changes to `2`; same Policy requirements, completed `GateUnsatisfied` → `ReviewAdmissionRejected`. |
| `invalid` | Method status `999` fails Result construction before Result publication; unchanged Journal head, no completed Policy or Admission. |
| `unsupported` | Valid Result followed deliberately by the **generic**, not selected, Admission route: `PolicyScopeApplicabilityUnavailable`, unchanged head, no completed Policy or Admission. This does not mean the selected route is unsupported. |

All four controls return **exit 0 only after their expected behavior and separate-process cold readback succeed**. That exit is demo completion, not a common accepted verdict. Exit `1` is unexpected failure/uncertainty; `2` is usage error. Preserve partial roots and raw streams. `inspect` can successfully report an unresolved Request; it is not another Admission attempt.

Inspect the retained stdout for exact five-component Journal references, Manifest/Subject/Anchor identities, controlled claim, Policy/disposition, and live publication durability. The child opens the Store anew, enumerates the retained references, validates Requests/Results and reads Manifest-matched retained payload through public APIs. Its `live_publication_receipt=NOT_RECOVERED` is intentional: cold readback does not reconstruct the earlier live receipt. This is a small demo inspector, not a general Store UI. The [agent report template](#during-execution-recovery-and-reporting) applies; mark Binder/agent-execution fields not applicable rather than inventing them.

The selected demo tree contains `source/{target.txt,nested/check.txt}` and a distinct `store/` with `registry/genesis.cbor`, `records/`, numeric `journal/` slots, `roots/<FreezeAttemptId>/payload/{target.txt,nested/check.txt}`, and `coordination/{publication.lock,freeze/,staging/}`. The source is synthetic and capture is bounded to two files and 4096 content bytes. The attempt ID is an isolated demo input, not production identity generation. Source capture establishes retained bytes, not correctness, authorship or strongest filesystem durability.

### Binder companion: controlled fake example

[`binder`](agent-evidence-binder/examples/binder.rs), with its [dispatch implementation](agent-evidence-binder/examples/support/dispatch.rs), is a **separate executable example**, not an extension of `evidence-registry journal verify`. It uses public Core/Binder APIs for fresh initialization, a single explicitly requested literal-argv dispatch per `run`, and read-only all-attempt `inspect`. There is no historical-log import, resume, automatic retry, Policy adjustment or separate `--help` command.

The recipe below follows the shared setup and completed Core controls. It deliberately reuses **only the Core demo's synthetic source directory** as input, not its Store, Request or Admission. Binder creates its own fresh workspace and target/Request. Input and workspace must be disjoint, approved native absolute paths. The initialized identifiers below are explicitly selected **synthetic inputs for this disposable demo**, not reported output identities or authentication. Use fresh identifiers for real work.

**Before dispatch:** approve the fake executable, synthetic source, private workspace, capture set and explicit limits. Init fixes Freeze Scope profile 2 and Review Scope/Method/Check profile 1 (version 1), HOSTILE role 1/count 1, a minimal Freeze Policy and a Review Policy requiring Method status `[1]`, finding state `[1]` and Anchor relations `[1,2]`. `--approve-demo-policy` acknowledges this fixed **DEMO** Policy; it does not choose a semantic correctness evaluator. Core creates the target Freeze and Request before any run. `binding.json` and `prompt.json` are untrusted locators/prompt material, not capabilities.

```bash
record binder-build cargo build --locked -p ai-agent-evidence-binder --example binder || exit 1
BINDER="$CARGO_TARGET_DIR/debug/examples/binder$EXE"
if command -v sha256sum >/dev/null 2>&1; then
  record binder-binary-sha256 sha256sum "$BINDER" || exit 1
else
  record binder-binary-sha256 shasum -a 256 "$BINDER" || exit 1
fi
SOURCE="$EXAMPLE_ROOT/core-accepted/source"
WORKSPACE="$EXAMPLE_ROOT/binder-workspace"
REGISTRY_ID="$(printf '%064x' 1)"
TARGET_ATTEMPT_ID="$(printf '%064x' 2)"
record binder-init "$BINDER" init --workspace "$WORKSPACE" --source "$SOURCE" \
  --registry-id "$REGISTRY_ID" --target-attempt "$TARGET_ATTEMPT_ID" \
  --source-files 8 --source-bytes 65536 --approve-demo-policy || exit 1
record binder-before "$BINDER" inspect --workspace "$WORKSPACE" || exit 1
```

`binder-before` exposes the Request with no dispatch yet; init exit `0` is not review acceptance. The following helper records **one** dispatch and a separate-process cold inspection even when that dispatch returns nonzero. Each call specifies its expected control exit; an unexpected exit or failed inspection stops the recipe with evidence retained. The four explicit calls are a preselected fake lifecycle demonstration, **not a retry-until-CLEAN loop**. They retain one Request/target/Anchor/Policy and the immediate predecessor chain `a → b → c → d`.

```bash
run_fake() {
  local attempt="$1" mode="$2" capture_number="$3" predecessor="$4" expected="$5"
  local capture_id status=0
  local predecessor_args=()
  capture_id="$(printf '%064x' "$capture_number")"
  if [ -n "$predecessor" ]; then predecessor_args=(--predecessor "$predecessor"); fi
  record "binder-$attempt" "$BINDER" run --workspace "$WORKSPACE" \
    --attempt "$attempt" --capture-attempt "$capture_id" \
    --output "$WORKSPACE/outputs/$attempt" --approve-input-root "$SOURCE" \
    --tool-permissions fake-only-no-network --runtime-ms 10000 --pipe-drain-ms 1000 \
    --stdout-bytes 65536 --stderr-bytes 65536 --output-bytes 1048576 \
    --output-files 16 --open-descriptors 64 "${predecessor_args[@]}" \
    -- "$BINDER" fake-reviewer --workspace "$WORKSPACE" --mode "$mode" || status=$?
  record "binder-inspect-$attempt" "$BINDER" inspect --workspace "$WORKSPACE" || return 1
  [ "$status" -eq "$expected" ]
}
run_fake a nonzero 3 '' 11 || exit 1
run_fake b malformed 4 a 12 || exit 1
run_fake c rejected 5 b 10 || exit 1
run_fake d clean 6 c 0 || exit 1
```

Each subsequent attempt requires a new attempt name, capture ID, absent direct child of `workspace/outputs`, and the exact immediate `--predecessor`; the first attempt must omit it. Reuse is refused even after process failure. Existing/partial workspaces and reservations are not repaired or deleted to permit retries. For real work, a later attempt needs explicit approval rather than being implied by these fake controls.

Capture includes the entire bounded closed output root, with mandatory `review.txt`, `submission.json`, `binder-stdout.bin` and `binder-stderr.bin`. The agent's `artifacts` list is only a candidate list, not a subset mode or permission to read elsewhere. Strict JSON rejects unknown/duplicate keys and unknown statuses; no LLM converts prose to a successful Result. Inspect generated `prompt.json` and the [submission parser](agent-evidence-binder/src/submission.rs) for the actual grammar. Missing/incomplete/excessive outputs do not become completed Policy rejection or silent omissions. A completed output Freeze remains reportable even if later submission parsing fails.

`--tool-permissions` is a caller declaration, **not sandbox enforcement** or a Hermes tool configuration flag. Binder inherits the environment, uses null stdin and the approved output root as cwd, and starts the literal argv after `--` without adding a shell. Windows requires a native `.exe`, not implicit `.cmd`/`.bat` execution. Resource declarations do not authenticate the reviewer, contain the whole process tree or constrain every filesystem/network action. Keep private ledger/approvals/capture material; argv omission in displayed inspection frames is not a general secret sanitizer.

| Companion operation / exit | Narrow meaning |
|---|---|
| `init` / `0` | Target and exact Request retained and cold-validated; no review yet. |
| `run` / `0`, `10` | Store returned accepted or rejected publication respectively; read `admission.reference` and actual durability separately from process exit. |
| `run` / `11` | Managed-process failure; inspect launch, exit, pipe and failure observations. |
| `run` / `12` | Capture/submission/Result/derivation/preflight failure or preterminal Store outcome; **not** completed Policy rejection. |
| `run` / `13` | Publication/ledger/failure-recording uncertainty. Missing receipt is not proof of absent publication. |
| `inspect` / `0` | Valid captured Store view, ledger prefix and complete candidate enumeration; not a verdict that every attempt passed. |
| `inspect` / `14` | Store validation unavailable/failed, corrupt ledger prefix or incomplete candidate enumeration; preserve the available report. |
| `2` | Usage/preflight/init/unreadable-inspection error; partial initialization may already exist. Preserve it. |

**Actual development observation, not a prediction for these new roots:** the retained Windows fake sample's raw `run-a` through `run-d` JSON and command exits yield the following selected-field summary. No output IDs are synthesized, and these are not real-Hermes results:

| Attempt / fake mode | Companion exit | `process_exit` | `result_recorded` | `disposition` | `failure_stage` |
|---|---:|---:|---|---|---|
| a / nonzero | 11 | 7 | false | not_completed | process |
| b / malformed | 12 | 0 | false | not_completed | submission |
| c / rejected | 10 | 0 | true | rejected | null |
| d / clean | 0 | 0 | true | accepted | null |

The same sample's cold `inspect` JSON reports **4 attempts**, **2 `ObservedFailure`** and **2 `HistoricalReferencesValidated`**, **1 Request**, **2 Results**, **1 accepted** and **1 rejected Admission**, and **0 independent reviews established**. Its predecessor fields are null, `a`, `b`, `c`, in that order. This is a projection of retained actual JSON, not replacement raw evidence or a claim that independent reviews occurred.

A completed `run` report emits `inspection_required=true`; usage/preflight errors can instead emit an error report. Read each private `binder-*.stdout/.stderr/.exit/.argv` log and each `binder-inspect-*.stdout` report, not just the latest Admission. Report exact Request, `launch`/`process_exit`, `result_recorded`, Result/output-Freeze/Admission references, failure stage, receipt and all attempt statuses. A null `result_recorded` means Result mutation was attempted without a known returned reference; false means publication was not attempted. Null references alone are never absence proof. Cold inspection revalidates known retained references/payloads without recreating a live witness or historical durability receipt. Displayed Intent argv is redacted and `frames_redacted=true`; preserve the private originals and review all other text before sharing. See [For AI Agents](#for-ai-agents) for recovery and reporting boundaries.

**Real-agent observations:** [a bounded Windows Request-first Hermes run](docs/HERMES-BINDER-DOGFOOD-1bbaeea.md) first exercised the actual installed Hermes executable through Binder and completed a fresh Result/accepted-Admission path with cold inspection. [Later bounded Windows observations](docs/BINDER-REAL-AGENT-OBSERVATIONS-v0.3.0.md) exercised the same literal-argv route with Codex CLI and Claude Code. Do not replace the fake command with guessed runtime flags, treat a copied historical log as a new review, or treat any accepted Admission as a reviewer-correctness or independence verdict. These are runtime/OS observations, not authentication, sandbox, formal-adapter, or general-agent claims.

## Store namespaces and operational boundary

The **selected profile** requires `registry/`, `journal/`, `records/`, `roots/`, and `coordination/{freeze,staging}/`. It validates its retained selected-terminal chain on every cold open and is bounded by the Store limits in [the implementation](src/authoritative_store.rs). It never initializes, repairs, or upgrades an incomplete generic/legacy history. The [Core example](#bounded-positive-core-example) describes its actual selected layout. Binder adds a separate local ledger and private workspace files, not new Core authority namespaces.

For comparison, `AuthoritativeRegistryStore::open(root)` is the **legacy** three-namespace read/derivation seam:

```text
<root>/
  registry/genesis.cbor
  journal/00000000000000000000.cbor
  journal/00000000000000000001.cbor
  ... contiguous twenty-digit slots ...
  records/<lowercase-64-hex-record-id>.cbor
```

`inspect_selected_terminal` performs a read-only selected cold reopen and reports a captured retained head and the latest retained selected Admission, if any. Latest-Admission inspection is not a substitute for Binder's required all-attempt reconciliation.

Do not probe a live or sole-copy Registry with mutation APIs. Every write requires explicit exact-root authorization, a selected-capable adapter, and the route's own retained-state/readback checks. `PublishedReceiptUncertain` remains uncertain: later inspection cannot upgrade it to `Published`, attest historical flushes, or report filesystem state after its captured view.

The selected profile does **not** establish universal filesystem durability, custody, remote replication, hostile same-principal writer exclusion, trusted-producer attestation, external authorization, or production readiness.

## Build and verify

CI and native qualification workflows use Rust/Cargo `1.97.1` with `rustfmt` and `clippy`; this is a tested toolchain pin, not an independently declared universal MSRV. Confirm the actual installed toolchain/components before building. Qualified native platforms are Windows x86_64, Linux x86_64, and macOS arm64. Windows requires the MSVC target (`1.97.1-x86_64-pc-windows-msvc`) and MSVC C++ Build Tools; Linux/macOS require their native linker/toolchain. The Linux suite additionally needs `python3`. Missing prerequisites are not permission to install tools, alter agent configuration, or acquire credentials silently.

### Quick start: Journal-only replay

Use this Bash-compatible recipe from an authorized checkout. It supports Windows Git Bash, Linux Bash, and macOS Bash, with the platform-specific environment setup described below. The implementation tree completed native qualification on Windows x86_64, Linux x86_64, and macOS arm64; that qualification does not itself verify every documented shell recipe.

**Recipe observation:** this exact build/demo/verify sequence ran on Windows x86_64 with Git Bash and Rust/Cargo 1.97.1 on 2026-09-11 (UTC+09:00). It is a **dirty working-tree observation**, separate from the current source candidate's Windows qualification and not a published-release qualification. The build, demo, and replay completed successfully. This recipe itself has not been rerun from the current documentation revision.

**Before writing:** use a trusted, authorized local checkout and a fresh demo leaf. The build writes under `target/readme-restoration-build/`; the example always writes under the compile-time checkout's `target/`, regardless of `CARGO_TARGET_DIR`. It does not open a Store or launch an agent. Keep partial or refused runs; do not delete an unknown directory to reuse its name.

**Shell note:** on Windows Git Bash, native Windows programs need Windows-form environment paths, hence `pwd -W` rather than POSIX `$PWD`. On Linux and macOS, use the POSIX setup in [POSIX shell note](#posix-shell-note) and invoke the binary without `.exe`.

```bash
case "$(uname -s)" in
  MINGW*|MSYS*)
    export RUSTUP_TOOLCHAIN=1.97.1-x86_64-pc-windows-msvc
    export CARGO_TARGET_DIR="$(pwd -W)/target/readme-restoration-build"
    CLI="$CARGO_TARGET_DIR/release/evidence-registry.exe"
    ;;
  Linux*|Darwin*)
    export RUSTUP_TOOLCHAIN=1.97.1
    export CARGO_TARGET_DIR="$PWD/target/readme-restoration-build"
    CLI="$CARGO_TARGET_DIR/release/evidence-registry"
    ;;
  *) printf 'Unsupported recipe shell/host\n' >&2; exit 1 ;;
esac
export RUSTUP_AUTO_INSTALL=0
export TMPDIR="$CARGO_TARGET_DIR/tmp" TEMP="$CARGO_TARGET_DIR/tmp" TMP="$CARGO_TARGET_DIR/tmp"
mkdir -p "$TMPDIR" || exit 1
cargo build --release --locked || exit 1
cargo run --example inspect_demo --locked -- readme-restoration-demo-v1 || exit 1
"$CLI" journal verify --genesis ./target/readme-restoration-demo-v1/genesis-entry.cbor
status=$?
printf 'exit=%s\n' "$status"
```

For a later run, choose a new leaf and change both occurrences of `readme-restoration-demo-v1` consistently. The [example](examples/inspect_demo.rs) accepts a name of 1–80 ASCII letters/digits, hyphens, or underscores, starting with a letter/digit; existing directories are refused. This disposable example is not a hostile-writer-safe publisher.

Observed generated files (loose structural examples, **not** a Store layout):

```text
target/readme-restoration-demo-v1/
  genesis-record.cbor                    # typed Genesis Record
  genesis-entry.cbor                     # the Journal replay input
  ordinary-verification-standalone.cbor   # independent index-24 vector, NOT a successor
  SHA256SUMS.txt                         # SHA-256 of the three CBOR files
  DEMO.txt                              # exact synthetic-input explanation
```

The registry ID is `0x11` repeated 32 times; the capability and environment IDs are respectively `0x22` and `0x33` repeated 32 times. These are **unresolved synthetic references**, not observed capabilities. The ordinary Verification file comes from the [independent vector](vectors/journal-ordinary-verification-v1.txt); its predecessor, dependencies, and Record payload are unresolved. Do not append it to this Genesis with `--entry`.

Actual CLI stdout, formatted from the fresh captured output (process exit `0`, empty CLI stderr; Cargo build messages were retained separately):

```json
{
  "output_schema_version": 1,
  "operation": "journal verify",
  "outcome": "JOURNAL_ONLY_REPLAY",
  "structural_status": "VALID",
  "authority_status": "UNAVAILABLE",
  "admission_status": "UNAVAILABLE",
  "entry_count": 1,
  "journal_head_index": 0,
  "journal_head_hash": "517a93f9463685e478ef1729d5b759eb903b6b82dbe1541fe977473848f27e5c"
}
```

The head hash was checked against SHA-256 of the generated `genesis-entry.cbor`. This replay establishes neither resolved Record prerequisites nor selected Store authority, so both authority and Admission remain `UNAVAILABLE`. The sample is not a Binder execution, positive Store authority example, or proof that a published binary produced this result.

### Windows PowerShell note

The final native recipe must set Windows-form build/temp paths, run the locked build and fresh demo, invoke the `.exe`, and capture `$LASTEXITCODE` immediately after each native command. It must preserve raw stdout/stderr and stop on failure. The Git Bash run above does **not** verify PowerShell syntax or execution. No unexecuted PowerShell demo is presented as runnable evidence here.

### POSIX shell note

For Linux and macOS POSIX shells, use the following environment/build setup. Do not apply its `$PWD` environment setup to Windows native tools. Obtain permission for build/temp writes, run commands separately, and stop on failure:

```sh
export RUSTUP_TOOLCHAIN=1.97.1 RUSTUP_AUTO_INSTALL=0
export CARGO_TARGET_DIR="$PWD/target"
export TMPDIR="$CARGO_TARGET_DIR/tmp" TEMP="$CARGO_TARGET_DIR/tmp" TMP="$CARGO_TARGET_DIR/tmp"
mkdir -p "$TMPDIR"
cargo build --release --locked
```

The POSIX binary is `target/release/evidence-registry` when using that target directory. This particular build → fresh demo → ordered replay recipe has not been executed on Linux or macOS from this documentation revision; do not infer recipe execution from the Windows observation or from broader native qualification.

### Developer verification

With an approved platform build/temp environment, retain every failure as well as later reruns. These existing Core gates are preserved; **they were not rerun as a suite in this documentation draft**:

```sh
cargo fmt --check
cargo test --all-targets --locked
cargo build --release --locked
cargo clippy --all-targets --locked -- -D warnings
cargo test --release --test review_admission_runtime --locked
cargo test --release --test freeze_committed_binding --locked
cargo test --doc --locked
git diff --check
```

Release closeout must bind the annotated v0.3.0 tag, its resolved commit/tree, and the GitHub Release to the completed implementation qualification records. Exact commands and retained evidence belong in the release record, not an invented universal test command.

### Platform qualification

| Scope | Current status | Boundary / next action |
|---|---|---|
| Released Core authority path | Historical release qualification exists for the Core-only v0.2.0 release. | Historical qualification does not qualify the v0.3.0 source candidate. |
| Current Binder implementation tree | Native qualification completed on Windows x86_64, Linux x86_64, and macOS arm64. | The qualification records bind the implementation tree, not semantic correctness or a publication identity. |
| Current Journal quick start | A shared Bash recipe is provided for Windows Git Bash, Linux Bash, and macOS Bash; a Windows x86_64 / Git Bash working-tree build, synthetic demo, and replay were observed. | This specific recipe has not been rerun from the current documentation revision; PowerShell, Linux, and macOS recipe execution is not inferred. |
| Core positive / Binder fake examples | Retained development executions and the native qualification fixture routes exist. | Fixtures remain fixtures; documented shell recipes need their own execution records if represented as tested recipes. |
| Binder common code / subprocess fixtures | `1bbaeea` / `adde784a` passed full native qualification on Windows x86_64, Linux x86_64, and macOS arm64. | This is clearly historical implementation-tree evidence, not qualification of the current documentation revision. |
| Hermes real-agent observation | One Windows/Git-Bash Request-first Hermes run on `1bbaeea` / `adde784a` retained a Result and accepted selected-lane Admission; its exact boundary is in [the dogfood record](docs/HERMES-BINDER-DOGFOOD-1bbaeea.md). | Historical integration observation; native qualification used deterministic fixture routes. It does not establish semantic correctness, reviewer identity, independence, or other adapters. |
| Codex CLI / Claude Code real-agent observations | Separate post-release Windows x86_64 runs on the public v0.3.0 release completed the Request → Result → Admission → cold-inspection chain; their retained attempt histories and runtime bounds are in [the observation record](docs/BINDER-REAL-AGENT-OBSERVATIONS-v0.3.0.md). | Neither runtime is thereby a built-in/formally qualified adapter, cross-platform-qualified, semantically correct, authenticated, independent, or sandbox-proven. |

Neither native tests nor hosted CI establish all-filesystem support, hostile same-principal writer exclusion, universal durability, custody, or production readiness. Unsupported capability and unavailable/preterminal outcomes must remain explicit. Historical platform details are in the [release notes](docs/RELEASE-NOTES-v0.2.0.md); they are not fresh observations of Binder.

## Policy, AI, and downstream interpretation

| Layer | Meaning |
|---|---|
| Individual evaluator | `Pass`, `Fail`, or `Indeterminate` for one evaluator; not completed Policy. |
| Completed Policy (§46) | `Satisfied`, `GateUnsatisfied`, or `GateIndeterminate`; not a publication receipt. |
| Preterminal/context gate | `PreTerminal` or `PolicyContextPrecondition`; no completed Policy result, not a completed rejection. |
| Disposition (§83) | Completed `Satisfied` maps to accepted; completed `GateUnsatisfied`/`GateIndeterminate` map to rejected. Mapping alone is not an Admission Record or Journal append. |
| Publication | Retention/publication outcome is separate from Policy disposition; uncertainty remains uncertainty after later readback. |

The selected terminal route evaluates only its fixed adopted Scope/evaluator constraints. Generic Policy applicability remains unavailable, and generic completion remains preterminal. Invalid, unsupported, missing, and unresolved input must not be silently converted into completed Policy rejection—or acceptance.

### Agent claim / Binder observation / Store decision

The table describes the Binder trust boundary and the existing Core decision boundary. The controlled fake example exercises a bounded route; it is not final Binder or real-agent runtime qualification.

| Material | Authority by itself? | Can influence an authoritative decision? | Established boundary |
|---|---|---|---|
| Exact Store-resolved Review Request | Authoritative prerequisite only in its selected context | Yes: fixes the target and selected review context | Its retained, validated prerequisite chain; not general execution permission. |
| Agent `CLEAN`/`PASS`, structured status, findings | No; untrusted claim | Yes: strictly ingested Result fields can satisfy or fail applicable Policy evaluators | The submission's claim, not its semantic correctness. Free prose is not translated into successful status. |
| Agent artifact list | No; untrusted selection | Selection affects retained material and can prevent ingestion | Candidate paths, not authorization to read arbitrary files or proof of completeness. |
| Binder-owned top-level process exit and pipe bytes | No; direct observations | Only under an explicit adopted evaluator rule; otherwise local completion/capture gates | A managed process/pipe observation, not proof of internal work or the truth of its text. |
| Store-captured bytes | Retained evidence, not independent truth or authorship | Only through defined Record/reference/Policy relations | Bytes read and retained at capture; output Freeze does not automatically become an original Review Policy dependency. |
| Agent internal delegation logs | No; agent-reported material | Mapped structured claims may affect Result; opaque logs are not a new evaluator | Reports about internal tools, subagents and retries, not direct Binder observations of them. |
| Binder build identity | No; provenance | No automatic Policy effect | Build/source/binary linkage to the extent measured, not execution authentication or attestation. |
| Selected Admission | Authoritative disposition within the selected lane | Only under applicable downstream semantics | Policy-derived local decision, not semantic truth, deployment permission, or external authorization. |

A Request's role/reviewer identity text does not authenticate the actual submitter or an internal subagent. Likewise, a request token, matching ID, PID, path, mtime, or output directory is not an authentication boundary. AI explanations, charts, recommendations, and proposed actions remain downstream interpretations of the retained facts.

### Binder workflow and capture boundary

**Implemented controlled route; qualification status is reported above, and bounded Windows real-agent observations are historical/post-release records:**

```text
Target Freeze → exact Review Request / fixed Anchor → retained dispatch intent
  → managed top-level execution → structured submission + output capture
  → Result for the original Request → Store Policy / Admission → cold replay
```

The post-review output Freeze is separate from the target Freeze. It cannot replace the original Request's target or Anchor or make postdispatch evidence appear to have existed before dispatch. The Binder-local attempt/capture/Result join is not a new Core supporting-evidence relation or authority Record. Core retains authority construction and evidence intake; Binder must not assign an accepted terminal outcome itself.

Before dispatch, select the authorized source/capture roots, required outputs, capture-set rule and resource limits. A **closed output set** means all eligible files within that predeclared, verified set—not all agent artifacts or all evidence needed for a correct review. An **agent-selected subset** must be labeled as a subset if supported; it cannot silently stand in for a complete set. The current design selects a closed output-root capture, not a general subset mode. Missing required files, out-of-root paths, excessive output and incomplete streams are failures or unresolved states, never silent omissions.

Store-owned intake performs filesystem identity/link checks, bounded reads, retained copying, digesting and Manifest/Subject construction. Agent digests or a separate Binder copy routine are not substitutes. Capture establishes bytes read at capture time, not authorship, equality with process-exit bytes, or protection against every non-cooperating same-principal writer.

Top-level exit `0`, pipe EOF, and submission completeness are separate observations. Text claiming that a delegated review finished remains agent-reported. Retain every managed dispatch intent before spawn; if required persistence fails, do not launch. Preserve failures, refusals, malformed output, timeout, cancellation, crash, missing results and publication uncertainty alongside any successful attempt. A crash between intent and observed launch remains launch-unresolved. Do not automatically retry until `CLEAN`, erase previous attempts, or count duplicate delivery as another independent review.

Cold reopen must reconcile all managed attempts and known exact Store references, not just the latest Admission. It does not recreate a live process witness or authorize resume/republication after an uncertain interruption. The [companion's `inspect --workspace` command](#binder-companion-controlled-fake-example) provides this read-only all-attempt projection, including available corrupt-prefix/uncertainty reports; read its status and completeness fields. These history boundaries cover Binder-managed records, not hidden internal retries, Binder-external executions, tail/whole-ledger deletion by an administrator, or all activity in the world. Historical logs cannot be retroactively registered as proof of a new dispatch; even a new dispatch cannot prove that an agent did not copy an old answer.

## For AI Agents

### Before installation or execution

1. **Choose the operation and identity.** Ask whether the user needs read-only Journal replay, selected Store operations, or the separate Binder route. Verify the release or exact source commit/tree; do not treat moving `main`, a Cargo version, successful installation, or a build as an authoritative result. Follow [installation](#installation-and-release-identity); v0.2.0 is Core-only, and Binder is available in the v0.3.0 source release.
2. **Distinguish a fresh consumer from an existing checkout.** A fresh consumer starts in an authorized empty location using public HTTPS source. In an existing checkout, inspect dirty/in-use state and ownership and refuse revision changes/builds against unknown concurrent work. Do not clean, reset, overwrite, terminate other work, or force tags to make the instructions fit. Author archive paths are not consumer prerequisites.
3. **Check real prerequisites.** Confirm OS/shell, installed Rust toolchain, components and linker. Keep `Cargo.lock` and use locked builds. Core requires no agent runtime. The [Bash quick-start setup](#quick-start-setup--bash) uses installed Rust/Cargo 1.97.1, native absolute paths and `sha256sum` or `shasum` for binary identity. Before any real-agent route is used, separately obtain approval for the selected runtime configuration, network/provider use, and runtime bounds; do not install or launch an agent merely because this section mentions one. The fake companion recipe is available above; a new runtime's arguments must be explicitly supplied and checked against its installed CLI help. Native implementation qualification is complete; verify the annotated tag and matching GitHub Release as the publication identity.

### Real-agent CLI preflight and launcher contract

Before a Binder `run` dispatches a real-agent CLI, make these checks explicit:

1. **Resolve the actual executable.** On Windows, the managed route requires an absolute native `.exe`; it does not implicitly invoke `.cmd`/`.bat` wrappers through a shell. This is a Windows launch constraint, not a claim that the generic Binder route is Windows-only.
2. **Check authentication and installed CLI help first.** A child can spawn but still fail unauthenticated. Record the installed version and use that version's `--help`; do not transplant flags from another CLI release.
3. **Construct literal argv, not a shell command.** Binder passes each declared element after `--` literally, with null stdin and the approved output root as the child working directory. Keep an argv prompt on one line: an element containing CR, LF, TAB, or another forbidden control character fails preflight. Do not select stdin prompt modes unless the route explicitly supports them.
4. **Place flags at the correct CLI boundary.** Some flags are global while others belong to a subcommand; a syntactically valid option can still fail when placed after the wrong subcommand. The Codex observation records one such retained failure.
5. **Confirm the real output/capture contract.** The child must create required `review.txt` and strict `submission.json` in the approved output root. `--tool-permissions` is a caller declaration, not Binder sandbox enforcement; agent-specific permission/sandbox flags are not proven secure by a completed run.
6. **Preserve attempts; distinguish later execution.** Do not retry automatically for acceptance or `CLEAN`: preserve nonzero/preflight/capture failures, and make a later attempt distinguishable with predecessor history where applicable.
7. **Cold-inspect every outcome.** Process exit, agent claim, Result, completed Policy, Admission, publication state, and all-attempt cold inspection are distinct layers. Inspect after nonzero outcomes too.

### Before writes, agent spawn, or capture

4. **Obtain scoped permission first.** Confirm the exact Store root, build/output destinations, review source, allowed capture roots/set, required files, retention destination and limits. Store mutation, external process execution, network use and artifact reading are separate permissions. Do not test mutation on a live or sole-copy Store. An Admission does not authorize external actions.
5. **Protect secrets at the capture boundary.** Avoid credential/environment dumps and unbounded transcript capture. Do not upload private transcripts, credentials, personal data or capture sets to public repositories, fixtures or services. Read permission is not publication permission. If sharing requires redaction, retain its provenance separately: redacted bytes are not the original captured bytes.
6. **Bind inputs before execution.** For Journal replay retain the exact ordered files and arguments. For Binder, use a Store-validated Request issued for the fixed target before dispatch; keep the intended executable/build identity, capture contract and all-attempt ledger. Do not let an agent-provided token, artifact list or claimed digest choose authority or broaden read permissions. Do not dispatch when required intent retention fails.

### During execution, recovery, and reporting

7. **Preserve facts and all attempts.** Keep source/binary identity, exact input identities/order, command/API inputs, raw stdout/stderr and exit, stream truncation/EOF, capture coverage/missing items, publication outcomes and failure records. Never convert free-form `CLEAN` into Result fields. A structurally valid structured claim may influence Policy; it is not thereby true.
8. **Stop on unknown, refused or uncertain outcomes.** Do not weaken Policy, recreate Requests, change target/Anchor, or repeatedly invoke the agent to get acceptance. An explicit new attempt must retain and expose predecessors. After interruption, reconcile the local ledger and exact retained Store references read-only; preserve corrupt/torn history and unresolved launch/publication. Do not fabricate a live witness, resume an old dispatch, or append a replacement merely because a result is missing.
9. **Report distinct layers.** Installation/build success, process exit, the agent's claim, strict Result ingestion, completed Policy, Admission, publication receipt and cold replay are different results. Say `UNAVAILABLE`, unknown, not observed, or not applicable when appropriate; never fill absent fields with plausible values. Latest Admission alone is not a complete attempt report.

Use this human-readable report template (these labels are **not** promised JSON fields or CLI flags):

| Report item | What to provide when actually available |
|---|---|
| Source / release | Release/tag object if used, exact commit/tree, dirty-source caveat, build command/toolchain and binary digest. |
| Binder build | Actual measured identity and linkage; explicitly state what was not authenticated. Not applicable for Core-only replay. |
| Request and target | Exact retained Request reference, original Freeze/Manifest, selected Policy/Scope/Anchor as exposed by the operation. |
| Direct observation | Managed top-level execution and pipe scope, exit, EOF/truncation, timeout/cancel/launch uncertainty. Do not list internal delegation as directly observed. |
| Agent claim | Submitted statuses/findings and opaque report location, labeled agent-reported. |
| Capture | Authorized closed set or explicit subset, retained identities, required/missing files, limits/truncation and capture-time caveats. |
| Store result | Actual Result binding, Policy completion or preterminal reason, Admission disposition/reference if produced. |
| Publication receipt | The exact reported outcome, including uncertainty; later readback is a separate observation. |
| Cold replay | New-process inspection scope/captured head, exact references/payloads checked, discrepancies and unknowns. |
| Every managed attempt | Successful, rejected, refused, failed, cancelled, timed-out, crash/incomplete and unresolved attempts, with predecessor relationships where recorded. |
| Limits and permission | What remains unestablished and which next action, if any, needs user permission. |

### README verification ledger and recipe-specific execution

The [README verification ledger](docs/README-VERIFICATION-LEDGER.md) defines
the required raw-evidence fields and records historical implementation-tree
qualification plus bounded real-agent observations. The Binder implementation tree
completed native qualification on Windows x86_64, Linux x86_64, and macOS arm64.
Those records do not create an annotated v0.3.0 tag or GitHub Release. Bash
recipes mean Windows Git Bash, Linux Bash and macOS Bash; they are not native
PowerShell recipes.

| Section / recipe | Precise remaining deliverable |
|---|---|
| Empty-consumer installation | Public HTTPS acquisition in isolated authorized scratch; immutable Core vs Binder-containing revision selection; actual Windows PowerShell/Git Bash, Linux/macOS shell runs and identity checks. |
| Existing-checkout installation | Execute safe clean/not-in-use revision selection and identity checks; separately demonstrate dirty/in-use refusal. Do not mutate the canonical development checkout to test this. |
| Journal minimum experience | Rerun build → fresh demo → exact Journal inputs → raw JSON/stderr/exit on the identified v0.3.0 source release; retain hashes and replace this working-tree sample with that evidence. Execute native PowerShell/Linux/macOS variants. |
| Public-example setup / identity / binary hashes | Execute the shared Bash initialization, fresh-root/refusal, raw-stream/exit/argv recorder, installed 1.97.1/toolchain checks and platform hash utilities on Windows Git Bash, Linux Bash and macOS Bash. Supply and execute the native PowerShell equivalent separately. |
| Local path consumer and positive library example | Build/run the documented `selected_review_demo` accepted/rejected/invalid/generic-unsupported controls and separate-process `inspect` from the identified v0.3.0 source release, with retained logs/layout/payload readback. Assemble and execute an isolated path-dependency consumer; a built in-repo example alone is not that fresh-consumer test. The unexecuted platform/shell coverage remains explicit rather than inferred. |
| Binder installation and configuration | Use the annotated v0.3.0 release revision that contains `binder`; acquire it over public HTTPS and verify the matching GitHub Release. Execute its locked build, fresh `init` and predispatch `inspect` with the documented fixed DEMO Policy/source/capture contract. Real-agent runtime configuration is separate; do not point new commands at v0.2.0. |
| Binder fake lifecycle | Execute the explicit `a/nonzero → b/malformed → c/rejected → d/clean` recipe, fresh capture IDs and required immediate predecessors, retaining each nonzero exit and every cold inspection. These commands are intended for the shared Bash setup; PowerShell translation/execution is pending. Regenerate sample summaries from final raw JSON, not this development table. |
| Binder real-agent observations | The historical Windows Hermes run on `1bbaeea` / `adde784a` is recorded in [the dogfood record](docs/HERMES-BINDER-DOGFOOD-1bbaeea.md). [Post-release Windows Codex CLI and Claude Code observations](docs/BINDER-REAL-AGENT-OBSERVATIONS-v0.3.0.md) separately completed the route on the public v0.3.0 source identity. A new runtime/version/platform observation needs its own Request-first run and cold inspection; fake subprocess fixtures do not substitute. |
| Binder interruption and all-attempt inspection | Execute `binder inspect --workspace` from the identified v0.3.0 source release and perform failure/corruption/candidate-completeness checks, with payload/reference validation and all prior failed/unresolved attempts. The fake chain demonstrates observed failures, not every crash/publication-uncertainty scenario. No historical-log import or automatic resume recipe. |
| Release verification and platform report | Bind every release-claimed README command/example to its exact source tree, OS/shell/toolchain, raw stdout/stderr/exit/hashes and status. Preserve Core, Binder/workspace, release-focused, doctest, formal, formatting, warning-denied clippy and Markdown/link gates. Separate real-agent E2E observations from native fixture tests. |

The [restoration inventory](docs/README-RESTORATION-LEDGER.md) records the historical reader capabilities being restored; it does not assert these pending executions passed.

## Related tools and standards

Official sources below were retrieved and checked on **2026-09-10 (UTC+09:00)**. The comparison is about neighboring responsibilities, not a ranking or novelty claim. **No integration with any listed neighbor is implemented**; a shared hash, CBOR encoding, review concept, or eventual Hermes adapter does not imply compatibility.

| Neighbor / type | Main question and useful fit | Relationship to EvidenceRegistry | Implemented ER integration |
|---|---|---|---|
| **in-toto — framework/tools** | Does a supply chain follow its owner-specified layout, authorized functionaries, signed links and artifact rules?[1] | Prefer for planned supply-chain verification. Artifacts must be explicitly supplied for recording; it is not an automatic observer of every file a command touches.[1] ER does not inherit its signing/authorization. | None; evidence-reference/import mapping unimplemented. |
| **Witness — attestation and policy tooling** | Capture SDLC attestations using in-toto and verify with a policy engine including OPA Rego; signing integrations and Archivista storage are upstream features.[2] | Prefer pipeline attestation capture and verification, not merely collection. Process tracing/tampering prevention is marked **Experimental** upstream.[2] | None; Witness's own integrations are not ER integrations. |
| **Archivista — graph/storage service** | Store/discover/retrieve in-toto attestations and query relationships via GraphQL, including review/test/scan context.[3] | Prefer attestation discovery and related metadata; review metadata is not unique to ER. Not an existing ER Store backend. | None; backend/schema/identity mapping unimplemented. |
| **Sigstore Rekor — transparency log** | Record/query signed metadata, obtain inclusion proofs and verify log integrity/consistency.[4] | Prefer externally verifiable signature transparency. Local Journal replay is not a Rekor proof or substitute for a transparency service. | None; anchoring/upload/proof verification unimplemented. |
| **SCITT — standards architecture** | Signed-statement registration under a transparency service's policy and verifiable receipts; registration does not establish statement accuracy.[5] | Evaluate the architecture and a concrete implementation's conformance separately. Shared CBOR does not establish compatibility or issuer authentication. | None; statement/receipt conversion and conformance not established. |
| **Grafeas — metadata API/reference implementation** | Store/query/retrieve artifact metadata using higher-level notes and resource-specific occurrences.[7] | Prefer cross-tool software metadata aggregation; an occurrence is not automatically an ER Record, Policy or Admission. | None; occurrence/reference mapping unimplemented. |
| **Ratify — verification engine** | Verify artifact security metadata and admit compliant artifacts for deployment under configured policy, with binary/Kubernetes use described upstream.[8] | Prefer deployment-policy enforcement; that admission boundary is different from selected local Review Admission. | None; ER-result consumption/deployment-policy bridge unimplemented. |

**SCITT publication status:** RFC 9943, *An Architecture for Trustworthy and Transparent Digital Supply Chains*, is a Standards Track architecture RFC published in **June 2026**.[5] The separate SCRAPI status page showed **`draft-ietf-scitt-scrapi-11`**, Active Internet-Draft, in **RFC Ed Queue** at the observation date—not a published API RFC.[6] Check a later official status before making a newer publication claim.

A future bridge would need explicit schema/version, digest/subject identity, trust, privacy, retention and authority mappings plus real tests. None of these systems' signatures, transparency proofs, policy engines, execution authentication or deployment permissions transfer to EvidenceRegistry merely through this comparison.

## Governing authority and provenance

Do not select authority by filename version, source-tree presence, selector equality, or API naming. Begin with the detached adoption records and verify their bound raw document bytes.

1. [Terminal Authority Closure Package Adoption Record](docs/FREEZE-RECORD-TERMINAL-AUTHORITY-CLOSURE-PACKAGE-v0.1.md) binds the terminal-closure package. Its [Core](docs/TERMINAL-AUTHORITY-CLOSURE-CORE-v0.1.md) and [Schema](docs/TERMINAL-AUTHORITY-CLOSURE-SCHEMA-v0.1.md) are normative package members; [Plan](docs/TERMINAL-AUTHORITY-CLOSURE-PLAN-v0.1.md) and [Inputs](docs/TERMINAL-AUTHORITY-CLOSURE-INPUTS-v0.1.json) remain nonnormative analysis/provenance material.
2. [Authority-Path Successor Package Adoption Record](docs/FREEZE-RECORD-AUTHORITY-PATH-SUCCESSOR-PACKAGE-v0.1.md) binds the selected Store, Request/Result, and Freeze path inputs.
3. [Terminal Review Admission Successor-Selector Package Adoption Record](docs/FREEZE-RECORD-TERMINAL-REVIEW-ADMISSION-SUCCESSOR-SELECTOR-PACKAGE-v0.1.md) binds the selected terminal Admission successors.
4. [Lifecycle/Cross-Reference Freeze Record](docs/FREEZE-RECORD-LIFECYCLE-v0.10.4-CROSS-REFERENCE-v0.5.md) binds the earlier adopted lifecycle pair and its frozen foundations.

Some reviewed candidate documents intentionally retain historical `DRAFT` or `NOT IMPLEMENTATION AUTHORITY` headers. The detached adoption records preserve those reviewed bytes and supply the governing post-review disposition; they do not amend the candidate text or broaden its claims. When the adopted specifications do not uniquely determine a semantic lane, the implementation must fail closed rather than infer a reconciliation.

[Formal provenance tooling](scripts/formal/) captures and verifies future SupportImpact evidence without rewriting historical evidence. It is not an EvidenceRegistry authority source.

## License

EvidenceRegistry is available under either the [MIT License](LICENSE-MIT) or the [Apache License, Version 2.0](LICENSE-APACHE), at your option (`MIT OR Apache-2.0`). See [third-party notices](THIRD-PARTY-NOTICES.md) for the locked dependency inventory and source-versus-binary distribution boundary. Licensing does not alter frozen semantics or authorize mutation of a live Registry.

## Sources

[1] https://raw.githubusercontent.com/in-toto/in-toto/develop/README.md

[2] https://raw.githubusercontent.com/in-toto/witness/main/README.md

[3] https://raw.githubusercontent.com/in-toto/archivista/main/README.md

[4] https://docs.sigstore.dev/logging/overview

[5] https://www.rfc-editor.org/rfc/rfc9943.txt

[6] https://datatracker.ietf.org/doc/draft-ietf-scitt-scrapi

[7] https://raw.githubusercontent.com/grafeas/grafeas/master/README.md

[8] https://ratify.dev/docs/what-is-ratify
