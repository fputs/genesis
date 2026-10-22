//! Prone, flanking, off-guard, and Take Cover.
//!
//! [`Side`] is the encounter-turns enum. This module re-exports it and does not declare
//! another. Posture and the Take Cover benefit are values the caller keeps.
//!
//! Player Core, "Drop Prone", "Stand", Prone, "Flanking", "Take Cover", Cover, and "Off-Guard".

use crate::combatant::CombatantId;
use crate::grid::{DiagonalCounter, GridError, Size, Square, in_reach, move_trait_allowed};
use crate::modifiers::{
    Cover, Modifier, ModifierError, ModifierTotal, ac_modifiers, attack_modifiers, total,
};

pub use crate::encounter::Side;

/// Standing or prone. The caller stores this. It is not a field of a creature.
///
/// Player Core, Prone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Posture {
    /// Not prone. Player Core, Prone.
    Standing,
    /// Prone. Off-guard, and attacks take a circumstance −2. Player Core, Prone.
    Prone,
}

/// Whether a creature can act. Flanking treats [`Readiness::Able`] as able to act (DEV-25).
///
/// Player Core, "Flanking".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Readiness {
    /// Able to act. Player Core, "Flanking".
    Able,
    /// Not able to act. Does not flank. Player Core, "Flanking".
    Unable,
}

/// A terrain feature the caller already judged to be nearby. This module has no wall test.
///
/// Player Core, "Take Cover".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NearbyFeature {
    /// A feature Take Cover can use. Player Core, "Take Cover".
    Present,
    /// No such feature. Player Core, "Take Cover".
    Absent,
}

/// Another creature in the square. Stand does not read this as a block (DEV-27).
///
/// Player Core, "Sharing Space".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OccupantPresence {
    /// Another creature is in the square. Player Core, "Sharing Space".
    Present,
    /// The square has no other creature. Player Core, "Sharing Space".
    Absent,
}

/// Melee or ranged, for cover that differs by attack and for off-guard.
///
/// Player Core, Cover. Player Core, "Off-Guard".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttackKind {
    /// A melee attack. Player Core, "Off-Guard".
    Melee,
    /// A ranged attack. Player Core, Cover.
    Ranged,
}

/// The benefit Take Cover stores. Lesser cover stays on the caller and is not a variant.
///
/// Player Core, "Take Cover". Player Core, Cover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoverBenefit {
    /// +2 circumstance to Armor Class against melee and ranged. Player Core, Cover.
    Standard,
    /// +4 circumstance to Armor Class against melee and ranged. Player Core, Cover.
    Greater,
    /// +4 circumstance against ranged attacks only. Player Core, Prone.
    GreaterVsRanged,
    /// +2 against melee and +4 against ranged. Player Core, "Take Cover". Player Core, Prone.
    StandardPlusGreaterVsRanged,
}

/// Why a stored Take Cover benefit ends. Ending it does not change posture.
///
/// Player Core, "Take Cover".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndReason {
    /// The creature left its square. Player Core, "Take Cover".
    LeftSquare,
    /// The creature used an attack action. Player Core, "Take Cover".
    AttackAction,
    /// The creature became unconscious. Player Core, "Take Cover".
    Unconscious,
    /// The creature dismissed the benefit. Player Core, "Take Cover".
    Dismiss,
    /// The creature was moved by force. Player Core, "Take Cover".
    ForcedMovement,
}

/// A trait named on an action cost.
///
/// `Move` is the move trait. `Manipulate` is the manipulate trait. `Ranged` means this
/// action is a ranged attack. It is not a Player Core action trait. Reactive Strike reads
/// these three (FIX-06, FIX-08).
///
/// Player Core, "Drop Prone". Player Core, "Stand". Player Core, "Interact".
/// Player Core, "Reactive Strike".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionTrait {
    /// The move trait. Player Core, "Drop Prone".
    Move,
    /// The manipulate trait. Player Core, "Interact".
    Manipulate,
    /// This action is a ranged attack. Not a Player Core action trait.
    ///
    /// Player Core, "Reactive Strike".
    Ranged,
}

/// What an action costs. Positioning reports the cost and does not spend it.
///
/// Player Core, Actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionCost {
    /// `count` actions, with the listed traits. Player Core, Actions.
    Actions {
        /// How many actions the activity costs. Player Core, Actions.
        count: u8,
        /// Traits on that cost. Empty means no move trait. Player Core, "Take Cover".
        traits: &'static [ActionTrait],
    },
    /// A free action. The traits still describe it. Player Core, "Take Cover".
    /// Player Core, "Release".
    Free {
        /// Traits on that free action. Empty is not manipulate. Player Core, "Take Cover".
        traits: &'static [ActionTrait],
    },
}

/// Actions the prone condition names. Stride and Step stay implemented in grid movement.
///
/// Leap and Crawl are not variants. Player Core, Prone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveAction {
    /// Named, and rejected while prone. The Stride distance stays in grid movement.
    ///
    /// Player Core, Prone.
    Stride,
    /// Named, and rejected while prone. The Step distance stays in grid movement.
    ///
    /// Player Core, Prone.
    Step,
    /// Drop Prone, rejected when the creature is already prone. Player Core, Prone.
    DropProne,
}

/// The outcome of Drop Prone or Stand.
///
/// The cost carries the move trait. Change 8 treats that trait as the reactive trigger
/// (FIX-06). There is no trigger flag, and the action is not spent.
///
/// Player Core, "Drop Prone" and "Stand".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PostureChange {
    /// Posture after the action. Player Core, "Drop Prone".
    pub posture: Posture,
    /// The square the creature already occupied. Player Core, "Sharing Space".
    pub square: Square,
    /// Take Cover benefit the caller keeps after the posture change. Player Core, "Take Cover".
    pub benefit: Option<CoverBenefit>,
    /// Reported cost. Not spent here. Player Core, "Drop Prone".
    pub cost: ActionCost,
}

/// The outcome of a successful Take Cover.
///
/// Player Core, "Take Cover".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TakeCoverResult {
    /// Benefit the caller stores. Player Core, "Take Cover".
    pub benefit: CoverBenefit,
    /// The square the creature already occupied. Player Core, "Take Cover".
    pub square: Square,
    /// One action and no move trait. Not spent here. Player Core, "Take Cover".
    pub cost: ActionCost,
}

/// Whether a move-trait action is legal. Not a bool, so the two states stay named.
///
/// Player Core, "Immobilized".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mobility {
    /// The move trait is legal. Player Core, "Immobilized".
    Free,
    /// The move trait is blocked. Player Core, "Immobilized".
    Immobilized,
}

/// Whether Armor Class takes the one off-guard penalty.
///
/// Player Core, "Off-Guard".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OffGuard {
    /// Circumstance −2 to Armor Class, once. Player Core, "Off-Guard".
    Yes,
    /// No off-guard penalty. Player Core, "Off-Guard".
    No,
}

/// Why a positioning action or total was rejected.
///
/// Player Core, Prone, "Take Cover", "Flanking", and Penalties.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositioningError {
    /// Immobilized blocks a move-trait action. Player Core, "Immobilized".
    Immobilized,
    /// Stand requires prone. Player Core, "Stand".
    NotProne,
    /// Standing Leap belongs to `add-crawl-and-leap`. This is not [`PositioningError::UnsupportedSize`].
    ///
    /// Player Core, Prone.
    Unsupported,
    /// Prone allows Stand, not Stride, Step, Drop Prone, or Leap. Player Core, Prone.
    ProneRestrictsMovement,
    /// Take Cover's requirement is not met, and nothing is stored. Player Core, "Take Cover".
    RequirementNotMet,
    /// The same id appears twice among the attacker and partners. Player Core, "Flanking".
    DuplicateId,
    /// A flanker id is the defender's id. Player Core, "Flanking".
    SelfId,
    /// A creature is not Small or Medium. Player Core, "Size, Space, and Reach".
    UnsupportedSize,
    /// A sum or a center did not fit. The value is not wrapped. Player Core, Penalties.
    Overflow,
    /// Reach is not a multiple of 5. Player Core, "Range and Reach".
    InvalidReach,
    /// The modifier list could not be built or totaled. Player Core, Penalties.
    Modifier(ModifierError),
}

/// Where one creature stands for a flanking or off-guard query.
///
/// No facing, faction, consent, or charm field. The caller supplies reach.
///
/// Player Core, "Flanking".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CombatantPos {
    /// Identity compared for duplicates and self-flanking. Player Core, "Flanking".
    pub id: CombatantId,
    /// Square on a grid this module does not bound. Player Core, "Grid Movement".
    pub square: Square,
    /// Size passed to reach. Only Small and Medium flank. Player Core, "Size, Space, and Reach".
    pub size: Size,
    /// Reach in feet. A multiple of 5 can threaten. Player Core, "Range and Reach".
    pub reach_feet: u32,
    /// Team. Allies share one side, and the target has the other (DEV-26). Player Core, "Flanking".
    pub side: Side,
    /// Able to act is the ledger's `able_to_act` (DEV-25). Player Core, "Flanking".
    pub readiness: Readiness,
    /// Posture. Prone does not clear Able. Player Core, Prone.
    pub posture: Posture,
}

const MOVE_TRAITS: &[ActionTrait] = &[ActionTrait::Move];
const NO_TRAITS: &[ActionTrait] = &[];

/// Drops prone in the same square. Copies `benefit`. Does not take combatant state.
///
/// Immobilized is checked before the prone restriction. The cost is one action with the
/// move trait and no trigger flag.
///
/// Player Core, "Drop Prone". Player Core, "Immobilized".
pub fn drop_prone(
    posture: Posture,
    square: Square,
    mobility: Mobility,
    benefit: Option<CoverBenefit>,
) -> Result<PostureChange, PositioningError> {
    require_mobile(mobility)?;
    match posture {
        Posture::Prone => Err(PositioningError::ProneRestrictsMovement),
        Posture::Standing => Ok(PostureChange {
            posture: Posture::Prone,
            square,
            benefit,
            cost: one_move(),
        }),
    }
}

/// Stands in the same square. An occupant does not block it (DEV-27).
///
/// The stored benefit drops only the prone-only part (R-11). Immobilized is checked
/// before [`PositioningError::NotProne`]. The cost is one action with the move trait
/// and no trigger flag.
///
/// Player Core, "Stand". Player Core, "Immobilized". Player Core, "Sharing Space".
pub fn stand(
    posture: Posture,
    square: Square,
    mobility: Mobility,
    benefit: Option<CoverBenefit>,
    _occupant: OccupantPresence,
) -> Result<PostureChange, PositioningError> {
    require_mobile(mobility)?;
    match posture {
        Posture::Standing => Err(PositioningError::NotProne),
        Posture::Prone => Ok(PostureChange {
            posture: Posture::Standing,
            square,
            benefit: benefit_after_stand(benefit),
            cost: one_move(),
        }),
    }
}

