# Design

## Context

`AGENTS.md` is a short pointer at the OpenSpec workflow and `openspec/config.yaml`. The config has a `context` block and a `specs` rule, and no `tasks` rule. See proposal.md for why that is not enough.

## Goals / Non-Goals

**Goals:**

- Put the constitution in root `AGENTS.md`, verbatim, with the section 2 commands in a fenced code block.
- Record two additions in `openspec/config.yaml`: the constitution sentence in `context`, and a `tasks` rule beside `specs`.
- Specify that behavior as ADDED requirements on `coding-constitution`.

**Non-Goals:**

- Rules procedures, new crates, CI, the changelog, or architect decisions.
- Archiving this change. That runs after merge.

## Decisions

### The constitution is `AGENTS.md`

Agents already read root `AGENTS.md`. Replacing that file makes the constitution the document they read before every change.

Alternative: leave the short pointer and add a second guide under `docs/`. Rejected, because the approved draft is the root file.

### This change applies the two doc edits

The implementation is the `AGENTS.md` replacement and the two `openspec/config.yaml` additions. Later changes follow the constitution; they do not copy it again.

### Config additions stay to two lines

The context sentence is one indented line after the existing paragraph. `tasks:` sits at the same level as `specs:` and has one item. No other line in `openspec/config.yaml` changes.

## Risks / Trade-offs

- [The `project-foundation` spec still calls `AGENTS.md` a short pointer] → This change does not edit that spec. A later amendment can update it. Until then, root `AGENTS.md` is the constitution.
- [Archive is not in this pull request] → The last task stays open until Evan merges, then `openspec archive add-coding-constitution --yes` runs.

## Migration Plan

Land the file replacement and the two config lines in this pull request. After merge, archive the change. Rollback is reverting that commit.

## Open Questions

None.
