# Bounded real-agent CLI observations on v0.3.0

This record preserves bounded **post-release Windows x86_64 observations** of the
AI Agent Evidence Binder's generic literal-argv managed-process route. It does
not amend the annotated `v0.3.0` release, qualify a runtime as a formal adapter,
or change Core/Binder authority semantics.

## Shared source and trust boundary

Both observations used the public v0.3.0 source identity:

| Identity | Value |
|---|---|
| Annotated tag object | `5732feda91508a3b254632dd9be13f74c5fe1a9e` |
| Resolved commit | `db4490f88b12b7fce2da24786b7802405d2ee16c` |
| Source tree | `632e323a21a4bd1ec0fb9b8656bdd1d9524cc95b` |
| Platform | Windows x86_64 |

In each completed observation, Binder retained a Target Freeze and exact Review
Request before dispatch, managed one literal-argv child, captured its bounded
outputs, strictly parsed `submission.json`, recorded a Result against the
original Request, completed selected Policy/Admission, and then used a separate
process for cold inspection. The separate historical Hermes dogfood record
remains at [HERMES-BINDER-DOGFOOD-1bbaeea.md](HERMES-BINDER-DOGFOOD-1bbaeea.md).

These are observations of a managed top-level process, retained files, strict
submission handling, and the selected Store path. They do **not** establish
semantic correctness, reviewer correctness or identity, authorship,
authentication, independence, sandbox correctness, complete process-tree
containment, output completeness, external authorization, production readiness,
or that another version/configuration/platform will behave the same way. An
accepted Admission remains a policy-derived local disposition, not proof that an
agent claim is true.

## Codex CLI observation

**Classification:** `CODEX_REAL_AGENT_BINDER_OBSERVATION_COMPLETED`

| Field | Observed value |
|---|---|
| Runtime | Codex CLI `0.154.0-alpha.6.2` |
| Executable | Native Windows executable at a private local path |
| Executable SHA-256 | `081E4DE4BE8E38FAC6ED4D95E3B1A0B9F6D31C090DDC36E1696B349FE406F575` |
| Observed CLI login state | ChatGPT-authenticated CLI observed; no account/session material is published |
| Model identity | `UNKNOWN` |
| Observed configuration | `workspace-write`; approval `never` |
| Invocation shape | `codex.exe --ask-for-approval never exec --ephemeral --sandbox workspace-write --cd <approved-output-root> "<single-line prompt>"` |

The final managed Codex process was observed to spawn and exit `0`. It produced
`review.txt` and strict `submission.json`; the submission parsed with
`method_status: 1`, `finding_state: 1`, no reason codes, and no findings. The
Binder recorded the Result against the original Request, an Output Freeze, and a
completed Policy with accepted Admission. Separate-process cold inspection
succeeded with complete candidate enumeration, a `ValidPrefix` ledger, and no
unmatched Request/Result.

Two earlier failed observations were retained rather than overwritten: one was
rejected by Binder argv preflight because the prompt contained forbidden control
characters; another spawned Codex successfully but failed in Codex CLI argument
parsing because a global option was placed after the `exec` subcommand. A later
explicitly initiated attempt corrected the invocation and completed. This is not
retry-until-`CLEAN`; the failed observations remain part of the attempt history.

## Claude Code observation

**Classification:** `CLAUDE_CODE_REAL_AGENT_BINDER_OBSERVATION_COMPLETED`

| Field | Observed value |
|---|---|
| Runtime | Claude Code `2.1.201` |
| Executable | Native Windows PE32+ executable at a private local path; not a node/npm wrapper |
| Executable SHA-256 | `fb804ee019bfbb8d7e85abf965e528e53b5aa5a4e4ebc0f164139dc10a9e0320` |
| Observed CLI login state | claude.ai / Pro authenticated CLI observed; no account/session material is published |
| Model on completed attempt | `claude-sonnet-5` |
| Relevant caller-selected flags | `--permission-mode acceptEdits`, `--add-dir <workspace>`, `--add-dir <source>`, `--no-session-persistence`, `--tools "Read,Write,Glob"`, `--max-budget-usd 5.00` |

The final managed Claude Code process was observed to spawn and exit `0`. It
produced `review.txt` and strict `submission.json`; Binder parsed the submission,
recorded its Result against the original Request, completed Policy as `Satisfied`,
and recorded an accepted Admission. Separate-process cold inspection succeeded
with complete candidate enumeration, `ValidPrefix` ledger status,
`ValidatedCapturedView` Store status, and no unresolved discrepancy.

One earlier attempt failed because the CLI was not logged in. A subsequent
authenticated attempt reached real model execution but was stopped by a
caller-selected budget cap. After explicit approval for a later predecessor-linked
attempt with a higher cap, the lifecycle completed. This is not
retry-until-`CLEAN`: the failed attempts remain retained and the later attempt was
separately authorized.

## What the observations mean operationally

Hermes remains the first real-agent integration and its historical dogfood record
is unchanged. Codex CLI and Claude Code were subsequently exercised through the
same generic literal-argv process route. Neither becomes a built-in or formally
qualified adapter from these observations, and no real Codex/Claude Code execution
is inferred for Linux or macOS. The existing three-OS native qualification applies
to the Binder implementation tree and deterministic fixture routes, not to every
real-agent runtime.

Private raw argv, transcripts, authentication output, provider/session data, and
other personal material are intentionally not reproduced here. The retained
observations support the bounded statements above; they are not public proof of
account identity or provider-side execution details.