/// Names a move the prone condition rejects. Does not call Stride or Step.
///
/// `counter` is taken by value, so the caller's counter stays as it was. Success is
/// `()` and carries no distance. Immobilized is checked before the prone restriction.
///
/// Player Core, Prone. Player Core, "Immobilized".
pub fn prone_move(
    posture: Posture,
    mobility: Mobility,
    action: MoveAction,
    _counter: DiagonalCounter,
) -> Result<(), PositioningError> {
    require_mobile(mobility)?;
    match (posture, action) {
        (Posture::Prone, MoveAction::Stride | MoveAction::Step | MoveAction::DropProne) => {
            Err(PositioningError::ProneRestrictsMovement)
        }
        (Posture::Standing, MoveAction::Stride | MoveAction::Step | MoveAction::DropProne) => {
            Ok(())
        }
    }
}

/// Rejects Leap. While prone the error is the prone restriction. While standing, Leap
/// belongs to `add-crawl-and-leap` and is [`PositioningError::Unsupported`].
///
/// Neither result carries a distance. Immobilized is checked first. There is no Crawl
/// function.
///
/// Player Core, Prone. Player Core, "Immobilized".
pub fn leap_while_prone(posture: Posture, mobility: Mobility) -> Result<(), PositioningError> {
    require_mobile(mobility)?;
    match posture {
        Posture::Prone => Err(PositioningError::ProneRestrictsMovement),
        Posture::Standing => Err(PositioningError::Unsupported),
    }
}

/// Stores a better Take Cover benefit, or rejects the attempt when the requirement is unmet.
///
/// Not prone: standard or greater becomes greater; lesser plus a feature, or a feature
/// alone, becomes standard; lesser alone, or nothing, is [`PositioningError::RequirementNotMet`]
/// and does not replace `stored`. Prone always succeeds and keeps the higher bonus per
/// attack, including greater cover against ranged (DEV-24). The cost is one action with
/// an empty trait list, so immobilized does not apply.
///
/// Player Core, "Take Cover". Player Core, Cover. Player Core, Prone.
pub fn take_cover(
    posture: Posture,
    square: Square,
    stored: Option<CoverBenefit>,
    caller_cover: Option<Cover>,
    feature: NearbyFeature,
) -> Result<TakeCoverResult, PositioningError> {
    let ordinary = match ordinary_benefit(caller_cover, feature) {
        Ok(benefit) => Some(benefit),
        Err(PositioningError::RequirementNotMet) if posture == Posture::Prone => None,
        Err(error) => return Err(error),
    };
    let fresh = match posture {
        Posture::Prone => max_benefit(ordinary, Some(CoverBenefit::GreaterVsRanged)),
        Posture::Standing => ordinary,
    };
    let Some(benefit) = max_benefit(fresh, stored) else {
        return Err(PositioningError::RequirementNotMet);
    };
    Ok(TakeCoverResult {
        benefit,
        square,
        cost: ActionCost::Actions {
            count: 1,
            traits: NO_TRAITS,
        },
    })
}

/// Clears a stored benefit. `None` in is `Ok(None)`.
///
/// Player Core, "Take Cover".
pub fn dismiss(_stored: Option<CoverBenefit>) -> Result<Option<CoverBenefit>, PositioningError> {
    Ok(None)
}

/// Dismiss is a free action with no traits.
///
/// Player Core, "Take Cover".
pub fn dismiss_cost() -> ActionCost {
    ActionCost::Free { traits: &[] }
}

/// Clears the stored benefit. Does not change posture; the caller still holds that.
///
/// Player Core, "Take Cover".
pub fn end_benefit(reason: EndReason, _benefit: Option<CoverBenefit>) -> Option<CoverBenefit> {
    match reason {
        EndReason::LeftSquare
        | EndReason::AttackAction
        | EndReason::Unconscious
        | EndReason::Dismiss
        | EndReason::ForcedMovement => None,
    }
}

/// Circumstance penalty on an attack from prone, from the existing attack modifiers.
///
/// [`Posture::Prone`] passes `true`. [`Posture::Standing`] passes `false`. The zeros omit
/// the multiple attack penalty and Aid.
///
/// Player Core, Prone.
pub fn attack_penalty(posture: Posture) -> Result<Vec<Modifier>, PositioningError> {
    let prone = match posture {
        Posture::Prone => true,
        Posture::Standing => false,
    };
    attack_modifiers(0, prone, 0).map_err(PositioningError::Modifier)
}

/// Armor Class for one attack. Cover is the higher of the stored benefit and the caller
/// cover for that attack, passed once. Off-guard is one circumstance −2.
///
/// The third modifier argument stays false: unconscious is not applied here. A caller
/// status penalty belongs in `extra`. Two cover bonuses are not both passed.
///
/// Player Core, Cover. Player Core, "Off-Guard". Player Core, Penalties.
pub fn armor_class(
    benefit: Option<CoverBenefit>,
    caller_cover: Option<Cover>,
    off_guard: OffGuard,
    attack: AttackKind,
    extra: &[Modifier],
) -> Result<ModifierTotal, PositioningError> {
    let stored = benefit
        .map(|benefit| degree_of_benefit(benefit, attack))
        .unwrap_or(Degree::None);
    let caller = caller_cover.map(degree_of_cover).unwrap_or(Degree::None);
    let cover = cover_of(stored.max(caller));
    let off_guard = match off_guard {
        OffGuard::Yes => true,
        OffGuard::No => false,
    };
    let mut modifiers =
        ac_modifiers(cover, off_guard, false).map_err(PositioningError::Modifier)?;
    modifiers.extend(extra.iter().copied());
    total(&modifiers).map_err(PositioningError::Modifier)
}

