//! Per-observer detection, concealment, Seek, Hide, and targeting flat checks.
//!
//! The caller owns [`Detections`]. This module does not spend actions, call Strike, or change Hit Points.
//!
//! Player Core pg. 434, "Detecting Creatures".

use std::collections::{BTreeMap, BTreeSet};

use crate::check::{
    CheckModifier, DcModifier, DegreeOfSuccess, dc_from_modifier, degree_of_success,
    fallback_unlisted_critical, flat_check, flat_checks_for_same_thing, resolve_check,
};
use crate::combatant::{CombatantId, Vital};
use crate::dice::Dice;
use crate::modifiers::Cover;
use crate::positioning::{MoveAction, OffGuard};

/// Whether one observer has found a subject.
///
/// Concealment sits beside this value and does not replace it.
///
/// Player Core pg. 434, "Detecting Creatures".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Detection {
    /// The observer knows where the subject is. Player Core pg. 434, Observed.
    Observed,
    /// The observer knows the space and not which creature. Player Core pg. 434, Hidden.
    Hidden,
    /// The observer does not know the space. Player Core pg. 447, Undetected.
    Undetected,
}

/// Whether the subject is concealed from the observer.
///
/// Player Core pg. 434, "Detecting Creatures". Player Core pg. 442, Concealed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Concealment {
    /// A DC 5 flat check is required to affect an observed subject. Player Core pg. 442, Concealed.
    Concealed,
    /// No concealment flat check. Player Core pg. 442, Concealed.
    NotConcealed,
}

/// One observer's detection and concealment of one subject.
///
/// Player Core pg. 434, "Detecting Creatures".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Relation {
    /// Observed, hidden, or undetected. Player Core pg. 434, "Detecting Creatures".
    pub detection: Detection,
    /// Concealed or not, beside the detection. Player Core pg. 442, Concealed.
    pub concealment: Concealment,
}

/// Why Seek, Hide, a stored relation, or a targeting check was rejected.
///
/// Player Core pg. 434, "Detecting Creatures".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetectionError {
    /// The seeker or hider is not alive. Player Core pg. 436, "Step 2: Act".
    NotConscious,
    /// The observer and the subject are the same creature. Player Core pg. 434, "Detecting Creatures".
    SameCreature,
    /// Hidden or undetected was stored, or Hide was attempted, with no Stealth DC.
    ///
    /// Player Core pg. 401, "Getting a DC from a Modifier".
    NoStealthDc,
    /// A non-area effect chose an undetected subject. Player Core pg. 447, Undetected.
    CannotTarget,
    /// A Seek area has a dimension of 0 feet. Player Core pg. 417, Seek.
    EmptyArea,
    /// A Seek area is longer than 30 feet in some dimension. Player Core pg. 417, Seek.
    AreaTooLarge,
    /// The same id appears twice in the subject or observer list. Player Core pg. 417, Seek.
    DuplicateId,
    /// A Stealth modifier or a Perception DC does not fit in `i32`. Player Core pg. 244, Hide.
    Overflow,
    /// Perception cannot be added to 20. Player Core pg. 417, Seek.
    PerceptionOutOfRange,
    /// No observer has standard cover, greater cover, or concealment. Player Core pg. 244, Hide.
    RequirementNotMet,
}

/// What Seek or Hide costs. The caller spends the action.
///
/// This is not positioning's action cost. Positioning names only the move trait.
///
/// Player Core pg. 417, Seek. Player Core pg. 244, Hide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetectionCost {
    /// 1 action with the concentrate trait. Player Core pg. 417, Seek.
    Seek,
    /// 1 action, with none of the concentrate, move, or manipulate traits. Player Core pg. 244, Hide.
    Hide,
}

impl DetectionCost {
    /// How many actions the caller spends. Seek and Hide are each 1 action.
    ///
    /// Player Core pg. 417, Seek. Player Core pg. 244, Hide.
    pub const fn actions(self) -> u8 {
        match self {
            Self::Seek | Self::Hide => 1,
        }
    }

    /// Seek has the concentrate trait. Hide does not.
    ///
    /// Player Core pg. 417, Seek. Player Core pg. 244, Hide.
    pub const fn concentrate(self) -> bool {
        match self {
            Self::Seek => true,
            Self::Hide => false,
        }
    }

    /// Neither Seek nor Hide has the move trait.
    ///
    /// Player Core pg. 244, Hide.
    pub const fn move_trait(self) -> bool {
        match self {
            Self::Seek | Self::Hide => false,
        }
    }

    /// Neither Seek nor Hide has the manipulate trait.
    ///
    /// Player Core pg. 244, Hide.
    pub const fn manipulate(self) -> bool {
        match self {
            Self::Seek | Self::Hide => false,
        }
    }
}

/// A rectangular Seek area, in feet. The caller chooses which subjects are inside it.
///
/// Player Core pg. 417, Seek.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Area {
    /// Length of the area. Player Core pg. 417, Seek.
    pub length_feet: u32,
    /// Width of the area. Player Core pg. 417, Seek.
    pub width_feet: u32,
    /// Height of the area. Player Core pg. 417, Seek.
    pub height_feet: u32,
}

/// One open d20 check: the face, the total, and the numeric degree.
///
/// Player Core pg. 417, Seek. Player Core pg. 244, Hide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenRoll {
    /// The d20 face. Player Core pg. 417, Seek.
    pub natural: u8,
    /// Face plus the check modifier. Player Core pg. 417, Seek.
    pub total: i32,
    /// Degree after the natural 20 or natural 1 step, before an unlisted-critical fallback.
    ///
    /// Player Core pg. 400, "Step 4: Degree of Success".
    pub degree: DegreeOfSuccess,
}

/// The outcome of Seek, including the relations after the check.
///
/// Player Core pg. 417, Seek.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeekResult {
    /// 1 action with the concentrate trait. Not spent here. Player Core pg. 417, Seek.
    pub cost: DetectionCost,
    /// The open Perception check, or nothing when no subject was compared. Player Core pg. 417, Seek.
    pub roll: Option<OpenRoll>,
    /// Relations after the check. Player Core pg. 417, Seek.
    pub detections: Detections,
}

/// One creature Hide is rolled against.
///
/// `perception_modifier` is the caller's final value. This module does not read that creature's vital.
///
/// Player Core pg. 244, Hide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HideObserver {
    /// Who might lose sight of the subject. Player Core pg. 244, Hide.
    pub id: CombatantId,
    /// Perception modifier already including every penalty. Player Core pg. 401, "Getting a DC from a Modifier".
    pub perception_modifier: i32,
    /// Cover between this observer and the subject. Player Core pg. 244, Hide.
    pub cover: Option<Cover>,
}

/// The outcome of Hide, including the relations after the check.
///
/// Player Core pg. 244, Hide.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HideResult {
    /// 1 action with none of the concentrate, move, or manipulate traits. Not spent here.
    ///
    /// Player Core pg. 244, Hide.
    pub cost: DetectionCost,
    /// The open Stealth check, or nothing when no observer was compared. Player Core pg. 244, Hide.
    pub roll: Option<OpenRoll>,
    /// Relations after the check. Player Core pg. 244, Hide.
    pub detections: Detections,
}

/// Whether a targeted effect hits, and whether a failed hidden check still spends the attack.
///
/// Player Core pg. 442, Concealed. Player Core pg. 444, Hidden.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetingResult {
    /// The effect happens. Player Core pg. 400, "Checks (Flat Checks)".
    pub affected: bool,
    /// The flat-check DC, or nothing when no die was drawn. Player Core pg. 400, "Checks (Flat Checks)".
    pub dc: Option<i32>,
    /// The numeric degree, or nothing when no die was drawn. Player Core pg. 400, "Checks (Flat Checks)".
    pub degree: Option<DegreeOfSuccess>,
    /// A failed hidden flat check still uses the action. This module does not spend it.
    ///
    /// Player Core pg. 444, Hidden.
    pub action_spent: bool,
    /// A failed hidden flat check still counts for the multiple attack penalty.
    ///
    /// Player Core pg. 444, Hidden.
    pub counts_for_map: bool,
}

/// Off-guard read from detection, then the relations after a Strike reveals the striker.
///
/// Player Core pg. 244, Hide. Player Core pg. 245, Sneak.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrikeNotice {
    /// Off-guard from detection only. The caller combines other sources.
    ///
    /// Player Core pg. 445, "Off-Guard".
    pub off_guard: OffGuard,
    /// Relations after every observer of the striker becomes observed. Player Core pg. 244, Hide.
    pub detections: Detections,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Stored {
    relation: Relation,
    stealth_dc: Option<i32>,
}

/// Caller-owned detection map, keyed by observer then subject.
///
/// A missing pair is observed and not concealed. The same inputs return the same result.
///
/// Player Core pg. 434, "Detecting Creatures".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Detections {
    relations: BTreeMap<(CombatantId, CombatantId), Stored>,
}

impl Default for Detections {
    /// An empty map. Every unread pair is observed and not concealed.
    ///
    /// Player Core pg. 434, "Detecting Creatures".
    fn default() -> Self {
        Self::new()
    }
}

