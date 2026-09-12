# Hermes Binder dogfood on `1bbaeea`

This is a bounded Windows execution record for implementation commit
`1bbaeea170322602fe60664bf6e4c4f10fadcb8e`, tree
`adde784afd6a45818739ab122a08aa3416390d90`. It is neither a release
qualification nor a correctness, authorship, independence, sandboxing, or
complete-review verdict.

## Run boundary

- Platform: Windows x86_64, Git Bash launcher.
- Adapter: the installed Hermes executable was a direct managed Binder child.
- Request: Store-issued event `300`, entry `5`, record
  `10e3ea10191104ebd6ae595bdfb63523403ff4541c71bac0b764884e8d58f001`.
- Capture: a fresh closed output root with mandatory `review.txt` and
  `submission.json`, plus Binder-owned stdout/stderr spools.
- Limits: 300000 ms runtime, 5000 ms pipe drain, 1 MiB each stdout/stderr,
  4 MiB across 32 output files, and 64 open descriptors.
- Evidence root: `target/hermes-dogfood-adde784a-final/` (ignored, private).
  Its `identity.txt`, `logs/*.argv`, `logs/*.stdout`, `logs/*.stderr`,
  `logs/*.exit`, workspace Store/ledger, and `evidence.sha256` retain the raw
  execution material. The private literal argv is redacted in cold inspection.

## Observed attempt and cold inspection

The single Request-first attempt, `hermes-final`, was persisted before spawn.
Binder recorded `SpawnObserved`, stdout EOF (392 bytes), stderr EOF (249
bytes), and child exit `0`. The child supplied both required agent files.

After producer, reader, and writer shutdown, read-only `binder inspect` exited
`0`. It reported one attempt, thirteen ledger frames, one Request, one Result,
one accepted Admission, zero rejected Admissions, zero observed failures, zero
unresolved attempts, and zero independently established reviews. The Request,
Output Preparation, Output Capture, Review Result, and Admission retained
references each revalidated as `Validated`; the attempt status was
`HistoricalReferencesValidated`.

The script-level `hermes-run=0` and `inspect_after=0` are retained in
`outcome.txt`. The evidence manifest hash is
`f1058d022c43a51cb0354e30b399e8d5eacf1cfad55384fc303e24af74cd85ed`.

## What this establishes—and does not

The Store issued and validated the Request before managed dispatch. Binder
observed the direct child lifecycle, bounded pipe capture, closed-root capture,
and retained Store references. The resulting Store path retained a Result and
an accepted selected-lane Admission.

This does not establish semantic truth, Hermes' internal work, reviewer
identity or authorship, independent review, full process-tree containment,
sandboxing, hostile-writer resistance, output completeness, or external
authorization. The accepted Admission is a policy outcome for the retained
Request/Result path; it is not proof that the review is correct.
