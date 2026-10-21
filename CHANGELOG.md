# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

v1 history lives on the `v1-reference` branch.

## [Unreleased]

### Added
- Modifier stacking for circumstance, item, status, and untyped penalties, and a damage minimum of 1 before critical doubling. ([#25](https://github.com/fputs/genesis/pull/25))
- 5-foot grid movement for Stride and Step, diagonal cost across a turn, and range-increment data. ([#26](https://github.com/fputs/genesis/pull/26))
- Pull-request hygiene job `pr-hygiene` (`scripts/pr-hygiene.mjs`, `.github/workflows/pr-hygiene.yml`). An implementation pull request must add a bullet under `## [Unreleased]`, and when a named OpenSpec change is removed the archive commit must be last. ([#23](https://github.com/fputs/genesis/pull/23))
- Dice, checks, and creature templates: injected dice, degree of success, flat checks, the multiple attack penalty, and a character build that derives a creature from its validated base. Folding a later contribution returns an error when the list is empty, the damage die cannot be stored, or a sum overflows. ([#7](https://github.com/fputs/genesis/pull/7))
- Spec-drift CI check (`scripts/spec-drift-check.mjs`), now run in `ci.yml` with its tests. It gates OpenSpec validation and flags completed changes that haven't been archived. Any PR that touches code or `.github/workflows/**` must also reference an OpenSpec change id with `OpenSpec-Change:`. ([#10](https://github.com/fputs/genesis/pull/10))
- `openspec/specs/spec-drift-check` spec, archived from the add-spec-drift-check change.

### Changed

- Genesis v2 restart. v1 is preserved on branch `v1-reference`.

### Fixed
- Duplicate effects collapse on (source id, statistic, type, bonus or penalty), and an equal sequence keeps the worse penalty. ([#30](https://github.com/fputs/genesis/pull/30))
- pr-hygiene and spec-drift fail closed on an invalid OpenSpec change id, an archive that is not the head commit, and a moved, edited, or fenced changelog bullet. Editing a pull request re-runs spec-drift without skipping Cargo. ([#28](https://github.com/fputs/genesis/pull/28))

### Security
- CI workflow token is now limited to read-only access (`permissions: contents: read`). ([#10](https://github.com/fputs/genesis/pull/10))
