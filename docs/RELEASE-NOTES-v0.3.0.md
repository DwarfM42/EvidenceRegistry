# EvidenceRegistry v0.3.0 release notes

## Release identity

EvidenceRegistry v0.3.0 is a source-only release of the Rust Core and the **AI
Agent Evidence Binder** companion. Verify the annotated `v0.3.0` tag, its
resolved commit and source tree, and the matching GitHub Release before
building. The release is not published to crates.io (`publish = false`) and
provides no prebuilt binaries or binary assets.

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

Binder uses a generic literal-argv managed-process route. Hermes is the first
real-agent integration/dogfood target; [bounded post-release Windows
observations](BINDER-REAL-AGENT-OBSERVATIONS-v0.3.0.md) subsequently exercised
the same route with Codex CLI and Claude Code. These are not built-in or formally
qualified adapters, and they do not establish every runtime version/configuration
or platform. Core and the read-only `evidence-registry journal verify` CLI require
and configure no agent runtime. The Binder ledger is bounded local history, not a
Core Record, signature, custody proof, semantic authority, or
execution-authentication mechanism.

## Qualification status and boundaries

The exact Binder implementation tree completed native qualification on Windows
x86_64, Linux x86_64, and macOS arm64; the [README verification
ledger](README-VERIFICATION-LEDGER.md) identifies the three records. This is
current implementation qualification, not semantic correctness. The annotated
tag, its resolved commit/tree, and matching GitHub Release establish the public
release locator. Any future real-agent observation must bind its own bounded
Request-first runtime run, retained Result/Admission, and cold all-attempt
inspection.

Historical implementation-tree qualification, Hermes dogfood, and post-release
Codex CLI/Claude Code observation records remain useful provenance but do not
qualify this documentation revision beyond their stated bounded facts.

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
Release is the public locator for the final tag object, commit, tree, release
body, and release-time evidence pointers; verify it and the annotated tag
directly.
