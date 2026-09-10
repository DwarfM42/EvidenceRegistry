# EvidenceRegistry

EvidenceRegistry lets you inspect exact Record bytes and replay a retained Journal without turning a successful hash check into a claim of trust. It provides a Rust library and a read-only JSON CLI for Record framing/identity, Journal structural/state replay, and bounded local namespace validation.

> **Claim boundary:** `STRUCTURALLY_VALID != AUTHORITATIVELY_VALID`. A successful decode, replay, or CLI result proves only the facts reported by that operation. It does not by itself establish external authority, admission, policy satisfaction, custody, durability, production readiness, or permission to publish.

## What you can inspect

- **Record bytes:** canonical framing, duplicated type/schema fields, and SHA-256 identity; separate typed decoders check supported Record-local schemas.
- **Journal history:** caller-ordered entries, exact hashes/references, and the implemented state-only transition/reconstruction rules.
- **Local storage:** the Rust store API validates a bounded, exact namespace and retained generation. It is not a general evidence database or collection service.

The Rust library implements one narrow, Store-owned selected terminal-closure lane: selected EMBEDDED Freeze, selected Review Request and Result, selected §82 derivation, and selected Admission publication. It is not exposed by the Journal-only CLI, does not upgrade legacy or generic paths, and is not a production or cross-platform qualification claim. Neither an API name containing `authoritative` nor an outcome enum variant establishes a reachable positive result outside that exact selected profile.

## Installation

