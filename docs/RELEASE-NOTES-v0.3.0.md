# EvidenceRegistry v0.3.0 release notes

## Release identity

EvidenceRegistry v0.3.0 is a source-only release of the Rust Core and the
**AI Agent Evidence Binder** companion. The annotated `v0.3.0` tag, its
resolved commit and source tree, and the GitHub Release are the release
identity; verify those immutable objects before building. The release is not
published to crates.io (`publish = false`) and provides no prebuilt binaries
or binary assets.

Both workspace packages are version `0.3.0`:

- `evidence-registry` — the Rust Core and read-only Journal CLI;
- `ai-agent-evidence-binder` — the separate companion workspace member.

`v0.2.0` remains an immutable Core-only historical source release. It did not
contain Binder and its tag, GitHub Release, qualification records, and release
notes are not rewritten by v0.3.0.

## Added companion surface

The Binder is a separate workspace component with the public `binder` example:

- `init` creates a fresh Store-backed target and exact Review Request;
- `run` persists intent before a single literal-argv managed process dispatch,
  retains every attempt, captures a predeclared bounded output set, and uses
  Store-owned Result/Admission operations where the retained inputs permit;
- `inspect` is read-only and reconciles the local bounded ledger with known
  retained Store references across all attempts.

Hermes is the first real-agent adapter/dogfood target. Core and the read-only
`evidence-registry journal verify` CLI neither require nor configure Hermes.
The Binder ledger is bounded local history, not a Core Record, signature,
custody proof, semantic authority, or execution-authentication mechanism.

## Qualification record and boundaries

The release record binds retained native qualification evidence for the final
v0.3.0 source subject on Windows x86_64, Linux x86_64, and native macOS arm64,
plus the required locked/offline workspace gates, focused regressions, and
formal support-impact test. It separately retains a bounded Windows
Request-first Hermes run with one Store Result and one accepted selected-lane
Admission followed by cold all-attempt inspection.

These observations do **not** establish semantic correctness, reviewer
correctness or identity, independent review, agent authorship, a sandbox,
complete process-tree containment, all adapters/platforms, hostile-writer
resistance, external authorization, universal durability, or production
readiness. `STRUCTURALLY_VALID != AUTHORITATIVELY_VALID`; Policy acceptance is
not semantic truth.

## Repository surface

The release retains the Core's strict Record framing, retained Journal replay,
and selected Store-owned Freeze/Request/Result/Policy/Admission path. The
Journal CLI remains read-only and Journal-only: successful replay leaves
`authority_status` and `admission_status` unavailable. The selected lane is not
a general authority engine and does not upgrade generic or legacy paths.

For commands, trust boundaries, and raw-evidence requirements, see the
[README](../README.md), [README verification ledger](README-VERIFICATION-LEDGER.md),
and [Binder design and trust contract](AGENT-BINDER-DESIGN.md). The GitHub
Release is the authoritative public locator for the final tag object, commit,
tree, release body, and release-time evidence pointers.
