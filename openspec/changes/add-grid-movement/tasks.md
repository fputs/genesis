# Tasks

Each scenario in `specs/grid-movement/spec.md` has exactly one test. The test name is the scenario title in snake_case. Checklist edge cases E1–E22 are marked on those tasks. The other tests are the Designer QA answers and the architecture rules. `design.md` maps QA questions Q1–Q9 to those scenarios and to the out-of-scope lines. Do not edit `check.rs`, `combatant.rs`, or `dice.rs`. Add no dependency.

## 1. Pure grid module

- [x] 1.1 Add `crates/rules/src/grid.rs` and `mod grid` in `lib.rs`. Re-export the public items from design.md. Keep `#![forbid(unsafe_code)]` and leave `crates/rules/Cargo.toml` with no dependencies. Fallible functions return `Result<_, GridError>`. No `unwrap`, `expect`, `panic!`, or unchecked indexing. Use checked arithmetic. Give every public item a `///` comment, citing the Player Core section when it states a rule. Add `the_same_stride_twice_matches`: two identical one-square Strides match, and a `ScriptedDice` holding 8 is unused. Cite Player Core "Grid Movement". Verify `cargo test -p rules the_same_stride_twice_matches` passes and `crates/rules/Cargo.toml` has no `dependencies` table entries

## 2. Sizes

- [x] 2.1 Add `size_data` and `the_size_table_keeps_every_size` for Tiny through Gargantuan, including Tiny under 5 feet and Gargantuan at least 20 feet. Cite Player Core "Size, Space, and Reach". Verify `cargo test -p rules the_size_table_keeps_every_size` passes
- [x] 2.2 Add `only_small_and_medium_can_move`: Large and Tiny Strides return `UnsupportedSize` and do not move; Small and Medium each Stride one open square for 5 feet. Cite Player Core "Size, Space, and Reach". Verify `cargo test -p rules only_small_and_medium_can_move` passes

## 3. Diagonal counter and terrain

- [x] 3.1 Add `DiagonalCounter::fresh`, `end_turn`, and `diagonals`. Do not store the counter on `Creature` or `CombatantState`. Add `diagonal_parity_across_actions` (E2): two one-square diagonal Strides cost 5 then 10 on one counter. Cite Player Core "Diagonal Movement". Verify `cargo test -p rules diagonal_parity_across_actions` passes
- [x] 3.2 Add `odd_diagonal_at_turn_end` (E3): after one diagonal, `end_turn` makes the next diagonal cost 5. Cite Player Core "Diagonal Movement". Verify `cargo test -p rules odd_diagonal_at_turn_end` passes
- [x] 3.3 Add `orthogonal_does_not_flip_parity` (E4): diagonal, orthogonal, diagonal costs 5, then 5, then 10, total 20. Cite Player Core "Diagonal Movement". Verify `cargo test -p rules orthogonal_does_not_flip_parity` passes
- [x] 3.4 Add `the_counter_is_not_on_the_creature`: a diagonal Stride updates only the separate counter; the `Creature` and a `CombatantState` built from it compare equal to their values from before the call. Cite Player Core "Diagonal Movement". Verify `cargo test -p rules the_counter_is_not_on_the_creature` passes
- [x] 3.5 Add `off_turn_movement_uses_a_fresh_counter` (D2 = DEV-15): encounter-core passes `DiagonalCounter::fresh()` and discards the result. A stored counter that already counts one diagonal is not passed in; a fresh counter makes one diagonal cost 5 and two diagonals cost 15, and the stored counter still counts one. Cite Player Core "Diagonal Movement" and record the DEV-15 substitute in the test comment. Verify `cargo test -p rules off_turn_movement_uses_a_fresh_counter` passes
- [x] 3.6 Add `difficult_on_a_diagonal` (E5): the second diagonal on difficult terrain costs 15, not 20. Cite Player Core "Difficult Terrain". Verify `cargo test -p rules difficult_on_a_diagonal` passes
- [x] 3.7 Add `greater_difficult_on_a_diagonal` (E6): the first greater-difficult diagonal costs 15, and the second costs 20. Cite Player Core "Difficult Terrain". Verify `cargo test -p rules greater_difficult_on_a_diagonal` passes
- [x] 3.8 Add `leaving_difficult_terrain` (E7): leaving a difficult square for an open orthogonal square costs 5. Cite Player Core "Difficult Terrain". Verify `cargo test -p rules leaving_difficult_terrain` passes
- [x] 3.9 Add `step_and_crawl_share_the_diagonal_counter` (Q1): one fresh counter; a diagonal Step advances it to one diagonal; a diagonal Stride on that counter costs 10. Crawl and Leap share that counter and are not implemented. Cite Player Core "Diagonal Movement", "Crawl", and "Leap". Verify `cargo test -p rules step_and_crawl_share_the_diagonal_counter` passes and `rg "fn crawl|fn leap" crates/rules/src/grid.rs` finds nothing

