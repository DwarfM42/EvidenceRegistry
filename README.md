# EvidenceRegistry

EvidenceRegistry is a Rust implementation of strict EvidenceRegistry Record and Journal framing, retained-Journal replay, bounded authoritative-store validation, and the current Review Admission runtime surface.

> **Claim boundary:** `STRUCTURALLY_VALID != AUTHORITATIVELY_VALID`. A successful decode, replay, or CLI result proves only the facts reported by that operation. It does not by itself establish external authority, admission, policy satisfaction, custody, durability, production readiness, or permission to publish.

## Repository layout

- `src/lib.rs` — public Rust library surface.
- `src/main.rs` — owner-facing read-only CLI.
- `src/authoritative_store.rs` — bounded authoritative Registry-store implementation, re-exported selectively from the crate root.
- `tests/` — executable behavior and boundary tests.
- `docs/` — specifications, freeze/governance records, and non-normative gap records.
- `formal/` — formal models and retained verification provenance.
- `vectors/` — exact test-vector material.

## Build a release binary

The supported local workspace is:

```text
D:\Desktop\Sandbox\hermes\EvidenceRegistry
```

From Git Bash, build with Cargo output and temporary files inside the repository's ignored `target/` tree:

```bash
REPO='D:/Desktop/Sandbox/hermes/EvidenceRegistry'
cd /d/Desktop/Sandbox/hermes/EvidenceRegistry
export CARGO_HOME="$HOME/.cargo"
export CARGO_TARGET_DIR="$REPO/target"
export TEMP="$REPO/target/tmp"
export TMP="$TEMP"
export TMPDIR="$TEMP"
mkdir -p "$CARGO_TARGET_DIR" "$TEMP"
cargo build --manifest-path "$REPO/Cargo.toml" --release --locked
```

Use the explicit native-style `D:/...` value above for environment variables consumed by Windows-native tools. Do not replace it with Git Bash's `/d/...` `$PWD`; some native-tool paths can otherwise be misinterpreted as `D:/d/...`.

The release executable is:

```text
D:\Desktop\Sandbox\hermes\EvidenceRegistry\target\release\evidence-registry.exe
```

`target/` is intentionally ignored by Git. `D:\AgentData` is output-only for this project: do not use files, caches, tools, copied sources, fixtures, or previous evidence from that directory as build or runtime inputs. Historical frozen documents may retain old external-path observations as immutable evidence; those paths are not operational dependencies and must not be followed to obtain inputs.

## Read-only Journal verification CLI

The binary currently exposes one owner-facing command:

```text
evidence-registry journal verify --genesis <path> [--entry <path> ...]
```

The caller selects the exact Genesis bytes and each subsequent Journal Entry byte file in order. The command does not scan a directory, discover missing entries, resolve Record payloads, or infer authority.

Example from Git Bash:

```bash
mkdir -p target/evidence/journal-verify
set +e
./target/release/evidence-registry.exe journal verify \
  --genesis '/path/to/genesis.cbor' \
  --entry '/path/to/00000000000000000001.cbor' \
  >target/evidence/journal-verify/result.json \
  2>target/evidence/journal-verify/stderr.txt
status=$?
set -e
printf '%s\n' "$status" >target/evidence/journal-verify/exit-status.txt
```

Before execution, record the ordered input paths, sizes, and SHA-256 values. Do not source the input files from `D:\AgentData`.

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

Add this repository as a local path dependency from the consuming Cargo project:

```toml
[dependencies]
evidence-registry = { path = "D:/Desktop/Sandbox/hermes/EvidenceRegistry" }
```

Strict Record-frame decoding is available at the crate root:

```rust
use evidence_registry::StrictRecordFrame;

let bytes = std::fs::read("/path/to/record.cbor")?;
let frame = StrictRecordFrame::decode_authoritative(&bytes)
    .map_err(|_| "record framing rejected")?;

println!("record type: {:?}", frame.record_type_id());
println!("schema version: {}", frame.schema_version());
println!("record id: {:02x?}", frame.record_id().as_bytes());
```

`StrictRecordFrame` validates canonical Record framing, duplicated type/schema fields, and exact-byte self-hash identity. It deliberately does not validate every type-local body schema, resolve retained references, or establish authority or admission.

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

Opening and validation APIs are read/derivation seams. `complete_authoritative_review_admission` can publish a new Record and Journal Entry. Do not call that terminal mutation API against a live or sole-copy Registry without explicit authorization for the exact root, accepted inputs, expected mutation, and post-operation replay/readback.

## Verification

Run the repository quality gates with the same project-local target and temporary environment used for the release build:

```bash
cargo fmt --check
cargo test --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo test --release --test review_admission_runtime --locked
cargo test --release --test freeze_committed_binding --locked
cargo test --doc --locked
git diff --check
```

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

The crate is `UNLICENSED`, has `publish = false`, and is not configured for crates.io publication. Remote push, release, deployment, and live Registry mutation are separate authorized operations; a local build or passing test suite does not authorize them.
