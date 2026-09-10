# EvidenceRegistry

EvidenceRegistry is a Rust library for strict Record and Journal handling plus one narrowly selected, local authority path. Its `evidence-registry` CLI is deliberately separate: it is read-only, Journal-only structural/state replay and never reports authority or Review Admission as established.

> **Claim boundary:** `STRUCTURALLY_VALID != AUTHORITATIVELY_VALID`. A successful decode, replay, inspection, or CLI result establishes only the facts reported by that operation. It does not by itself establish external authority, admission, policy satisfaction, custody, durability, production readiness, or permission to publish.

## Two supported surfaces

### Read-only CLI

The CLI exposes exactly one command:

```text
evidence-registry journal verify --genesis <path> [--entry <path> ...]
```

The caller supplies the exact Genesis Journal Entry and each later Journal Entry byte file in order. The command does not scan a directory, discover missing entries, open a Store, resolve Record payloads, inspect selected terminals, mutate state, or infer authority.

It emits JSON with `output_schema_version: 1` and operation `journal verify`.

| Exit | Meaning |
|---:|---|
| `0` | Supported Journal-only structural/state replay completed (`JOURNAL_ONLY_REPLAY`). |
| `1` | Exact input was structurally rejected (`STRUCTURAL_REPLAY_REJECTED`). |
| `2` | Command-line usage was invalid. |
| `3` | A retained entry is outside the supported replay subset. |
| `6` | A caller-selected input file was unavailable. |

Even on exit `0`, `authority_status` and `admission_status` remain `UNAVAILABLE`. Preserve the input bytes, hashes, order, arguments, stdout/stderr, and process exit together; a narrative about that output is not evidence or authority.

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

EvidenceRegistry is distributed as tagged source from the [official GitHub repository](https://github.com/DwarfM42/EvidenceRegistry) and [GitHub Releases](https://github.com/DwarfM42/EvidenceRegistry/releases), not crates.io. `publish = false` is intentional. Prebuilt binaries are outside this release's distribution scope. See the [v0.2.0 release notes](docs/RELEASE-NOTES-v0.2.0.md) for the supported-surface and qualification boundaries.

For a release, retain the annotated tag object, resolved commit, source tree, build commands, and binary hash. A moving branch is not a release identity. For `v0.2.0`:

```sh
git status --short
git fetch https://github.com/DwarfM42/EvidenceRegistry.git tag v0.2.0
git checkout --detach v0.2.0
git rev-parse "v0.2.0^{tag}" "v0.2.0^{commit}" HEAD
```

Stop if the checkout is dirty or in use by another process. Do not force-replace an existing tag. Check the resolved commit against the corresponding GitHub Release before building.

### Local path dependency

Use a local path dependency when consuming source directly:

```toml
[dependencies]
evidence-registry = { path = "../EvidenceRegistry" }
```

Cargo resolves the path relative to the consuming manifest; adjust it to the actual checkout location. Begin with [the executable example](examples/inspect_demo.rs), [crate-root APIs](src/lib.rs), and the focused tests:

- [`tests/retained_journal_replay.rs`](tests/retained_journal_replay.rs)
- [`tests/freeze_committed_binding.rs`](tests/freeze_committed_binding.rs)
- [`tests/review_admission_runtime.rs`](tests/review_admission_runtime.rs)
- [`tests/journal_verify_cli.rs`](tests/journal_verify_cli.rs)

## Store namespaces and operational boundary

`AuthoritativeRegistryStore::open(root)` is the legacy three-namespace read/derivation seam:

```text
<root>/
  registry/genesis.cbor
  journal/00000000000000000000.cbor
  journal/00000000000000000001.cbor
  ... contiguous twenty-digit slots ...
  records/<lowercase-64-hex-record-id>.cbor
```

The selected profile additionally requires `roots/` and `coordination/{freeze,staging}/`, validates its retained selected-terminal chain on every cold open, and is bounded by the Store limits in [the implementation](src/authoritative_store.rs). It never initializes, repairs, or upgrades an incomplete generic/legacy history.

Do not probe a live or sole-copy Registry with mutation APIs. Every write requires explicit exact-root authorization, a selected-capable adapter, and the route's own retained-state/readback checks. `PublishedReceiptUncertain` remains uncertain: later inspection cannot upgrade it to `Published`, attest historical flushes, or report filesystem state after its captured view.

The selected profile does **not** establish universal filesystem durability, custody, remote replication, hostile same-principal writer exclusion, trusted-producer attestation, external authorization, or production readiness.

## Build and verify

CI and native qualification workflows use Rust/Cargo `1.97.1` with `rustfmt` and `clippy`; this is a tested toolchain pin, not an independently declared universal MSRV. The Linux suite additionally needs `python3`.

```sh
export RUSTUP_TOOLCHAIN=1.97.1 RUSTUP_AUTO_INSTALL=0
export CARGO_TARGET_DIR="$PWD/target"
export TMPDIR="$CARGO_TARGET_DIR/tmp" TEMP="$CARGO_TARGET_DIR/tmp" TMP="$CARGO_TARGET_DIR/tmp"
mkdir -p "$TMPDIR"
cargo build --release --locked
cargo fmt --check
cargo test --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo test --release --test review_admission_runtime --locked
cargo test --release --test freeze_committed_binding --locked
cargo test --doc --locked
git diff --check
```

On Windows, use the equivalent CI toolchain target (`1.97.1-x86_64-pc-windows-msvc`) and MSVC C++ Build Tools. The compiled binary is `target/release/evidence-registry` on Linux/macOS and `target/release/evidence-registry.exe` on Windows.

The [demo](examples/inspect_demo.rs) creates a new `target/<name>/` directory and refuses to overwrite existing files. Its inputs are synthetic and unresolved by design; it neither opens nor mutates a live Store. The ordinary Verification vector is a standalone structural input, not a Genesis-successor replay fixture.

Release-specific native qualification is bound to its exact commit and tree in the release evidence. The authority-path implementation was previously exercised natively on Windows x86_64, Linux x86_64, and macOS arm64; that bounded result does not qualify changed trees, all filesystems, hostile-writer behavior, or production deployment.

## Policy, AI, and downstream interpretation

| Layer | Meaning |
|---|---|
| Individual evaluator | `Pass`, `Fail`, or `Indeterminate` for one evaluator; not completed Policy. |
| Completed Policy (§46) | `Satisfied`, `GateUnsatisfied`, or `GateIndeterminate`; not a publication receipt. |
| Preterminal/context gate | `PreTerminal` or `PolicyContextPrecondition`; no completed Policy result. |

The selected terminal route evaluates only its fixed adopted Scope/evaluator constraints. Generic Policy applicability remains unavailable, and generic completion remains preterminal.

AI agents and downstream systems may invoke the read-only CLI and explain its exact output. Their prose, recommendations, charts, or proposed next actions are downstream interpretations—not canonical evidence, completed Policy, or authorization. No external evidence collector or automatic admission bridge is implemented.

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