/// Whether `attacker` and any partner flank `defender`.
///
/// Checks run in order: duplicate id, self id, size for the defender, the attacker, and
/// every partner, then reach and distance, centers, reach that is not a multiple of 5,
/// then the boolean. Size is before any reach or distance, so unsupported size beats
/// overflow. An empty partner list is not a flank. Partners are not paired with each other.
///
/// Player Core, "Flanking". Player Core, "Range and Reach". Player Core, "Size, Space, and Reach".
pub fn flanks(
    defender: &CombatantPos,
    attacker: &CombatantPos,
    partners: &[CombatantPos],
) -> Result<bool, PositioningError> {
    reject_duplicate_ids(attacker, partners)?;
    reject_self_ids(defender, attacker, partners)?;
    reject_unsupported_sizes(defender, attacker, partners)?;
    let attacker_reaches = reaches_target(attacker, defender)?;
    let mut partner_reaches = Vec::with_capacity(partners.len());
    for partner in partners {
        partner_reaches.push(reaches_target(partner, defender)?);
    }
    let attacker_at = local_center(attacker.square, defender.square)?;
    let mut partner_at = Vec::with_capacity(partners.len());
    for partner in partners {
        partner_at.push(local_center(partner.square, defender.square)?);
    }
    if !multiple_of_five(attacker.reach_feet)
        || partners
            .iter()
            .any(|partner| !multiple_of_five(partner.reach_feet))
    {
        return Err(PositioningError::InvalidReach);
    }
    for ((partner, partner_reaches_target), partner_center) in
        partners.iter().zip(partner_reaches).zip(partner_at)
    {
        if !attacker_reaches || !partner_reaches_target {
            continue;
        }
        if attacker.readiness != Readiness::Able || partner.readiness != Readiness::Able {
            continue;
        }
        if attacker.side != partner.side || attacker.side == defender.side {
            continue;
        }
        if center_line_flanks(attacker_at, partner_center) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Whether `defender` is off-guard to `attack` from `attacker`.
///
/// Prone is off-guard for melee and ranged before partner errors are read. Otherwise
/// the flanking error is returned unchanged. A melee flank or `other` is one off-guard.
/// Ranged flanking alone is not.
///
/// Player Core, "Flanking". Player Core, "Off-Guard".
pub fn off_guard(
    defender: &CombatantPos,
    attacker: &CombatantPos,
    attack: AttackKind,
    partners: &[CombatantPos],
    other: OffGuard,
) -> Result<bool, PositioningError> {
    if defender.posture == Posture::Prone {
        return Ok(true);
    }
    let flanked = flanks(defender, attacker, partners)?;
    Ok(match attack {
        AttackKind::Melee => flanked || other == OffGuard::Yes,
        AttackKind::Ranged => other == OffGuard::Yes,
    })
}

/// `move_trait_allowed` rejects only immobilized movement. Any rejection from it is
/// [`PositioningError::Immobilized`].
fn require_mobile(mobility: Mobility) -> Result<(), PositioningError> {
    let immobilized = match mobility {
        Mobility::Free => false,
        Mobility::Immobilized => true,
    };
    if move_trait_allowed(immobilized).is_ok() {
        Ok(())
    } else {
        Err(PositioningError::Immobilized)
    }
}

fn one_move() -> ActionCost {
    ActionCost::Actions {
        count: 1,
        traits: MOVE_TRAITS,
    }
}

fn benefit_after_stand(benefit: Option<CoverBenefit>) -> Option<CoverBenefit> {
    match benefit {
        Some(CoverBenefit::StandardPlusGreaterVsRanged) => Some(CoverBenefit::Standard),
        Some(CoverBenefit::GreaterVsRanged) => None,
        Some(CoverBenefit::Greater | CoverBenefit::Standard) | None => benefit,
    }
}

fn ordinary_benefit(
    caller_cover: Option<Cover>,
    feature: NearbyFeature,
) -> Result<CoverBenefit, PositioningError> {
    match (caller_cover, feature) {
        (Some(Cover::Standard | Cover::Greater), _) => Ok(CoverBenefit::Greater),
        (Some(Cover::Lesser) | None, NearbyFeature::Present) => Ok(CoverBenefit::Standard),
        (Some(Cover::Lesser) | None, NearbyFeature::Absent) => {
            Err(PositioningError::RequirementNotMet)
        }
    }
}

/// Bonus rank for one attack. Lesser exists only on caller cover, not on a stored benefit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Degree {
    None = 0,
    Lesser = 1,
    Standard = 2,
    Greater = 4,
}

fn degree_of_benefit(benefit: CoverBenefit, attack: AttackKind) -> Degree {
    match (benefit, attack) {
        (CoverBenefit::Standard, _) => Degree::Standard,
        (CoverBenefit::Greater, _) => Degree::Greater,
        (CoverBenefit::GreaterVsRanged, AttackKind::Melee) => Degree::None,
        (CoverBenefit::GreaterVsRanged, AttackKind::Ranged) => Degree::Greater,
        (CoverBenefit::StandardPlusGreaterVsRanged, AttackKind::Melee) => Degree::Standard,
        (CoverBenefit::StandardPlusGreaterVsRanged, AttackKind::Ranged) => Degree::Greater,
    }
}

fn degree_of_cover(cover: Cover) -> Degree {
    match cover {
        Cover::Lesser => Degree::Lesser,
        Cover::Standard => Degree::Standard,
        Cover::Greater => Degree::Greater,
    }
}

fn cover_of(degree: Degree) -> Option<Cover> {
    match degree {
        Degree::None => None,
        Degree::Lesser => Some(Cover::Lesser),
        Degree::Standard => Some(Cover::Standard),
        Degree::Greater => Some(Cover::Greater),
    }
}

/// The Take Cover tables only produce five pairs. Lesser is caller cover and is not stored.
fn encode(melee: Degree, ranged: Degree) -> Option<CoverBenefit> {
    match (melee, ranged) {
        (Degree::Greater, _) => Some(CoverBenefit::Greater),
        (Degree::Standard, Degree::Greater) => Some(CoverBenefit::StandardPlusGreaterVsRanged),
        (Degree::Standard, _) => Some(CoverBenefit::Standard),
        (Degree::None | Degree::Lesser, Degree::Greater) => Some(CoverBenefit::GreaterVsRanged),
        (Degree::None | Degree::Lesser, _) => None,
    }
}

fn max_benefit(left: Option<CoverBenefit>, right: Option<CoverBenefit>) -> Option<CoverBenefit> {
    let (left_melee, left_ranged) = pair(left);
    let (right_melee, right_ranged) = pair(right);
    encode(left_melee.max(right_melee), left_ranged.max(right_ranged))
}

fn pair(benefit: Option<CoverBenefit>) -> (Degree, Degree) {
    match benefit {
        Some(benefit) => (
            degree_of_benefit(benefit, AttackKind::Melee),
            degree_of_benefit(benefit, AttackKind::Ranged),
        ),
        None => (Degree::None, Degree::None),
    }
}

fn reject_duplicate_ids(
    attacker: &CombatantPos,
    partners: &[CombatantPos],
) -> Result<(), PositioningError> {
    for (index, partner) in partners.iter().enumerate() {
        if partner.id == attacker.id {
            return Err(PositioningError::DuplicateId);
        }
        for other in partners.iter().skip(index + 1) {
            if other.id == partner.id {
                return Err(PositioningError::DuplicateId);
            }
        }
    }
    Ok(())
}

fn reject_self_ids(
    defender: &CombatantPos,
    attacker: &CombatantPos,
    partners: &[CombatantPos],
) -> Result<(), PositioningError> {
    if attacker.id == defender.id || partners.iter().any(|partner| partner.id == defender.id) {
        return Err(PositioningError::SelfId);
    }
    Ok(())
}

/// Size before reach: an unsupported partner is not hidden by another creature's distance.
fn reject_unsupported_sizes(
    defender: &CombatantPos,
    attacker: &CombatantPos,
    partners: &[CombatantPos],
) -> Result<(), PositioningError> {
    flank_size(defender.size)?;
    flank_size(attacker.size)?;
    for partner in partners {
        flank_size(partner.size)?;
    }
    Ok(())
}

fn flank_size(size: Size) -> Result<(), PositioningError> {
    match size {
        Size::Small | Size::Medium => Ok(()),
        Size::Tiny | Size::Large | Size::Huge | Size::Gargantuan => {
            Err(PositioningError::UnsupportedSize)
        }
    }
}

fn reaches_target(
    flanker: &CombatantPos,
    defender: &CombatantPos,
) -> Result<bool, PositioningError> {
    in_reach(
        flanker.square,
        defender.square,
        flanker.size,
        defender.size,
        flanker.reach_feet,
    )
    .map_err(grid_failure)
}

/// `in_reach` returns unsupported size or overflow. Each grid error has its own arm so a
/// new variant fails this match.
fn grid_failure(error: GridError) -> PositioningError {
    match error {
        GridError::UnsupportedSize => PositioningError::UnsupportedSize,
        GridError::Overflow => PositioningError::Overflow,
        GridError::ExceedsSpeed => PositioningError::Overflow,
        GridError::SpeedNotMultipleOfFive => PositioningError::Overflow,
        GridError::NegativeSpeed => PositioningError::Overflow,
        GridError::OccupiedEnd => PositioningError::Overflow,
        GridError::EnemySquare => PositioningError::Overflow,
        GridError::StepIntoDifficultTerrain => PositioningError::Overflow,
        GridError::StepRequiresTenFeet => PositioningError::Overflow,
        GridError::StepNotOneSquare => PositioningError::Overflow,
        GridError::PathNotAdjacent => PositioningError::Overflow,
        GridError::OffGrid => PositioningError::Overflow,
        GridError::Immobilized => PositioningError::Overflow,
        GridError::InvalidRangeIncrement => PositioningError::Overflow,
        GridError::NegativeDistance => PositioningError::Overflow,
        GridError::BeyondRange => PositioningError::Overflow,
    }
}

fn multiple_of_five(reach_feet: u32) -> bool {
    reach_feet.is_multiple_of(5)
}

/// Half-square center `(2x + 1, 2y + 1)`. A value that does not fit in `i32` is overflow,
/// including when a relative offset would have fit.
fn half_center(square: Square) -> Result<(i32, i32), PositioningError> {
    let x = square
        .x
        .checked_mul(2)
        .and_then(|value| value.checked_add(1))
        .ok_or(PositioningError::Overflow)?;
    let y = square
        .y
        .checked_mul(2)
        .and_then(|value| value.checked_add(1))
        .ok_or(PositioningError::Overflow)?;
    Ok((x, y))
}

/// Center of `square` in the target square's half-square frame, where that square is
/// `[0, 2] × [0, 2]`. Differences are widened so a later cross product cannot wrap.
fn local_center(square: Square, target: Square) -> Result<Pt, PositioningError> {
    let center = half_center(square)?;
    let origin = half_center(target)?;
    Ok(Pt {
        x: i128::from(center.0) - (i128::from(origin.0) - 1),
        y: i128::from(center.1) - (i128::from(origin.1) - 1),
    })
}

#[derive(Clone, Copy)]
struct Pt {
    x: i128,
    y: i128,
}

fn pt(x: i128, y: i128) -> Pt {
    Pt { x, y }
}

/// A corner counts as both edges (DEV-23). Opposite edges, or two opposite corners, flank.
fn center_line_flanks(first: Pt, second: Pt) -> bool {
    let left = segments_touch(first, second, pt(0, 0), pt(0, 2));
    let right = segments_touch(first, second, pt(2, 0), pt(2, 2));
    let bottom = segments_touch(first, second, pt(0, 0), pt(2, 0));
    let top = segments_touch(first, second, pt(0, 2), pt(2, 2));
    let opposite_edges = (left && right) || (bottom && top);
    let opposite_corners = (passes_through(first, second, pt(0, 0))
        && passes_through(first, second, pt(2, 2)))
        || (passes_through(first, second, pt(0, 2)) && passes_through(first, second, pt(2, 0)));
    opposite_edges || opposite_corners
}

fn segments_touch(a: Pt, b: Pt, c: Pt, d: Pt) -> bool {
    let first = orient(a, b, c);
    let second = orient(a, b, d);
    let third = orient(c, d, a);
    let fourth = orient(c, d, b);
    if first != 0 && second != 0 && third != 0 && fourth != 0 {
        return (first > 0) != (second > 0) && (third > 0) != (fourth > 0);
    }
    (first == 0 && on_segment(a, b, c))
        || (second == 0 && on_segment(a, b, d))
        || (third == 0 && on_segment(c, d, a))
        || (fourth == 0 && on_segment(c, d, b))
}

fn passes_through(a: Pt, b: Pt, point: Pt) -> bool {
    orient(a, b, point) == 0 && on_segment(a, b, point)
}

fn orient(a: Pt, b: Pt, c: Pt) -> i128 {
    let ab_x = b.x - a.x;
    let ab_y = b.y - a.y;
    let ac_x = c.x - a.x;
    let ac_y = c.y - a.y;
    ab_x * ac_y - ab_y * ac_x
}

fn on_segment(a: Pt, b: Pt, point: Pt) -> bool {
    point.x >= a.x.min(b.x)
        && point.x <= a.x.max(b.x)
        && point.y >= a.y.min(b.y)
        && point.y <= a.y.max(b.y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BonusType, BuildParts, CharacterBuild, CombatantId, CombatantState, DieCount, Faces,
        FactionId, MapBounds, MeleeStrike, Modifier, ModifierError, MoveInput, PenaltyType,
        ReachFeet, ScriptedDice, Size, SourceId, Statistic, derive, new_combatant_state, stride,
        total,
    };

    fn at(x: i32, y: i32) -> Square {
        Square { x, y }
    }

    fn held_die() -> ScriptedDice {
        ScriptedDice::new([8])
    }

    fn assert_held(dice: &ScriptedDice) {
        assert_eq!(dice.remaining(), 1);
    }

    fn actions_left(actions: u8) -> CombatantState {
        let parts = BuildParts {
            hit_points: 20,
            armor_class: 18,
            perception: 5,
            speed_feet: 25,
            strike: MeleeStrike {
                attack_bonus: 7,
                damage_dice: DieCount::new(1).expect("one die"),
                damage_faces: Faces::new(8).expect("d8"),
                damage_bonus: 4,
                reach_feet: ReachFeet::new(5).expect("reach 5"),
                agile: false,
            },
            shield: None,
            stealth_dc: None,
            aid_modifier: None,
            aid_rank: None,
            uses_dying_rules: None,
        };
        let creature = derive(&CharacterBuild::try_new(parts).expect("valid build"));
        let mut state = new_combatant_state(CombatantId(1), &creature);
        state.actions_remaining = actions;
        state.reaction_ready = false;
        state
    }

    fn one_diagonal() -> DiagonalCounter {
        let outcome = stride(MoveInput {
            start: at(0, 0),
            path: &[at(1, 1)],
            speed_feet: 25,
            size: Size::Medium,
            immobilized: false,
            faction: FactionId(1),
            bounds: MapBounds {
                width: 10,
                height: 10,
            },
            terrain: &[],
            occupants: &[],
            counter: DiagonalCounter::fresh(),
        })
        .expect("diagonal stride");
        assert_eq!(outcome.counter.diagonals(), 1);
        outcome.counter
    }

    fn assert_move_cost(cost: ActionCost) {
        assert_eq!(
            cost,
            ActionCost::Actions {
                count: 1,
                traits: &[ActionTrait::Move],
            }
        );
    }

    fn ac(
        benefit: Option<CoverBenefit>,
        caller: Option<Cover>,
        off_guard: OffGuard,
        attack: AttackKind,
        extra: &[Modifier],
    ) -> ModifierTotal {
        let first = armor_class(benefit, caller, off_guard, attack, extra).expect("armor class");
        let second = armor_class(benefit, caller, off_guard, attack, extra).expect("armor class");
        assert_eq!(first, second);
        first
    }

    fn ac_value(
        benefit: Option<CoverBenefit>,
        caller: Option<Cover>,
        off_guard: OffGuard,
        attack: AttackKind,
        extra: &[Modifier],
    ) -> i32 {
        ac(benefit, caller, off_guard, attack, extra).get(Statistic::ArmorClass)
    }

    fn placed(id: u32, x: i32, y: i32, side: Side) -> CombatantPos {
        CombatantPos {
            id: CombatantId(id),
            square: at(x, y),
            size: Size::Medium,
            reach_feet: 5,
            side,
            readiness: Readiness::Able,
            posture: Posture::Standing,
        }
    }

    fn circumstance_bonuses(total: &ModifierTotal) -> Vec<i32> {
        total
            .applied()
            .iter()
            .filter(|modifier| {
                modifier.bonus_type() == Some(BonusType::Circumstance)
                    && modifier.statistic() == Statistic::ArmorClass
            })
            .map(|modifier| modifier.value())
            .collect()
    }

    /// Player Core, "Drop Prone".
    #[test]
    fn the_same_drop_prone_twice_matches() {
        let dice = held_die();
        let first = drop_prone(Posture::Standing, at(3, 4), Mobility::Free, None).expect("prone");
        let second = drop_prone(Posture::Standing, at(3, 4), Mobility::Free, None).expect("prone");
        assert_eq!(first, second);
        assert_eq!(first.posture, Posture::Prone);
        assert_eq!(first.square, at(3, 4));
        assert_held(&dice);
    }

    /// Player Core, Prone.
    #[test]
    fn drop_prone_leaves_the_combatant_state() {
        let dice = held_die();
        let state = actions_left(2);
        let changed = drop_prone(Posture::Standing, at(3, 4), Mobility::Free, None).expect("prone");
        assert_eq!(changed.posture, Posture::Prone);
        assert_eq!(changed.square, at(3, 4));
        assert_eq!(state.actions_remaining, 2);
        assert!(!state.reaction_ready);
        assert_held(&dice);
    }

    /// Player Core, "Drop Prone".
    #[test]
    fn drop_prone_reports_one_move_action() {
        let dice = held_die();
        let changed = drop_prone(Posture::Standing, at(3, 4), Mobility::Free, None).expect("prone");
        let PostureChange {
            posture,
            square,
            benefit,
            cost,
        } = changed;
        assert_eq!(posture, Posture::Prone);
        assert_eq!(square, at(3, 4));
        assert_eq!(benefit, None);
        assert_move_cost(cost);
        assert_held(&dice);
    }

    /// Player Core, "Stand".
    #[test]
    fn stand_reports_one_move_action() {
        let dice = held_die();
        let first = stand(
            Posture::Prone,
            at(3, 4),
            Mobility::Free,
            None,
            OccupantPresence::Absent,
        )
        .expect("stand");
        let second = stand(
            Posture::Prone,
            at(3, 4),
            Mobility::Free,
            None,
            OccupantPresence::Absent,
        )
        .expect("stand");
        assert_eq!(first, second);
        let PostureChange {
            posture,
            square: _,
            benefit: _,
            cost,
        } = first;
        assert_eq!(posture, Posture::Standing);
        assert_move_cost(cost);
        assert_held(&dice);
    }

    /// Player Core, "Immobilized".
    #[test]
    fn an_immobilized_creature_cannot_drop_prone_or_stand() {
        let dice = held_die();
        let square = at(3, 4);
        let benefit = Some(CoverBenefit::Standard);
        for posture in [Posture::Standing, Posture::Prone] {
            assert_eq!(
                drop_prone(posture, square, Mobility::Immobilized, benefit),
                Err(PositioningError::Immobilized)
            );
            assert_eq!(
                stand(
                    posture,
                    square,
                    Mobility::Immobilized,
                    benefit,
                    OccupantPresence::Absent,
                ),
                Err(PositioningError::Immobilized)
            );
        }
        assert_eq!(square, at(3, 4));
        assert_eq!(benefit, Some(CoverBenefit::Standard));
        assert_held(&dice);
    }

    /// Player Core, "Stand".
    #[test]
    fn standing_cannot_stand() {
        let dice = held_die();
        assert_eq!(
            stand(
                Posture::Standing,
                at(3, 4),
                Mobility::Free,
                None,
                OccupantPresence::Absent,
            ),
            Err(PositioningError::NotProne)
        );
        assert_held(&dice);
    }

    /// Player Core, "Immobilized".
    #[test]
    fn immobilized_wins_over_not_prone() {
        let dice = held_die();
        assert_eq!(
            stand(
                Posture::Standing,
                at(3, 4),
                Mobility::Immobilized,
                None,
                OccupantPresence::Absent,
            ),
            Err(PositioningError::Immobilized)
        );
        assert_held(&dice);
    }

    /// Player Core, "Immobilized".
    #[test]
    fn immobilized_wins_over_a_prone_move() {
        let dice = held_die();
        let counter = one_diagonal();
        assert_eq!(
            prone_move(
                Posture::Prone,
                Mobility::Immobilized,
                MoveAction::Step,
                counter,
            ),
            Err(PositioningError::Immobilized)
        );
        assert_eq!(counter.diagonals(), 1);
        assert_held(&dice);
    }

    /// Player Core, "Immobilized". Player Core, Prone.
    #[test]
    fn immobilized_wins_over_leap_while_prone() {
        let dice = held_die();
        let result: Result<(), PositioningError> =
            leap_while_prone(Posture::Prone, Mobility::Immobilized);
        assert_eq!(result, Err(PositioningError::Immobilized));
        assert_held(&dice);
    }

    /// Player Core, Prone.
    #[test]
    fn standing_prone_move_step_is_ok() {
        let dice = held_die();
        let counter = one_diagonal();
        let first: Result<(), PositioningError> =
            prone_move(Posture::Standing, Mobility::Free, MoveAction::Step, counter);
        let second: Result<(), PositioningError> =
            prone_move(Posture::Standing, Mobility::Free, MoveAction::Step, counter);
        assert_eq!(first, Ok(()));
        assert_eq!(first, second);
        assert_eq!(counter.diagonals(), 1);
        assert_held(&dice);
    }

    /// Player Core, Prone.
    #[test]
    fn only_stand_is_legal_while_prone() {
        let dice = held_die();
        let square = at(3, 4);
        let stood = stand(
            Posture::Prone,
            square,
            Mobility::Free,
            None,
            OccupantPresence::Absent,
        )
        .expect("stand");
        assert_eq!(stood.posture, Posture::Standing);
        assert_eq!(stood.square, square);
        assert_eq!(
            drop_prone(Posture::Prone, square, Mobility::Free, None),
            Err(PositioningError::ProneRestrictsMovement)
        );
        let counter = DiagonalCounter::fresh();
        assert_eq!(
            prone_move(Posture::Prone, Mobility::Free, MoveAction::Stride, counter),
            Err(PositioningError::ProneRestrictsMovement)
        );
        assert_eq!(
            prone_move(Posture::Prone, Mobility::Free, MoveAction::Step, counter),
            Err(PositioningError::ProneRestrictsMovement)
        );
        assert_held(&dice);
    }

    /// Player Core, Prone.
    #[test]
    fn no_prone_move_variant_returns_a_distance() {
        let dice = held_die();
        let counter = one_diagonal();
        for action in [MoveAction::Stride, MoveAction::Step, MoveAction::DropProne] {
            let result: Result<(), PositioningError> =
                prone_move(Posture::Prone, Mobility::Free, action, counter);
            assert_eq!(result, Err(PositioningError::ProneRestrictsMovement));
        }
        assert_eq!(counter.diagonals(), 1);
        assert_held(&dice);
    }

    /// Player Core, Prone.
    #[test]
    fn leap_while_prone_is_rejected() {
        let dice = held_die();
        let result: Result<(), PositioningError> = leap_while_prone(Posture::Prone, Mobility::Free);
        assert_eq!(result, Err(PositioningError::ProneRestrictsMovement));
        assert_held(&dice);
    }

    /// Player Core, Prone.
    #[test]
    fn a_standing_leap_is_not_resolved() {
        let dice = held_die();
        let first: Result<(), PositioningError> =
            leap_while_prone(Posture::Standing, Mobility::Free);
        let second: Result<(), PositioningError> =
            leap_while_prone(Posture::Standing, Mobility::Free);
        assert_eq!(first, Err(PositioningError::Unsupported));
        assert_eq!(first, second);
        assert_held(&dice);
    }

    /// Player Core, Prone.
    #[test]
    fn move_actions_are_stride_step_and_drop_prone() {
        let dice = held_die();
        let listed = [MoveAction::Stride, MoveAction::Step, MoveAction::DropProne];
        let names: Vec<&str> = listed
            .into_iter()
            .map(|action| match action {
                MoveAction::Stride => "stride",
                MoveAction::Step => "step",
                MoveAction::DropProne => "drop prone",
            })
            .collect();
        assert_eq!(names, ["stride", "step", "drop prone"]);
        let counter = DiagonalCounter::fresh();
        let stepped: Result<(), PositioningError> =
            prone_move(Posture::Standing, Mobility::Free, MoveAction::Step, counter);
        let strided: Result<(), PositioningError> = prone_move(
            Posture::Standing,
            Mobility::Free,
            MoveAction::Stride,
            counter,
        );
        assert_eq!(stepped, Ok(()));
        assert_eq!(strided, Ok(()));
        assert_eq!(counter.diagonals(), 0);
        assert_held(&dice);
    }

    /// Player Core, "Sharing Space".
    #[test]
    fn the_returned_square_equals_the_input_square() {
        let dice = held_die();
        let square = at(3, 4);
        let dropped =
            drop_prone(Posture::Standing, square, Mobility::Free, None).expect("drop prone");
        let stood = stand(
            Posture::Prone,
            square,
            Mobility::Free,
            None,
            OccupantPresence::Absent,
        )
        .expect("stand");
        let covered = take_cover(
            Posture::Standing,
            square,
            None,
            Some(Cover::Standard),
            NearbyFeature::Absent,
        )
        .expect("take cover");
        assert_eq!(dropped.square, square);
        assert_eq!(stood.square, square);
        assert_eq!(covered.square, square);
        assert_held(&dice);
    }

    /// Player Core, "Sharing Space". Player Core, "Stand".
    #[test]
    fn stand_ignores_an_occupant() {
        let dice = held_die();
        let changed = stand(
            Posture::Prone,
            at(3, 4),
            Mobility::Free,
            None,
            OccupantPresence::Present,
        )
        .expect("stand");
        let PostureChange {
            posture,
            square,
            benefit,
            cost,
        } = changed;
        assert_eq!(posture, Posture::Standing);
        assert_eq!(square, at(3, 4));
        assert_eq!(benefit, None);
        assert_move_cost(cost);
        assert_held(&dice);
    }

    /// Player Core, "Drop Prone". Player Core, Cover.
    #[test]
    fn drop_prone_keeps_the_square_and_the_cover_benefit() {
        let dice = held_die();
        let changed = drop_prone(
            Posture::Standing,
            at(3, 4),
            Mobility::Free,
            Some(CoverBenefit::Standard),
        )
        .expect("drop prone");
        assert_eq!(changed.posture, Posture::Prone);
        assert_eq!(changed.square, at(3, 4));
        assert_eq!(changed.benefit, Some(CoverBenefit::Standard));
        assert_held(&dice);
    }

    /// Player Core, Prone. Player Core, "Off-Guard".
    #[test]
    fn prone_off_guard_and_the_attack_penalty() {
        let dice = held_die();
        let first = attack_penalty(Posture::Prone).expect("prone penalty");
        let second = attack_penalty(Posture::Prone).expect("prone penalty");
        assert_eq!(first, second);
        let attack = total(&first).expect("attack total");
        assert_eq!(attack.get(Statistic::Attack), -2);
        assert_eq!(
            ac_value(None, None, OffGuard::Yes, AttackKind::Melee, &[]),
            -2
        );
        assert_eq!(
            ac_value(None, None, OffGuard::Yes, AttackKind::Ranged, &[]),
            -2
        );
        assert_held(&dice);
    }

    /// Player Core, Prone.
    #[test]
    fn a_standing_attacker_has_no_prone_penalty() {
        let dice = held_die();
        let modifiers = attack_penalty(Posture::Standing).expect("standing");
        let attack = total(&modifiers).expect("attack total");
        assert_eq!(attack.get(Statistic::Attack), 0);
        assert!(
            modifiers
                .iter()
                .all(|modifier| { modifier.penalty_type() != Some(PenaltyType::Circumstance) })
        );
        assert_held(&dice);
    }

    /// Player Core, Prone. Player Core, "Off-Guard".
    #[test]
    fn a_prone_attacker_and_an_off_guard_defender() {
        let dice = held_die();
        let modifiers = attack_penalty(Posture::Prone).expect("prone penalty");
        let attack = total(&modifiers).expect("attack total");
        assert_eq!(attack.get(Statistic::Attack), -2);
        assert_eq!(
            ac_value(None, None, OffGuard::Yes, AttackKind::Melee, &[]),
            -2
        );
        assert_held(&dice);
    }

    /// Player Core, Prone. Player Core, Penalties.
    #[test]
    fn prone_penalty_and_an_untyped_penalty_total_minus_7() {
        let dice = held_die();
        let mut modifiers = attack_penalty(Posture::Prone).expect("prone penalty");
        let untyped = Modifier::penalty(
            PenaltyType::Untyped,
            -5,
            SourceId(5),
            8,
            None,
            Statistic::Attack,
        )
        .expect("untyped penalty");
        modifiers.push(untyped);
        let attack = total(&modifiers).expect("attack total");
        assert_eq!(attack.get(Statistic::Attack), -7);
        assert_held(&dice);
    }

    /// Player Core, Cover. Player Core, "Off-Guard".
    #[test]
    fn standard_cover_and_off_guard_total_0() {
        let dice = held_die();
        assert_eq!(
            ac_value(
                None,
                Some(Cover::Standard),
                OffGuard::Yes,
                AttackKind::Melee,
                &[],
            ),
            0
        );
        assert_eq!(
            ac_value(
                None,
                Some(Cover::Standard),
                OffGuard::Yes,
                AttackKind::Ranged,
                &[],
            ),
            0
        );
        assert_held(&dice);
    }

    /// Player Core, Cover. Player Core, "Off-Guard".
    #[test]
    fn greater_cover_and_off_guard_total_plus_2() {
        let dice = held_die();
        assert_eq!(
            ac_value(
                None,
                Some(Cover::Greater),
                OffGuard::Yes,
                AttackKind::Melee,
                &[],
            ),
            2
        );
        assert_eq!(
            ac_value(
                None,
                Some(Cover::Greater),
                OffGuard::Yes,
                AttackKind::Ranged,
                &[],
            ),
            2
        );
        assert_held(&dice);
    }

    /// Player Core, Cover.
    #[test]
    fn stored_standard_and_caller_greater_total_plus_4() {
        let dice = held_die();
        for attack in [AttackKind::Melee, AttackKind::Ranged] {
            let total = ac(
                Some(CoverBenefit::Standard),
                Some(Cover::Greater),
                OffGuard::No,
                attack,
                &[],
            );
            assert_eq!(total.get(Statistic::ArmorClass), 4);
            assert_eq!(circumstance_bonuses(&total), vec![4]);
        }
        assert_held(&dice);
    }

    /// Player Core, Penalties. Player Core, "Off-Guard".
    #[test]
    fn off_guard_and_a_circumstance_penalty_total_minus_2() {
        let dice = held_die();
        let extra = Modifier::penalty(
            PenaltyType::Circumstance,
            -1,
            SourceId(1),
            9,
            None,
            Statistic::ArmorClass,
        )
        .expect("circumstance penalty");
        assert_eq!(
            ac_value(None, None, OffGuard::Yes, AttackKind::Melee, &[extra]),
            -2
        );
        assert_held(&dice);
    }

    /// Player Core, Penalties. Player Core, "Off-Guard".
    #[test]
    fn unconscious_status_and_off_guard_total_minus_6() {
        let dice = held_die();
        let status = Modifier::penalty(
            PenaltyType::Status,
            -4,
            SourceId(11),
            100,
            None,
            Statistic::ArmorClass,
        )
        .expect("status penalty");
        let total = ac(None, None, OffGuard::Yes, AttackKind::Melee, &[status]);
        assert_eq!(total.get(Statistic::ArmorClass), -6);
        assert_ne!(total.get(Statistic::ArmorClass), -8);
        let status_applied: Vec<_> = total
            .applied()
            .iter()
            .filter(|modifier| modifier.penalty_type() == Some(PenaltyType::Status))
            .collect();
        assert_eq!(status_applied.len(), 1);
        assert_eq!(status_applied[0].source(), Some(SourceId(11)));
        assert_eq!(status_applied[0].value(), -4);
        assert_held(&dice);
    }

    /// Player Core, Penalties.
    #[test]
    fn an_armor_class_overflow_does_not_wrap() {
        let dice = held_die();
        let extra = Modifier::penalty(
            PenaltyType::Untyped,
            i32::MIN,
            SourceId(12),
            12,
            None,
            Statistic::ArmorClass,
        )
        .expect("minimum penalty");
        assert_eq!(
            armor_class(None, None, OffGuard::Yes, AttackKind::Melee, &[extra]),
            Err(PositioningError::Modifier(ModifierError::Overflow))
        );
        assert_held(&dice);
    }

    /// Player Core, "Raise a Shield". Player Core, Cover.
    #[test]
    fn greater_cover_does_not_add_a_shield() {
        let dice = held_die();
        let value = ac_value(
            None,
            Some(Cover::Greater),
            OffGuard::No,
            AttackKind::Melee,
            &[],
        );
        assert_eq!(value, 4);
        assert_ne!(value, 6);
        assert_held(&dice);
    }

    /// Player Core, Cover.
    #[test]
    fn cover_is_an_armor_class_bonus_only() {
        let dice = held_die();
        let total = ac(
            None,
            Some(Cover::Greater),
            OffGuard::No,
            AttackKind::Ranged,
            &[],
        );
        assert_eq!(total.get(Statistic::ArmorClass), 4);
        assert_eq!(total.applied().len(), 1);
        assert_eq!(circumstance_bonuses(&total), vec![4]);
        assert_held(&dice);
    }

    /// Player Core, "Take Cover".
    #[test]
    fn the_same_take_cover_twice_matches() {
        let dice = held_die();
        let first = take_cover(
            Posture::Standing,
            at(3, 4),
            None,
            Some(Cover::Standard),
            NearbyFeature::Absent,
        )
        .expect("take cover");
        let second = take_cover(
            Posture::Standing,
            at(3, 4),
            None,
            Some(Cover::Standard),
            NearbyFeature::Absent,
        )
        .expect("take cover");
        assert_eq!(first, second);
        assert_eq!(first.benefit, CoverBenefit::Greater);
        assert_held(&dice);
    }

    /// Player Core, "Take Cover". Player Core, Cover.
    #[test]
    fn standard_cover_becomes_greater() {
        let dice = held_die();
        let covered = take_cover(
            Posture::Standing,
            at(3, 4),
            None,
            Some(Cover::Standard),
            NearbyFeature::Absent,
        )
        .expect("take cover");
        assert_eq!(covered.benefit, CoverBenefit::Greater);
        let total = ac(
            Some(covered.benefit),
            Some(Cover::Standard),
            OffGuard::No,
            AttackKind::Melee,
            &[],
        );
        assert_eq!(total.get(Statistic::ArmorClass), 4);
        assert_eq!(circumstance_bonuses(&total), vec![4]);
        assert_held(&dice);
    }

    /// Player Core, "Take Cover".
    #[test]
    fn lesser_cover_alone_is_not_enough() {
        let dice = held_die();
        let state = actions_left(2);
        let stored = None;
        assert_eq!(
            take_cover(
                Posture::Standing,
                at(3, 4),
                stored,
                Some(Cover::Lesser),
                NearbyFeature::Absent,
            ),
            Err(PositioningError::RequirementNotMet)
        );
        assert_eq!(stored, None);
        assert_eq!(
            ac_value(
                stored,
                Some(Cover::Lesser),
                OffGuard::No,
                AttackKind::Melee,
                &[],
            ),
            1
        );
        assert_eq!(state.actions_remaining, 2);
        assert_held(&dice);
    }

    /// Player Core, "Take Cover".
    #[test]
    fn lesser_cover_and_a_feature_become_standard() {
        let dice = held_die();
        let covered = take_cover(
            Posture::Standing,
            at(3, 4),
            None,
            Some(Cover::Lesser),
            NearbyFeature::Present,
        )
        .expect("take cover");
        assert_eq!(covered.benefit, CoverBenefit::Standard);
        assert_eq!(
            ac_value(
                Some(covered.benefit),
                Some(Cover::Lesser),
                OffGuard::No,
                AttackKind::Melee,
                &[],
            ),
            2
        );
        assert_held(&dice);
    }

    /// Player Core, "Take Cover".
    #[test]
    fn a_nearby_feature_grants_standard_cover() {
        let dice = held_die();
        let covered = take_cover(
            Posture::Standing,
            at(3, 4),
            None,
            None,
            NearbyFeature::Present,
        )
        .expect("take cover");
        assert_eq!(covered.benefit, CoverBenefit::Standard);
        assert_eq!(
            ac_value(
                Some(covered.benefit),
                None,
                OffGuard::No,
                AttackKind::Melee,
                &[]
            ),
            2
        );
        assert_held(&dice);
    }

    /// Player Core, "Take Cover".
    #[test]
    fn no_cover_and_not_prone() {
        let dice = held_die();
        let stored = None;
        assert_eq!(
            take_cover(
                Posture::Standing,
                at(3, 4),
                stored,
                None,
                NearbyFeature::Absent,
            ),
            Err(PositioningError::RequirementNotMet)
        );
        assert_eq!(stored, None);
        assert_held(&dice);
    }

    /// Player Core, "Take Cover". Player Core, Cover.
    #[test]
    fn greater_cover_is_not_reduced() {
        let dice = held_die();
        let covered = take_cover(
            Posture::Standing,
            at(3, 4),
            None,
            Some(Cover::Greater),
            NearbyFeature::Absent,
        )
        .expect("take cover");
        assert_eq!(covered.benefit, CoverBenefit::Greater);
        assert_eq!(
            ac_value(
                Some(covered.benefit),
                Some(Cover::Greater),
                OffGuard::No,
                AttackKind::Melee,
                &[],
            ),
            4
        );
        assert_held(&dice);
    }

    /// Player Core, Cover. A creature two sizes larger is not a cover argument.
    #[test]
    fn a_larger_neighbour_is_not_cover() {
        let dice = held_die();
        let neighbour_size = Size::Large;
        let neighbour_square = at(1, 0);
        assert_eq!(
            take_cover(
                Posture::Standing,
                at(0, 0),
                None,
                None,
                NearbyFeature::Absent,
            ),
            Err(PositioningError::RequirementNotMet)
        );
        assert_eq!(neighbour_size, Size::Large);
        assert_eq!(neighbour_square, at(1, 0));
        assert_held(&dice);
    }

    /// Player Core, "Take Cover". Player Core, Prone.
    #[test]
    fn prone_with_standard_cover_totals_plus_2() {
        let dice = held_die();
        let covered = take_cover(
            Posture::Prone,
            at(3, 4),
            None,
            Some(Cover::Standard),
            NearbyFeature::Absent,
        )
        .expect("take cover");
        assert_eq!(covered.benefit, CoverBenefit::Greater);
        assert_eq!(
            ac_value(
                Some(covered.benefit),
                Some(Cover::Standard),
                OffGuard::Yes,
                AttackKind::Melee,
                &[],
            ),
            2
        );
        assert_eq!(
            ac_value(
                Some(covered.benefit),
                Some(Cover::Standard),
                OffGuard::Yes,
                AttackKind::Ranged,
                &[],
            ),
            2
        );
        assert_held(&dice);
    }

    /// Player Core, "Take Cover". Player Core, Prone.
    #[test]
    fn prone_with_greater_cover_totals_plus_2() {
        let dice = held_die();
        let covered = take_cover(
            Posture::Prone,
            at(3, 4),
            None,
            Some(Cover::Greater),
            NearbyFeature::Absent,
        )
        .expect("take cover");
        assert_eq!(covered.benefit, CoverBenefit::Greater);
        assert_eq!(
            ac_value(
                Some(covered.benefit),
                Some(Cover::Greater),
                OffGuard::Yes,
                AttackKind::Melee,
                &[],
            ),
            2
        );
        assert_eq!(
            ac_value(
                Some(covered.benefit),
                Some(Cover::Greater),
                OffGuard::Yes,
                AttackKind::Ranged,
                &[],
            ),
            2
        );
        assert_held(&dice);
    }

    /// Player Core, "Take Cover". Player Core, Prone.
    #[test]
    fn prone_with_a_feature_and_no_cover_totals_0_and_plus_2() {
        let dice = held_die();
        let covered = take_cover(Posture::Prone, at(3, 4), None, None, NearbyFeature::Present)
            .expect("take cover");
        assert_eq!(covered.benefit, CoverBenefit::StandardPlusGreaterVsRanged);
        assert_eq!(
            ac_value(
                Some(covered.benefit),
                None,
                OffGuard::Yes,
                AttackKind::Melee,
                &[],
            ),
            0
        );
        assert_eq!(
            ac_value(
                Some(covered.benefit),
                None,
                OffGuard::Yes,
                AttackKind::Ranged,
                &[],
            ),
            2
        );
        assert_held(&dice);
    }

    /// Player Core, "Take Cover". Player Core, Prone.
    #[test]
    fn prone_with_lesser_cover_totals_minus_1_and_plus_2() {
        let dice = held_die();
        let covered = take_cover(
            Posture::Prone,
            at(3, 4),
            None,
            Some(Cover::Lesser),
            NearbyFeature::Absent,
        )
        .expect("take cover");
        assert_eq!(covered.benefit, CoverBenefit::GreaterVsRanged);
        assert_eq!(
            ac_value(
                Some(covered.benefit),
                Some(Cover::Lesser),
                OffGuard::Yes,
                AttackKind::Melee,
                &[],
            ),
            -1
        );
        assert_eq!(
            ac_value(
                Some(covered.benefit),
                Some(Cover::Lesser),
                OffGuard::Yes,
                AttackKind::Ranged,
                &[],
            ),
            2
        );
        assert_held(&dice);
    }

    /// Player Core, "Take Cover". Player Core, Prone.
    #[test]
    fn prone_with_lesser_cover_and_a_feature_totals_0_and_plus_2() {
        let dice = held_die();
        let covered = take_cover(
            Posture::Prone,
            at(3, 4),
            None,
            Some(Cover::Lesser),
            NearbyFeature::Present,
        )
        .expect("take cover");
        assert_eq!(covered.benefit, CoverBenefit::StandardPlusGreaterVsRanged);
        assert_eq!(
            ac_value(
                Some(covered.benefit),
                Some(Cover::Lesser),
                OffGuard::Yes,
                AttackKind::Melee,
                &[],
            ),
            0
        );
        assert_eq!(
            ac_value(
                Some(covered.benefit),
                Some(Cover::Lesser),
                OffGuard::Yes,
                AttackKind::Ranged,
                &[],
            ),
            2
        );
        assert_held(&dice);
    }

    /// Player Core, "Take Cover". Player Core, Prone.
    #[test]
    fn prone_with_no_cover_totals_minus_2_and_plus_2() {
        let dice = held_die();
        let covered = take_cover(Posture::Prone, at(3, 4), None, None, NearbyFeature::Absent)
            .expect("take cover");
        assert_eq!(covered.benefit, CoverBenefit::GreaterVsRanged);
        assert_eq!(
            ac_value(
                Some(covered.benefit),
                None,
                OffGuard::Yes,
                AttackKind::Melee,
                &[],
            ),
            -2
        );
        assert_eq!(
            ac_value(
                Some(covered.benefit),
                None,
                OffGuard::Yes,
                AttackKind::Ranged,
                &[],
            ),
            2
        );
        assert_held(&dice);
    }

    /// Player Core, "Take Cover".
    #[test]
    fn stored_greater_survives_a_later_standard() {
        let dice = held_die();
        let covered = take_cover(
            Posture::Standing,
            at(3, 4),
            Some(CoverBenefit::Greater),
            Some(Cover::Standard),
            NearbyFeature::Absent,
        )
        .expect("take cover");
        assert_eq!(covered.benefit, CoverBenefit::Greater);
        assert_eq!(
            ac_value(
                Some(covered.benefit),
                Some(Cover::Standard),
                OffGuard::No,
                AttackKind::Melee,
                &[],
            ),
            4
        );
        let kept = take_cover(
            Posture::Standing,
            at(3, 4),
            Some(CoverBenefit::StandardPlusGreaterVsRanged),
            None,
            NearbyFeature::Present,
        )
        .expect("take cover");
        assert_eq!(
            armor_class(
                Some(kept.benefit),
                None,
                OffGuard::No,
                AttackKind::Ranged,
                &[]
            )
            .expect("armor class")
            .get(Statistic::ArmorClass),
            4
        );
        assert_held(&dice);
    }

    /// Player Core, "Take Cover".
    #[test]
    fn a_failed_requirement_keeps_the_stored_benefit() {
        let dice = held_die();
        let stored = Some(CoverBenefit::Greater);
        assert_eq!(
            take_cover(
                Posture::Standing,
                at(3, 4),
                stored,
                Some(Cover::Lesser),
                NearbyFeature::Absent,
            ),
            Err(PositioningError::RequirementNotMet)
        );
        assert_eq!(
            armor_class(
                stored,
                Some(Cover::Lesser),
                OffGuard::No,
                AttackKind::Melee,
                &[],
            )
            .expect("armor class")
            .get(Statistic::ArmorClass),
            4
        );
        assert_held(&dice);
    }

    /// Player Core, "Take Cover".
    #[test]
    fn dismissing_nothing_stores_nothing() {
        let dice = held_die();
        let first = dismiss(None);
        let second = dismiss(None);
        assert_eq!(first, Ok(None));
        assert_eq!(first, second);
        assert_held(&dice);
    }

    /// Player Core, "Take Cover". Player Core, "Immobilized".
    #[test]
    fn take_cover_is_one_action_without_the_move_trait() {
        let dice = held_die();
        let mobility = Mobility::Immobilized;
        assert!(move_trait_allowed(matches!(mobility, Mobility::Immobilized)).is_err());
        let covered = take_cover(
            Posture::Standing,
            at(3, 4),
            None,
            Some(Cover::Standard),
            NearbyFeature::Absent,
        )
        .expect("take cover");
        assert_eq!(
            covered.cost,
            ActionCost::Actions {
                count: 1,
                traits: &[],
            }
        );
        assert_eq!(covered.benefit, CoverBenefit::Greater);
        assert_held(&dice);
    }

    /// Player Core, "Take Cover".
    #[test]
    fn dismiss_is_free() {
        let dice = held_die();
        assert_eq!(dismiss_cost(), ActionCost::Free { traits: &[] });
        assert_eq!(dismiss(Some(CoverBenefit::Greater)).expect("dismiss"), None);
        assert_held(&dice);
    }

    /// Player Core, "Take Cover". Player Core, Prone.
    #[test]
    fn leaving_the_square_clears_the_benefit() {
        let dice = held_die();
        let posture = Posture::Prone;
        let first = end_benefit(EndReason::LeftSquare, Some(CoverBenefit::Standard));
        let second = end_benefit(EndReason::LeftSquare, Some(CoverBenefit::Standard));
        assert_eq!(first, second);
        assert_eq!(first, None);
        assert_eq!(posture, Posture::Prone);
        assert_eq!(
            ac_value(
                first,
                Some(Cover::Standard),
                OffGuard::Yes,
                AttackKind::Melee,
                &[],
            ),
            0
        );
        assert_eq!(
            ac_value(
                first,
                Some(Cover::Standard),
                OffGuard::Yes,
                AttackKind::Ranged,
                &[],
            ),
            0
        );
        assert_held(&dice);
    }

    /// Player Core, "Take Cover". Player Core, Prone.
    #[test]
    fn an_attack_action_clears_the_benefit() {
        let dice = held_die();
        let posture = Posture::Prone;
        assert_eq!(
            end_benefit(EndReason::AttackAction, Some(CoverBenefit::Greater)),
            None
        );
        assert_eq!(posture, Posture::Prone);
        assert_held(&dice);
    }

    /// Player Core, "Take Cover". Player Core, Prone. Player Core, "Off-Guard".
    #[test]
    fn unconscious_clears_the_benefit_only() {
        let dice = held_die();
        let posture = Posture::Prone;
        let benefit = end_benefit(EndReason::Unconscious, Some(CoverBenefit::Greater));
        assert_eq!(benefit, None);
        assert_eq!(posture, Posture::Prone);
        assert_eq!(
            ac_value(benefit, None, OffGuard::Yes, AttackKind::Melee, &[]),
            -2
        );
        assert_held(&dice);
    }

    /// Player Core, "Take Cover". Player Core, Prone.
    #[test]
    fn forced_movement_clears_the_benefit() {
        let dice = held_die();
        let posture = Posture::Prone;
        assert_eq!(
            end_benefit(EndReason::ForcedMovement, Some(CoverBenefit::Standard)),
            None
        );
        assert_eq!(posture, Posture::Prone);
        assert_held(&dice);
    }

    /// Player Core, "Take Cover". Player Core, Prone.
    #[test]
    fn stand_turns_standard_plus_greater_against_ranged_into_standard() {
        let dice = held_die();
        let changed = stand(
            Posture::Prone,
            at(3, 4),
            Mobility::Free,
            Some(CoverBenefit::StandardPlusGreaterVsRanged),
            OccupantPresence::Absent,
        )
        .expect("stand");
        assert_eq!(changed.posture, Posture::Standing);
        assert_eq!(changed.benefit, Some(CoverBenefit::Standard));
        assert_eq!(
            ac_value(changed.benefit, None, OffGuard::No, AttackKind::Melee, &[],),
            2
        );
        assert_eq!(
            ac_value(changed.benefit, None, OffGuard::No, AttackKind::Ranged, &[],),
            2
        );
        assert_held(&dice);
    }

    /// Player Core, "Take Cover". Player Core, Prone.
    #[test]
    fn stand_ends_a_greater_against_ranged_benefit() {
        let dice = held_die();
        let changed = stand(
            Posture::Prone,
            at(3, 4),
            Mobility::Free,
            Some(CoverBenefit::GreaterVsRanged),
            OccupantPresence::Absent,
        )
        .expect("stand");
        assert_eq!(changed.posture, Posture::Standing);
        assert_eq!(changed.benefit, None);
        assert_eq!(
            ac_value(changed.benefit, None, OffGuard::No, AttackKind::Melee, &[]),
            0
        );
        assert_eq!(
            ac_value(changed.benefit, None, OffGuard::No, AttackKind::Ranged, &[]),
            0
        );
        assert_held(&dice);
    }

    /// Player Core, "Take Cover". Player Core, Prone.
    #[test]
    fn stand_keeps_greater_cover() {
        let dice = held_die();
        let changed = stand(
            Posture::Prone,
            at(3, 4),
            Mobility::Free,
            Some(CoverBenefit::Greater),
            OccupantPresence::Absent,
        )
        .expect("stand");
        assert_eq!(changed.posture, Posture::Standing);
        assert_eq!(changed.benefit, Some(CoverBenefit::Greater));
        assert_eq!(
            ac_value(changed.benefit, None, OffGuard::No, AttackKind::Melee, &[],),
            4
        );
        assert_eq!(
            ac_value(changed.benefit, None, OffGuard::No, AttackKind::Ranged, &[],),
            4
        );
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn the_same_flanking_query_twice_matches() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(2, 0, 1, Side::Pc);
        let partner = placed(3, 0, -1, Side::Pc);
        let first = flanks(&defender, &attacker, &[partner]).expect("flank");
        let second = flanks(&defender, &attacker, &[partner]).expect("flank");
        assert_eq!(first, second);
        assert!(first);
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn the_same_off_guard_query_twice_matches() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(2, 0, 1, Side::Pc);
        let partner = placed(3, 0, -1, Side::Pc);
        let first = off_guard(
            &defender,
            &attacker,
            AttackKind::Melee,
            &[partner],
            OffGuard::No,
        )
        .expect("off-guard");
        let second = off_guard(
            &defender,
            &attacker,
            AttackKind::Melee,
            &[partner],
            OffGuard::No,
        )
        .expect("off-guard");
        assert_eq!(first, second);
        assert!(first);
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn opposite_sides_at_reach_5() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(2, 0, 1, Side::Pc);
        let partner = placed(3, 0, -1, Side::Pc);
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(true));
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn opposite_corners_at_reach_5() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(2, 1, 1, Side::Pc);
        let partner = placed(3, -1, -1, Side::Pc);
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(true));
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn the_same_side_does_not_flank() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(2, -1, 1, Side::Pc);
        let partner = placed(3, 0, 1, Side::Pc);
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(false));
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn one_corner_does_not_flank() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(2, 0, 1, Side::Pc);
        let partner = placed(3, 1, 0, Side::Pc);
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(false));
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn a_corner_counts_as_both_edges() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let mut attacker = placed(2, -2, -1, Side::Pc);
        attacker.reach_feet = 10;
        let partner = placed(3, 1, 0, Side::Pc);
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(true));
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn adjacent_side_interiors_do_not_flank() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let mut attacker = placed(2, -2, -2, Side::Pc);
        attacker.reach_feet = 10;
        let partner = placed(3, 0, 1, Side::Pc);
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(false));
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn reach_10_opposite_sides() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let mut attacker = placed(2, 0, 2, Side::Pc);
        attacker.reach_feet = 10;
        let partner = placed(3, 0, -1, Side::Pc);
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(true));
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn reach_10_opposite_corners() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let mut attacker = placed(2, 2, 2, Side::Pc);
        attacker.reach_feet = 10;
        let mut partner = placed(3, -2, -2, Side::Pc);
        partner.reach_feet = 10;
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(true));
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn reach_10_off_axis_opposite_sides() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let mut attacker = placed(2, -2, -1, Side::Pc);
        attacker.reach_feet = 10;
        let mut partner = placed(3, 2, 1, Side::Pc);
        partner.reach_feet = 10;
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(true));
        assert_held(&dice);
    }

    /// Player Core, "Grid Movement". Player Core, "Flanking".
    #[test]
    fn negative_squares_are_legal() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(2, 1, 0, Side::Pc);
        let partner = placed(3, -1, 0, Side::Pc);
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(true));
        assert_held(&dice);
    }

    /// Player Core, "Grid Movement". Player Core, "Flanking".
    #[test]
    fn extreme_coordinates_overflow() {
        let dice = held_die();
        let defender = placed(1, i32::MAX - 1, 0, Side::Enemy);
        let attacker = placed(2, i32::MAX, 0, Side::Pc);
        let partner = placed(3, i32::MAX - 2, 0, Side::Pc);
        assert_eq!(
            flanks(&defender, &attacker, &[partner]),
            Err(PositioningError::Overflow)
        );
        let mut odd_attacker = placed(2, i32::MAX, 0, Side::Pc);
        odd_attacker.reach_feet = 7;
        let mut odd_partner = placed(3, i32::MAX - 2, 0, Side::Pc);
        odd_partner.reach_feet = 7;
        assert_eq!(
            flanks(&defender, &odd_attacker, &[odd_partner]),
            Err(PositioningError::Overflow)
        );
        assert_held(&dice);
    }

    /// Player Core, "Range and Reach". Player Core, "Flanking".
    #[test]
    fn reach_at_the_maximum_u32_does_not_wrap() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let mut attacker = placed(2, 0, 1, Side::Pc);
        attacker.reach_feet = u32::MAX;
        let mut partner = placed(3, 0, -1, Side::Pc);
        partner.reach_feet = u32::MAX;
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(true));
        assert_held(&dice);
    }

    /// Player Core, "Range and Reach".
    #[test]
    fn out_of_reach_does_not_flank() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(2, 0, 1, Side::Pc);
        let partner = placed(3, 0, -2, Side::Pc);
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(false));
        let far_attacker = placed(4, 0, -2, Side::Pc);
        let near_partner = placed(3, 0, 1, Side::Pc);
        assert_eq!(flanks(&defender, &far_attacker, &[near_partner]), Ok(false));
        assert_held(&dice);
    }

    /// Player Core, "Range and Reach".
    #[test]
    fn a_reach_that_is_not_a_multiple_of_5_is_rejected() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let mut attacker = placed(2, 0, 1, Side::Pc);
        attacker.reach_feet = 7;
        let mut partner = placed(3, 0, -1, Side::Pc);
        partner.reach_feet = 7;
        assert_eq!(
            flanks(&defender, &attacker, &[partner]),
            Err(PositioningError::InvalidReach)
        );
        assert_held(&dice);
    }

    /// Player Core, "Range and Reach".
    #[test]
    fn reach_0_does_not_threaten() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let mut attacker = placed(2, 0, 1, Side::Pc);
        attacker.reach_feet = 0;
        let mut partner = placed(3, 0, -1, Side::Pc);
        partner.reach_feet = 0;
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(false));
        assert_held(&dice);
    }

    /// Player Core, "Range and Reach". Player Core, "Flanking".
    #[test]
    fn reach_15_opposite_corners_flank() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let mut attacker = placed(2, 2, 2, Side::Pc);
        attacker.reach_feet = 15;
        let mut partner = placed(3, -2, -2, Side::Pc);
        partner.reach_feet = 15;
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(true));
        assert_held(&dice);
    }

    /// Player Core, "Range and Reach". Player Core, "Flanking".
    #[test]
    fn reach_15_far_corners_do_not_flank() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let mut attacker = placed(2, 3, 3, Side::Pc);
        attacker.reach_feet = 15;
        let mut partner = placed(3, -3, -3, Side::Pc);
        partner.reach_feet = 15;
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(false));
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn the_target_cannot_flank_itself() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(1, 0, 1, Side::Enemy);
        let partner = placed(2, 0, -1, Side::Pc);
        assert_eq!(
            flanks(&defender, &attacker, &[partner]),
            Err(PositioningError::SelfId)
        );
        let other = placed(2, 0, 1, Side::Pc);
        let same_as_defender = placed(1, 0, -1, Side::Pc);
        assert_eq!(
            flanks(&defender, &other, &[same_as_defender]),
            Err(PositioningError::SelfId)
        );
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn duplicate_id_wins_over_self_id() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Pc);
        let attacker = placed(1, 0, 1, Side::Pc);
        let partner = placed(1, 0, -1, Side::Pc);
        assert_eq!(
            flanks(&defender, &attacker, &[partner]),
            Err(PositioningError::DuplicateId)
        );
        assert_held(&dice);
    }

    /// Player Core, "Flanking". Player Core, "Size, Space, and Reach".
    #[test]
    fn self_id_wins_over_unsupported_size() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Pc);
        let mut attacker = placed(1, 0, 1, Side::Pc);
        attacker.size = Size::Large;
        let partner = placed(2, 0, -1, Side::Pc);
        assert_eq!(
            flanks(&defender, &attacker, &[partner]),
            Err(PositioningError::SelfId)
        );
        assert_held(&dice);
    }

    /// Player Core, "Size, Space, and Reach". Player Core, "Flanking".
    #[test]
    fn unsupported_size_wins_over_overflow() {
        let dice = held_die();
        let mut defender = placed(1, i32::MAX - 1, 0, Side::Enemy);
        defender.size = Size::Large;
        let attacker = placed(2, i32::MAX, 0, Side::Pc);
        let partner = placed(3, i32::MAX - 2, 0, Side::Pc);
        assert_eq!(
            flanks(&defender, &attacker, &[partner]),
            Err(PositioningError::UnsupportedSize)
        );
        let mut large_partner = placed(3, 0, -1, Side::Pc);
        large_partner.size = Size::Large;
        assert_eq!(
            flanks(
                &placed(1, 0, 0, Side::Enemy),
                &placed(2, 1_000_000_000, 0, Side::Pc),
                &[large_partner],
            ),
            Err(PositioningError::UnsupportedSize)
        );
        assert_held(&dice);
    }

    /// Player Core, "Size, Space, and Reach".
    #[test]
    fn large_is_unsupported() {
        let dice = held_die();
        let mut defender = placed(1, 0, 0, Side::Enemy);
        defender.size = Size::Large;
        let attacker = placed(2, 0, 1, Side::Pc);
        let partner = placed(3, 0, -1, Side::Pc);
        assert_eq!(
            flanks(&defender, &attacker, &[partner]),
            Err(PositioningError::UnsupportedSize)
        );
        assert_held(&dice);
    }

    /// Player Core, "Size, Space, and Reach".
    #[test]
    fn a_tiny_flanker_is_unsupported() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let mut attacker = placed(2, 0, 1, Side::Pc);
        attacker.size = Size::Tiny;
        attacker.reach_feet = 0;
        let partner = placed(3, 0, -1, Side::Pc);
        assert_eq!(
            flanks(&defender, &attacker, &[partner]),
            Err(PositioningError::UnsupportedSize)
        );
        assert_held(&dice);
    }

    /// Player Core, "Size, Space, and Reach".
    #[test]
    fn a_large_unable_flanker_is_unsupported() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let mut attacker = placed(2, 0, 1, Side::Pc);
        attacker.size = Size::Large;
        attacker.readiness = Readiness::Unable;
        let partner = placed(3, 0, -1, Side::Pc);
        assert_eq!(
            flanks(&defender, &attacker, &[partner]),
            Err(PositioningError::UnsupportedSize)
        );
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn different_sides_do_not_flank() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(2, 0, 1, Side::Pc);
        let partner = placed(3, 0, -1, Side::Enemy);
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(false));
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn same_side_as_the_target_does_not_flank() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(2, 0, 1, Side::Enemy);
        let partner = placed(3, 0, -1, Side::Enemy);
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(false));
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn unable_to_act_does_not_flank() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let mut attacker = placed(2, 0, 1, Side::Pc);
        attacker.readiness = Readiness::Unable;
        let partner = placed(3, 0, -1, Side::Pc);
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(false));
        let able = placed(2, 0, 1, Side::Pc);
        let mut unable_partner = placed(3, 0, -1, Side::Pc);
        unable_partner.readiness = Readiness::Unable;
        assert_eq!(flanks(&defender, &able, &[unable_partner]), Ok(false));
        assert_held(&dice);
    }

    /// Player Core, "Flanking". Player Core, Prone.
    #[test]
    fn prone_does_not_clear_able_to_act() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let mut attacker = placed(2, 0, 1, Side::Pc);
        attacker.posture = Posture::Prone;
        let mut partner = placed(3, 0, -1, Side::Pc);
        partner.posture = Posture::Prone;
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(true));
        assert_held(&dice);
    }

    /// Player Core, "Flanking". Player Core, Prone.
    #[test]
    fn a_prone_target_can_be_flanked() {
        let dice = held_die();
        let mut defender = placed(1, 0, 0, Side::Enemy);
        defender.posture = Posture::Prone;
        let attacker = placed(2, 0, 1, Side::Pc);
        let partner = placed(3, 0, -1, Side::Pc);
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(true));
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn ally_order_does_not_matter() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(2, 0, 1, Side::Pc);
        let opposite = placed(3, 0, -1, Side::Pc);
        let beside = placed(4, 1, 1, Side::Pc);
        assert_eq!(flanks(&defender, &attacker, &[opposite, beside]), Ok(true));
        assert_eq!(flanks(&defender, &attacker, &[beside, opposite]), Ok(true));
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn an_attacker_missing_from_partners_does_not_flank() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(2, 0, 1, Side::Pc);
        assert_eq!(flanks(&defender, &attacker, &[]), Ok(false));
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn two_flankers_on_one_square_do_not_flank() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(2, 0, 1, Side::Pc);
        let partner = placed(3, 0, 1, Side::Pc);
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(false));
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn a_flanker_on_the_target_square_does_not_flank() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(2, 0, 1, Side::Pc);
        let partner = placed(3, 0, 0, Side::Pc);
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(false));
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn terrain_and_creatures_do_not_block_flanking() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let mut attacker = placed(2, 0, 2, Side::Pc);
        attacker.reach_feet = 10;
        let partner = placed(3, 0, -1, Side::Pc);
        assert_eq!(flanks(&defender, &attacker, &[partner]), Ok(true));
        assert_held(&dice);
    }

    /// Player Core, "Flanking". Player Core, "Off-Guard".
    #[test]
    fn melee_from_a_flanker_is_off_guard() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(2, 0, 1, Side::Pc);
        let partner = placed(3, 0, -1, Side::Pc);
        assert_eq!(
            off_guard(
                &defender,
                &attacker,
                AttackKind::Melee,
                &[partner],
                OffGuard::No
            ),
            Ok(true)
        );
        assert_held(&dice);
    }

    /// Player Core, "Flanking". Player Core, "Off-Guard".
    #[test]
    fn ranged_from_a_flanker_is_not_off_guard() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(2, 0, 1, Side::Pc);
        let partner = placed(3, 0, -1, Side::Pc);
        assert_eq!(
            off_guard(
                &defender,
                &attacker,
                AttackKind::Ranged,
                &[partner],
                OffGuard::No
            ),
            Ok(false)
        );
        assert_held(&dice);
    }

    /// Player Core, "Flanking". Player Core, "Off-Guard".
    #[test]
    fn one_flanking_partner_is_enough() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(2, 0, 1, Side::Pc);
        let opposite = placed(3, 0, -1, Side::Pc);
        let beside = placed(4, 1, 1, Side::Pc);
        assert_eq!(
            off_guard(
                &defender,
                &attacker,
                AttackKind::Melee,
                &[opposite, beside],
                OffGuard::No
            ),
            Ok(true)
        );
        assert_held(&dice);
    }

    /// Player Core, "Off-Guard". Player Core, Prone. Player Core, "Size, Space, and Reach".
    #[test]
    fn prone_off_guard_does_not_wait_on_partner_geometry() {
        let dice = held_die();
        let mut defender = placed(1, 0, 0, Side::Enemy);
        defender.posture = Posture::Prone;
        let attacker = placed(3, 0, -1, Side::Pc);
        let mut partner = placed(2, 0, 1, Side::Pc);
        partner.size = Size::Large;
        assert_eq!(
            off_guard(
                &defender,
                &attacker,
                AttackKind::Melee,
                &[partner],
                OffGuard::No
            ),
            Ok(true)
        );
        assert_eq!(
            off_guard(
                &defender,
                &attacker,
                AttackKind::Ranged,
                &[partner],
                OffGuard::No
            ),
            Ok(true)
        );
        assert_held(&dice);
    }

    /// Player Core, Prone. Player Core, "Off-Guard".
    #[test]
    fn prone_off_guard_applies_to_melee_and_ranged() {
        let dice = held_die();
        let mut defender = placed(1, 0, 0, Side::Enemy);
        defender.posture = Posture::Prone;
        let attacker = placed(2, 0, 1, Side::Pc);
        assert_eq!(
            off_guard(&defender, &attacker, AttackKind::Melee, &[], OffGuard::No),
            Ok(true)
        );
        assert_eq!(
            off_guard(&defender, &attacker, AttackKind::Ranged, &[], OffGuard::No),
            Ok(true)
        );
        assert_held(&dice);
    }

    /// Player Core, "Off-Guard".
    #[test]
    fn a_standing_defender_with_no_flank_is_not_off_guard() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(2, 0, 1, Side::Pc);
        assert_eq!(
            off_guard(&defender, &attacker, AttackKind::Melee, &[], OffGuard::No),
            Ok(false)
        );
        assert_eq!(
            off_guard(&defender, &attacker, AttackKind::Melee, &[], OffGuard::Yes),
            Ok(true)
        );
        assert_held(&dice);
    }

    /// Player Core, "Off-Guard". Player Core, Penalties.
    #[test]
    fn grabbed_and_flanked_are_off_guard_once() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(2, 0, 1, Side::Pc);
        let partner = placed(3, 0, -1, Side::Pc);
        let queried = off_guard(
            &defender,
            &attacker,
            AttackKind::Melee,
            &[partner],
            OffGuard::Yes,
        )
        .expect("off-guard");
        assert!(queried);
        let cause = if queried { OffGuard::Yes } else { OffGuard::No };
        let total = armor_class(None, None, cause, AttackKind::Melee, &[]).expect("armor class");
        assert_eq!(total.get(Statistic::ArmorClass), -2);
        assert_held(&dice);
    }

    /// Player Core, "Off-Guard". Player Core, Prone. Player Core, "Flanking".
    #[test]
    fn prone_and_flanking_are_off_guard_once() {
        let dice = held_die();
        let mut defender = placed(1, 0, 0, Side::Enemy);
        defender.posture = Posture::Prone;
        let attacker = placed(2, 0, 1, Side::Pc);
        let partner = placed(3, 0, -1, Side::Pc);
        let queried = off_guard(
            &defender,
            &attacker,
            AttackKind::Melee,
            &[partner],
            OffGuard::No,
        )
        .expect("off-guard");
        assert!(queried);
        let cause = if queried { OffGuard::Yes } else { OffGuard::No };
        let total = armor_class(None, None, cause, AttackKind::Melee, &[]).expect("armor class");
        assert_eq!(total.get(Statistic::ArmorClass), -2);
        assert_held(&dice);
    }

    /// Player Core, "Flanking". Player Core, "Off-Guard".
    #[test]
    fn a_bystander_is_not_a_flanker() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(2, 1, 0, Side::Pc);
        let north = placed(3, 0, 1, Side::Pc);
        let south = placed(4, 0, -1, Side::Pc);
        assert_eq!(
            off_guard(
                &defender,
                &attacker,
                AttackKind::Melee,
                &[north, south],
                OffGuard::No
            ),
            Ok(false)
        );
        assert_held(&dice);
    }

    /// Player Core, "Flanking".
    #[test]
    fn a_duplicate_id_is_rejected() {
        let dice = held_die();
        let defender = placed(1, 0, 0, Side::Enemy);
        let attacker = placed(2, 0, 1, Side::Pc);
        let partner = placed(2, 0, -1, Side::Pc);
        assert_eq!(
            flanks(&defender, &attacker, &[partner]),
            Err(PositioningError::DuplicateId)
        );
        assert_eq!(
            off_guard(
                &defender,
                &attacker,
                AttackKind::Melee,
                &[partner],
                OffGuard::No
            ),
            Err(PositioningError::DuplicateId)
        );
        let separate = placed(4, 0, 1, Side::Pc);
        let first = placed(3, 0, -1, Side::Pc);
        let second = placed(3, 1, 0, Side::Pc);
        assert_eq!(
            flanks(&defender, &separate, &[first, second]),
            Err(PositioningError::DuplicateId)
        );
        assert_eq!(
            off_guard(
                &defender,
                &separate,
                AttackKind::Melee,
                &[first, second],
                OffGuard::No,
            ),
            Err(PositioningError::DuplicateId)
        );
        assert_held(&dice);
    }
}
