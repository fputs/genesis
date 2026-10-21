# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

v1 history lives on the `v1-reference` branch.

## [Unreleased]

### Added
- Dice, checks, and creature templates: injected dice, degree of success, flat checks, the multiple attack penalty, and a character build that derives a creature. ([#7](https://github.com/fputs/genesis/pull/7))
- Spec-drift CI check (`scripts/spec-drift-check.mjs`), now run in `ci.yml` with its tests. It gates OpenSpec validation and flags completed changes that haven't been archived. Any PR that touches code or `.github/workflows/**` must also reference an OpenSpec change id with `OpenSpec-Change:`. ([#10](https://github.com/fputs/genesis/pull/10))
- `openspec/specs/spec-drift-check` spec, archived from the add-spec-drift-check change.

### Changed

- Genesis v2 restart. v1 is preserved on branch `v1-reference`.

### Security
- CI workflow token is now limited to read-only access (`permissions: contents: read`). ([#10](https://github.com/fputs/genesis/pull/10))