The first release uses exact tagged source from the [official GitHub repository](https://github.com/DwarfM42/EvidenceRegistry) and [GitHub Releases](https://github.com/DwarfM42/EvidenceRegistry/releases), deliberately not crates.io. Prebuilt binaries are outside this first release's distribution scope. Use the release's annotated tag and retain its commit identity; an unqualified moving branch is not a release identity.

## Repository layout

- [`publications/`](publications/) contains non-authoritative publication/article-support material, including fact-check material and retained claim-checking context. It is not part of the frozen EvidenceRegistry authority set.
- [`scripts/formal/`](scripts/formal/) contains formal-verification provenance capture/verification tooling for future SupportImpact verification evidence. It is not part of the frozen EvidenceRegistry authority set.

### Publication support

[`publications/`](publications/) materials may record which article claims were confirmed, incomplete, or not rerun. They are supporting records, not frozen EvidenceRegistry authority; later tooling improvements do not close historical evidence gaps. See the [article fact-check](publications/evidence-registry-article-fact-check.md).

## Read-only Journal verification CLI

The [CLI](src/main.rs) exposes one command (there is no separate `--help` command):

```text
evidence-registry journal verify --genesis <path> [--entry <path> ...]
```

The caller selects the exact Genesis bytes and each subsequent Journal Entry byte file in order. The command does not scan a directory, discover missing entries, resolve Record payloads, or infer authority.

For your own retained history, repeat `--entry` in Journal order after `--genesis`; pass the Genesis **Journal Entry**, not the Genesis Record. Preserve the binary hash, ordered input paths/sizes/hashes, exact arguments, raw stdout/stderr, and process exit together. Do not substitute a directory scan or AI-selected order. The sample's ordinary vector must not be passed as `--entry`.

The command emits JSON on stdout with `output_schema_version: 1` and operation `journal verify`.

| Exit | Meaning |
|---:|---|
| `0` | Supported Journal-only structural/state replay completed (`JOURNAL_ONLY_REPLAY`). |
| `1` | Exact input was structurally rejected (`STRUCTURAL_REPLAY_REJECTED`). |
| `2` | Command-line usage was invalid. |
| `3` | A retained entry is outside the supported replay subset. |
| `6` | A caller-selected input file was unavailable. |

Always interpret the JSON outcome and `error_class` together with the actual process exit code. A successful CLI run still leaves Record resolution, Record payload semantics, external authority, policy satisfaction, Review Admission, storage custody, durability, and production qualification unestablished.

## Rust library use

Use a local path dependency; crates.io publication is intentionally outside the first-release scope. In this example, the consuming project's `Cargo.toml` is one directory below the common parent of that project and the EvidenceRegistry checkout:

```toml
[dependencies]
evidence-registry = { path = "../EvidenceRegistry" }
```

Cargo resolves this path relative to the consuming manifest. Adjust it to the actual relationship between the two directories; neighboring directories and this checkout directory name are illustrative, not required.

Start with [the executable example](examples/inspect_demo.rs) and [crate-root APIs](src/lib.rs): `GenesisRecord::new` / `decode_authoritative`, `StrictRecordFrame::decode_authoritative`, `GenesisJournalEntry`, `OrdinaryVerificationJournalEntry`, and `RetainedJournal`. `StrictRecordFrame` does **not** validate a type-local body schema; use the corresponding strict typed decoder where implemented. Neither layer resolves references or supplies semantic authority merely by decoding.

For retained replay, authoritative-store opening, and Review Admission integration, inspect the exact public API in `src/lib.rs` and use these tests as executable examples:

- [`tests/retained_journal_replay.rs`](tests/retained_journal_replay.rs)
- [`tests/freeze_committed_binding.rs`](tests/freeze_committed_binding.rs)
- [`tests/review_admission_runtime.rs`](tests/review_admission_runtime.rs)
- [`tests/journal_verify_cli.rs`](tests/journal_verify_cli.rs)

### Authoritative Registry namespace

`AuthoritativeRegistryStore::open(root)` expects an exact local namespace:

```text
<root>/
  registry/genesis.cbor
  journal/00000000000000000000.cbor
  journal/00000000000000000001.cbor
  ... contiguous twenty-digit slots ...
  records/<lowercase-64-hex-record-id>.cbor
```

`AuthoritativeRegistryStore::open(root)` is the legacy three-namespace read/derivation seam. The selected profile instead requires the exact `registry/`, `journal/`, `records/`, `roots/`, and `coordination/{freeze,staging}/` namespace, validates its selected terminal chain on every cold open, and remains bounded by the Store limits in [the implementation](src/authoritative_store.rs). It does not upgrade legacy or generic histories.

The selected APIs are Store-derived boundaries: `initialize_selected_profile`, `prepare_selected_embedded_freeze`, `commit_prepared_selected_embedded_freeze`, `record_selected_review_request`, `record_selected_review_result`, `derive_selected_review_admission_section_82`, and `complete_selected_review_admission`. Inputs select only the narrow documented semantic choices; Records, references, anchors, journal positions, and publication state are derived from retained Store state. `inspect_selected_terminal(root)` performs a separate selected cold reopen and reports a captured retained head plus the latest retained selected Admission, if any. It is read-only and cannot turn a historical `PublishedReceiptUncertain` outcome into `Published`, attest historical flushes, or report the filesystem after its captured view.

Do not probe a live or sole-copy Registry with any mutation API. Every write requires explicit exact-root authorization, a selected-capable adapter, and the route's own required readback. The implementation has not established a universal durability, custody, trusted-producer, or cross-platform qualification claim.

### Policy results are different layers

| Layer | Meaning |
|---|---|
| Individual evaluator | `Pass`, `Fail`, `Indeterminate`: one evaluator's result, not completed Policy. |
| Completed Policy (§46) | `Satisfied`, `GateUnsatisfied`, `GateIndeterminate`: composed completion, not a publication receipt. |
| Preterminal/context gate | `PreTerminal`, `PolicyContextPrecondition`: no completed Policy result; not `GateUnsatisfied` or `GateIndeterminate`. |

Section 83 maps **completed** `Satisfied` to accepted and **completed** `GateUnsatisfied`/`GateIndeterminate` to rejected; disposition alone is not an Admission Record or Journal append. Generic Policy applicability remains unavailable and generic completion remains preterminal. The selected terminal route evaluates only its fixed adopted Scope/evaluator constraints; it does not provide a general evaluator library or generic Scope inference.

### AI and downstream consumers

An agent can invoke the same read-only CLI and explain its JSON alongside the exact input bytes, hashes, order, and source/binary identity. Its narrative, charts, recommendations, or proposed next actions are **downstream interpretations**, not canonical evidence, completed Policy, or authorization. Keep `UNAVAILABLE` explicit rather than converting it to pass/fail. No AI adapter, external evidence collector, or automatic admission bridge is implemented.

## For AI agents

EvidenceRegistry is intended especially for evidence-sensitive, AI-agent-assisted workflows. The goal is not just to install it, but to retain which qualified release was used: a moving branch must not silently become the execution subject. The initial release deliberately favors inspectable, pinned GitHub source over package-registry convenience; crates.io publication is outside its scope. This is a project distribution choice, not a claim that crates.io lacks version identity or is unsafe. Technically experienced human users can follow the same workflow. **Exact release identity is not an authoritative EvidenceRegistry result.**

Obtain the [official repository over HTTPS](https://github.com/DwarfM42/EvidenceRegistry) and select its [v0.1.0 release](https://github.com/DwarfM42/EvidenceRegistry/releases/tag/v0.1.0), once published. The commands below start at an existing checkout's repository root; no SSH credentials or private development setup are required. Stop if the checkout is dirty or in use by another process. Run each command separately and stop on any failure; do not force-replace an existing tag:

```sh
git status --short
git fetch https://github.com/DwarfM42/EvidenceRegistry.git tag v0.1.0
git checkout --detach v0.1.0
git rev-parse "v0.1.0^{tag}" "v0.1.0^{commit}" HEAD
```

Compare the resolved commit and checked-out `HEAD` with the official release record. Retain the annotated tag object, release tag, and source commit. Then follow [Build a release binary](#build-a-release-binary) immediately below, including its `cargo build --release --locked` step, and retain the built binary hash. Use only the documented [read-only CLI](#read-only-journal-verification-cli) or [public Rust library](#rust-library-use); the demo below is not a live Registry.

Retain the exact command, ordered input identities, raw stdout/stderr, exit code, and supported Record/Journal identities exposed by the operation (such as `journal_head_hash`). Report the release and source identity, the actual result, the bounded claim it establishes, and stronger claims left unestablished. Successful installation/build does not make a result trusted. CLI `JOURNAL_ONLY_REPLAY` with structural `VALID` does not establish terminal authority, Policy satisfaction, Review Admission, custody, or production trust: `authority_status` and `admission_status` remain `UNAVAILABLE`. An AI explanation is not authority.

## Build a release binary

From the EvidenceRegistry repository root, with Rust/Cargo **1.97.1**, rustfmt, and the platform linker installed (MSVC C++ Build Tools on Windows). Use your normal Cargo dependency configuration; Cargo may download locked dependencies if they are not cached. The commands keep build/temp output under repository-relative `target/`. The Linux test suite additionally requires `python3`.

### Windows PowerShell

```powershell
$env:RUSTUP_TOOLCHAIN = '1.97.1-x86_64-pc-windows-msvc'
$env:RUSTUP_AUTO_INSTALL = '0'
$env:CARGO_TARGET_DIR = Join-Path $PWD.Path 'target'
$env:TEMP = Join-Path $env:CARGO_TARGET_DIR 'tmp'
$env:TMP = $env:TEMP
$env:TMPDIR = $env:TEMP
New-Item -ItemType Directory -Force $env:TEMP | Out-Null
cargo build --release --locked
if ($LASTEXITCODE -ne 0) { throw 'Build failed' }
cargo run --example inspect_demo --locked
if ($LASTEXITCODE -ne 0) { throw 'Demo refused or failed; inspect the error' }
$er = '.\target\release\evidence-registry.exe'
& $er journal verify --genesis '.\target\readme-demo\genesis-entry.cbor'
$LASTEXITCODE
```

### Windows Git Bash

Use Git Bash's native Windows path form when setting environment variables for Cargo. From the repository root:

```bash
export RUSTUP_TOOLCHAIN=1.97.1-x86_64-pc-windows-msvc RUSTUP_AUTO_INSTALL=0
export CARGO_TARGET_DIR="$(pwd -W)/target"
export TMPDIR="$CARGO_TARGET_DIR/tmp" TEMP="$CARGO_TARGET_DIR/tmp" TMP="$CARGO_TARGET_DIR/tmp"
mkdir -p "$TMPDIR"
cargo build --release --locked || exit 1
```

The binary is `target/release/evidence-registry.exe`. For the demo, use the following Linux/macOS demo and verification commands with `er=./target/release/evidence-registry.exe`; do not apply that section's environment setup on Windows.

### Linux / macOS

```sh
export RUSTUP_TOOLCHAIN=1.97.1 RUSTUP_AUTO_INSTALL=0
export CARGO_TARGET_DIR="$PWD/target"
export TMPDIR="$CARGO_TARGET_DIR/tmp" TEMP="$CARGO_TARGET_DIR/tmp" TMP="$CARGO_TARGET_DIR/tmp"
mkdir -p "$TMPDIR"
cargo build --release --locked || exit 1
cargo run --example inspect_demo --locked || exit 1
er=./target/release/evidence-registry
"$er" journal verify --genesis ./target/readme-demo/genesis-entry.cbor
status=$?
printf 'exit=%s\n' "$status"
```

The [compiled example](examples/inspect_demo.rs) creates a **new** `target/readme-demo/` directory. Existing directories/files are refused, never replaced. For another run, pass a fresh leaf name: `cargo run --example inspect_demo --locked -- readme-demo-2`, then use that name in the verification path. Names use ASCII letters/digits, hyphens, or underscores and start with a letter/digit. Use a trusted local checkout, not a shared directory with hostile writers; this example is not a transactional or durability-guaranteed publisher. An interrupted run may leave partial files—inspect them rather than retrying over them.

```text
target/readme-demo/
  genesis-record.cbor                    # typed Genesis Record
  genesis-entry.cbor                     # Genesis Journal Entry referencing that Record
  ordinary-verification-standalone.cbor   # independent index-24 vector; NOT a successor
  SHA256SUMS.txt                         # hashes of the three CBOR files
  DEMO.txt                              # synthetic inputs and explicit limits
```

The Genesis registry ID is `0x11` repeated 32 times. Its capability and environment IDs are respectively `0x22` and `0x33` repeated 32 times: **unresolved synthetic references**, not observed capabilities or environment evidence. The example neither opens nor mutates a live Registry and deliberately does not create a store namespace.

The ordinary Verification file comes directly from the [independent vector](vectors/journal-ordinary-verification-v1.txt), checked against its retained digest and strict decoder without using the production encoder to establish expected bytes. Its index is **24**, with unresolved predecessor, dependencies, and Record payload. It is a standalone structural example, **never a Genesis-successor replay fixture**.

## Sample output

Actual stdout from the source-built Windows CLI on the generated Genesis Entry (exit `0`, empty stderr):

```json
{"output_schema_version":1,"operation":"journal verify","outcome":"JOURNAL_ONLY_REPLAY","structural_status":"VALID","authority_status":"UNAVAILABLE","admission_status":"UNAVAILABLE","entry_count":1,"journal_head_index":0,"journal_head_hash":"517a93f9463685e478ef1729d5b759eb903b6b82dbe1541fe977473848f27e5c"}
```

The head hash is SHA-256 of `genesis-entry.cbor`. This is a source-example execution, not a receipt for a published release binary or a new cross-platform qualification. The `VALID` field describes structural replay only; both authority and admission remain `UNAVAILABLE`.

## Verification

From the repository root with the build environment above:

```bash
cargo fmt --check
cargo test --all-targets --locked
cargo build --release --locked
cargo clippy --all-targets --locked -- -D warnings
cargo test --release --test review_admission_runtime --locked
cargo test --release --test freeze_committed_binding --locked
cargo test --doc --locked
git diff --check
```

### Post-release formal provenance follow-up

Post-release fact-checking identified weaker historical source/log provenance binding for SupportImpact than for Lifecycle verification provenance. The historical evidence was not rewritten or retroactively strengthened. PR #4 added the future-only [SupportImpact capture/verifier path](scripts/formal/) (`run_supportimpact.py` and `verify_supportimpact.py`); a fresh Dafny 4.11.0 verification through that path reported `11 verified, 0 errors`. This post-release formal/provenance work is not part of the original v0.1.0 release qualification, does not change the v0.1.0 release subject, and does not establish EvidenceRegistry authority or admission.

### Platform boundaries

The integrated source contains Windows x86_64, Linux x86_64, and macOS arm64 storage adapters. Platform tests and hosted CI exercise bounded behavior, not all operating systems/filesystems or production suitability. The new onboarding example/output above was exercised locally on Windows; its Linux/macOS shell forms are provided without claiming new native executions here.

Windows retains deny-write sharing protection. Linux and macOS instead use cooperative root-object
`flock` serialization and exact identity/byte revalidation; neither constrains writers that ignore
the protocol. Linux explicitly unlocks on orderly owner-process drop, including post-lock errors.
macOS retains its bare File/O_CLOEXEC lifecycle, excluding surviving fork-without-exec descriptor
holders. Neither adapter establishes universal filesystem durability or terminal Review authority.

Native-byte CLI arguments are preserved. Linux tests existing byte-FF filenames; the macOS test
is a filesystem-capability probe and reports errno 92 as unsupported, not as successful replay of
an existing APFS byte-FF name. Missing native-byte paths are tested separately. Linux tests require
`python3`; the default suite runs its six ignored workers through bounded, exact-name supervisors.

## Related tools / how this differs

**in-toto** verifies planned supply-chain steps and authorized actors.[1] **Witness** captures SDLC attestations and verifies them with its own policy engine, including embedded OPA Rego and signing integrations.[2] **Archivista** stores/discovers in-toto attestations and exposes relationships through GraphQL, including reviews, tests, and scans.[3] Prefer these responsibilities for supply-chain verification, attestation capture, and discovery; they are not collection-only tools or capabilities unique to EvidenceRegistry.

**Sigstore Rekor** provides signature transparency, entry inclusion proofs, and log-integrity verification.[4] Prefer a transparency service when that is the requirement; local hash-linked Journal replay is not a substitute.

**SCITT** is a standards architecture, not one executable: **RFC 9943** (Standards Track, June 2026) covers signed-statement registration under service policy and verifiable receipts; registration does not establish statement accuracy.[18] **SCRAPI is separate**: the retained 2026-09-08 source snapshot lists `draft-ietf-scitt-scrapi-11` in `RFC Ed Queue`, not a published API RFC.[8] Check an implementation's actual standard/version alignment rather than inferring compatibility from shared CBOR encoding.

For focus, this short comparison omits standalone treatments of **Grafeas**, an artifact-metadata API using notes/occurrences (overlapping the storage/API discussion), and **Ratify**, an artifact security-metadata verification/deployment-admission engine (a different admission boundary).[11][35] This is editorial scope, not a ranking or a claim that either lacks relevant features.

**All proposed bridges are UNIMPLEMENTED:** in-toto/Witness evidence references, an Archivista backend/discovery mapping, Rekor anchoring, SCITT statement/receipt conversion, Grafeas occurrence references, and Ratify deployment-policy consumption. None is a supported ER integration. Each would need explicit schema, digest, trust, privacy, and authority mappings; ER does not inherit their signing, transparency, policy, or deployment-enforcement guarantees.

## Governing documents

Do not select authority by filename version alone. Start with the detached freeze record and authenticate the exact adopted document bytes:

- [Specification Freeze Record: Lifecycle v0.10.4 + Cross-Reference v0.5](docs/FREEZE-RECORD-LIFECYCLE-v0.10.4-CROSS-REFERENCE-v0.5.md)
- [Evidence Lifecycle Specification v0.10.4](docs/EVIDENCE-REGISTRY-LIFECYCLE-SPEC-v0.10.4.md)
- [Normative Event & Transition Cross-Reference v0.5](docs/CROSS-REFERENCE-v0.5.md)
- [Formal Verification / Implementation Boundary v0.5.4](docs/FORMAL-VERIFICATION-IMPLEMENTATION-BOUNDARY-SPEC-v0.5.4.md)
- [Identity Format v0.3](docs/IDENTITY-FORMAT-v0.3.md)
- [Record Schema v0.3](docs/RECORD-SCHEMA-v0.3.md)
- [Specification Gap Candidates](docs/SPECIFICATION-GAP-CANDIDATES.md) — non-normative unless separately adopted.

Later-numbered candidate documents are not automatically adopted authority. If the adopted specifications do not uniquely determine required behavior, fail closed for that semantic lane rather than inventing or reconciling semantics by intuition.

## License and publication

EvidenceRegistry is licensed under either the [MIT License](LICENSE-MIT) or the [Apache License, Version 2.0](LICENSE-APACHE), at your option (`MIT OR Apache-2.0`). Cargo version `0.1.0` retains `publish = false`: the first release is intentionally distributed through GitHub, not crates.io.

This project license does not override, relicense, or replace third-party dependency, toolchain, or redistributed-component licenses. See [Third-party notices](THIRD-PARTY-NOTICES.md) for the locked dependency inventory, original notices, and source-versus-binary distribution boundary. Licensing changes distribution permissions, not frozen EvidenceRegistry semantics, runtime authority, or permission to mutate a live Registry.

Build and demo outputs belong under the ignored, repository-relative `target/` directory. Historical frozen documents may retain archival paths; those paths are not installation or runtime requirements.

## Sources

- [1] https://raw.githubusercontent.com/in-toto/in-toto/develop/README.md — in-toto
- [2] https://raw.githubusercontent.com/in-toto/witness/main/README.md — witness
- [3] https://raw.githubusercontent.com/in-toto/archivista/main/README.md — archivista
- [4] https://raw.githubusercontent.com/sigstore/rekor/main/README.md — rekor
- [8] https://datatracker.ietf.org/doc/draft-ietf-scitt-scrapi — scitt-api-status
- [11] https://raw.githubusercontent.com/grafeas/grafeas/master/README.md — grafeas
- [18] https://www.rfc-editor.org/rfc/rfc9943.txt — scitt-rfc9943
- [35] https://ratify.dev/docs/what-is-ratify — ratify-docs
