# Tasks

## 1. Definition of done

- [ ] 1.1 Add one line to `AGENTS.md` section 2, immediately after the paragraph that starts "All must pass." The line is: `An implementation pull request adds a changelog entry under ## [Unreleased] before the work is done. Spec-only, docs-only, and archive-only pull requests do not.` Verify that line is present and that no other line in section 2 changed
- [ ] 1.2 In `AGENTS.md` section 10, replace `Update CHANGELOG.md only through the process in docs/producer/process.md.` with `An implementation pull request writes its CHANGELOG.md entry under ## [Unreleased] before the work is done. There is no separate changelog pull request after merge.` Verify that line is the only change in section 10
- [ ] 1.3 In `docs/producer/process.md`, keep the sentences that say where `CHANGELOG.md` lives, that it uses Keep a Changelog with an Unreleased section, and that v2 starts here while v1 history is on `v1-reference`. Replace the rest of the Changelog bullet so it says the implementation pull request writes that Unreleased entry before the work is done, spec-only, docs-only, and archive-only pull requests do not, and there is no separate changelog pull request after merge. In the Review and escalation bullet, delete the sentence that starts "Changelog entries are automated". Verify the remaining review text is unchanged
- [ ] 1.4 Retire the post-merge changelog step wherever a living doc still describes it. Search `docs/` and `AGENTS.md` for Producer or DevOps drafting a changelog entry after merge, or for a later pull request whose job is that entry. The two sentences edited in task 1.3 are the descriptions on main. If the search finds another living doc that still describes that step, remove it there too. Do not edit `openspec/changes/archive/`. Verify the search of `docs/` and `AGENTS.md` finds no separate post-merge changelog step

## 2. Changelog entry

- [ ] 2.1 Edit the existing root `CHANGELOG.md`. Do not replace the file. After the Semantic Versioning sentence, and before `## [Unreleased]`, add the sentence "v1 history lives on the `v1-reference` branch." Keep the existing `### Changed` bullet about the v2 restart. Under `## [Unreleased]`, add the pull request #10 entry below, verbatim, with `### Added` first and `### Security` after the existing `### Changed` section. Verify the header sentence, the restart bullet, and both #10 bullets with their links

```
### Added
- Spec-drift CI check (`scripts/spec-drift-check.mjs`), now run in `ci.yml` with its tests. It gates OpenSpec validation and flags completed changes that haven't been archived. Any PR that touches code or `.github/workflows/**` must also reference an OpenSpec change id with `OpenSpec-Change:`. ([#10](https://github.com/fputs/genesis/pull/10))
- `openspec/specs/spec-drift-check` spec, archived from the add-spec-drift-check change.

### Security
- CI workflow token is now limited to read-only access (`permissions: contents: read`). ([#10](https://github.com/fputs/genesis/pull/10))
```

## 3. Proof and archive

- [ ] 3.1 Run the AGENTS.md proof commands and verify each exits 0: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`, and `npx @fission-ai/openspec@1.14.3 validate --all --strict`
- [ ] 3.2 As the final commit of the implementation pull request, run `openspec archive add-changelog --yes` and verify the change is no longer active under `openspec/changes/` and sits under `openspec/changes/archive/`