impl Detections {
    /// An empty map. Every unread pair is observed and not concealed.
    ///
    /// Player Core pg. 434, "Detecting Creatures".
    pub fn new() -> Self {
        Self {
            relations: BTreeMap::new(),
        }
    }

    /// The stored relation, or observed and not concealed when the pair is absent.
    ///
    /// This does not read vital and does not roll.
    ///
    /// Player Core pg. 434, "Detecting Creatures".
    pub fn get(&self, observer: CombatantId, subject: CombatantId) -> Relation {
        self.relations
            .get(&(observer, subject))
            .map(|stored| stored.relation)
            .unwrap_or(Relation {
                detection: Detection::Observed,
                concealment: Concealment::NotConcealed,
            })
    }

    /// Returns a map with `relation` stored for the pair.
    ///
    /// The same creature is [`DetectionError::SameCreature`] and the map is unchanged.
    /// Hidden or undetected without a Stealth DC is [`DetectionError::NoStealthDc`].
    /// Observed, with either concealment, needs no Stealth DC. Vital is not an argument:
    /// a dead observer or a dead subject stores when the other checks pass.
    ///
    /// Player Core pg. 434, "Detecting Creatures". Player Core pg. 442, Concealed.
    pub fn store(
        &self,
        observer: CombatantId,
        subject: CombatantId,
        relation: Relation,
        stealth_dc: Option<i32>,
    ) -> Result<Self, DetectionError> {
        if observer == subject {
            return Err(DetectionError::SameCreature);
        }
        if matches!(
            relation.detection,
            Detection::Hidden | Detection::Undetected
        ) && stealth_dc.is_none()
        {
            return Err(DetectionError::NoStealthDc);
        }
        let mut next = self.clone();
        next.relations.insert(
            (observer, subject),
            Stored {
                relation,
                stealth_dc,
            },
        );
        Ok(next)
    }
}

/// Hidden or undetected is off-guard. Otherwise `other` is kept. Concealment does not apply.
///
/// Pass the result once to Armor Class. A second yes is still one yes.
/// This function does not spend an action.
///
/// Player Core pg. 444, Hidden. Player Core pg. 447, Undetected. Player Core pg. 445, "Off-Guard".
pub fn off_guard_against(relation: Relation, other: OffGuard) -> OffGuard {
    match relation.detection {
        Detection::Hidden | Detection::Undetected => OffGuard::Yes,
        Detection::Observed => other,
    }
}

/// Resolves the flat check to affect `subject`, or rejects an undetected choice.
///
/// An observed creature that is not concealed, and every area effect, is affected and draws no die.
/// Hidden and concealed roll once, at DC 11. A hidden failure reports the action and the attack
/// count and does not call `spend_actions` or Strike. The relation is not changed.
///
/// Player Core pg. 442, Concealed. Player Core pg. 444, Hidden. Player Core pg. 400, "Checks (Flat Checks)".
pub fn resolve_targeting(
    detections: &Detections,
    observer: CombatantId,
    subject: CombatantId,
    area_effect: bool,
    dice: &mut impl Dice,
) -> Result<TargetingResult, DetectionError> {
    let relation = detections.get(observer, subject);
    if area_effect
        || (relation.detection == Detection::Observed
            && relation.concealment == Concealment::NotConcealed)
    {
        return Ok(untargeted());
    }
    if relation.detection == Detection::Undetected {
        return Err(DetectionError::CannotTarget);
    }
    let roll = match (relation.detection, relation.concealment) {
        (Detection::Hidden, Concealment::Concealed) => flat_checks_for_same_thing(dice, 5, &[11]),
        (Detection::Hidden, Concealment::NotConcealed) => flat_check(dice, 11),
        (Detection::Observed, Concealment::Concealed) => flat_check(dice, 5),
        (Detection::Observed, Concealment::NotConcealed)
        | (Detection::Undetected, Concealment::Concealed | Concealment::NotConcealed) => {
            return Ok(untargeted());
        }
    };
    let effect = fallback_unlisted_critical(roll.degree, false, false);
    let affected = effect == DegreeOfSuccess::Success;
    let hidden_failure = relation.detection == Detection::Hidden && !affected && roll.dc == 11;
    Ok(TargetingResult {
        affected,
        dc: Some(roll.dc),
        degree: Some(roll.degree),
        action_spent: hidden_failure,
        counts_for_map: hidden_failure,
    })
}

/// Seeks every hidden or undetected subject that has a Stealth DC.
///
/// One Perception check is compared with each of those DCs, in ascending id.
/// The modifier is not a [`CheckModifier`]: the gate is `checked_add(20)`, the same gate as initiative,
/// so a Perception modifier of 2147483500 is rolled. Seek lists a critical success and a success.
/// A critical failure has no entry and leaves the relation. Concealment is copied.
/// The caller spends [`DetectionCost::Seek`].
///
/// Player Core pg. 417, Seek.
pub fn seek(
    detections: Detections,
    seeker: CombatantId,
    vital: Vital,
    perception_modifier: i32,
    area: Area,
    subjects: &[CombatantId],
    dice: &mut impl Dice,
) -> Result<SeekResult, DetectionError> {
    if vital != Vital::Alive {
        return Err(DetectionError::NotConscious);
    }
    if duplicate_ids(subjects.iter().copied()) {
        return Err(DetectionError::DuplicateId);
    }
    if subjects.contains(&seeker) {
        return Err(DetectionError::SameCreature);
    }
    validate_area(area)?;
    if perception_modifier.checked_add(20).is_none() {
        return Err(DetectionError::PerceptionOutOfRange);
    }

    let mut qualifying = Vec::new();
    for subject in subjects {
        let Some(stored) = detections.relations.get(&(seeker, *subject)) else {
            continue;
        };
        if !matches!(
            stored.relation.detection,
            Detection::Hidden | Detection::Undetected
        ) {
            continue;
        }
        let Some(dc) = stored.stealth_dc else {
            continue;
        };
        qualifying.push(SeekSubject { id: *subject, dc });
    }
    qualifying.sort_by_key(|subject| subject.id);
    let Some((first, rest)) = qualifying.split_first() else {
        return Ok(SeekResult {
            cost: DetectionCost::Seek,
            roll: None,
            detections,
        });
    };

    let natural = dice.d20();
    let Some(total) = perception_modifier.checked_add(i32::from(natural)) else {
        return Err(DetectionError::Overflow);
    };
    let degree = degree_of_success(natural, total, first.dc);
    let mut next = detections;
    apply_seek(&mut next, seeker, first.id, degree);
    for subject in rest {
        let numeric = degree_of_success(natural, total, subject.dc);
        apply_seek(&mut next, seeker, subject.id, numeric);
    }
    Ok(SeekResult {
        cost: DetectionCost::Seek,
        roll: Some(OpenRoll {
            natural,
            total,
            degree,
        }),
        detections: next,
    })
}

/// Hides the subject from each observer who has standard cover, greater cover, or concealment.
///
/// The Stealth modifier is the Stealth DC minus 10, plus 2 for standard cover, 4 for greater cover,
/// or 0 when concealment is the only qualifier. One Stealth check is compared with each qualifying
/// Perception DC, in ascending id. Hide lists a success only. An observer who already has the
/// subject hidden or undetected is skipped. The caller spends [`DetectionCost::Hide`].
///
/// Player Core pg. 244, Hide. Player Core pg. 401, "Getting a DC from a Modifier".
pub fn hide(
    detections: Detections,
    subject: CombatantId,
    vital: Vital,
    stealth_dc: Option<i32>,
    observers: &[HideObserver],
    dice: &mut impl Dice,
) -> Result<HideResult, DetectionError> {
    if vital != Vital::Alive {
        return Err(DetectionError::NotConscious);
    }
    if duplicate_ids(observers.iter().map(|observer| observer.id)) {
        return Err(DetectionError::DuplicateId);
    }
    if observers.iter().any(|observer| observer.id == subject) {
        return Err(DetectionError::SameCreature);
    }
    let Some(stealth_dc) = stealth_dc else {
        return Err(DetectionError::NoStealthDc);
    };

    let mut qualifiers = Vec::new();
    let mut saw_active = false;
    for observer in observers {
        let relation = detections.get(observer.id, subject);
        if matches!(
            relation.detection,
            Detection::Hidden | Detection::Undetected
        ) {
            continue;
        }
        saw_active = true;
        if hide_qualifies(observer.cover, relation.concealment) {
            qualifiers.push(HideCheck {
                id: observer.id,
                perception_modifier: observer.perception_modifier,
                cover: observer.cover,
                concealment: relation.concealment,
            });
        }
    }
    if qualifiers.is_empty() {
        if !saw_active && !observers.is_empty() {
            return Ok(HideResult {
                cost: DetectionCost::Hide,
                roll: None,
                detections,
            });
        }
        return Err(DetectionError::RequirementNotMet);
    }
    qualifiers.sort_by_key(|check| check.id);

    let mut checked = Vec::new();
    for qualifier in qualifiers {
        checked.push(CheckedHide {
            id: qualifier.id,
            modifier: stealth_modifier(stealth_dc, qualifier.cover)?,
            dc: perception_dc(qualifier.perception_modifier)?,
            concealment: qualifier.concealment,
        });
    }
    let Some((first, rest)) = checked.split_first() else {
        return Err(DetectionError::RequirementNotMet);
    };
    let roll = resolve_check(dice, first.modifier, first.dc);
    let mut next = detections;
    record_hide(&mut next, subject, stealth_dc, first, roll.degree);
    for item in rest {
        let total = match i32::from(roll.natural).checked_add(item.modifier.get()) {
            Some(total) => total,
            // `CheckModifier` keeps every `u8` face plus the modifier inside `i32`.
            None => i32::from(roll.natural),
        };
        let numeric = degree_of_success(roll.natural, total, item.dc);
        record_hide(&mut next, subject, stealth_dc, item, numeric);
    }
    Ok(HideResult {
        cost: DetectionCost::Hide,
        roll: Some(OpenRoll {
            natural: roll.natural,
            total: roll.total,
            degree: roll.degree,
        }),
        detections: next,
    })
}

