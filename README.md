# EvidenceRegistry

EvidenceRegistry lets you inspect exact Record bytes and replay a retained Journal without turning a successful hash check into a claim of trust. It provides a Rust library and a read-only JSON CLI for Record framing/identity, Journal structural/state replay, and bounded local namespace validation.

> **Claim boundary:** `STRUCTURALLY_VALID != AUTHORITATIVELY_VALID`. A successful decode, replay, or CLI result proves only the facts reported by that operation. It does not by itself establish external authority, admission, policy satisfaction, custody, durability, production readiness, or permission to publish.

## What you can inspect

- **Record bytes:** canonical framing, duplicated type/schema fields, and SHA-256 identity; separate typed decoders check supported Record-local schemas.
- **Journal history:** caller-ordered entries, exact hashes/references, and the implemented state-only transition/reconstruction rules.
- **Local storage:** the Rust store API validates a bounded, exact namespace and retained generation. It is not a general evidence database or collection service.

**Not currently available:** positive Freeze semantic authority or successful terminal Review Admission. The public runtime has typed gates and terminal machinery, but current semantic gates prevent that route from reaching publication. Neither an API name containing `authoritative` nor an outcome enum variant establishes a reachable positive result.

## Installation

Use an available matching release asset from [Releases](https://github.com/DwarfM42/EvidenceRegistry/releases), following that release's verified download/checksum instructions. The asset naming convention is:

| Platform | Asset filename (when available) |
|---|---|
| Windows x86_64 | `evidence-registry-windows-x86_64.exe` |
| Linux x86_64 | `evidence-registry-linux-x86_64` |
| macOS arm64 | `evidence-registry-macos-arm64` |

If the selected release does not provide your asset and checksum inventory, use the source quick start below. Verify the original download filename against the release inventory before launch; checksum agreement is not build provenance or proof of safety. This naming table does not establish asset publication, signing, notarization, or a successful download verification.

## Quick start from source

Start in an authorized checkout's repository root with Rust/Cargo **1.97.1**, rustfmt, and the platform linker installed (MSVC C++ Build Tools on Windows). These commands use the existing Cargo dependency cache and keep build/temp output under `target/`. The Linux test suite additionally requires `python3`.

### Windows PowerShell

```powershell
$env:RUSTUP_TOOLCHAIN = '1.97.1-x86_64-pc-windows-msvc'
$env:RUSTUP_AUTO_INSTALL = '0'
$env:CARGO_HOME = Join-Path $HOME '.cargo'
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

### Linux / macOS

```sh
export RUSTUP_TOOLCHAIN=1.97.1 RUSTUP_AUTO_INSTALL=0
export CARGO_HOME="$HOME/.cargo" CARGO_TARGET_DIR="$PWD/target"
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

Use a local path dependency from a neighboring Cargo project; there is no crates.io package installation route:

```toml
[dependencies]
evidence-registry = { path = "../EvidenceRegistry" }
```

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

`AuthoritativeRegistryStore::open(root)` and validation APIs are bounded read/derivation seams; inspect the [store implementation](src/authoritative_store.rs) for exact limits. Current `validate_freeze_committed_authority` fails closed at `SemanticAuthorityUnavailable` after structural checks. Terminal publication machinery exists, but the public `complete_authoritative_review_admission` route cannot currently reach successful publication. Do not treat it as a usable publishing API or probe a live/sole-copy Registry with it; any future reachable mutation requires explicit exact-root authorization and readback.

### Policy results are different layers

| Layer | Meaning |
|---|---|
| Individual evaluator | `Pass`, `Fail`, `Indeterminate`: one evaluator's result, not completed Policy. |
| Completed Policy (§46) | `Satisfied`, `GateUnsatisfied`, `GateIndeterminate`: composed completion, not a publication receipt. |
| Preterminal/context gate | `PreTerminal`, `PolicyContextPrecondition`: no completed Policy result; not `GateUnsatisfied` or `GateIndeterminate`. |

Section 83 maps **completed** `Satisfied` to accepted and **completed** `GateUnsatisfied`/`GateIndeterminate` to rejected; disposition alone is not an Admission Record or Journal append. The retained Policy evaluator currently always returns a preterminal authority-input failure. These type definitions are not examples of a successfully completed authority lane.

### AI and downstream consumers

An agent can invoke the same read-only CLI and explain its JSON alongside the exact input bytes, hashes, order, and source/binary identity. Its narrative, charts, recommendations, or proposed next actions are **downstream interpretations**, not canonical evidence, completed Policy, or authorization. Keep `UNAVAILABLE` explicit rather than converting it to pass/fail. No AI adapter, external evidence collector, or automatic admission bridge is implemented.

## Verification

From the repository root with the quick-start environment:

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

The crate is `UNLICENSED`, version `0.1.0`, and has `publish = false`. Source access does not grant an open-source license; this README does not change repository visibility. A GitHub release is not crates.io publication, a license grant, or permission to mutate a live Registry.

Contributor storage note: `target/` is ignored. Historical frozen documents can retain old external paths; those are not operational dependencies. For this project's controlled workspace, `D:\AgentData` remains output-only, never a build/runtime input.

## Sources

- [1] https://raw.githubusercontent.com/in-toto/in-toto/develop/README.md — in-toto
- [2] https://raw.githubusercontent.com/in-toto/witness/main/README.md — witness
- [3] https://raw.githubusercontent.com/in-toto/archivista/main/README.md — archivista
- [4] https://raw.githubusercontent.com/sigstore/rekor/main/README.md — rekor
- [8] https://datatracker.ietf.org/doc/draft-ietf-scitt-scrapi — scitt-api-status
- [11] https://raw.githubusercontent.com/grafeas/grafeas/master/README.md — grafeas
- [18] https://www.rfc-editor.org/rfc/rfc9943.txt — scitt-rfc9943
- [35] https://ratify.dev/docs/what-is-ratify — ratify-docs
