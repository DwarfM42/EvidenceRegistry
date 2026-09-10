# README verification ledger

This ledger defines the verification record required for README commands and
API examples. It is deliberately separate from
[`README-RESTORATION-LEDGER.md`](README-RESTORATION-LEDGER.md), which inventories
historical public material and does not prove that a command was executed.

A completed record binds each row to one exact Git tree and stores raw command
arguments, stdout, stderr, child exit status, platform/toolchain, and output
hashes under an ignored `target/` evidence root. The source document may name
that evidence root, but hashes alone are not a substitute for retained output.

## Required fields

| Field | Requirement |
|---|---|
| README section | Heading and the command/API example being exercised. |
| Exact candidate | Commit and tree resolved immediately before execution. |
| Platform | OS, architecture, shell, Rust/Cargo toolchain and relevant target. |
| Command/API | Literal executed argv or the public API/example scenario. |
| Exit | Actual exit code; nonzero expected controls are marked as such. |
| Output | Raw stdout/stderr locations and SHA-256, or an exact output hash where private. |
| Status | `PASS`, `EXPECTED_NONZERO`, `FAIL`, or `NOT_RUN`; no zero-match selection may be recorded as PASS. |
| Variation/limit | Intentional platform variation and the narrow conclusion. |
| Reality | `observed`, `controlled fake`, or `illustrative only`. |

## Current command surfaces

| README section | Required evidence | Reality / limit |
|---|---|---|
| Public-example setup | Identity, source-state safeguard, toolchain/components, recorder files, and fresh-root refusal. | Observed separately per native shell. A dirty/in-use safeguard must be reported rather than bypassed. |
| Bounded positive Core example | Locked build; accepted, rejected, invalid, unsupported, and cold-inspect commands. | Controlled fake semantic Result; not an AI review. |
| Binder controlled fake example | Locked Binder build; init; predispatch inspect; `a → b → c → d` with every cold inspection. | Controlled fake reviewer; each expected nonzero remains evidence, not a skipped failure. |
| Hermes Binder run | New Store Request; literal managed process; retained capture; Result/Admission where present; all-attempt cold inspection. | Actual adapter execution; Agent claims and internal delegation stay untrusted/agent-reported. |
| Journal quick start | Release build, fresh `inspect_demo`, ordered journal replay, raw JSON/stderr/exit. | Journal-only structural replay; authority/admission remain unavailable. |
| Developer gate | Formatting, workspace tests, doctests, warning-denied Clippy, formal script, Markdown/link check, release build and applicable focused suites. | A test pass is not platform qualification or reviewer correctness. |
| Native qualification | Same exact tree on Windows x86_64, Linux x86_64 and macOS arm64. | Native execution only; no Docker/cross-compilation substitution. |

## Current known records

- `target/readme-verification-c29482d1/` is a Windows Git Bash execution record
  for pre-ledger tree `b9a23ee2186c2123420a3a397378ca15232938f0` at commit
  `c29482d190263f83697e8c23dbf58eda7f5145a3`. It covers the Core controls and
  Binder fake lifecycle with raw argv/stdout/stderr/exit files. Its
  `source-state` safeguard correctly exposed an existing, preserved, zero-byte
  untracked `src/.hermes-tmp.laTrQG`; the subsequent disposable examples ran,
  but that local checkout did not satisfy the README's clean-checkout
  precondition. It is therefore useful execution evidence, not final candidate
  qualification.
- `docs/HERMES-BINDER-DOGFOOD-0a44589.md` records the separate actual Windows
  Hermes run on commit `0a44589`, including a fresh accepted Request-first
  path and cold inspection. It is not a replacement for
  exact-final-documentation-tree rerun or native multi-platform qualification.
- `docs/HERMES-BINDER-DOGFOOD-98b7646.md` remains retained historical context
  for the earlier launcher probes and accepted implementation-tree run.

After the final source/documentation tree is committed, create a new machine
record beneath `target/` using every field above. The final handoff must name
that evidence root and summarize each row without replacing raw output with a
narrative.
