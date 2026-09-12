# README restoration inventory

Status: restoration work inventory, **not** the final README command verification
ledger and not a claim that new Binder functionality has been implemented.

Comparison inputs are exact Git objects:

- PR #6 merged README at `ec37ebb736cc0dfb7092bc39f3dc1457db9053ff`.
- v0.2.0 README at `14aa2d2b4ef6342e53e1274acd393b87ab5ee6aa`.
- Initial current main at the same v0.2.0 commit, tree
  `702e2a4a497fb605e7c2b7b63d6904e6112ba3bd`.

The latter two README blobs are identical (`82c86e4` abbreviated Git blob shown by
Git); the older diff starts at blob `33b7f57`. Full revision identities above, not
these abbreviations, select the inputs. Old claims are discovery material only.

| Old location | User-facing role | v0.2.0 / initial main | Restoration destination | New support required |
|---|---|---|---|---|
| Opening / What you can inspect | Explain purpose before APIs | Replaced by technical surfaces; no Binder | README opening and surface table | Implemented Binder capabilities, honest three-layer boundary |
| Installation | Official source and release pin | Stable v0.2.0 checkout block exists, no empty consumer start | README installation plus For AI Agents | Fresh HTTPS source acquisition/build in authorized consumer scratch; stable vs exact Binder revision split |
| Repository layout / Publication support | Distinguish publication evidence from authority | Removed; formal scripts retain link | README provenance, not restoration of retired artifacts | Check active references; preserve ignored archive and old evidence |
| Read-only Journal verification CLI | Exact ordered inputs, exits and evidence retention | Mostly retained | README CLI quick start and output interpretation | Execute final binary with actual generated Genesis; retain stdout/stderr/exit and hashes |
| Rust library use / executable example | Concrete typed construction/decoding | Short API/test link list remains | README executable library walkthrough | New public-only Store provisioning, Request/Result/Admission/cold reopen example; do not repackage fixture direct writes as real production |
| Authoritative Registry namespace | Distinguish legacy and selected layouts | Legacy tree plus selected namespace prose retained | README selected/legacy layout | Run selected Store example; inspect actual layout and link exact mapping contract |
| Policy results are different layers | Separate evaluator, Policy, preterminal and publication | Table retained in shortened form | README outcome guide | Fresh accepted, completed rejected, and preterminal results; retain uncertainty separately |
| AI and downstream consumers | AI narrative is not authority | Short prose says no collector | README Agent claim / Binder observation / Store decision | Update only after Binder works; clarify internal Hermes logs remain Agent reports |
| For AI agents | Version-pinned installation→execution→retention→human report | Entire dedicated section removed | Dedicated README For AI Agents section | Fresh final-candidate commands, dirty/in-use stop, output permissions, whole attempt inspection and report template |
| Windows PowerShell build/demo | Native path/env/build/run/exit sequence | Replaced by 'equivalent' target prose | README Windows PowerShell instructions | Execute in actual PowerShell runtime; capture commands and exits; no assumed shell equivalence |
| Windows Git Bash build | Native Windows paths for Cargo environment | Removed | README Windows Git Bash instructions | Execute actual documented environment and commands; do not reuse POSIX `$PWD` as a native Windows path |
| Linux / macOS build/demo | Executable shell sequence and fresh demo leaf | Reduced to build/tests | README POSIX instructions | Execute on configured native Linux x86_64/macOS arm64 canonical environments; final tree only |
| Demo layout / synthetic reference explanation | Know exact inputs and why no authority | One paragraph remains; concrete files and hashes removed | README synthetic CLI demo | Fresh generated files/output; independent index-24 vector must not be passed as Genesis successor |
| Sample output | Real JSON and exact exit, meaningful limits | Removed | README newly observed sample blocks | Regenerate after final functionality; raw outputs retained, no manually manufactured/normalized semantics |
| Verification | Reproducible gates | Core gates retained | README verification / final command ledger | Run all-targets, focused release suites, docs, fmt, clippy, formal and Markdown/link gates without removing any |
| Formal provenance follow-up | Distinguish old gaps and future capture tooling | General tooling sentence remains | README historical provenance link | Historical counts stay historical; rerun any count presented as current, never infer older gaps closed |
| Platform boundaries | Actual adapter semantics, special-path limits | General native release scope retained | README qualification table | New exact-tree native output; retain failures; separate real Hermes E2E platform from subprocess fixtures |
| Related tools / how this differs | Select ER vs other responsibilities | Entire comparison removed | Short README table + public detailed comparison | Fresh official sources for in-toto, Witness, Archivista, Rekor, SCITT, Grafeas, Ratify; standards distinguished from products |
| Proposed bridges | Avoid inherited signing/transparency/admission claims | Removed with comparisons | README comparison integration column | All seven external bridges remain unimplemented; first Hermes adapter does not change that |
| Governing documents | Trace exact adopted semantics | Improved detached adoption chain | Preserve README adoption entry points | Verify final local links; retain historical headers without reopening C1–C3 |
| License and publication | Source licensing vs runtime authority | Retained and shortened | README license + release availability | No new release/tag/package publication; third-party notices reflect actual locked dependencies |

## Fresh evidence requirements

Every current README example must be executed against the final candidate source,
with README section, exact command/API example, platform/shell, exact commit/tree,
observed exit, stdout/stderr/result or hashes, PASS/FAIL and intentional platform
variation. This inventory is not that execution ledger. README text must never
claim unfinished rows are verified. Historical v0.2.0 evidence and illustrative
syntax are labeled separately from freshly observed current behavior.

A Binder demonstration starts at a real Store-issued Request, uses the actual managed
Hermes dispatch, retains submission and captured artifacts through Store intake,
records actual Result/Policy/Admission outcome and cold-reopens it. A controlled
fake reviewer demo is useful but cannot stand in for the real Agent run. The
Binder-capable exact source revision must not be mislabeled as part of v0.2.0.
