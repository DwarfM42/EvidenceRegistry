// PURE_FORMAL property kernel for FV v0.5.4 §§6.1 and 92.1–93.
//
// This model consumes no filesystem, runtime, platform, hashing, serialization,
// Journal, or lifecycle assumptions. It models only the frozen SupportImpact
// total order and max-reduction algebra.
module SupportImpactAlgebra {
  datatype SupportImpact = UNAFFECTED | AFFECTED_UNKNOWN | DEGRADED | AFFECTED

  datatype SupportState = SUPPORT_ACTIVE | SUPPORT_INDETERMINATE | SUPPORT_DEGRADED | SUPPORT_INVALIDATED

  function ImpactRank(impact: SupportImpact): nat {
    match impact
      case UNAFFECTED => 0
      case AFFECTED_UNKNOWN => 1
      case DEGRADED => 2
      case AFFECTED => 3
  }

  // FV v0.5.4 §93: max under the normative §92.1 rank order.
  function CombineImpact(a: SupportImpact, b: SupportImpact): SupportImpact {
    if ImpactRank(a) >= ImpactRank(b) then a else b
  }

  // FV v0.5.4 §93 support-state mapping.
  function SupportStateOf(impact: SupportImpact): SupportState {
    match impact
      case UNAFFECTED => SUPPORT_ACTIVE
      case AFFECTED_UNKNOWN => SUPPORT_INDETERMINATE
      case DEGRADED => SUPPORT_DEGRADED
      case AFFECTED => SUPPORT_INVALIDATED
  }

  lemma CombineImpactCommutative(a: SupportImpact, b: SupportImpact)
    ensures CombineImpact(a, b) == CombineImpact(b, a)
  {
  }

  lemma CombineImpactAssociative(a: SupportImpact, b: SupportImpact, c: SupportImpact)
    ensures CombineImpact(CombineImpact(a, b), c) == CombineImpact(a, CombineImpact(b, c))
  {
  }

  lemma CombineImpactIdempotent(a: SupportImpact)
    ensures CombineImpact(a, a) == a
  {
  }

  lemma UnaffectedIsIdentity(a: SupportImpact)
    ensures CombineImpact(a, UNAFFECTED) == a
  {
  }

  lemma UnaffectedIsLeftIdentity(a: SupportImpact)
    ensures CombineImpact(UNAFFECTED, a) == a
  {
  }

  // Smallest direct consequence toward general SupportStateIsOrderIndependent:
  // a two-element aggregate reaches the same SupportState in either order.
  lemma SupportStateIsOrderIndependentForPair(a: SupportImpact, b: SupportImpact)
    ensures SupportStateOf(CombineImpact(a, b)) == SupportStateOf(CombineImpact(b, a))
  {
    CombineImpactCommutative(a, b);
  }
}
