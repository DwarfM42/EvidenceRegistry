# Lifecycle formal proof frontier v1

**Status:** unverified theorem omitted from the verified model; not a specification gap and not a runtime blocker.

## Scope

`formal/Lifecycle.dfy` has a verified `PURE_FORMAL` state-classification and lifecycle-transition kernel. It proves only the model claims documented in `formal/Lifecycle-tool-provenance-v1.json`.

## Deferred theorem

The following intended symbolic-model theorem was attempted but is **not** present in `formal/Lifecycle.dfy` and must not be claimed as verified:

```text
TerminalPredecessorCanOnlyBeObserved(kind, before, event, after)
  requires LegalLifecycleTransition(kind, before, event, after)
  requires HasTerminality(kind)
  requires StateMatchesKind(kind, before)
  requires IsTerminal(kind, before)
  ensures after == before
```

This expresses the Cross-Reference v0.3 §93 candidate property `OneShotTerminalObjectsDoNotTransition` at the state-only model scope.

## Observed resource frontier

At `2026-08-28T17:52:02+09:00`, Dafny 4.11.0 / Z3 4.12.1 exhausted the retained standard resource limit for that theorem under:

```text
D:/AgentData/evidence-registry-tools/dafny-release-4.11.0/dafny/Dafny.exe verify --cores:1 --resource-limit:100000 formal/Lifecycle.dfy
```

Two semantics-preserving formulations were tried: a direct event match and an explicit exhaustive event split. Both reported:

```text
formal/Lifecycle.dfy(...): Error: Verification out of resource (LifecycleModel.TerminalPredecessorCanOnlyBeObserved)
Dafny program verifier finished with 21 verified, 0 errors, 1 out of resource
```

The unverified lemma was removed rather than published. The retained D:-resident execution evidence is `D:/AgentData/evidence-registry-evidence/lifecycle-transition-green-v1.log` for the succeeding verified model and the preceding red logs; the resource result is recorded here because the green log path was intentionally reused by the succeeding run.

## Disposition

This is a formal-tool resource frontier, not a frozen-spec ambiguity. Do not weaken the property. Continue independently with runtime/vector/lifecycle work whose correctness claim does not depend on this unverified theorem, and return to it only with a proof decomposition that retains the exact property.
