# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

v1 history lives on the `v1-reference` branch.

## [Unreleased]

### Added
- Behavior tests for Hide on both sides of the degree boundary and the rejected side of the +20 modifier gate, Raise a Shield off-turn with no shield, and an unconscious creature's shield state. ([#72](https://github.com/fputs/genesis/pull/72))
- Delay, return, and Ready. A stored action and a Reactive Strike on the same trigger resolve in initiative order. ([#70](https://github.com/fputs/genesis/pull/70))
- Aid preparation on your turn and the Aid reaction against DC 15. ([#68](https://github.com/fputs/genesis/pull/68))
- Reactive Strike: the in-reach trigger, one reaction, and disruption. ([#66](https://github.com/fputs/genesis/pull/66))
- Raise a Shield and Shield Block. ([#63](https://github.com/fputs/genesis/pull/63))
- Seek, Hide, targeting flat checks, and off-guard while hidden or undetected. ([#62](https://github.com/fputs/genesis/pull/62))
- Dying, recovery checks, temporary Hit Points, and which side has won. ([#59](https://github.com/fputs/genesis/pull/59))
- Prone, flanking, off-guard, and Take Cover. ([#50](https://github.com/fputs/genesis/pull/50))
- Initiative, the three-action turn, and Strike. ([#52](https://github.com/fputs/genesis/pull/52))
- Modifier stacking for circumstance, item, status, and untyped penalties, and a damage minimum of 1 before critical doubling. ([#25](https://github.com/fputs/genesis/pull/25))
- 5-foot grid movement for Stride and Step, diagonal cost across a turn, and range-increment data. ([#26](https://github.com/fputs/genesis/pull/26))
- Pull-request hygiene job `pr-hygiene` (`scripts/pr-hygiene.mjs`, `.github/workflows/pr-hygiene.yml`). An implementation pull request must add a bullet under `## [Unreleased]`, and when a named OpenSpec change is removed the archive commit must be last. ([#23](https://github.com/fputs/genesis/pull/23))
- Dice, checks, and creature templates: injected dice, degree of success, flat checks, the multiple attack penalty, and a character build that derives a creature from its validated base. Folding a later contribution returns an error when the list is empty, the damage die cannot be stored, or a sum overflows. ([#7](https://github.com/fputs/genesis/pull/7))
- Spec-drift CI check (`scripts/spec-drift-check.mjs`), now run in `ci.yml` with its tests. It gates OpenSpec validation and flags completed changes that haven't been archived. Any PR that touches code or `.github/workflows/**` must also reference an OpenSpec change id with `OpenSpec-Change:`. ([#10](https://github.com/fputs/genesis/pull/10))
- `openspec/specs/spec-drift-check` spec, archived from the add-spec-drift-check change.

### Changed

- **BREAKING:** Strike, Reactive Strike, and a readied Strike roll the targeting flat check before the attack roll. A lost Strike returns `StrikeDelivery::Lost` and does not roll the attack d20. ([#80](https://github.com/fputs/genesis/pull/80))
- **BREAKING:** Preparing Aid accepts an unconscious or dying ally and rejects a dead ally. The reaction against a dead ally is rejected and does not spend the reaction. The reaction is spent only when the ally attempts the check. ([#75](https://github.com/fputs/genesis/pull/75))
- A failed or critically failed concealed targeting flat check reports the action spent and the attack counting for MAP, and a success reports neither. ([#69](https://github.com/fputs/genesis/pull/69))
- A readied attack uses the multiple attack penalty for the attack count frozen when Ready resolved. ([#64](https://github.com/fputs/genesis/pull/64))
- **BREAKING:** `Vital::EnterDying` is replaced by Alive, Unconscious, Dying of 1, 2, or 3, and Dead. ([#59](https://github.com/fputs/genesis/pull/59))
- **BREAKING:** pr-hygiene skips archive-last while a pull request is a draft and fails a ready implementation pull request whose own OpenSpec change is still active. ([#43](https://github.com/fputs/genesis/pull/43))
- **BREAKING:** A frightened or sickened bonus is rejected at construction. ([#40](https://github.com/fputs/genesis/pull/40))
- **BREAKING:** Frightened and sickened modifiers with no rank or a non-status type are rejected at construction, and the attack check total's internal terms use reserved sources. ([#37](https://github.com/fputs/genesis/pull/37))
- The equal-value condition-id tie uses condition id 4. ([#37](https://github.com/fputs/genesis/pull/37))
- **BREAKING:** An `OpenSpec-Change` line fails unless it is the prefix, one space, and one kebab id. A dated archive folder name is not an id. Invalid ids log the whole remainder with non-printable characters and backslashes escaped as `\uXXXX`, and both spec-drift jobs share one pull request concurrency group. ([#33](https://github.com/fputs/genesis/pull/33))
- Genesis v2 restart. v1 is preserved on branch `v1-reference`.

### Fixed
- The condition group key is (condition, statistic, bonus or penalty, type) after the status rewrite, so different types stack. ([#30](https://github.com/fputs/genesis/pull/30))
- Condition grouping is by (condition, statistic), the source key leaves different conditions apart, and a tie keeps the lower source id. ([#30](https://github.com/fputs/genesis/pull/30))
- Duplicate effects collapse on (source id, statistic, type, bonus or penalty), and an equal sequence keeps the worse penalty. ([#30](https://github.com/fputs/genesis/pull/30))
- pr-hygiene and spec-drift fail closed on an invalid OpenSpec change id, an archive that is not the head commit, and a moved, edited, or fenced changelog bullet. Editing a pull request re-runs spec-drift without skipping Cargo. ([#28](https://github.com/fputs/genesis/pull/28))

### Security
- CI workflow token is now limited to read-only access (`permissions: contents: read`). ([#10](https://github.com/fputs/genesis/pull/10))
