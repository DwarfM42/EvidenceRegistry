# Hermes Binder dogfood on `d4f9f4d8`

This is a bounded execution record for implementation commit
`d4f9f4d8e18a144d86c6d3db6b0691d3a03a321e`, tree
`a7d22c056536b2df1674adc42e365b46a4834d27`. It is not a release
qualification, a correctness verdict, or a claim that Hermes' internal work was
observed by the Binder.

## Run boundary

- Platform: Windows x86_64.
- Adapter: the actual installed Hermes executable, dispatched as a direct child
  through the Binder example's literal-argument route.
- Request: Store-issued event `300`, entry `5`, record
  `7acaf95b85a0d007bf91b60162111b3c4636329df13742c724811b6b87d0a09b`.
- Registry: `933096fddfb927ba4edd05605117df7a697dad50345e1c621ee9c5cf3bbbcd9e`.
- Capture rule: closed fresh output root with mandatory `review.txt` and
  `submission.json`, plus Binder-owned stdout/stderr spools.
- The raw command line, prompt, agent stdout/stderr, output artifacts, and
  ledger remain private execution evidence under
  `target/hermes-dogfood-d4f9f4d8/`; this document intentionally does not make
  those private values a public API or a completeness claim.

## All retained attempts

| Attempt | Predecessor | Direct Binder observation | Store result |
|---|---|---|---|
| `hermes-real-1` | none | Spawn observed; process exit `0`; declared agent files were absent from the closed output root | Output capture committed; strict submission/capture completion failed; no Result or Admission |
| `hermes-real-2` | `hermes-real-1` | Spawn observed; process exit `0`; supplying Hermes `--in` did not make its file tool write into the Binder root | Output capture committed; strict submission/capture completion failed; no Result or Admission |
| `hermes-real-3` | `hermes-real-2` | Spawn observed; process exit `0`; required files were present at the explicitly named absolute Binder output paths | Output capture, Review Result, and Admission were retained; the Admission disposition was rejected |

The explicit retry chain is intentional. Attempts 1 and 2 were not replaced,
hidden, or reclassified after attempt 3.

## Retained third-attempt references

| Material | Event | Entry | Record ID |
|---|---:|---:|---|
| Output Freeze | 101 | 11 | `016801d96a7dbbc6df6931e7546fbe5f19a48b4efc9bec33cd6c5403a3a216df` |
| Review Result | 301 | 12 | `fdd8f39640dd5d1f4acec7f2ea9922bcd91a6242170671f050067d9153e03d0b` |
| Admission | 303 | 13 | `ac298e89b7a177377d2a863e65ddb40c774ce19aaec0f7198002d321c5ae81c7` |

The Binder's read-only cold inspection reopened the Store and ledger after the
run. It reported three attempts, two observed failures, one historical
reference-validation outcome, one retained Result, one rejected Admission, and
no accepted Admission.

## What this establishes

- **Authoritative Request:** the Review Request came from the Store and was
  validated before the managed dispatch.
- **Binder observation:** only the top-level child launch, process/pipe facts,
  and the closed-root capture gate were directly observed by Binder.
- **Agent claim:** Hermes supplied a structured submission with an
  environmentally-degraded, indeterminate claim. Its source discussion,
  tool use, internal delegation, and any claimed artifact selection are
  agent-reported rather than Binder-observed facts.
- **Retained Store evidence:** Store intake retained the third attempt's output
  bytes at capture time and produced the Result and Admission records shown
  above.
- **Policy and Admission:** the existing selected Policy evaluated the retained
  Request/Result path and the Store produced a rejected Admission. This does
  not establish that Hermes was objectively right or wrong.

## Important adapter limit observed

Hermes' file tool did not use the Binder child process working directory as its
write location in attempts 1 and 2. A fresh, explicitly named absolute output
path in the agent prompt was necessary for attempt 3. That path is still an
untrusted instruction to the Agent, not an acceptance credential or a sandbox:
closed-root Store capture and mandatory-file checks are what prevented the first
two attempts from becoming Results.

Capture establishes retained bytes at capture time only. It does not establish
that those bytes were present at Hermes completion, that the Agent reviewed a
complete artifact set, that a claim is correct, or that Hermes internal
subagents/transcripts were directly observed.
