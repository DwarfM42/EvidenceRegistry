# Hermes Binder dogfood on `98b7646`

This is a bounded Windows execution record for implementation commit
`98b76468fba3cca1462fbd5e5090998c629a2ed2`, tree
`d1f8a0dbdf48b5bde2cce458693359f6ee83cdeb`. It records one accepted
Request-first dispatch against a frozen source snapshot of that exact tree. It
is not release qualification, an independent-review quorum, a correctness
verdict, a claim that Hermes' internal work was observed, or a claim that the
subsequent documentation tree is the executed tree.

## Run boundary

- Platform: Windows x86_64 through Git Bash; the Binder and Hermes child were
  Windows-native executables.
- Adapter: the installed Hermes executable dispatched as a literal direct child
  by the public Binder example.
- Request: Store-issued event `300`, entry `5`, record
  `1c20bf889cd45fef97d80f36801fdbc948128d5616e061eaf6d5c3773d95a33d` in
  registry `0000000000000000000000000000000000000000000000000000000001`.
- Capture rule: fresh closed output root with mandatory `review.txt`,
  `submission.json`, and Binder-owned stdout/stderr spools.
- Source input: a `git archive` snapshot of only the reviewed Binder parser and
  its tests, marked with the exact commit/tree above before `init`.
- Private execution evidence: `target/hermes-dogfood-d1f8a0db-r6/`, including
  literal argv, query, raw streams, exit records, hashes, source identity and
  workspace. The path is not a public API or a completeness claim.

## Preserved prior probes and failed attempts

Earlier isolated path and Binder roots were retained rather than reused:

| Root | Observation | Narrow result |
|---|---|---|
| `D:/AgentData/temp/hermes-file-path-probe-d1f8a0db-r1/` | Hermes rejected an absolute `--in D:/...` launcher argument before file tools. | Launcher parsing failure; no Binder run. |
| `D:/AgentData/temp/hermes-file-path-probe-d1f8a0db-r2/` | With `--in .`, Hermes wrote and read back `path-probe-backslash-v1\n` at the exact canonical `D:\...\backslash.txt` path. | Canonical Windows backslash file addressing is usable by this Hermes installation. |
| `target/hermes-dogfood-d1f8a0db-r4/` | Managed child spawned and exited `1`; `../../source` resolved to the wrong source root. | Observed process failure; no Output Freeze or Result. |
| `target/hermes-dogfood-d1f8a0db-r5/` | Managed child spawned and exited `2`; `--query-file` was resolved after Hermes adopted `--in`. | Observed process failure; no Output Freeze or Result. |

The `r6` harness corrected those relative launch locators and generated the
prompt's output paths from one non-duplicated Windows absolute locator. None of
the failed roots were deleted, resumed, or used as the successful workspace.

## Retained `r6` outcome

The managed Hermes child was observed to spawn and exit `0`. Binder captured
both required agent files and its owned raw pipes, accepted the strict
submission, and cold-inspected the completed workspace.

| Material | Event | Entry | Record ID |
|---|---:|---:|---|
| Output Freeze | 101 | 7 | `a9f03ea7cc7f2b0ddb4a2e4b88e7221bc3a28d33e57a84017a8a72b0ba8f3fa9` |
| Review Result | 301 | 8 | `79d58a1cb3e7a1b26c1ac8327a70a779c9602dd44b3ab5648c97dc16c91a703f` |
| Admission | 302 | 9 | `75486215f37ab86733a0a39676d6f4dd184abac23511b3ac527e83ed825d82aa` |

The direct `run` report recorded `disposition: accepted`, `process_exit: 0`,
`result_recorded: true`, and one known authoritative Admission publication.
The separate cold `inspect` reported one attempt with
`HistoricalReferencesValidated`, one retained Result, one accepted Admission,
no observed-failure attempts, no unresolved attempts, and no established
independent-review quorum.

The captured review is an agent claim: it reported a static source-level
no-blocking conclusion, method status `1`, finding state `1`, no retained
findings and `artifacts: ["review.txt"]`. Binder observed only top-level child
and pipe facts, closed-root capture, strict ingestion, and Store-derived
references. It did not observe Hermes' internal work, establish the review's
semantic correctness, sandbox Hermes, establish authorship, or qualify another
platform or adapter.

## Adapter and evidence limits

The isolated probe established a local file-addressing fact only. The Binder
capture gate, not prompt text or a `--in` setting, determines whether required
output files are retained and eligible for strict Result ingestion. An absolute
output path in an agent prompt remains an untrusted instruction, not a
capability or sandbox boundary.

The stderr spool retained Hermes' warning that a prior update had not restarted
running gateways. This run did not change Hermes configuration or restart a
gateway, and the warning is not treated as a cause of the accepted execution.

This record does not replace exact-final-documentation-tree execution, Linux or
macOS native qualification, independent review, release qualification, or
publication authorization.
