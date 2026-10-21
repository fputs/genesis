# Design

## Context

See proposal.md for why. `with_condition` returns `Self` and cannot fail. A frightened or sickened penalty is rewritten to status `-rank` whatever type the caller declared. `attack_check_total` passes `SourceId(1)`, `SourceId(2)`, and `SourceId(3)`. The property oracle calls `derived_rank`, `same_duplicate_key`, and `source_ord`. The behavior to implement is `specs/modifiers/spec.md`.

## Goals / Non-Goals

**Goals:**

- Reject invalid frightened and sickened input at construction.
- Make the property oracle a separate RAW reference.
- Give the attack check's internal terms reserved sources.

**Non-Goals:**

- A new crate, dependency, or field on `Modifier`.
- Changing same-type stacking, the duplicate-key ladder, or the condition-group tie order.
- Editing `docs/designer/rules-rulings.md` after this change copies R-03.
- A type-order selection step.

## Decisions

### 1. Reject at construction

`with_condition` returns `Result`. Frightened or sickened with rank `None`, or with a type other than status, returns `ModifierError::InvalidValuedCondition`. Rank 0 is `Ok` and still contributes nothing when totaled. Off-guard and any other condition with no rank stay `Ok`. A status bonus with a rank stays `Ok` and keeps its value. Do not rewrite a non-status type into status.

A legal penalty of rank above 0 still applies as status `-rank`. That value step stays. The type rewrite does not.

Alternative considered: reject only while totaling. Rejected. R-03 says construction, and a bad modifier must not exist to be stacked.

### 2. Independent oracle

The test builds its own grouping, duplicate-key collapse, and tie sort. It does not call `derived_rank`, `same_duplicate_key`, `source_ord`, `condition_prefers`, `key_prefers`, `bonus_prefers`, `penalty_prefers`, `stack_tie`, or `group_kind`.

The sort follows the orders the other requirements already state. Inside a condition group: grouped value (frightened and sickened penalties use `-rank`; otherwise the written value), then highest rank, a rank over none, newest sequence, lowest source, higher level, a level over none, then the duplicate-key order. Duplicate key: higher level, a level over none, higher rank, a rank over none, greater sequence, then the higher bonus or the more negative penalty, then the lower condition id (a missing id loses), then the lower source id. Type is not a component. Rank 0 on frightened or sickened is omitted before grouping.

The test encodes source order itself: caller ids first, by number, then the reserved variants in a fixed order that includes the new attack-check variants. It does not call `source_ord`.

Random sets use sources 1–2, sequences 1–2, two condition ids (`ConditionId(10)` and frightened), magnitudes 1–3, level absent or 1, and ranks 0–2. Sickened uses the same `-rank` path and is covered by its rejection scenario. Keep the unconditioned untyped exact duplicate and the both-levels-absent set. Add a conditioned exact duplicate. At least 1,000 random sets, seed `0x0D1A_7E5B`, every permutation of a set of at most 4.

### 3. Reserved attack-check sources

Add three `ModifierSource` variants, one per internal term (untyped multiple attack penalty, circumstance penalty, other untyped penalty). They are distinct from each other and from `MultipleAttackPenalty`. Build the terms in the modifiers module. `attack_check_total` uses that builder and does not pass `SourceId`. A positive untyped term stays a raw addend, as it is today. Callers still cannot construct these variants.

Alternative considered: reuse `SourceId(1..3)` and tell callers to avoid them. Rejected. Caller source 1 is legal and would collapse an internal untyped term.

### 4. No type-order tiebreak

A grep of `openspec/specs`, `docs`, and unarchived design notes found no selection tiebreak ordered circumstance, item, status, untyped. Archived `amend-modifier-tiebreaks-2` already rejects that alternative. `kind_ord` orders the reported list after selection. It is not a selection step, and this change does not edit it. The group key includes type, so a type step inside a group cannot change the winner.

## Risks / Trade-offs

- [Existing tests chain `with_condition`] → They handle `Result`. `a_frightened_circumstance_penalty_becomes_status_minus_rank` expects the error and is not deleted.
- [The oracle's source table drifts from production] → The table is the order in decision 2. A new reserved variant is added to that table in the same change.
- [The rulings log still says frightened and sickened are rewritten to status] → This change copies the designer's file exactly, including that earlier amendment and R-03. The implementation does not edit it. The spec replaces the rewrite.

## Migration Plan

Callers of `with_condition` handle the error. Rank 0, other unranked conditions, and a frightened or sickened status bonus with a rank keep today's success path. No stored data changes.
