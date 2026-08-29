# EvidenceRegistry Document Encoding Note v0.1

**Status:** NON-NORMATIVE REPOSITORY FACT
**Scope:** Line-ending state of tracked files as of this note
**Purpose:** Record why line endings differ across tracked paths, and prevent
normalization of frozen documents.

Normative specification documents under `docs/` are stored with CRLF line
endings, except `docs/EVIDENCE-REGISTRY-LIFECYCLE-SPEC-v0.10.2.md`, which was
committed with LF and is frozen at that byte identity.

Source, test, vector, and formal-model files use LF.

The frozen specifications and their successors carry the `-text` gitattribute.
Git performs no line-ending conversion on them. Their committed bytes are their
identity.

This note records an existing repository fact. It is non-normative. It does not
define EvidenceRegistry encoding semantics and does not alter any frozen
specification.

It does not authorize normalizing the line endings of any frozen document.
Changing the line endings of a frozen document would change its SHA-256 and
invalidate every audit binding that cites it.

Lifecycle §46 requires that LF and CRLF not be silently normalized. The mixed
state recorded here is therefore a consequence of treating document bytes as
identity, not a defect to be repaired.
