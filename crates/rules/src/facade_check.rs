//! Crate-root visibility. Not a rules procedure.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const PUBLIC: &[&str] = &[
    "ActionCost",
    "ActionIdentity",
    "ActionTrait",
    "Agility",
    "AidCircumstance",
    "AidOutcome",
    "AidPreparation",
    "AidRank",
    "ArithmeticOverflow",
    "AttackBasis",
    "BonusType",
    "BuildParts",
    "CharacterBuild",
    "CombatantId",
    "CombatantState",
    "Concealment",
    "ConditionId",
    "ConditionDuration",
    "ConditionError",
    "ConditionKind",
    "ConditionSlot",
    "CoverBenefit",
    "Creature",
    "DamageBasis",
    "DamageDefenses",
    "DamageType",
    "Detection",
    "DetectionError",
    "Detections",
    "DiagonalCounter",
    "Dice",
    "DieCount",
    "DieCountError",
    "DegreeOfSuccess",
    "Disruption",
    "DyingValue",
    "DyingValueError",
    "Encounter",
    "EncounterError",
    "FaceCountError",
    "Faces",
    "FactionId",
    "GridError",
    "HpError",
    "InitiativeMove",
    "LEVEL_RANGE",
    "Level",
    "LevelError",
    "LostStrike",
    "MapBounds",
    "MeleeStrike",
    "Mobility",
    "Modifier",
    "ModifierError",
    "MoveAction",
    "MoveOutcome",
    "Occupant",
    "OccupantPresence",
    "ObservedAction",
    "Participant",
    "PenaltyType",
    "PositioningError",
    "Posture",
    "PostureChange",
    "Proficiency",
    "ReachError",
    "ReachFeet",
    "ReactionField",
    "ReactionPlace",
    "ReactiveError",
    "ReactiveOutcome",
    "ReactiveStrike",
    "ReactiveStrikeInput",
    "ReadyActivity",
    "ReadyEncounter",
    "ReadyError",
    "ReadyStrike",
    "ReadySuccess",
    "ReadyTrigger",
    "Relation",
    "Resistance",
    "ResolvedReaction",
    "Save",
    "SaveRoll",
    "SavingThrows",
    "SeededDice",
    "ShieldBlock",
    "ShieldTemplate",
    "Side",
    "Skill",
    "SkillRoll",
    "Skills",
    "Size",
    "SourceId",
    "Square",
    "SquareEvent",
    "Statistic",
    "StoredReady",
    "StrikeDelivery",
    "StrikeOutcome",
    "StrikeProfile",
    "StrikeRange",
    "TemplateError",
    "Terrain",
    "TurnSlot",
    "Vital",
    "Weakness",
    "begin_encounter",
    "derive",
    "new_combatant_state",
];

const PRIVATE: &[&str] = &[
    "Area",
    "AttackBlowError",
    "AttackKind",
    "AttackTotalTerm",
    "CheckModifier",
    "CheckRoll",
    "CombatantPos",
    "Contribution",
    "Cover",
    "DcModifier",
    "DetectionCost",
    "EncounterOutcome",
    "EndReason",
    "FlatRoll",
    "FoldError",
    "HideObserver",
    "HideResult",
    "MapInput",
    "ModifierTotal",
    "MoveInput",
    "NearbyFeature",
    "OffGuard",
    "OpenRoll",
    "RangeBand",
    "Readiness",
    "RosterEntry",
    "SaveError",
    "SeekResult",
    "ShieldError",
    "SizeData",
    "Space",
    "StrikeNotice",
    "TakeCoverResult",
    "TargetingResult",
    "ac_modifiers",
    "adjacent",
    "aid_check",
    "aid_circumstance_bonus",
    "apply_attack_blow",
    "apply_dice_damage",
    "apply_healing",
    "apply_resolved_blow",
    "armor_class",
    "attack_check_total",
    "attack_modifiers",
    "attack_penalty",
    "base_contribution",
    "basic_save_amount",
    "contributions",
    "damage_dealt",
    "dc_from_modifier",
    "dc_from_total",
    "degree_of_success",
    "dismiss",
    "dismiss_cost",
    "distance",
    "drop_prone",
    "encounter_outcome",
    "end_benefit",
    "fallback_unlisted_critical",
    "flanks",
    "flat_check",
    "flat_checks_for_same_thing",
    "fold_contributions",
    "grant_temporary_hit_points",
    "hide",
    "in_reach",
    "leap_while_prone",
    "lower_raised_shield",
    "move_trait_allowed",
    "multiple_attack_penalty",
    "note_action",
    "note_strike",
    "off_guard",
    "off_guard_against",
    "perception_modifiers",
    "proficiency_bonus",
    "prone_move",
    "raise_shield",
    "range_band",
    "ready_cost",
    "reflex_modifiers",
    "refresh",
    "resolve_check",
    "resolve_targeting",
    "save_dc",
    "save_modifier",
    "skill_dc",
    "skill_modifier",
    "seek",
    "shield_ac_modifier",
    "size_data",
    "stand",
    "start_of_turn",
    "stealth_dc_from_modifier",
    "step",
    "stride",
    "take_cover",
    "take_initiative_move",
    "total",
    "validate_speed",
];

