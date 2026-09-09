# Specification adoption handoff checklist

**Status: NON-AUTHORITATIVE operational guidance.** This is not an adoption or
freeze record, a semantic specification, or an implementation qualification.
Completing this checklist does not grant adoption, runtime authority, or
publication permission. It does not alter any historical decision or bytes.

## When to use it

Use this checklist before a future successor is described as adopted in another
document, selected as an implementation input, or handed to a reviewer as an
adopted contract. Its purpose is to make that claim's evidence discoverable and
re-verifiable, not to require a successor or impose a new specification rule.

The [existing detached freeze record](../docs/FREEZE-RECORD-LIFECYCLE-v0.10.4-CROSS-REFERENCE-v0.5.md)
already demonstrates exact package binding, preserved candidate status, review
bindings, Owner disposition, and separation from implementation qualification.
The [README](../README.md#governing-documents) explains authority selection.
This checklist adds a repeatable handoff and evidence-availability check; it does
not replace either document or infer adoption from similar version histories.

## Future handoff

- [ ] **Name the claim and its boundary.** Identify the proposed successor,
  affected contract, required companions, and unchanged predecessors. Distinguish
  frozen semantic predecessors, adopted contract inputs, and rejected or merely
  historical candidates. Do not infer package membership from a later citation.
- [ ] **Bind exact bytes.** For each predecessor, successor, and required
  companion, record a repository-relative path, Git blob ID, byte size, and full
  SHA-256 from raw bytes. Retain the reviewed candidate commit and tree. Check
  these identities again at handoff; any changed bytes need their own review
  disposition rather than inherited approval.
- [ ] **Locate the actual decision.** Where Owner acceptance applies, retain the
  explicit decision and a verifiable identity for its source, with exact scope
  and disposition. Bind the detached decision artifact by locator, size, and
  full SHA-256. Separate the reviewed candidate commit/tree from the decision's
  recording commit/tree and the explicitly selected effective commit/tree. If
  the effective boundary is unspecified, report it as unspecified; a commit
  message or timestamp alone does not select it.
- [ ] **Bind review and acceptance receipts.** Retain the required raw review
  reports, authentication/closeout receipts, and acceptance evidence, each with
  locator, byte size, full SHA-256, actual disposition, and exact subject binding.
  Check that their subjects match the proposed package. A reported clean review
  is not equivalent to re-authenticating its raw report; neither is a hash a
  substitute for available bytes.
- [ ] **Check availability at the next handoff.** Confirm which exact artifacts
  the intended reviewer can retrieve through authorized access and verify those
  available bytes. Record unavailable or access-restricted items and the resulting
  limit on the claim. For public guidance, use repository-relative links or
  approved public artifact locators, not workstation paths or private receipts.
  Do not publish private evidence merely to make this checklist complete.
- [ ] **Preserve history and stop at the evidence limit.** Keep reviewed candidate
  headers, blocked candidates, and missing historical evidence unchanged. Do not
  backdate acceptance, reconstruct missing receipts as originals, or claim that
  an unavailable decision proves rejection or non-adoption. A later recovered
  artifact needs a separately dated verification; it does not rewrite an earlier
  availability observation. Leave adoption unestablished by the available
  evidence when its binding cannot be authenticated.
- [ ] **Keep the result in its lane.** Link the authenticated decision rather than
  copying semantic rules into an operational note. State separately what the
  specification decision establishes and what remains unestablished about
  implementation, formal verification, runtime qualification, or production.
  Any semantic ambiguity still belongs to its specification owner.

No historical gap is closed by this checklist. Future adoption decisions and any
needed specification changes remain separately authorized work.