/// Ends hidden when the caller says cover and concealment are both gone.
///
/// Lesser cover does not keep the subject hidden. Undetected stays undetected.
/// Nothing calls this except the caller.
///
/// Player Core pg. 244, Hide.
pub fn refresh(
    mut detections: Detections,
    subject: CombatantId,
    observer: CombatantId,
    cover: Option<Cover>,
) -> Detections {
    let Some(stored) = detections.relations.get_mut(&(observer, subject)) else {
        return detections;
    };
    let keeps_hidden = matches!(cover, Some(Cover::Standard | Cover::Greater))
        || stored.relation.concealment == Concealment::Concealed;
    if stored.relation.detection == Detection::Hidden && !keeps_hidden {
        stored.relation.detection = Detection::Observed;
    }
    detections
}

/// Reads detection off-guard, then makes every observer see the striker.
///
/// The off-guard value covers detection only. Concealment on those pairs stays.
/// A pair whose subject is not the striker stays, including the other direction.
/// This function does not roll.
///
/// Player Core pg. 244, Hide. Player Core pg. 245, Sneak.
pub fn note_strike(
    mut detections: Detections,
    striker: CombatantId,
    target: CombatantId,
) -> StrikeNotice {
    let off_guard = off_guard_against(detections.get(target, striker), OffGuard::No);
    let reveal: Vec<CombatantId> = detections
        .relations
        .iter()
        .filter(|((_, subject), stored)| {
            *subject == striker
                && matches!(
                    stored.relation.detection,
                    Detection::Hidden | Detection::Undetected
                )
        })
        .map(|((observer, _), _)| *observer)
        .collect();
    for observer in reveal {
        if let Some(stored) = detections.relations.get_mut(&(observer, striker)) {
            stored.relation.detection = Detection::Observed;
        }
    }
    StrikeNotice {
        off_guard,
        detections,
    }
}

/// Records a move action without ending hidden and without calling [`refresh`].
///
/// Stride, Step, and Drop Prone leave every relation as it was. This function does not roll.
///
/// Player Core pg. 244, Hide.
pub fn note_action(detections: Detections, actor: CombatantId, action: MoveAction) -> Detections {
    let _ = actor;
    match action {
        MoveAction::Stride | MoveAction::Step | MoveAction::DropProne => detections,
    }
}

struct SeekSubject {
    id: CombatantId,
    dc: i32,
}

struct HideCheck {
    id: CombatantId,
    perception_modifier: i32,
    cover: Option<Cover>,
    concealment: Concealment,
}

struct CheckedHide {
    id: CombatantId,
    modifier: CheckModifier,
    dc: i32,
    concealment: Concealment,
}

fn untargeted() -> TargetingResult {
    TargetingResult {
        affected: true,
        dc: None,
        degree: None,
        action_spent: false,
        counts_for_map: false,
    }
}

fn duplicate_ids(ids: impl IntoIterator<Item = CombatantId>) -> bool {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id) {
            return true;
        }
    }
    false
}

fn validate_area(area: Area) -> Result<(), DetectionError> {
    if area.length_feet == 0 || area.width_feet == 0 || area.height_feet == 0 {
        return Err(DetectionError::EmptyArea);
    }
    if area.length_feet > 30 || area.width_feet > 30 || area.height_feet > 30 {
        return Err(DetectionError::AreaTooLarge);
    }
    Ok(())
}

fn apply_seek(
    detections: &mut Detections,
    seeker: CombatantId,
    subject: CombatantId,
    numeric: DegreeOfSuccess,
) {
    let Some(stored) = detections.relations.get_mut(&(seeker, subject)) else {
        return;
    };
    let effect = fallback_unlisted_critical(numeric, true, false);
    stored.relation.detection = match (stored.relation.detection, effect) {
        (Detection::Hidden | Detection::Undetected, DegreeOfSuccess::CriticalSuccess) => {
            Detection::Observed
        }
        (Detection::Hidden, DegreeOfSuccess::Success) => Detection::Observed,
        (Detection::Undetected, DegreeOfSuccess::Success) => Detection::Hidden,
        (detection, _) => detection,
    };
}

fn hide_qualifies(cover: Option<Cover>, concealment: Concealment) -> bool {
    matches!(cover, Some(Cover::Standard | Cover::Greater)) || concealment == Concealment::Concealed
}

fn stealth_modifier(
    stealth_dc: i32,
    cover: Option<Cover>,
) -> Result<CheckModifier, DetectionError> {
    let base = stealth_dc.checked_sub(10).ok_or(DetectionError::Overflow)?;
    let bonus = match cover {
        Some(Cover::Standard) => 2,
        Some(Cover::Greater) => 4,
        Some(Cover::Lesser) | None => 0,
    };
    let modifier = base.checked_add(bonus).ok_or(DetectionError::Overflow)?;
    CheckModifier::new(modifier).map_err(|_| DetectionError::Overflow)
}

fn perception_dc(perception_modifier: i32) -> Result<i32, DetectionError> {
    let modifier = DcModifier::new(perception_modifier).map_err(|_| DetectionError::Overflow)?;
    Ok(dc_from_modifier(modifier))
}

