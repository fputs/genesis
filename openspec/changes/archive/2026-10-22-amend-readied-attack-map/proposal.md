# Proposal

## Why

The merged dice-and-checks requirement says a reaction on another creature's turn takes no multiple attack penalty. Ready is the exception: a readied attack uses the multiple attack penalty from when the creature Readied (Player Core pg. 417, https://2e.aonprd.com/Actions.aspx?ID=2299). That exception is FIX-16, ruled as R-05 in `docs/designer/rules-rulings.md`. The blanket sentence is what this change narrows, before a later delay and ready change.

## What Changes

- Narrow the `dice-and-checks` requirement "Multiple attack penalty on your turn". A reaction that is not a readied attack still takes no penalty and still does not add to the creature's own-turn count. An attack-trait check made as a readied action takes the penalty for the attack count frozen when Ready resolved, including the third and later attacks. The readied attack's own weapon decides agile: 0, -5, or -10, or 0, -4, or -8. The frozen value is the attack count, not a penalty number. A readied attack does not add to any own-turn count. Ruling R-05 is cited beside the Source lines.
- Keep every existing scenario title under that requirement. Add the R-05 cases on the penalty function, each with a full GIVEN: frozen count 1 and a non-agile weapon is -5; frozen count 0 is 0; frozen count 1 and an agile weapon after a non-agile check is -4; frozen count 2 and an agile weapon is -8; frozen count 2 and a non-agile weapon is -10; frozen count 3 is -10 or -8; frozen count 255 stays at that later penalty; after a -5 readied attack the next own turn starts at 0; a Reactive Strike that is not a readied attack is 0. A readied attack on the creature's own turn with prior count 0 and frozen count 2 is -10. A reaction that is not a Reactive Strike, on the creature's own turn, takes the own-turn penalty.
- The Ready action and the Delay action stay out of this change. The later ready change owns the trigger, the action cost, and the encounter snapshot. The later delay change owns whether a reaction is available while delayed.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dice-and-checks`: The off-turn sentence of "Multiple attack penalty on your turn" gains the readied-attack exception from R-05. The penalty stays an untyped penalty under the existing untyped requirement.

## Impact

- The implementation edits `multiple_attack_penalty` and `MapInput` in `crates/rules/src/check.rs`. A readied attack passes the frozen attack count. `attack_modifiers` keeps its caller-supplied penalty argument and does not grow a second table. `attack_trait_penalty` in `crates/rules/src/encounter.rs` keeps returning 0 off the creature's open turn: `openspec/specs/encounter-turns/spec.md` still says an off-turn attack-trait check takes penalty 0 and that the encounter does not keep a penalty frozen from Ready. Call sites that build `MapInput` pass no frozen count until the ready change.
- No new crate and no new dependency. `crates/rules` stays free of Bevy, serde, files, network, the clock, and global randomness. Callers still inject dice. No file from `v1-reference` is ported.
- Hooks for the later ready change: snapshot the attack count when Ready resolves, call `multiple_attack_penalty` with that count and the readied weapon's agile trait, do not increment the own-turn count, and amend encounter-turns requirement "A reaction attack penalty stays 0", keeping its scenario title for a reaction that is not readied. Ready expires at the start of the creature's next turn. That change also covers a readied attack against a Dying, Unconscious, or Dead target. It owns the trigger scenario from `docs/designer/backfill-findings.md` B-06 (two own-turn attacks, then the readied Strike fires at -10) and keeps R-05's freeze-the-count reading in that scenario. The later delay change treats a readied attack as a reaction whose penalty comes from this function; Delay's availability rule stays in that change (Player Core pg. 416, https://2e.aonprd.com/Actions.aspx?ID=2294).
- The implementation pull request adds an Unreleased changelog bullet, then archives with `openspec archive amend-readied-attack-map --yes` as its last commit. This pull request is spec only, so it adds no changelog bullet and does not archive.