const SCRIPT_CFG: &str = "#[cfg(any(test, feature = \"test-support\"))]";

#[test]
fn root_exports_match_the_facade() {
    let src = include_str!("lib.rs");
    let (plain, gated) = pub_use_names(src);
    let public: BTreeSet<String> = PUBLIC.iter().map(|name| (*name).to_string()).collect();
    assert_eq!(
        gated,
        BTreeSet::from([String::from("ScriptedDice")]),
        "ScriptedDice must stay on the test-support cfg line"
    );
    if plain != public {
        let extras: Vec<_> = plain.difference(&public).cloned().collect();
        let missing: Vec<_> = public.difference(&plain).cloned().collect();
        panic!(
            "private-list export still at the root: {}; missing facade names: {}",
            extras.join(", "),
            missing.join(", ")
        );
    }
}

#[test]
fn removed_names_are_pub_crate() {
    let private: BTreeSet<&str> = PRIVATE.iter().copied().collect();
    let mut hits = Vec::new();
    for path in rust_sources() {
        let text = fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!("read {}: {error}", path.display());
        });
        for (number, line) in text.lines().enumerate() {
            if let Some(name) = column0_pub_name(line)
                && private.contains(name)
            {
                hits.push(format!("{}:{}: {name}", path.display(), number + 1));
            }
        }
    }
    assert!(
        hits.is_empty(),
        "column-0 pub definition of a private-list name:\n{}",
        hits.join("\n")
    );
}

#[test]
fn nothing_carries_a_removal_attribute() {
    // The needle is split so this file does not contain the attribute's name.
    // Task 3.3 searches the crate for that name and must find nothing.
    let needle = concat!("#[", "depre", "cated");
    for path in rust_sources() {
        let text = fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!("read {}: {error}", path.display());
        });
        assert!(
            !text.contains(needle),
            "{} carries a removal attribute",
            path.display()
        );
    }
}

#[test]
fn workspace_stays_one_crate() {
    let rules_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace = rules_dir
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .join("Cargo.toml");
    let workspace_text = fs::read_to_string(&workspace).expect("read workspace manifest");
    assert!(
        workspace_text.contains("members = [\"crates/rules\"]"),
        "workspace members changed"
    );
    let rules_text = fs::read_to_string(rules_dir.join("Cargo.toml")).expect("read rules manifest");
    assert!(
        !rules_text.contains("[dependencies]") && !rules_text.contains("[dev-dependencies]"),
        "rules gained a dependency"
    );
}

fn pub_use_names(src: &str) -> (BTreeSet<String>, BTreeSet<String>) {
    let lines: Vec<&str> = src.lines().collect();
    let mut plain = BTreeSet::new();
    let mut gated = BTreeSet::new();
    let mut index = 0;
    while index < lines.len() {
        if !lines[index].trim_start().starts_with("pub use ") {
            index += 1;
            continue;
        }
        let gated_use = previous_code_line(&lines, index) == Some(SCRIPT_CFG);
        let mut block = String::new();
        loop {
            block.push_str(lines[index].trim());
            let done = lines[index].contains(';');
            index += 1;
            if done {
                break;
            }
            assert!(index < lines.len(), "unclosed pub use");
        }
        let body = block
            .strip_prefix("pub use ")
            .and_then(|rest| rest.strip_suffix(';'))
            .expect("pub use statement");
        let names = imported_names(body);
        let dest = if gated_use { &mut gated } else { &mut plain };
        dest.extend(names.into_iter().map(str::to_string));
    }
    (plain, gated)
}

fn previous_code_line<'a>(lines: &[&'a str], index: usize) -> Option<&'a str> {
    lines[..index]
        .iter()
        .rev()
        .map(|line| line.trim())
        .find(|line| !line.is_empty())
}

fn imported_names(body: &str) -> Vec<&str> {
    let inside = if let Some(start) = body.find('{') {
        let end = body.rfind('}').expect("pub use braces");
        &body[start + 1..end]
    } else {
        body.rsplit("::").next().expect("pub use path")
    };
    inside
        .split(',')
        .filter_map(|part| {
            let token = part.trim();
            if token.is_empty() {
                return None;
            }
            let name = token.rsplit("::").next().expect("path segment").trim();
            Some(name)
        })
        .collect()
}

fn column0_pub_name(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("pub ")?;
    let rest = rest
        .strip_prefix("fn ")
        .or_else(|| rest.strip_prefix("struct "))
        .or_else(|| rest.strip_prefix("enum "))
        .or_else(|| rest.strip_prefix("trait "))
        .or_else(|| rest.strip_prefix("const "))
        .or_else(|| rest.strip_prefix("type "))?;
    let end = rest
        .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .unwrap_or(rest.len());
    let name = &rest[..end];
    if name.is_empty() { None } else { Some(name) }
}

fn rust_sources() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir).unwrap_or_else(|error| {
            panic!("read {}: {error}", dir.display());
        });
        for entry in entries {
            let entry = entry.unwrap_or_else(|error| {
                panic!("dirent {}: {error}", dir.display());
            });
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}
