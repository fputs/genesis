# Design

## Context

`multiple_attack_penalty` in `crates/rules/src/check.rs` takes `MapInput` (`on_own_turn`, `prior_attack_trait_checks_this_turn`, `current_weapon_agile`) and returns `i32`. Off the creature's own turn it returns 0 and ignores the count. The caller owns the count. `attack_modifiers` takes that `i32` and has no penalty table. `Encounter::attack_trait_penalty` passes `on_own_turn` from the open turn and returns 0 otherwise. See proposal.md for why the off-turn zero is too wide. The behavior to implement is `specs/dice-and-checks/spec.md`.

## Goals / Non-Goals

**Goals:**

- One penalty table. A readied attack uses the attack count frozen when Ready resolved, and the weapon of that attack sets agile.
- The function still returns `i32` and still does not write a count.
- Existing own-turn results and the non-readied reaction result stay as they are.

**Non-Goals:**

- The Ready action: cost, concentrate, trigger, ending the turn, or spending the reaction.
- The Delay action, including whether a reaction is available while delayed.
- Storing a snapshot on the encounter. `openspec/specs/encounter-turns/spec.md` requirement "A reaction attack penalty stays 0" still says an off-turn attack-trait check takes penalty 0 and that the encounter does not keep a penalty frozen from Ready.
- A second penalty table in `attack_modifiers`, a new crate, a new dependency, or a file from `v1-reference`.

## Decisions

### 1. The snapshot is an attack count on `MapInput`

```rust
pub struct MapInput {
    pub on_own_turn: bool,
    pub prior_attack_trait_checks_this_turn: u8,
    pub current_weapon_agile: bool,
    /// `Some(count)` is a readied attack. `count` is the attack-trait checks
    /// already made when Ready resolved, not a penalty. `None` is every other check.
    pub readied_attack_count: Option<u8>,
}
```

When `readied_attack_count` is `Some(count)`, the existing match uses `count` and `current_weapon_agile`: 0, then -5, then -10, or 0, then -4, then -8. `on_own_turn` does not zero that result. When it is `None` and `on_own_turn` is false, the result is 0. When it is `None` and `on_own_turn` is true, the match uses `prior_attack_trait_checks_this_turn`, as it does today.

The function takes `MapInput` by value and returns `i32`. It does not increment a count. A caller-held `u8` is unchanged after the call.

Alternative considered: store the penalty number at Ready time. Rejected. Ruling R-05 says the snapshot is the count, because the weapon used for the readied attack sets agile. Two non-agile checks then an agile readied attack are -8, not a stored -10. One agile check then a non-agile readied attack is -5, not a stored -4.

Alternative considered: a second function for readied attacks. Rejected. A second match would drift from the own-turn table.

### 2. Call sites pass `None` until the ready change

Every current `MapInput` literal sets `readied_attack_count: None`. That is `attack_trait_penalty`, the two helpers in `modifiers.rs`, and the helpers in `check.rs`. `attack_trait_penalty` still returns 0 off the open turn. `attack_modifiers` keeps its signature and its body. Its comment may say the caller passes the penalty this function returns; it must not grow a table or a stored penalty.

Alternative considered: teach `attack_trait_penalty` to store the count now. Rejected. That is the ready change, and the encounter-turns requirement still forbids keeping a frozen penalty.

### 3. No new error

`Some` together with `on_own_turn: true` still uses the frozen count. A readied attack is on another creature's turn, and the scenarios pass it that way. The function stays `i32`, with no new error variant and no panic. `u8` needs no overflow check here, because the function does not add to the count.

## Risks / Trade-offs

- [Encounter-turns still says every off-turn attack-trait check takes penalty 0 and that the encounter does not keep a frozen penalty] → This change does not amend that requirement and does not pass `Some` from `attack_trait_penalty`. The later ready change amends "A reaction attack penalty stays 0", keeps the scenario title "A reaction after two Strikes keeps penalty 0" for a reaction that is not readied, snapshots the `u8` count when Ready resolves, and calls `multiple_attack_penalty` with that count and the readied weapon's agile trait. It does not increment the own-turn count. It owns the B-06 trigger scenario: two own-turn attacks, then the readied non-agile Strike fires at -10. `attack_modifiers` already accepts the `i32` this function returns.
- [Delay might assume every reaction has penalty 0] → The later delay change treats a readied attack as a reaction whose penalty comes from this function. Delay's own rule is availability: no reactions while delayed (Player Core pg. 416, Actions 2294). This change does not specify Delay.
- [The requirement sentence is 496 characters, and strict validation fails a main-spec requirement past 500] → Keep the sentence within that limit. The table and the count-versus-number cases live in the scenarios.
- [A readied attack does not add to the count, but this function does not hold the count] → Tests keep a local `u8`, pass `Some` of that value, and assert the local value is unchanged. The same inputs twice return the same `i32`.

## Migration Plan

No persisted data and no public caller outside this crate. The new field lands in the same change as the literals that set it to `None`. Rolling back means dropping the field and the readied scenarios together.
