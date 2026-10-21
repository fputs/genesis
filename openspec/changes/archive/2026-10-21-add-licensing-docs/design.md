# Design

## Context

The ORC research dated 2026-10-21 is the source for the licensing rules. It is research, not legal advice. Items it marks unverified or ambiguous stay marked in `docs/architect/licensing.md`.

`crates/rules/Cargo.toml` is a spec-drift code path because it is under `crates/`. The MIT `license` field is what makes this pull request an implementation change. The numbered rules are documentation. They are written so a later change can turn each one into a check. This change adds no check.

## Goals

- One document holds LIC-1 onward.
- `AGENTS.md` tells every contributor to follow that document and to cite book and page.
- The existing changelog sentences in `AGENTS.md` sections 2 and 10 stay as they are.
- The notice keeps the author-line TODO and the generic reserved list.

## Decisions

The normative rule text lives in `docs/architect/licensing.md`, not in a second copy inside the spec. The spec requires that document, the notice, the MIT manifest field, and the `AGENTS.md` sentence.

## Risks

The research flags notice placement, author lines, Archives of Nethys terms, and user-made levels as unverified. Those stay open questions in the document. Shipping the notice in the game, the build bundle, and the store page is not part of this change.