fn record_hide(
    detections: &mut Detections,
    subject: CombatantId,
    stealth_dc: i32,
    check: &CheckedHide,
    numeric: DegreeOfSuccess,
) {
    let effect = fallback_unlisted_critical(numeric, false, false);
    if effect != DegreeOfSuccess::Success {
        return;
    }
    detections.relations.insert(
        (check.id, subject),
        Stored {
            relation: Relation {
                detection: Detection::Hidden,
                concealment: check.concealment,
            },
            stealth_dc: Some(stealth_dc),
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Cover, DegreeOfSuccess, DyingValue, MoveAction, OffGuard, ScriptedDice, Side, Statistic,
        ac_modifiers, degree_of_success, total,
    };

    fn cid(value: u32) -> CombatantId {
        CombatantId(value)
    }

    fn feet(length_feet: u32, width_feet: u32, height_feet: u32) -> Area {
        Area {
            length_feet,
            width_feet,
            height_feet,
        }
    }

    fn rel(detection: Detection, concealment: Concealment) -> Relation {
        Relation {
            detection,
            concealment,
        }
    }

    fn remember(
        detections: Detections,
        observer: u32,
        subject: u32,
        detection: Detection,
        concealment: Concealment,
        stealth_dc: Option<i32>,
    ) -> Detections {
        detections
            .store(
                cid(observer),
                cid(subject),
                rel(detection, concealment),
                stealth_dc,
            )
            .expect("stored relation")
    }

    fn seen(detections: &Detections, observer: u32, subject: u32) -> Detection {
        detections.get(cid(observer), cid(subject)).detection
    }

    fn veil(detections: &Detections, observer: u32, subject: u32) -> Concealment {
        detections.get(cid(observer), cid(subject)).concealment
    }

    fn watcher(id: u32, perception_modifier: i32, cover: Option<Cover>) -> HideObserver {
        HideObserver {
            id: cid(id),
            perception_modifier,
            cover,
        }
    }

    fn armor_total(off_guard: OffGuard) -> i32 {
        let modifiers =
            ac_modifiers(None, off_guard == OffGuard::Yes, false).expect("ac modifiers");
        total(&modifiers).expect("total").get(Statistic::ArmorClass)
    }

    fn run_seek(
        detections: Detections,
        seeker: CombatantId,
        vital: Vital,
        perception: i32,
        area: Area,
        subjects: &[CombatantId],
        faces: &[u8],
    ) -> (Result<SeekResult, DetectionError>, ScriptedDice) {
        let mut dice = ScriptedDice::new(faces.iter().copied());
        let result = seek(
            detections, seeker, vital, perception, area, subjects, &mut dice,
        );
        (result, dice)
    }

    fn run_hide(
        detections: Detections,
        subject: CombatantId,
        vital: Vital,
        stealth_dc: Option<i32>,
        observers: &[HideObserver],
        faces: &[u8],
    ) -> (Result<HideResult, DetectionError>, ScriptedDice) {
        let mut dice = ScriptedDice::new(faces.iter().copied());
        let result = hide(detections, subject, vital, stealth_dc, observers, &mut dice);
        (result, dice)
    }

    fn run_target(
        detections: &Detections,
        observer: u32,
        subject: u32,
        area_effect: bool,
        faces: &[u8],
    ) -> (Result<TargetingResult, DetectionError>, ScriptedDice) {
        let mut dice = ScriptedDice::new(faces.iter().copied());
        let result = resolve_targeting(
            detections,
            cid(observer),
            cid(subject),
            area_effect,
            &mut dice,
        );
        (result, dice)
    }

    fn dying_one() -> Vital {
        Vital::Dying(DyingValue::new(1).expect("dying 1"))
    }

    /// Player Core pg. 434, "Detecting Creatures".
    #[test]
    fn the_same_relation_twice_matches() {
        let dice = ScriptedDice::new([8]);
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let first = detections.get(cid(1), cid(2));
        let second = detections.get(cid(1), cid(2));
        assert_eq!(first, second);
        assert_eq!(first.detection, Detection::Hidden);
        assert_eq!(first.concealment, Concealment::NotConcealed);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn the_same_seek_twice_matches() {
        let stored = remember(
            Detections::new(),
            1,
            2,
            Detection::Undetected,
            Concealment::NotConcealed,
            Some(17),
        );
        let (first, dice_a) = run_seek(
            stored.clone(),
            cid(1),
            Vital::Alive,
            5,
            feet(30, 10, 10),
            &[cid(2)],
            &[12],
        );
        let (second, dice_b) = run_seek(
            stored,
            cid(1),
            Vital::Alive,
            5,
            feet(30, 10, 10),
            &[cid(2)],
            &[12],
        );
        let first = first.expect("first seek");
        let second = second.expect("second seek");
        assert_eq!(first, second);
        let roll = first.roll.expect("roll");
        assert_eq!(roll.total, 17);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
        assert_eq!(seen(&first.detections, 1, 2), Detection::Hidden);
        assert_eq!(dice_a.remaining(), 0);
        assert_eq!(dice_b.remaining(), 0);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn the_same_hide_twice_matches() {
        let stored = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            Some(17),
        );
        let observers = [watcher(1, 5, Some(Cover::Standard))];
        let (first, dice_a) = run_hide(
            stored.clone(),
            cid(2),
            Vital::Alive,
            Some(17),
            &observers,
            &[6],
        );
        let (second, dice_b) = run_hide(stored, cid(2), Vital::Alive, Some(17), &observers, &[6]);
        let first = first.expect("first hide");
        let second = second.expect("second hide");
        assert_eq!(first, second);
        let roll = first.roll.expect("roll");
        assert_eq!(roll.total, 15);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
        assert_eq!(seen(&first.detections, 1, 2), Detection::Hidden);
        assert_eq!(dice_a.remaining(), 0);
        assert_eq!(dice_b.remaining(), 0);
    }

    /// Player Core pg. 442, Concealed. Player Core pg. 400, "Checks (Flat Checks)".
    #[test]
    fn the_same_flat_check_twice_matches() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::Concealed,
            None,
        );
        let (first, dice_a) = run_target(&detections, 1, 2, false, &[5]);
        let (second, dice_b) = run_target(&detections, 1, 2, false, &[5]);
        let first = first.expect("first flat check");
        let second = second.expect("second flat check");
        assert_eq!(first, second);
        assert_eq!(first.degree, Some(DegreeOfSuccess::Success));
        assert_eq!(first.dc, Some(5));
        assert!(first.affected);
        assert_eq!(dice_a.remaining(), 0);
        assert_eq!(dice_b.remaining(), 0);
    }

    /// Player Core pg. 434, "Detecting Creatures".
    #[test]
    fn one_subject_is_hidden_from_one_observer_and_observed_by_another() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        assert_eq!(seen(&detections, 1, 2), Detection::Hidden);
        assert_eq!(seen(&detections, 3, 2), Detection::Observed);
        assert_eq!(veil(&detections, 3, 2), Concealment::NotConcealed);
    }

    /// Player Core pg. 434, "Detecting Creatures".
    #[test]
    fn a_missing_relation_is_observed() {
        let detections = Detections::new();
        let relation = detections.get(cid(1), cid(2));
        assert_eq!(relation.detection, Detection::Observed);
        assert_eq!(relation.concealment, Concealment::NotConcealed);
    }

    /// Player Core pg. 434, "Detecting Creatures".
    #[test]
    fn the_observer_and_the_subject_cannot_be_the_same() {
        let dice = ScriptedDice::new([8]);
        let detections = Detections::new();
        let error = detections
            .store(
                cid(1),
                cid(1),
                rel(Detection::Hidden, Concealment::NotConcealed),
                Some(17),
            )
            .expect_err("same creature");
        assert_eq!(error, DetectionError::SameCreature);
        assert_eq!(seen(&detections, 1, 1), Detection::Observed);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 434, "Detecting Creatures".
    #[test]
    fn a_dead_subject_or_observer_is_stored() {
        let dice = ScriptedDice::new([8]);
        let _vitals = (Vital::Dead, Vital::Dead);
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        assert_eq!(seen(&detections, 1, 2), Detection::Hidden);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 434, "Detecting Creatures".
    #[test]
    fn a_pc_and_an_enemy_observe_each_other() {
        let _sides = (Side::Pc, Side::Enemy);
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let detections = remember(
            detections,
            2,
            1,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        assert_eq!(seen(&detections, 1, 2), Detection::Hidden);
        assert_eq!(seen(&detections, 2, 1), Detection::Hidden);
    }

    /// Player Core pg. 434, "Detecting Creatures". Player Core pg. 442, Concealed.
    #[test]
    fn an_observed_creature_can_be_concealed() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::Concealed,
            None,
        );
        assert_eq!(seen(&detections, 1, 2), Detection::Observed);
        assert_eq!(veil(&detections, 1, 2), Concealment::Concealed);
    }

    /// Player Core pg. 434, "Detecting Creatures". Player Core pg. 442, Concealed.
    #[test]
    fn a_hidden_creature_can_be_concealed() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::Concealed,
            Some(17),
        );
        assert_eq!(seen(&detections, 1, 2), Detection::Hidden);
        assert_eq!(veil(&detections, 1, 2), Concealment::Concealed);
    }

    /// Player Core pg. 401, "Getting a DC from a Modifier".
    #[test]
    fn hidden_without_a_stealth_dc_is_rejected() {
        let detections = Detections::new();
        let error = detections
            .store(
                cid(1),
                cid(2),
                rel(Detection::Hidden, Concealment::NotConcealed),
                None,
            )
            .expect_err("no stealth dc");
        assert_eq!(error, DetectionError::NoStealthDc);
        assert_eq!(seen(&detections, 1, 2), Detection::Observed);
    }

    /// Player Core pg. 401, "Getting a DC from a Modifier".
    #[test]
    fn undetected_without_a_stealth_dc_is_rejected() {
        let detections = Detections::new();
        let error = detections
            .store(
                cid(1),
                cid(2),
                rel(Detection::Undetected, Concealment::NotConcealed),
                None,
            )
            .expect_err("no stealth dc");
        assert_eq!(error, DetectionError::NoStealthDc);
        assert_eq!(seen(&detections, 1, 2), Detection::Observed);
    }

    /// Player Core pg. 401, "Getting a DC from a Modifier".
    #[test]
    fn concealed_without_a_stealth_dc_is_stored() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::Concealed,
            None,
        );
        assert_eq!(seen(&detections, 1, 2), Detection::Observed);
        assert_eq!(veil(&detections, 1, 2), Concealment::Concealed);
    }

    /// Player Core pg. 442, Concealed. Player Core pg. 400, "Checks (Flat Checks)".
    #[test]
    fn a_natural_5_at_dc_5_affects_the_target() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::Concealed,
            None,
        );
        let (result, dice) = run_target(&detections, 1, 2, false, &[5]);
        let result = result.expect("flat check");
        assert_eq!(dice.remaining(), 0);
        assert_eq!(result.dc, Some(5));
        assert_eq!(result.degree, Some(DegreeOfSuccess::Success));
        assert!(result.affected);
        assert!(!result.action_spent);
        assert!(!result.counts_for_map);
    }

    /// Player Core pg. 442, Concealed. Player Core pg. 400, "Checks (Flat Checks)".
    #[test]
    fn a_natural_4_at_dc_5_does_not_affect_the_target() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::Concealed,
            None,
        );
        let (result, dice) = run_target(&detections, 1, 2, false, &[4]);
        let result = result.expect("flat check");
        assert_eq!(dice.remaining(), 0);
        assert_eq!(result.dc, Some(5));
        assert_eq!(result.degree, Some(DegreeOfSuccess::Failure));
        assert!(!result.affected);
    }

    /// Player Core pg. 442, Concealed. Player Core pg. 400, "Checks (Flat Checks)".
    #[test]
    fn a_natural_1_at_dc_5_does_not_affect_the_target() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::Concealed,
            None,
        );
        let (result, dice) = run_target(&detections, 1, 2, false, &[1]);
        let result = result.expect("flat check");
        assert_eq!(dice.remaining(), 0);
        assert_eq!(result.degree, Some(DegreeOfSuccess::CriticalFailure));
        assert!(!result.affected);
    }

    /// Player Core pg. 442, Concealed. Player Core pg. 400, "Checks (Flat Checks)".
    #[test]
    fn a_natural_20_at_dc_5_affects_the_target() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::Concealed,
            None,
        );
        let (result, dice) = run_target(&detections, 1, 2, false, &[20]);
        let result = result.expect("flat check");
        assert_eq!(dice.remaining(), 0);
        assert_eq!(result.degree, Some(DegreeOfSuccess::CriticalSuccess));
        assert!(result.affected);
    }

    /// Player Core pg. 444, Hidden. Player Core pg. 434, Hidden.
    #[test]
    fn a_natural_11_at_dc_11_affects_the_target() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, dice) = run_target(&detections, 1, 2, false, &[11]);
        let result = result.expect("flat check");
        assert_eq!(dice.remaining(), 0);
        assert_eq!(result.dc, Some(11));
        assert_eq!(result.degree, Some(DegreeOfSuccess::Success));
        assert!(result.affected);
        assert!(!result.action_spent);
        assert!(!result.counts_for_map);
    }

    /// Player Core pg. 444, Hidden. Player Core pg. 434, Hidden.
    #[test]
    fn a_natural_10_at_dc_11_does_not_affect_the_target() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, dice) = run_target(&detections, 1, 2, false, &[10]);
        let result = result.expect("flat check");
        assert_eq!(dice.remaining(), 0);
        assert_eq!(result.dc, Some(11));
        assert_eq!(result.degree, Some(DegreeOfSuccess::Failure));
        assert!(!result.affected);
        assert!(result.action_spent);
        assert!(result.counts_for_map);
        assert_eq!(seen(&detections, 1, 2), Detection::Hidden);
    }

    /// Player Core pg. 444, Hidden. Player Core pg. 434, Hidden.
    #[test]
    fn a_natural_20_at_dc_11_affects_the_target() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, dice) = run_target(&detections, 1, 2, false, &[20]);
        let result = result.expect("flat check");
        assert_eq!(dice.remaining(), 0);
        assert_eq!(result.degree, Some(DegreeOfSuccess::CriticalSuccess));
        assert!(result.affected);
    }

    /// Player Core pg. 400, "Checks (Flat Checks)".
    #[test]
    fn hidden_and_concealed_fail_one_dc_11_roll() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::Concealed,
            Some(17),
        );
        let (result, dice) = run_target(&detections, 1, 2, false, &[10, 20]);
        let result = result.expect("flat check");
        assert_eq!(result.dc, Some(11));
        assert_eq!(result.degree, Some(DegreeOfSuccess::Failure));
        assert!(!result.affected);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 400, "Checks (Flat Checks)".
    #[test]
    fn hidden_and_concealed_succeed_at_the_reported_dc_11() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::Concealed,
            Some(17),
        );
        let (result, dice) = run_target(&detections, 1, 2, false, &[11]);
        let result = result.expect("flat check");
        assert_eq!(dice.remaining(), 0);
        assert_eq!(result.dc, Some(11));
        assert_eq!(result.degree, Some(DegreeOfSuccess::Success));
        assert!(result.affected);
    }

    /// Player Core pg. 434, Observed. Player Core pg. 444, Hidden.
    #[test]
    fn an_observed_target_rolls_nothing() {
        let detections = Detections::new();
        let (result, dice) = run_target(&detections, 1, 2, false, &[4]);
        let result = result.expect("no roll");
        assert!(result.affected);
        assert_eq!(result.dc, None);
        assert_eq!(result.degree, None);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 434, Observed. Player Core pg. 444, Hidden.
    #[test]
    fn an_area_effect_rolls_nothing() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::Concealed,
            Some(17),
        );
        let (result, dice) = run_target(&detections, 1, 2, true, &[4]);
        let result = result.expect("area");
        assert!(result.affected);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 447, Undetected.
    #[test]
    fn an_undetected_target_cannot_be_chosen() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Undetected,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, dice) = run_target(&detections, 1, 2, false, &[11]);
        assert_eq!(
            result.expect_err("cannot target"),
            DetectionError::CannotTarget
        );
        assert_eq!(seen(&detections, 1, 2), Detection::Undetected);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 444, Hidden. Player Core pg. 445, "Off-Guard".
    #[test]
    fn hidden_is_off_guard_at_minus_2() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let off_guard = off_guard_against(detections.get(cid(1), cid(2)), OffGuard::No);
        assert_eq!(off_guard, OffGuard::Yes);
        assert_eq!(armor_total(off_guard), -2);
    }

    /// Player Core pg. 447, Undetected. Player Core pg. 445, "Off-Guard".
    #[test]
    fn undetected_is_off_guard_at_minus_2() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Undetected,
            Concealment::NotConcealed,
            Some(17),
        );
        let off_guard = off_guard_against(detections.get(cid(1), cid(2)), OffGuard::No);
        assert_eq!(off_guard, OffGuard::Yes);
        assert_eq!(armor_total(off_guard), -2);
    }

    /// Player Core pg. 442, Concealed. Player Core pg. 445, "Off-Guard".
    #[test]
    fn concealed_is_not_off_guard() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::Concealed,
            None,
        );
        let off_guard = off_guard_against(detections.get(cid(1), cid(2)), OffGuard::No);
        assert_eq!(off_guard, OffGuard::No);
        assert_eq!(armor_total(off_guard), 0);
    }

    /// Player Core pg. 444, Hidden. Player Core pg. 445, "Off-Guard".
    #[test]
    fn hidden_and_concealed_is_still_off_guard() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::Concealed,
            Some(17),
        );
        let off_guard = off_guard_against(detections.get(cid(1), cid(2)), OffGuard::No);
        assert_eq!(off_guard, OffGuard::Yes);
        assert_eq!(armor_total(off_guard), -2);
    }

    /// Player Core pg. 444, Hidden. Player Core pg. 445, "Off-Guard".
    #[test]
    fn hidden_plus_another_off_guard_is_still_minus_2() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let off_guard = off_guard_against(detections.get(cid(1), cid(2)), OffGuard::Yes);
        assert_eq!(off_guard, OffGuard::Yes);
        let total = armor_total(off_guard);
        assert_eq!(total, -2);
        assert_ne!(total, -4);
    }

    /// Player Core pg. 436, "Step 2: Act". Player Core pg. 417, Seek.
    #[test]
    fn a_dying_seeker_rolls_nothing() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Undetected,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, dice) = run_seek(
            detections.clone(),
            cid(1),
            dying_one(),
            5,
            feet(35, 5, 5),
            &[cid(2)],
            &[12],
        );
        let error = result.expect_err("not conscious");
        assert_eq!(error, DetectionError::NotConscious);
        assert_ne!(error, DetectionError::AreaTooLarge);
        assert_eq!(seen(&detections, 1, 2), Detection::Undetected);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 436, "Step 2: Act". Player Core pg. 244, Hide.
    #[test]
    fn an_unconscious_hider_rolls_nothing() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let (result, dice) = run_hide(
            detections.clone(),
            cid(2),
            Vital::Unconscious,
            None,
            &[watcher(1, 5, Some(Cover::Lesser))],
            &[15],
        );
        let error = result.expect_err("not conscious");
        assert_eq!(error, DetectionError::NotConscious);
        assert_ne!(error, DetectionError::NoStealthDc);
        assert_eq!(seen(&detections, 1, 2), Detection::Observed);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 417, Seek. Player Core pg. 436, "Step 2: Act".
    #[test]
    fn seek_reports_one_concentrate_action() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Undetected,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, dice) = run_seek(
            detections,
            cid(1),
            Vital::Alive,
            5,
            feet(30, 10, 10),
            &[cid(2)],
            &[12],
        );
        let result = result.expect("seek");
        assert_eq!(result.cost, DetectionCost::Seek);
        assert_eq!(result.cost.actions(), 1);
        assert!(result.cost.concentrate());
        assert!(!result.cost.move_trait());
        assert!(!result.cost.manipulate());
        let roll = result.roll.expect("roll");
        assert_eq!(roll.natural, 12);
        assert_eq!(roll.total, 17);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn a_critical_success_moves_hidden_to_observed() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, _) = run_seek(
            detections,
            cid(1),
            Vital::Alive,
            5,
            feet(30, 10, 10),
            &[cid(2)],
            &[20],
        );
        let result = result.expect("seek");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.total, 25);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalSuccess);
        assert_eq!(seen(&result.detections, 1, 2), Detection::Observed);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn a_critical_success_moves_undetected_to_observed() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Undetected,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, _) = run_seek(
            detections,
            cid(1),
            Vital::Alive,
            5,
            feet(30, 10, 10),
            &[cid(2)],
            &[20],
        );
        let result = result.expect("seek");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.total, 25);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalSuccess);
        assert_eq!(seen(&result.detections, 1, 2), Detection::Observed);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn a_success_moves_hidden_to_observed() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, _) = run_seek(
            detections,
            cid(1),
            Vital::Alive,
            5,
            feet(30, 10, 10),
            &[cid(2)],
            &[12],
        );
        let result = result.expect("seek");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.total, 17);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
        assert_eq!(seen(&result.detections, 1, 2), Detection::Observed);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn a_success_moves_undetected_to_hidden() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Undetected,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, _) = run_seek(
            detections,
            cid(1),
            Vital::Alive,
            5,
            feet(30, 10, 10),
            &[cid(2)],
            &[12],
        );
        let result = result.expect("seek");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.natural, 12);
        assert_eq!(roll.total, 17);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
        assert_eq!(seen(&result.detections, 1, 2), Detection::Hidden);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn a_failure_leaves_undetected() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Undetected,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, _) = run_seek(
            detections,
            cid(1),
            Vital::Alive,
            5,
            feet(30, 10, 10),
            &[cid(2)],
            &[11],
        );
        let result = result.expect("seek");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.total, 16);
        assert_eq!(roll.degree, DegreeOfSuccess::Failure);
        assert_eq!(seen(&result.detections, 1, 2), Detection::Undetected);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn a_critical_failure_leaves_hidden() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, _) = run_seek(
            detections,
            cid(1),
            Vital::Alive,
            0,
            feet(30, 10, 10),
            &[cid(2)],
            &[1],
        );
        let result = result.expect("seek");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.total, 1);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalFailure);
        assert_eq!(seen(&result.detections, 1, 2), Detection::Hidden);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn a_critical_failure_leaves_undetected() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Undetected,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, _) = run_seek(
            detections,
            cid(1),
            Vital::Alive,
            0,
            feet(30, 10, 10),
            &[cid(2)],
            &[1],
        );
        let result = result.expect("seek");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.total, 1);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalFailure);
        assert_eq!(seen(&result.detections, 1, 2), Detection::Undetected);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn a_natural_20_moves_hidden_to_observed() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(25),
        );
        let (result, _) = run_seek(
            detections,
            cid(1),
            Vital::Alive,
            0,
            feet(30, 10, 10),
            &[cid(2)],
            &[20],
        );
        let result = result.expect("seek");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.total, 20);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
        assert_eq!(seen(&result.detections, 1, 2), Detection::Observed);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn a_success_leaves_concealment_set() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::Concealed,
            Some(17),
        );
        let (result, _) = run_seek(
            detections,
            cid(1),
            Vital::Alive,
            5,
            feet(30, 10, 10),
            &[cid(2)],
            &[12],
        );
        let result = result.expect("seek");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.total, 17);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
        assert_eq!(seen(&result.detections, 1, 2), Detection::Observed);
        assert_eq!(veil(&result.detections, 1, 2), Concealment::Concealed);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn one_perception_roll_meets_one_stealth_dc_and_misses_another() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Undetected,
            Concealment::NotConcealed,
            Some(17),
        );
        let detections = remember(
            detections,
            1,
            4,
            Detection::Undetected,
            Concealment::NotConcealed,
            Some(27),
        );
        let (result, dice) = run_seek(
            detections,
            cid(1),
            Vital::Alive,
            5,
            feet(30, 10, 10),
            &[cid(2), cid(4)],
            &[13],
        );
        let result = result.expect("seek");
        let roll = result.roll.expect("one roll");
        assert_eq!(roll.natural, 13);
        assert_eq!(roll.total, 18);
        assert_eq!(degree_of_success(13, 18, 17), DegreeOfSuccess::Success);
        assert_eq!(degree_of_success(13, 18, 27), DegreeOfSuccess::Failure);
        assert_eq!(seen(&result.detections, 1, 2), Detection::Hidden);
        assert_eq!(seen(&result.detections, 1, 4), Detection::Undetected);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn observed_and_a_missing_stealth_dc_are_not_compared() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            Some(17),
        );
        let detections = remember(
            detections,
            1,
            3,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let detections = remember(
            detections,
            1,
            4,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, dice) = run_seek(
            detections,
            cid(1),
            Vital::Alive,
            5,
            feet(30, 10, 10),
            &[cid(2), cid(3), cid(4)],
            &[12],
        );
        let result = result.expect("seek");
        let roll = result.roll.expect("one roll");
        assert_eq!(roll.total, 17);
        assert_eq!(seen(&result.detections, 1, 4), Detection::Observed);
        assert_eq!(seen(&result.detections, 1, 2), Detection::Observed);
        assert_eq!(seen(&result.detections, 1, 3), Detection::Observed);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn only_an_observed_subject_rolls_nothing() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, dice) = run_seek(
            detections,
            cid(1),
            Vital::Alive,
            5,
            feet(30, 10, 10),
            &[cid(2)],
            &[12],
        );
        let result = result.expect("seek");
        assert_eq!(result.roll, None);
        assert_eq!(seen(&result.detections, 1, 2), Detection::Observed);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn an_area_beyond_30_feet_rolls_nothing() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Undetected,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, dice) = run_seek(
            detections.clone(),
            cid(1),
            Vital::Alive,
            5,
            feet(35, 5, 5),
            &[cid(2)],
            &[12],
        );
        assert_eq!(result.expect_err("area"), DetectionError::AreaTooLarge);
        assert_eq!(seen(&detections, 1, 2), Detection::Undetected);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn a_zero_foot_area_rolls_nothing() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Undetected,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, dice) = run_seek(
            detections.clone(),
            cid(1),
            Vital::Alive,
            5,
            feet(0, 0, 0),
            &[cid(2)],
            &[12],
        );
        assert_eq!(result.expect_err("empty"), DetectionError::EmptyArea);
        assert_eq!(seen(&detections, 1, 2), Detection::Undetected);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn a_duplicate_subject_rolls_nothing() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Undetected,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, dice) = run_seek(
            detections.clone(),
            cid(1),
            Vital::Alive,
            5,
            feet(35, 5, 5),
            &[cid(2), cid(2)],
            &[12],
        );
        let error = result.expect_err("duplicate");
        assert_eq!(error, DetectionError::DuplicateId);
        assert_ne!(error, DetectionError::AreaTooLarge);
        assert_eq!(seen(&detections, 1, 2), Detection::Undetected);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn the_seeker_as_a_subject_rolls_nothing() {
        let (result, dice) = run_seek(
            Detections::new(),
            cid(1),
            Vital::Alive,
            5,
            feet(35, 5, 5),
            &[cid(1)],
            &[12],
        );
        let error = result.expect_err("self");
        assert_eq!(error, DetectionError::SameCreature);
        assert_ne!(error, DetectionError::AreaTooLarge);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn the_seeker_listed_twice_rolls_nothing() {
        let (result, dice) = run_seek(
            Detections::new(),
            cid(1),
            Vital::Alive,
            5,
            feet(30, 10, 10),
            &[cid(1), cid(1)],
            &[12],
        );
        let error = result.expect_err("duplicate");
        assert_eq!(error, DetectionError::DuplicateId);
        assert_ne!(error, DetectionError::SameCreature);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn a_seek_area_with_a_zero_dimension_rolls_nothing() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Undetected,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, dice) = run_seek(
            detections.clone(),
            cid(1),
            Vital::Alive,
            5,
            feet(30, 0, 10),
            &[cid(2)],
            &[12],
        );
        assert_eq!(result.expect_err("empty"), DetectionError::EmptyArea);
        assert_eq!(seen(&detections, 1, 2), Detection::Undetected);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn a_seek_at_perception_2147483628_rolls_nothing() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, dice) = run_seek(
            detections.clone(),
            cid(1),
            Vital::Alive,
            2_147_483_628,
            feet(30, 10, 10),
            &[cid(2)],
            &[12],
        );
        assert_eq!(
            result.expect_err("range"),
            DetectionError::PerceptionOutOfRange
        );
        assert_eq!(seen(&detections, 1, 2), Detection::Hidden);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn a_seek_at_perception_2147483627_is_a_critical_success() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, _) = run_seek(
            detections,
            cid(1),
            Vital::Alive,
            2_147_483_627,
            feet(30, 10, 10),
            &[cid(2)],
            &[20],
        );
        let result = result.expect("seek");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.natural, 20);
        assert_eq!(roll.total, 2_147_483_647);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalSuccess);
        assert_eq!(seen(&result.detections, 1, 2), Detection::Observed);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn a_seek_at_perception_i32_minimum_stays_hidden() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, _) = run_seek(
            detections,
            cid(1),
            Vital::Alive,
            i32::MIN,
            feet(30, 10, 10),
            &[cid(2)],
            &[20],
        );
        let result = result.expect("seek");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.natural, 20);
        assert_eq!(roll.total, -2_147_483_628);
        assert_eq!(roll.degree, DegreeOfSuccess::Failure);
        assert_eq!(seen(&result.detections, 1, 2), Detection::Hidden);
    }

    /// Player Core pg. 417, Seek.
    #[test]
    fn perception_2147483500_is_a_critical_success() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, _) = run_seek(
            detections,
            cid(1),
            Vital::Alive,
            2_147_483_500,
            feet(30, 10, 10),
            &[cid(2)],
            &[12],
        );
        let result = result.expect("seek");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.natural, 12);
        assert_eq!(roll.total, 2_147_483_512);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalSuccess);
        assert_eq!(seen(&result.detections, 1, 2), Detection::Observed);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn one_observer_without_cover_is_left_observed() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let detections = remember(
            detections,
            5,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let (result, dice) = run_hide(
            detections,
            cid(2),
            Vital::Alive,
            Some(17),
            &[watcher(1, 5, Some(Cover::Standard)), watcher(5, 5, None)],
            &[6],
        );
        let result = result.expect("hide");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.total, 15);
        assert_eq!(seen(&result.detections, 1, 2), Detection::Hidden);
        assert_eq!(seen(&result.detections, 5, 2), Detection::Observed);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn concealment_from_one_observer_hides_from_that_observer() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::Concealed,
            None,
        );
        let detections = remember(
            detections,
            5,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let (result, _) = run_hide(
            detections,
            cid(2),
            Vital::Alive,
            Some(17),
            &[watcher(1, 5, None), watcher(5, 5, None)],
            &[8],
        );
        let result = result.expect("hide");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.natural, 8);
        assert_eq!(roll.total, 15);
        assert_eq!(roll.total - i32::from(roll.natural), 7);
        assert_eq!(roll.degree, degree_of_success(8, 15, 15));
        assert_eq!(seen(&result.detections, 1, 2), Detection::Hidden);
        assert_eq!(veil(&result.detections, 1, 2), Concealment::Concealed);
        assert_eq!(seen(&result.detections, 5, 2), Detection::Observed);
        assert_eq!(veil(&result.detections, 5, 2), Concealment::NotConcealed);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn lesser_cover_is_not_enough() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let (result, dice) = run_hide(
            detections.clone(),
            cid(2),
            Vital::Alive,
            Some(17),
            &[watcher(1, 5, Some(Cover::Lesser))],
            &[15],
        );
        assert_eq!(
            result.expect_err("requirement"),
            DetectionError::RequirementNotMet
        );
        assert_eq!(seen(&detections, 1, 2), Detection::Observed);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn no_cover_and_not_concealed_is_not_enough() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let (result, dice) = run_hide(
            detections.clone(),
            cid(2),
            Vital::Alive,
            Some(17),
            &[watcher(1, 5, None)],
            &[15],
        );
        assert_eq!(
            result.expect_err("requirement"),
            DetectionError::RequirementNotMet
        );
        assert_eq!(seen(&detections, 1, 2), Detection::Observed);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn hide_without_a_stealth_dc_rolls_nothing() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let (result, dice) = run_hide(
            detections.clone(),
            cid(2),
            Vital::Alive,
            None,
            &[watcher(1, 5, Some(Cover::Lesser))],
            &[15],
        );
        let error = result.expect_err("no dc");
        assert_eq!(error, DetectionError::NoStealthDc);
        assert_ne!(error, DetectionError::RequirementNotMet);
        assert_eq!(seen(&detections, 1, 2), Detection::Observed);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn hiding_from_yourself_rolls_nothing() {
        let (result, dice) = run_hide(
            Detections::new(),
            cid(2),
            Vital::Alive,
            Some(17),
            &[watcher(2, 5, Some(Cover::Standard))],
            &[6],
        );
        assert_eq!(result.expect_err("self"), DetectionError::SameCreature);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn hiding_from_yourself_twice_rolls_nothing() {
        let (result, dice) = run_hide(
            Detections::new(),
            cid(2),
            Vital::Alive,
            Some(17),
            &[
                watcher(2, 5, Some(Cover::Standard)),
                watcher(2, 5, Some(Cover::Standard)),
            ],
            &[6],
        );
        let error = result.expect_err("duplicate");
        assert_eq!(error, DetectionError::DuplicateId);
        assert_ne!(error, DetectionError::SameCreature);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn a_duplicate_observer_rolls_nothing() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let (result, dice) = run_hide(
            detections.clone(),
            cid(2),
            Vital::Alive,
            Some(17),
            &[
                watcher(1, 5, Some(Cover::Standard)),
                watcher(1, 5, Some(Cover::Standard)),
            ],
            &[6],
        );
        assert_eq!(result.expect_err("duplicate"), DetectionError::DuplicateId);
        assert_eq!(seen(&detections, 1, 2), Detection::Observed);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn a_perception_dc_that_does_not_fit_rolls_nothing() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let (result, dice) = run_hide(
            detections.clone(),
            cid(2),
            Vital::Alive,
            Some(17),
            &[watcher(1, 2_147_483_638, Some(Cover::Standard))],
            &[10],
        );
        let error = result.expect_err("overflow");
        assert_eq!(error, DetectionError::Overflow);
        assert_ne!(error, DetectionError::PerceptionOutOfRange);
        assert_eq!(seen(&detections, 1, 2), Detection::Observed);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn the_lowest_stealth_dc_with_standard_cover_rolls_nothing() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let (result, dice) = run_hide(
            detections.clone(),
            cid(2),
            Vital::Alive,
            Some(i32::MIN),
            &[watcher(1, 5, Some(Cover::Standard))],
            &[6],
        );
        assert_eq!(result.expect_err("overflow"), DetectionError::Overflow);
        assert_eq!(seen(&detections, 1, 2), Detection::Observed);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn the_highest_stealth_dc_with_greater_cover_rolls_nothing() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let (result, dice) = run_hide(
            detections.clone(),
            cid(2),
            Vital::Alive,
            Some(i32::MAX),
            &[watcher(1, 5, Some(Cover::Greater))],
            &[6],
        );
        assert_eq!(result.expect_err("overflow"), DetectionError::Overflow);
        assert_eq!(seen(&detections, 1, 2), Detection::Observed);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 244, Hide. Player Core pg. 401, "Getting a DC from a Modifier".
    #[test]
    fn standard_cover_and_a_natural_6_hides() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let (result, _) = run_hide(
            detections,
            cid(2),
            Vital::Alive,
            Some(17),
            &[watcher(1, 5, Some(Cover::Standard))],
            &[6],
        );
        let result = result.expect("hide");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.natural, 6);
        assert_eq!(roll.total, 15);
        assert_eq!(roll.total - i32::from(roll.natural), 9);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
        assert_eq!(roll.degree, degree_of_success(6, 15, 15));
        assert_eq!(seen(&result.detections, 1, 2), Detection::Hidden);
    }

    /// Player Core pg. 244, Hide. Player Core pg. 401, "Getting a DC from a Modifier".
    #[test]
    fn standard_cover_and_a_natural_5_stays_observed() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let (result, _) = run_hide(
            detections,
            cid(2),
            Vital::Alive,
            Some(17),
            &[watcher(1, 5, Some(Cover::Standard))],
            &[5],
        );
        let result = result.expect("hide");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.total, 14);
        assert_eq!(roll.total - i32::from(roll.natural), 9);
        assert_eq!(roll.degree, DegreeOfSuccess::Failure);
        assert_eq!(roll.degree, degree_of_success(5, 14, 15));
        assert_eq!(seen(&result.detections, 1, 2), Detection::Observed);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn greater_cover_adds_4() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let (result, _) = run_hide(
            detections,
            cid(2),
            Vital::Alive,
            Some(17),
            &[watcher(1, 5, Some(Cover::Greater))],
            &[4],
        );
        let result = result.expect("hide");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.natural, 4);
        assert_eq!(roll.total, 15);
        assert_eq!(roll.total - i32::from(roll.natural), 11);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
        assert_eq!(roll.degree, degree_of_success(4, 15, 15));
        assert_eq!(seen(&result.detections, 1, 2), Detection::Hidden);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn concealment_alone_and_a_natural_8_hides() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::Concealed,
            None,
        );
        let (result, _) = run_hide(
            detections,
            cid(2),
            Vital::Alive,
            Some(17),
            &[watcher(1, 5, None)],
            &[8],
        );
        let result = result.expect("hide");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.total, 15);
        assert_eq!(roll.total - i32::from(roll.natural), 7);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
        assert_eq!(roll.degree, degree_of_success(8, 15, 15));
        assert_eq!(seen(&result.detections, 1, 2), Detection::Hidden);
        assert_eq!(veil(&result.detections, 1, 2), Concealment::Concealed);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn concealment_alone_and_a_natural_7_stays_observed() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::Concealed,
            None,
        );
        let (result, _) = run_hide(
            detections,
            cid(2),
            Vital::Alive,
            Some(17),
            &[watcher(1, 5, None)],
            &[7],
        );
        let result = result.expect("hide");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.total, 14);
        assert_eq!(roll.total - i32::from(roll.natural), 7);
        assert_eq!(roll.degree, DegreeOfSuccess::Failure);
        assert_eq!(roll.degree, degree_of_success(7, 14, 15));
        assert_eq!(seen(&result.detections, 1, 2), Detection::Observed);
        assert_eq!(veil(&result.detections, 1, 2), Concealment::Concealed);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn one_stealth_roll_meets_one_perception_dc_and_misses_another() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let detections = remember(
            detections,
            5,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let (result, dice) = run_hide(
            detections,
            cid(2),
            Vital::Alive,
            Some(17),
            &[
                watcher(1, 5, Some(Cover::Standard)),
                watcher(5, 10, Some(Cover::Standard)),
            ],
            &[10],
        );
        let result = result.expect("hide");
        let roll = result.roll.expect("one roll");
        assert_eq!(roll.natural, 10);
        assert_eq!(roll.total, 19);
        assert_eq!(degree_of_success(10, 19, 15), DegreeOfSuccess::Success);
        assert_eq!(degree_of_success(10, 19, 20), DegreeOfSuccess::Failure);
        assert_eq!(seen(&result.detections, 1, 2), Detection::Hidden);
        assert_eq!(seen(&result.detections, 5, 2), Detection::Observed);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn the_caller_passes_the_final_perception_modifier() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let (result, _) = run_hide(
            detections,
            cid(2),
            Vital::Alive,
            Some(17),
            &[watcher(1, 1, Some(Cover::Standard))],
            &[2],
        );
        let result = result.expect("hide");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.natural, 2);
        assert_eq!(roll.total, 11);
        assert_eq!(roll.total - i32::from(roll.natural), 9);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
        assert_eq!(roll.degree, degree_of_success(2, 11, 11));
        assert_eq!(seen(&result.detections, 1, 2), Detection::Hidden);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn hide_reports_one_action() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let (result, _) = run_hide(
            detections,
            cid(2),
            Vital::Alive,
            Some(17),
            &[watcher(1, 5, Some(Cover::Standard))],
            &[6],
        );
        let result = result.expect("hide");
        assert_eq!(result.cost, DetectionCost::Hide);
        assert_eq!(result.cost.actions(), 1);
        assert!(!result.cost.concentrate());
        assert!(!result.cost.move_trait());
        assert!(!result.cost.manipulate());
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn a_critical_success_on_hide_hides() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let (result, _) = run_hide(
            detections,
            cid(2),
            Vital::Alive,
            Some(17),
            &[watcher(1, 5, Some(Cover::Standard))],
            &[16],
        );
        let result = result.expect("hide");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.total, 25);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalSuccess);
        assert_eq!(seen(&result.detections, 1, 2), Detection::Hidden);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn a_natural_1_on_hide_stays_observed() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let (result, _) = run_hide(
            detections,
            cid(2),
            Vital::Alive,
            Some(17),
            &[watcher(1, 5, Some(Cover::Standard))],
            &[1],
        );
        let result = result.expect("hide");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.total - i32::from(roll.natural), 9);
        assert_eq!(roll.degree, degree_of_success(1, 10, 15));
        assert_eq!(roll.total, 10);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalFailure);
        assert_eq!(seen(&result.detections, 1, 2), Detection::Observed);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn a_natural_20_on_hide_hides() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let (result, _) = run_hide(
            detections,
            cid(2),
            Vital::Alive,
            Some(17),
            &[watcher(1, 25, Some(Cover::Standard))],
            &[20],
        );
        let result = result.expect("hide");
        let roll = result.roll.expect("roll");
        assert_eq!(roll.total - i32::from(roll.natural), 9);
        assert_eq!(roll.total, 29);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
        assert_eq!(roll.degree, degree_of_success(20, 29, 35));
        assert_eq!(seen(&result.detections, 1, 2), Detection::Hidden);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn an_observer_who_already_has_you_hidden_is_skipped() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, dice) = run_hide(
            detections,
            cid(2),
            Vital::Alive,
            Some(17),
            &[watcher(1, 5, Some(Cover::Standard))],
            &[6],
        );
        let result = result.expect("hide");
        assert_eq!(result.roll, None);
        assert_eq!(seen(&result.detections, 1, 2), Detection::Hidden);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn an_observer_who_already_has_you_undetected_is_skipped() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Undetected,
            Concealment::NotConcealed,
            Some(17),
        );
        let (result, dice) = run_hide(
            detections,
            cid(2),
            Vital::Alive,
            Some(17),
            &[watcher(1, 5, Some(Cover::Standard))],
            &[6],
        );
        let result = result.expect("hide");
        assert_eq!(result.roll, None);
        assert_eq!(seen(&result.detections, 1, 2), Detection::Undetected);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn losing_cover_and_concealment_ends_hidden() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let refreshed = refresh(detections, cid(2), cid(1), None);
        assert_eq!(seen(&refreshed, 1, 2), Detection::Observed);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn lesser_cover_does_not_keep_a_creature_hidden() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let refreshed = refresh(detections, cid(2), cid(1), Some(Cover::Lesser));
        assert_eq!(seen(&refreshed, 1, 2), Detection::Observed);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn concealment_keeps_a_hidden_creature_hidden() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::Concealed,
            Some(17),
        );
        let refreshed = refresh(detections, cid(2), cid(1), None);
        assert_eq!(seen(&refreshed, 1, 2), Detection::Hidden);
        assert_eq!(veil(&refreshed, 1, 2), Concealment::Concealed);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn losing_cover_does_not_end_undetected() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Undetected,
            Concealment::NotConcealed,
            Some(17),
        );
        let refreshed = refresh(detections, cid(2), cid(1), None);
        assert_eq!(seen(&refreshed, 1, 2), Detection::Undetected);
    }

    /// Player Core pg. 244, Hide. Player Core pg. 444, Hidden.
    #[test]
    fn a_hidden_strike_is_off_guard_then_every_observer_observes() {
        let dice = ScriptedDice::new([8]);
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let detections = remember(
            detections,
            3,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let notice = note_strike(detections, cid(2), cid(1));
        assert_eq!(notice.off_guard, OffGuard::Yes);
        assert_eq!(armor_total(notice.off_guard), -2);
        assert_eq!(seen(&notice.detections, 1, 2), Detection::Observed);
        assert_eq!(veil(&notice.detections, 1, 2), Concealment::NotConcealed);
        assert_eq!(seen(&notice.detections, 3, 2), Detection::Observed);
        assert_eq!(veil(&notice.detections, 3, 2), Concealment::NotConcealed);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn a_strike_does_not_flip_the_other_direction() {
        let dice = ScriptedDice::new([8]);
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let detections = remember(
            detections,
            1,
            3,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let detections = remember(
            detections,
            3,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let detections = remember(
            detections,
            3,
            1,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let notice = note_strike(detections, cid(2), cid(1));
        assert_eq!(seen(&notice.detections, 1, 2), Detection::Observed);
        assert_eq!(veil(&notice.detections, 1, 2), Concealment::NotConcealed);
        assert_eq!(seen(&notice.detections, 3, 2), Detection::Observed);
        assert_eq!(veil(&notice.detections, 3, 2), Concealment::NotConcealed);
        assert_eq!(seen(&notice.detections, 1, 3), Detection::Hidden);
        assert_eq!(seen(&notice.detections, 3, 1), Detection::Hidden);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn a_stride_does_not_refresh_hidden() {
        let dice = ScriptedDice::new([8]);
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let after = note_action(detections, cid(2), MoveAction::Stride);
        assert_eq!(seen(&after, 1, 2), Detection::Hidden);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 244, Hide.
    #[test]
    fn refresh_after_a_stride_ends_hidden() {
        let dice = ScriptedDice::new([8]);
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        );
        let after = note_action(detections, cid(2), MoveAction::Stride);
        let refreshed = refresh(after, cid(2), cid(1), None);
        assert_eq!(seen(&refreshed, 1, 2), Detection::Observed);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 245, Sneak. Player Core pg. 444, Hidden.
    #[test]
    fn an_undetected_strike_is_off_guard_then_the_striker_is_observed() {
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Undetected,
            Concealment::NotConcealed,
            Some(17),
        );
        let detections = remember(
            detections,
            3,
            2,
            Detection::Undetected,
            Concealment::NotConcealed,
            Some(17),
        );
        let notice = note_strike(detections, cid(2), cid(1));
        assert_eq!(notice.off_guard, OffGuard::Yes);
        assert_eq!(armor_total(notice.off_guard), -2);
        assert_eq!(seen(&notice.detections, 1, 2), Detection::Observed);
        assert_eq!(veil(&notice.detections, 1, 2), Concealment::NotConcealed);
        assert_eq!(seen(&notice.detections, 3, 2), Detection::Observed);
        assert_eq!(veil(&notice.detections, 3, 2), Concealment::NotConcealed);
    }

    /// Player Core pg. 445, "Off-Guard".
    #[test]
    fn an_observed_striker_is_not_off_guard_from_detection() {
        let dice = ScriptedDice::new([8]);
        let detections = remember(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::NotConcealed,
            None,
        );
        let notice = note_strike(detections, cid(2), cid(1));
        assert_eq!(notice.off_guard, OffGuard::No);
        let relation = notice.detections.get(cid(1), cid(2));
        assert_eq!(off_guard_against(relation, OffGuard::Yes), OffGuard::Yes);
        assert_eq!(relation.detection, Detection::Observed);
        assert_eq!(dice.remaining(), 1);
    }
}
