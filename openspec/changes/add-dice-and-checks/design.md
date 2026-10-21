# Design

## Context

`crates/rules` is an empty library. `v1-reference` already has `crates/rules/src/dice.rs` and `crates/rules/src/check.rs`. The degree helper there matches Player Core. The multiple-attack helper does not: it is `multiple_attack_penalty(prior_strikes: u8) -> i32` with only -5/-10, and `roll_attack` in `crates/rules/src/encounter.rs` then folds that number into the circumstance penalty (`map.min(aid_penalty)`, and prone's -2 the same way). See the proposal for why that fold is rejected. Typed bonus and penalty buckets are `add-modifiers`, which is not this change.

## Goals / Non-Goals

**Goals:**

- Public dice and check API the later changes can call.
- Port the v1 degree behavior and the v1 die sources, and replace the MAP signature.
- Name the v1 tests that become acceptance tests, and the v1 tests that must not be copied.

**Non-Goals:**

- Circumstance, status, and item buckets, and "same type keeps only the worst penalty." That is `add-modifiers`, next, before encounter-core.
- Strikes, damage dice, reactions, prone, or Aid's prepared state.
- A global RNG.

## Decisions

### Module layout

```text
crates/rules/src/lib.rs     mod dice; mod check; pub use the items below
crates/rules/src/dice.rs    Dice, SeededDice, ScriptedDice
crates/rules/src/check.rs   degrees, flat checks, MAP
```

Do not add `combatant`, `encounter`, or `grid` in this change. If `lib.rs` already declares other modules, leave them.

### Purity

`crates/rules` gains no dependencies, and `Cargo.toml` stays otherwise unchanged. `dice.rs` and `check.rs` do not read a clock, a file, the network, or a global. They do not use `Cell`, `RefCell`, `Mutex`, or `unsafe`. `lib.rs` carries `#![forbid(unsafe_code)]`. `SeededDice` and `ScriptedDice` keep their state in fields and advance it only through `&mut self` on `Dice::roll`. Callers pass the `Dice`.

### Public types and signatures

```rust
pub trait Dice {
    fn roll(&mut self, faces: u8) -> u8;
    fn d20(&mut self) -> u8 {
        self.roll(20)
    }
}

pub struct SeededDice { /* private generator, advanced only through &mut self */ }
impl SeededDice {
    pub fn new(seed: u64) -> Self;
}
impl Dice for SeededDice {}

pub struct ScriptedDice;
impl ScriptedDice {
    pub fn new(rolls: impl IntoIterator<Item = u8>) -> Self;
    pub fn remaining(&self) -> usize;
}
impl Dice for ScriptedDice {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DegreeOfSuccess {
    CriticalFailure,
    Failure,
    Success,
    CriticalSuccess,
}

impl DegreeOfSuccess {
    pub fn improve(self) -> Self;
    pub fn worsen(self) -> Self;
}

pub fn degree_of_success(natural: u8, total: i32, dc: i32) -> DegreeOfSuccess;

pub fn fallback_unlisted_critical(
    degree: DegreeOfSuccess,
    lists_critical_success: bool,
    lists_critical_failure: bool,
) -> DegreeOfSuccess;

pub fn dc_from_modifier(modifier: i32) -> i32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckRoll {
    pub natural: u8,
    pub total: i32,
    pub dc: i32,
    pub degree: DegreeOfSuccess,
}

pub fn resolve_check(dice: &mut impl Dice, modifier: i32, dc: i32) -> CheckRoll;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlatRoll {
    pub natural: Option<u8>,
    pub dc: i32,
    pub degree: DegreeOfSuccess,
}

pub fn flat_check(dice: &mut impl Dice, dc: i32) -> FlatRoll;
pub fn flat_checks_for_same_thing(dice: &mut impl Dice, dcs: &[i32]) -> FlatRoll;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapInput {
    pub on_own_turn: bool,
    pub prior_attack_trait_checks_this_turn: u8,
    pub current_weapon_agile: bool,
}

pub fn multiple_attack_penalty(input: MapInput) -> i32;

pub fn attack_check_total(
    natural: u8,
    attack_bonus: i32,
    map: i32,
    circumstance_penalty: i32,
    other_untyped_penalty: i32,
) -> i32;
```

`resolve_check` rolls `d20`, sets `total` to natural plus modifier, and sets `degree` from `degree_of_success`. That function applies the natural 20 or natural 1 before it returns. Later effects call `DegreeOfSuccess::improve` or `DegreeOfSuccess::worsen` on that result. They do not re-run the numeric bands.

`fallback_unlisted_critical` maps an unlisted critical success to success and an unlisted critical failure to failure.

`flat_check` does not add a modifier. DC <= 1 returns `FlatRoll { natural: None, degree: Success }` and does not roll. DC >= 21 returns `degree: Failure` the same way. Otherwise it rolls one d20 and uses `degree_of_success` with modifier 0. `flat_checks_for_same_thing` uses the highest DC in a non-empty `dcs` and calls `flat_check` once. It finds that maximum without `unwrap` or `panic`. Callers with two different things call `flat_check` twice.

`multiple_attack_penalty` returns 0 when `on_own_turn` is false. It does not increment `prior_attack_trait_checks_this_turn`; the caller does that only for an attack-trait check on the creature's own turn. On that turn the penalty is 0, then -5, then -10, or 0, then -4, then -8 when `current_weapon_agile` is true. The agile flag is the weapon of this check.

`attack_check_total` returns `natural + attack_bonus + map + circumstance_penalty + other_untyped_penalty`. Penalties are negative. It does not take the minimum of `map` and `circumstance_penalty`. Those two `i32` slots are a stand-in until `add-modifiers` supplies buckets. This change does not implement "worst penalty of a type."

`degree_of_success` on the boundaries: total >= dc + 10 is critical success, total >= dc is success, total <= dc - 10 is critical failure, otherwise failure. Natural 20 and natural 1 then move one step and stop at the ends.

### What is ported from v1-reference, and what changes

Port `crates/rules/src/dice.rs` as the behavior of `Dice`, `SeededDice`, and `ScriptedDice`. Keep the xorshift, the replacement of seed 0 with `0xA5A5_5A5A_1234_5678`, and the scripted panic when the queue is empty or the value is not on the die (including `faces < 1`).

Port v1's `Degree` enum as `DegreeOfSuccess`, and port `degree_of_success` and the private improve/worsen steps from `crates/rules/src/check.rs`. Make `improve` and `worsen` public methods. The numeric bands and the natural 20 / natural 1 order stay. The public name is `DegreeOfSuccess`.

Do not port `multiple_attack_penalty(prior_strikes: u8) -> i32` as that signature. Replace it with `MapInput`. The old function has no agile flag and no turn gate.

Do not port `roll_attack` from `crates/rules/src/encounter.rs`. Its comment treats MAP, prone, and a failed Aid as one circumstance pool and keeps the worst. `attack_check_total` adds the numbers instead.

### v1 tests that become acceptance tests

- Port `degrees_follow_dc_bands_then_natural_20_and_1` from `v1-reference` `crates/rules/src/check.rs`. Keep its cases: (10, 24, 14) critical success, (11, 18, 18) success, (8, 15, 18) failure, (2, 4, 18) critical failure, natural 20 on a failure to success `(20, 31, 40)`, natural 20 on a success to critical success `(20, 21, 18)`, natural 1 on a critical success to success `(1, 30, 18)`, natural 1 on a failure to critical failure `(1, 12, 18)`. Cite Player Core, Step 4: Degree of Success, and Natural 1 and Natural 20.
- Port `map_is_zero_then_minus_five_then_minus_ten` from that same file as the on-turn, non-agile cases: prior 0, 1, 2, and 3 yield 0, -5, -10, -10. Extend that test for agile -4/-8, the current weapon, an off-turn reaction, and a non-Strike attack-trait check. Cite Player Core, Multiple Attack Penalty.

v1 has no unit test for `SeededDice` or `ScriptedDice`. Add `the_same_seed_repeats`, `scripted_dice_panic_when_exhausted`, `scripted_dice_panic_out_of_range`, and `scripted_bounds_are_legal`.

Add `map_stacks_with_a_circumstance_penalty`: second on-turn non-agile check, attack bonus +9, circumstance penalty -2, DC 16, scripted natural 12, total 14, failure. Add `map_stacks_with_another_untyped_penalty` with the same numbers and a second untyped -2 instead of the circumstance penalty.

Do not port these v1 tests. They are not acceptance tests for this change:

- `prone_attack_penalty_does_not_stack_with_map` in `crates/rules/tests/procedures.rs`. It asserts MAP is a circumstance penalty. That result is rejected.
- `later_strikes_take_map_and_a_miss_does_not_roll_damage_dice`. Strike damage and the action economy belong to encounter-core.
- `reactive_strike_ignores_map`. Reactive Strike is a later change. The off-turn penalty of 0 is covered by the MAP unit test above.

## Risks / Trade-offs

- [Loose `i32` penalty arguments] → `add-modifiers` replaces them with buckets. Until then, callers pass negative penalties and this function adds them. Do not take `min` of MAP and a circumstance penalty.
- [Porting the xorshift locks the seed stream] → that is intentional, so a v1 seed stays reproducible. The acceptance test checks two instances, not a printed roll table.