## 4. Stride limits

- [x] 4.1 Add `exact_speed` (E1, Q2): five orthogonal squares at Speed 25 cost 25 and succeed; six squares return `ExceedsSpeed`, no square of that path is entered, the position stays the start, and the counter stays fresh. Cite Player Core "Stride". Verify `cargo test -p rules exact_speed` passes
- [x] 4.2 Add `terrain_cost_exceeds_the_remainder` (E8): Speed 25 and four open squares plus one difficult square would cost 30, return `ExceedsSpeed`, and leave the creature on the start square. Cite Player Core "Stride". Verify `cargo test -p rules terrain_cost_exceeds_the_remainder` passes
- [x] 4.3 Add `leftover_movement_is_not_banked` (D5): a 5-foot Stride does not reduce the next Stride below Speed 25. Cite Player Core "Stride". Verify `cargo test -p rules leftover_movement_is_not_banked` passes
- [x] 4.4 Add `speed_0` (E22): an empty path succeeds at cost 0; a one-square path returns `ExceedsSpeed`; Step returns `StepRequiresTenFeet`. Position stays. Cite Player Core "Speed" and "Step". Verify `cargo test -p rules speed_0` passes
- [x] 4.5 Add `validate_speed` and `speed_not_a_multiple_of_five` (Q3). add-modifiers (#19) validates Creature Speed as a multiple of 5 at load and fold. Do not add a creature-template delta. Movement keeps `SpeedNotMultipleOfFive` as a defensive guard: Speed 7 with no path, and a Stride of an empty path, both return that error, not `NegativeSpeed`, and the position stays. Rule: Player Core has no multiple-of-5 rule. Label the test comment validation (DEV-21) and cite Rules 2347 (Player Core pg. 420) and Rules 2356 (5-ft squares). Verify `cargo test -p rules speed_not_a_multiple_of_five` passes
- [x] 4.6 Add `a_gap_in_the_path_is_rejected`: a square two steps away with nothing between returns `PathNotAdjacent`, and the position and counter stay. Cite Player Core "Grid Movement". Verify `cargo test -p rules a_gap_in_the_path_is_rejected` passes
- [x] 4.7 Add `an_empty_path_does_not_search`: an empty path at Speed 25 costs 0, emits no events, ends on the start, and does not insert a destination three squares away. There is no path-search function. Cite Player Core "Stride". Verify `cargo test -p rules an_empty_path_does_not_search` passes and `rg find_path crates/rules/src/grid.rs` finds nothing
- [x] 4.8 Add `off_grid` (E15): a path square outside the caller-supplied width and height returns `OffGrid` and the position is unchanged. Cite Player Core "Grid Movement". Verify `cargo test -p rules off_grid` passes
- [x] 4.9 Add `stride_emits_left_and_entered_events` (D17): origin, then one orthogonal square, then one diagonal square, costs 10, `triggers` is true, and the events are Left origin, Entered orthogonal, Left orthogonal, Entered diagonal. `triggers` covers only this function's result. Stand, Drop Prone, and Crawl also trigger Reactive Strike elsewhere (FIX-06). Cite Player Core "Stride". Verify `cargo test -p rules stride_emits_left_and_entered_events` passes
- [x] 4.10 Add `a_stride_does_not_spend_actions` (Q9): a `CombatantState` with 2 actions remaining and the reaction not ready is not an argument; after a legal Stride those fields are unchanged, and the outcome has a foot cost and no action cost. Cite Player Core "Stride". Verify `cargo test -p rules a_stride_does_not_spend_actions` passes
- [x] 4.11 Add `a_diagonal_needs_no_wall_check` (Q6): a one-square open diagonal Stride from a fresh counter costs 5 and is not rejected for a blocked corner. There is no wall input. Record in the test comment the future ruling that a diagonal is blocked only when both orthogonal neighbours block, and that this change does not apply it. Verify `cargo test -p rules a_diagonal_needs_no_wall_check` passes
- [x] 4.12 Add `multiples_of_five_pass_speed_validation`: the defensive guard accepts 0, 5, 10, and 25 with no path. Cite validation (DEV-21), Rules 2347, and Rules 2356. Verify `cargo test -p rules multiples_of_five_pass_speed_validation` passes
- [x] 4.13 Add `a_step_uses_the_speed_validation`: Step at Speed 7 into an open square returns `SpeedNotMultipleOfFive` and does not move. Cite validation (DEV-21), Rules 2347, and Rules 2356. Verify `cargo test -p rules a_step_uses_the_speed_validation` passes

## 5. Step

- [x] 5.1 Add `step` per design.md. Add `step_into_difficult_terrain` (E9): Step into difficult terrain and Step into greater difficult terrain both return `StepIntoDifficultTerrain` and do not move. Cite Player Core "Step". Verify `cargo test -p rules step_into_difficult_terrain` passes
- [x] 5.2 Add `step_with_speed_5` (E10): Speed 5 into an open square returns `StepRequiresTenFeet` and does not move. Cite Player Core "Step". Verify `cargo test -p rules step_with_speed_5` passes
- [x] 5.3 Add `step_advances_parity` (E11): a diagonal Step from a fresh counter costs 5, then a diagonal Stride costs 10. Cite Player Core "Step" and record the D6 substitute in the test comment. Verify `cargo test -p rules step_advances_parity` passes
- [x] 5.4 Add `step_on_a_10_ft_diagonal_costs_5` (Q4): with the counter already counting one diagonal, the diagonal Step is allowed, moves 5 feet rather than 10, the counter then counts two, and the next diagonal Stride costs 5. Cite Player Core "Step". Verify `cargo test -p rules step_on_a_10_ft_diagonal_costs_5` passes
- [x] 5.5 Add `a_step_is_one_square`: a two-square Step and an empty Step both return `StepNotOneSquare` and do not move. Cite Player Core "Step". Verify `cargo test -p rules a_step_is_one_square` passes
- [x] 5.6 Add `shared_diagonal_counter_includes_step`: one counter, Stride one diagonal (5), Step one orthogonal (5, counter unchanged), Stride one diagonal (10). Cite Player Core "Diagonal Movement". Verify `cargo test -p rules shared_diagonal_counter_includes_step` passes
- [x] 5.7 Add `step_events_are_non_triggering`: one diagonal Step costs 5, `triggers` is false, and the events are Left the start, then Entered the destination. Cite Player Core "Step". Verify `cargo test -p rules step_events_are_non_triggering` passes

## 6. Occupancy

- [x] 6.1 Add `through_an_ally` (E12, Q5): a same-faction creature on the first square of a two-square path does not stop the Stride, the path enters that square, and the end square is empty. Cite Player Core "Moving Through a Creature's Space" and record the D7 substitute. Verify `cargo test -p rules through_an_ally` passes
- [x] 6.2 Add `through_an_enemy` (E13): entering an other-faction square, including ending on it, returns `EnemySquare` and does not move. Cite Player Core "Moving Through a Creature's Space". Verify `cargo test -p rules through_an_enemy` passes
- [x] 6.3 Add `ending_in_an_ally_square` (E14, Q5): ending on a same-faction creature returns `OccupiedEnd` and does not move. Cite Player Core "Moving Through a Creature's Space" and record the D8 substitute, including that the next-action move out is encounter-core. Verify `cargo test -p rules ending_in_an_ally_square` passes
- [x] 6.4 Add `prone_dead_willing_and_tiny_sharing_is_out_of_scope` (Q7): ending a Stride on a same-faction creature returns `OccupiedEnd` and does not move. The occupant has no prone, dead, willing, or Tiny exception. Cite Player Core "Sharing a Space" and record that those rules are out of scope. Verify `cargo test -p rules prone_dead_willing_and_tiny_sharing_is_out_of_scope` passes

## 7. Distance, reach, and range

- [x] 7.1 Add `distance` and `adjacent`. Add `distance_0_and_adjacency` (E16): one diagonal apart is 5 feet and adjacent, the reverse is 5, self is 0 and not adjacent, terrain and another creature do not change it, and a turn counter that already counts one diagonal is unchanged. Cite Player Core "Areas". Verify `cargo test -p rules distance_0_and_adjacency` passes
- [x] 7.2 Add `in_reach`. Add `reach_5_is_only_adjacency` (D11): reach 5 includes a diagonal neighbor only, not a creature two orthogonal squares away, and not itself. Cite Player Core "Range and Reach". Verify `cargo test -p rules reach_5_is_only_adjacency` passes
- [x] 7.3 Add `reach_10_diagonal_exception` (E20): reach 10 includes two diagonal squares (grid cost 15), includes a 2-by-1 offset at 10 feet, and excludes a 3-by-1 offset at 15 feet. Reach 20 includes three diagonal squares at 20 feet. Cite Player Core "Range and Reach". Verify `cargo test -p rules reach_10_diagonal_exception` passes
- [x] 7.4 Add `range_band`. Add `range_boundary` (E18): increment 60; distance 0 and 60 are count 1 penalty 0; 65 is count 2 penalty -2; 360 is count 6 penalty -10; 365 is `BeyondRange` and not penalty -12. Cite Player Core "Range Penalty". Verify `cargo test -p rules range_boundary` passes
- [x] 7.5 Add `a_boundary_uses_the_lower_increment`: increment 60; 120 feet is count 2 penalty -2; 121 feet is count 3 penalty -4. Cite Player Core "Range Penalty". Verify `cargo test -p rules a_boundary_uses_the_lower_increment` passes
- [x] 7.6 Add `range_penalty_is_plain_data` (E19): 130 feet at increment 60 returns count 3 and penalty -4, with no attack total and no call to `multiple_attack_penalty`. Cite Player Core "Range Penalty" and "Penalties", and record that the summed -9 is encounter-core. Verify `cargo test -p rules range_penalty_is_plain_data` passes and `rg multiple_attack_penalty crates/rules/src/grid.rs` finds nothing
- [x] 7.7 Add `negative_or_zero_inputs` (E17): Speed -5 is `NegativeSpeed` and not a stay; increment 0 is `InvalidRangeIncrement` and not penalty 0; distance -1 is `NegativeDistance`. None are clamped. Cite Player Core "Speed" and "Range Penalty". Verify `cargo test -p rules negative_or_zero_inputs` passes
- [x] 7.8 Add `coordinate_overflow_is_an_error`: distance between `i32::MAX` and `i32::MIN` returns `Overflow` and not a wrapped distance. Cite Player Core "Grid Movement". Verify `cargo test -p rules coordinate_overflow_is_an_error` passes

## 8. Immobilized

- [x] 8.1 Add `move_trait_allowed` and call it from Stride and Step. Add `immobilized` (E21): an immobilized creature with Speed 25 cannot Stride or Step into an open square; position and counter stay. Cite Player Core "Immobilized". Verify `cargo test -p rules immobilized` passes
- [x] 8.2 Add `the_immobilized_check_needs_no_path`: the flag set returns `Immobilized` with no path; the flag clear returns `Ok`. Do not implement Crawl, Stand, or Drop Prone. Cite Player Core "Immobilized". Verify `cargo test -p rules the_immobilized_check_needs_no_path` passes and `rg "fn crawl|fn stand|fn drop_prone" crates/rules/src/grid.rs` finds nothing

## 9. Changelog and proof

- [x] 9.1 Add one bullet under `## [Unreleased]` `### Added` in `CHANGELOG.md` describing 5-foot grid movement, diagonal cost, and range-increment data, and link that bullet to the implementation pull request. Put that edit in a commit before the archive commit. Do not add the bullet in the spec-only pull request. Verify the new bullet is under `### Added`, contains the pull request link, and that the archive commit does not modify `CHANGELOG.md`
- [x] 9.2 Run the AGENTS.md proof commands and verify each exits 0. Confirm `crates/rules/Cargo.toml` still has no dependencies
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `npx @fission-ai/openspec@1.14.3 validate --all --strict`

## Workflow follow-up

- The final commit of the implementation pull request is `openspec archive add-grid-movement --yes`. There is no separate archive pull request. That commit touches only `openspec/changes/archive/**`, `openspec/specs/**`, and the removed `openspec/changes/add-grid-movement/**`, and its message contains `openspec archive add-grid-movement`. The changelog bullet is already in an earlier commit.
