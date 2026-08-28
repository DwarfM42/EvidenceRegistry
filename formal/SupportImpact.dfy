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

  // Retained external theorem name from the initial verified bootstrap.
  lemma SupportStateIsOrderIndependentForPair(a: SupportImpact, b: SupportImpact)
    ensures SupportStateOf(CombineImpact(a, b)) == SupportStateOf(CombineImpact(b, a))
  {
    CombineImpactCommutative(a, b);
  }

  // FV v0.5.4 §99 records loss for one exact Closeout independently of later
  // raw support observations. It has no Journal or environment premise.
  datatype TerminalSupportFloor = NO_TERMINAL_FLOOR | DEGRADED_FLOOR | INVALIDATED_FLOOR

  function ApplyTerminalSupportFloor(
      floor: TerminalSupportFloor,
      rawState: SupportState
  ): SupportState {
    match floor
      case INVALIDATED_FLOOR => SUPPORT_INVALIDATED
      case DEGRADED_FLOOR => SUPPORT_DEGRADED
      case NO_TERMINAL_FLOOR => rawState
  }

  function ExtendTerminalSupportFloor(
      floor: TerminalSupportFloor,
      observed: SupportState
  ): TerminalSupportFloor {
    if floor == INVALIDATED_FLOOR || observed == SUPPORT_INVALIDATED then INVALIDATED_FLOOR
    else if floor == DEGRADED_FLOOR || observed == SUPPORT_DEGRADED then DEGRADED_FLOOR
    else NO_TERMINAL_FLOOR
  }

  lemma SupportInvalidationIsAbsorbing(rawState: SupportState)
    ensures ApplyTerminalSupportFloor(INVALIDATED_FLOOR, rawState) == SUPPORT_INVALIDATED
  {
  }

  lemma SupportDegradationIsNonReviving(rawState: SupportState)
    ensures ApplyTerminalSupportFloor(DEGRADED_FLOOR, rawState) == SUPPORT_DEGRADED
  {
  }

  lemma CloseoutSupportIsNonReviving(floor: TerminalSupportFloor, observed: SupportState)
    ensures floor == INVALIDATED_FLOOR ==> ExtendTerminalSupportFloor(floor, observed) == INVALIDATED_FLOOR
    ensures floor == DEGRADED_FLOOR ==> ExtendTerminalSupportFloor(floor, observed) != NO_TERMINAL_FLOOR
  {
  }

  lemma IndeterminateMayResolveUnaffected()
    ensures ApplyTerminalSupportFloor(NO_TERMINAL_FLOOR, SUPPORT_INDETERMINATE) == SUPPORT_INDETERMINATE
    ensures ApplyTerminalSupportFloor(NO_TERMINAL_FLOOR, SUPPORT_ACTIVE) == SUPPORT_ACTIVE
  {
  }
}
