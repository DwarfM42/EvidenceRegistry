# EvidenceRegistry v0.2.0 release notes

## Release identity

When published, the `v0.2.0` source release will be identified by its annotated `v0.2.0` tag, resolved commit, and source tree. Its GitHub Release will record the exact release subject and qualification evidence. EvidenceRegistry is intentionally not published to crates.io (`publish = false`), and no prebuilt binary distribution is provided.

## Added library capability

v0.2.0 adds a bounded, Store-owned selected terminal-closure path to the Rust library:

- selected `EMBEDDED` Freeze preparation, commit, and retained replay;
- selected Review Request (`300`) and Review Result (`301`) production and replay;
- selected nonpublishing Review Admission §82 derivation;
- selected terminal Admission publication (`302` accepted, `303` rejected);
- selected cold replay and bounded read-only terminal inspection.

The path derives Records, references, anchors, Journal positions, and publication state from retained Store state. It uses selected namespace validation, lock/revalidation, immutable no-replace publication, readback/replay validation, bounded resource controls, and an explicit `PublishedReceiptUncertain` outcome.

## CLI and scope boundaries

The `evidence-registry` CLI remains one read-only command:

```text
evidence-registry journal verify --genesis <path> [--entry <path> ...]
```

It performs caller-ordered Journal structural/state replay only. It does not open a Store, discover data, resolve Record payloads, mutate state, inspect selected terminals, or establish authority/admission. Its successful output continues to report `authority_status: "UNAVAILABLE"` and `admission_status: "UNAVAILABLE"`.

The selected library profile is not a generic authority engine and does not upgrade legacy/generic Store profiles. It does not establish generic Scope/evaluator inference, external admission authority, custody, trusted-producer attestation, universal durability, hostile-writer exclusion, remote replication, production readiness, or automatic AI admission.

## Qualification boundary

Native qualification is release-subject-specific. Before publication, the exact release candidate must complete native Windows x86_64, Linux x86_64, and macOS arm64 gates; the corresponding GitHub Release will identify that commit/tree and gate evidence. This qualifies the bounded tested adapter behavior only; it does not qualify all filesystems, hostile-writer behavior, or production deployment.

## Repository surface

The release refreshes the README to distinguish the read-only CLI from the selected mutable library profile, to name the controlling detached adoption records, and to state the operational/durability boundaries directly.

Six obsolete public support artifacts were retired after tracked-reference analysis and retained byte-for-byte in a local-only ignored archive in the release workspace. Governing authority documents, required build/test/formal/provenance inputs, and active vectors remain tracked. Git history remains the durable public record for retired source bytes.
