# Hermes Binder dogfood on `0a44589`

This is a bounded Windows execution record for commit
`0a4458938dca5cf08c1da9af581b3edf9727f911`, tree
`0fc482329d06580f880e7ad5bd525d6ffe5f5807`. It is the fresh Request-first
rerun after Binder began requiring an absolute executable locator on every
platform. It records one accepted dispatch against a frozen source snapshot of
that exact tree.

It is not release qualification, an independent-review quorum, a correctness
verdict, a claim that Hermes' internal work was observed, or a claim that a
subsequent documentation-only tree is the executed tree.

## Run boundary

- Platform: Windows x86_64 through Git Bash; Binder and Hermes were
  Windows-native executables.
- Adapter: the installed Hermes executable was the literal direct child of the
  public Binder example. The retained private argv is not reproduced here.
- Request: Store-issued event `300`, entry `5`, record
  `80bc7f1b77c57e186d1239ea678607b6057a86ab2386df0b47c3e973c2ec081b`, hash
  `739165f382f1d8b258abc706eedbde8ff37daeb8d1db5d3ddc1f9eed107b9b09`, in
  registry `0000000000000000000000000000000000000000000000000000000001`.
- Capture rule: a fresh closed output root with mandatory `review.txt`,
  `submission.json`, and Binder-owned stdout/stderr spools.
- Source input: a frozen subset marked with the exact commit/tree above before
  `init`; it was not an assertion that Hermes reviewed the complete repository.
- Limits: 300,000 ms runtime, 5,000 ms pipe drain, 1 MiB per pipe, 4 MiB
  output capture, 32 output files, and 64 open descriptors.
- Private execution evidence:
  `target/hermes-dogfood-0fc48232-r7/`, including literal argv, query, raw
  streams, exit records, hashes, source identity, workspace, and cold report.
  That path is neither public API nor a completeness claim.

## Retained outcome

The wrapper recorded `hermes_run=0 inspect_after=0`. Binder observed the child
spawn, process exit `0`, and the required capture set; it accepted the strict
submission, recorded Result, completed selected Admission, and then cold-opened
the workspace.

| Material | Event | Entry | Record ID |
|---|---:|---:|---|
| Output Freeze | 101 | 7 | `ddc7980fbc4aebf118c2ff6bd4b671443955e5a140adebcc6a95ebb07049f988` |
| Review Result | 301 | 8 | `ec9e2c1d48dcd41bd18d415ac048a719d202fed36f75adcbc4748a1f0a5f514c` |
| Admission | 302 | 9 | `926104256279124b7cec1bb6b726356c336d166b262597b5576667fee75ace51` |

The direct `run` report recorded `disposition: "accepted"`,
`launch: "observed_spawn"`, `process_exit: 0`, and `result_recorded: true`.
The separate cold `inspect` reported one attempt with
`HistoricalReferencesValidated`; 13 complete ledger frames; one Request, Output
Freeze, Result, and accepted Admission; no observed failures; no unresolved
attempts; and no established independent-review quorum.

The submitted claim had schema `1`, exact Request binding, method status `1`,
finding state `1`, empty reason codes/findings, and `artifacts: ["review.txt"]`.
The captured static review reported no blocking finding within its named source
boundary. This remains an agent claim. Binder did not observe Hermes' internal
work, prove review correctness/authorship, sandbox Hermes, establish an
independent review, or qualify another platform/adapter.

## Retained failed-attempt context

Earlier launcher and managed-run failures remain preserved in their own roots
and in [the earlier `98b7646` record](HERMES-BINDER-DOGFOOD-98b7646.md).
They were not deleted, resumed, or reused for this run. The literal malformed
`D/…` output tree from the older `r3` locator failure was archived outside the
canonical repository with its byte identities before clean-checkout restoration;
it is evidence of that closed failure, not a successful capture.

## Limits

An output path named in the prompt is untrusted instruction, not a capability or
sandbox boundary. This observed managed-child/capture/Store path does not
establish semantic truth, hidden-tool absence, authenticated identity,
process-tree containment, full artifact inventory, broad filesystem guarantees,
release readiness, or publication authorization.

This record does not replace final exact-tree README command execution, native
Linux/macOS qualification, independent review, protected PR checks, or release
qualification.
