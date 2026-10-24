//! Circumstance, item, status, and untyped modifiers.
//!
//! Callers supply sequence numbers. Newer means a greater sequence, not a clock.
//!
//! Player Core, Bonuses and Penalties.

#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "callers are this crate's tests; the facade keeps the procedure and drops the root export"
    )
)]

/// A bonus type. Proficiency is not a modifier: those numbers are already baked (DEV-01).
///
/// Player Core, Bonuses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BonusType {
    /// A circumstance bonus. Player Core, Bonuses.
    Circumstance,
    /// An item bonus. Player Core, Bonuses.
    Item,
    /// A status bonus. Player Core, Bonuses.
    Status,
}

/// A penalty type. Untyped penalties stack. An untyped bonus is not a penalty.
///
/// Player Core, Penalties.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PenaltyType {
    /// A circumstance penalty. Player Core, Penalties.
    Circumstance,
    /// An item penalty. Player Core, Penalties.
    Item,
    /// A status penalty. Player Core, Penalties.
    Status,
    /// An untyped penalty. These all add. Player Core, Penalties.
    Untyped,
}

/// Why a modifier could not be built or totaled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModifierError {
    /// The value was a positive untyped number. Penalties can be untyped. Bonuses cannot.
    ///
    /// Player Core, Penalties.
    UntypedBonus,
    /// The value was 0. A bonus is greater than 0 and a penalty is less than 0.
    ///
    /// Player Core, Penalties.
    Zero,
    /// A bonus was negative, or a typed penalty was positive.
    ///
    /// Player Core, Penalties.
    Sign,
    /// A sum did not fit in `i32`. The total is not saturated.
    ///
    /// Player Core, Penalties.
    Overflow,
    /// Frightened or sickened had no rank, a declared type other than status, or was a bonus.
    /// Unconscious had a rank, a type other than status, a value other than −4, or was a bonus.
    ///
    /// Player Core, Frightened. Player Core, Sickened. Player Core pg. 446, Unconscious.
    /// Player Core, "Redundant Conditions With Values".
    InvalidValuedCondition,
}

/// Identity of one effect. Equal ids are the same effect.
///
/// Player Core, Duplicate Effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceId(pub u32);

/// Identity of one condition. Equal ids are the same condition.
///
/// Player Core, "Redundant Conditions".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ConditionId(pub u32);

impl ConditionId {
    /// Frightened. A status penalty with this id and a rank applies as `-rank`.
    /// No rank, or a type other than status, is rejected. A status bonus with a rank
    /// keeps the type and value the caller declared.
    ///
    /// Player Core, Frightened. Player Core, "Redundant Conditions With Values".
    pub const FRIGHTENED: Self = Self(2);

    /// Sickened. A status penalty with this id and a rank applies as `-rank`.
    /// No rank, or a type other than status, is rejected. A status bonus with a rank
    /// keeps the type and value the caller declared.
    ///
    /// Player Core, Sickened. Player Core, "Redundant Conditions With Values".
    pub const SICKENED: Self = Self(3);

    /// Unconscious. A status penalty of −4 with no rank. A rank, another type, another
    /// value, or a bonus is rejected. This is not a valued condition.
    ///
    /// Player Core pg. 446, Unconscious.
    pub const UNCONSCIOUS: Self = Self(4);
}

/// The statistic a modifier applies to. Two statistics do not share a stack.
///
/// Player Core, Duplicate Effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Statistic {
    /// An attack roll. Player Core, Attack Rolls.
    Attack,
    /// Armor Class. Player Core, Armor Class.
    ArmorClass,
    /// Perception. Player Core, Perception.
    Perception,
    /// A damage roll. Player Core, "Damage Rolls".
    Damage,
    /// A Reflex save. It does not share a stack with Armor Class or Perception.
    ///
    /// Player Core pg. 446, Unconscious.
    Reflex,
    /// A Fortitude save. It does not share a stack with Reflex or Will.
    ///
    /// Player Core pg. 404, Saving Throws.
    Fortitude,
    /// A Will save. It does not share a stack with Reflex or Fortitude.
    ///
    /// Player Core pg. 404, Saving Throws.
    Will,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Condition {
    id: ConditionId,
    rank: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModifierKind {
    Bonus(BonusType),
    Penalty(PenaltyType),
}

/// Where a modifier came from. Caller ids and reserved rules do not compare equal.
///
/// A caller builds a [`SourceId`]. The multiple attack penalty, the range penalty, the
/// attack check's circumstance penalty, and the other producers in this module use a
/// reserved variant, so no `SourceId` a caller can construct names them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ModifierSource {
    Caller(SourceId),
    MultipleAttackPenalty,
    Prone,
    Aid,
    Cover,
    OffGuard,
    UnconsciousArmorClass,
    UnconsciousPerception,
    UnconsciousReflex,
    /// A raised shield's circumstance bonus to Armor Class. Player Core, "Raise a Shield".
    RaisedShield,
    /// Player Core, "Range Penalty".
    RangePenalty,
    /// The attack check total's circumstance penalty. Player Core, Penalties.
    AttackCircumstance,
}

impl ModifierSource {
    fn caller_id(self) -> Option<SourceId> {
        match self {
            Self::Caller(id) => Some(id),
            Self::MultipleAttackPenalty
            | Self::Prone
            | Self::Aid
            | Self::Cover
            | Self::OffGuard
            | Self::UnconsciousArmorClass
            | Self::UnconsciousPerception
            | Self::UnconsciousReflex
            | Self::RaisedShield
            | Self::RangePenalty
            | Self::AttackCircumstance => None,
        }
    }
}

/// One bonus or penalty. The value is greater than 0 for a bonus and less than 0 for a penalty.
///
/// Player Core, Bonuses and Penalties.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Modifier {
    source: ModifierSource,
    sequence: u32,
    level: Option<u32>,
    condition: Option<Condition>,
    statistic: Statistic,
    kind: ModifierKind,
    value: i32,
}

impl Modifier {
    /// Builds a bonus. The value must be greater than 0.
    ///
    /// `level` is the effect's level, when it has one. `sequence` is caller-supplied;
    /// a greater number is newer. `statistic` is required. Attack is not implied.
    ///
    /// Player Core, Bonuses. Player Core, Duplicate Effects.
    pub fn bonus(
        ty: BonusType,
        value: i32,
        source: SourceId,
        sequence: u32,
        level: Option<u32>,
        statistic: Statistic,
    ) -> Result<Self, ModifierError> {
        Self::bonus_from(
            ty,
            value,
            ModifierSource::Caller(source),
            sequence,
            level,
            statistic,
        )
    }

    fn bonus_from(
        ty: BonusType,
        value: i32,
        source: ModifierSource,
        sequence: u32,
        level: Option<u32>,
        statistic: Statistic,
    ) -> Result<Self, ModifierError> {
        if value == 0 {
            return Err(ModifierError::Zero);
        }
        if value < 0 {
            return Err(ModifierError::Sign);
        }
        Ok(Self {
            source,
            sequence,
            level,
            condition: None,
            statistic,
            kind: ModifierKind::Bonus(ty),
            value,
        })
    }

    /// Builds a penalty. The value must be less than 0.
    ///
    /// A positive untyped value is [`ModifierError::UntypedBonus`]. Any other positive
    /// value is [`ModifierError::Sign`]. `statistic` is required. Attack is not implied.
    ///
    /// Player Core, Penalties. Player Core, Duplicate Effects.
    pub fn penalty(
        ty: PenaltyType,
        value: i32,
        source: SourceId,
        sequence: u32,
        level: Option<u32>,
        statistic: Statistic,
    ) -> Result<Self, ModifierError> {
        Self::penalty_from(
            ty,
            value,
            ModifierSource::Caller(source),
            sequence,
            level,
            statistic,
        )
    }

    fn penalty_from(
        ty: PenaltyType,
        value: i32,
        source: ModifierSource,
        sequence: u32,
        level: Option<u32>,
        statistic: Statistic,
    ) -> Result<Self, ModifierError> {
        if value == 0 {
            return Err(ModifierError::Zero);
        }
        if value > 0 {
            return Err(if ty == PenaltyType::Untyped {
                ModifierError::UntypedBonus
            } else {
                ModifierError::Sign
            });
        }
        Ok(Self {
            source,
            sequence,
            level,
            condition: None,
            statistic,
            kind: ModifierKind::Penalty(ty),
            value,
        })
    }

    /// Marks this modifier as coming from a condition.
    ///
    /// `rank` is the condition's own value. `None` is a condition with no value, such as off-guard.
    /// Rank is not the algebraic value of the penalty. Frightened or sickened with no rank, a
    /// declared type other than status, or a bonus at any rank including 0, returns
    /// [`ModifierError::InvalidValuedCondition`] and is not rewritten into a penalty.
    /// Unconscious with a rank, a type other than status, a value other than −4, or a bonus
    /// returns the same error. It is not rewritten from its rank.
    ///
    /// Player Core, "Redundant Conditions With Values" and "Condition Values".
    /// Player Core, Frightened. Player Core, Sickened.
    pub fn with_condition(
        mut self,
        id: ConditionId,
        rank: Option<u32>,
    ) -> Result<Self, ModifierError> {
        let valued = id == ConditionId::FRIGHTENED || id == ConditionId::SICKENED;
        let bonus = matches!(self.kind, ModifierKind::Bonus(_));
        if valued && (rank.is_none() || !is_status_kind(self.kind) || bonus) {
            return Err(ModifierError::InvalidValuedCondition);
        }
        if id == ConditionId::UNCONSCIOUS {
            let status_penalty = matches!(self.kind, ModifierKind::Penalty(PenaltyType::Status));
            if rank.is_some() || !status_penalty || self.value != -4 {
                return Err(ModifierError::InvalidValuedCondition);
            }
        }
        self.condition = Some(Condition { id, rank });
        Ok(self)
    }

    /// The signed value. A bonus is greater than 0. A penalty is less than 0.
    pub fn value(self) -> i32 {
        self.value
    }

    /// Caller-supplied sequence. A greater number is newer.
    ///
    /// Player Core, Duplicate Effects.
    pub fn sequence(self) -> u32 {
        self.sequence
    }

    /// The effect's level, when it has one.
    ///
    /// Player Core, Duplicate Effects.
    pub fn level(self) -> Option<u32> {
        self.level
    }

    /// The caller-supplied effect id, when this modifier has one.
    ///
    /// Reserved producers, including the multiple attack penalty, have no caller id.
    ///
    /// Player Core, Duplicate Effects. Player Core, "Multiple Attack Penalty".
    pub fn source(self) -> Option<SourceId> {
        self.source.caller_id()
    }

    /// The statistic this modifier applies to.
    ///
    /// Player Core, Duplicate Effects.
    pub fn statistic(self) -> Statistic {
        self.statistic
    }

    /// The condition this modifier came from, when it has one.
    ///
    /// Player Core, "Redundant Conditions".
    pub fn condition_id(self) -> Option<ConditionId> {
        self.condition.map(|condition| condition.id)
    }

    /// The bonus type, or `None` when this is a penalty.
    pub fn bonus_type(self) -> Option<BonusType> {
        match self.kind {
            ModifierKind::Bonus(ty) => Some(ty),
            ModifierKind::Penalty(_) => None,
        }
    }

    /// The penalty type, or `None` when this is a bonus.
    pub fn penalty_type(self) -> Option<PenaltyType> {
        match self.kind {
            ModifierKind::Penalty(ty) => Some(ty),
            ModifierKind::Bonus(_) => None,
        }
    }

    /// The condition rank, when this modifier came from a valued condition.
    ///
    /// Player Core, "Condition Values".
    pub fn rank(self) -> Option<u32> {
        self.condition.and_then(|condition| condition.rank)
    }
}

/// The modifiers that apply after duplicate effects, conditions, and same-type stacking.
///
/// Each statistic has its own total. Player Core, Bonuses, Penalties, and Duplicate Effects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ModifierTotal {
    totals: StatisticTotals,
    applied: Vec<Modifier>,
}

impl ModifierTotal {
    /// The total for `statistic`. Modifiers on another statistic are not included, and the
    /// result is not clamped.
    ///
    /// Player Core, Duplicate Effects. Player Core, "Damage Rolls".
    pub fn get(&self, statistic: Statistic) -> i32 {
        self.totals.get(statistic)
    }

    /// The modifiers that were applied. A tie keeps the lowest sequence, then the lower
    /// condition id, and a missing condition id loses, then the lower source id.
    pub fn applied(&self) -> &[Modifier] {
        &self.applied
    }
}

/// Splits bonuses from penalties, reduces repeated conditions per statistic, collapses
/// the duplicate key, then stacks what remains on each statistic.
///
/// The key is (source id, statistic, type, bonus or penalty) and equal condition ids.
/// A missing condition id matches only another missing id, so different conditions stay
/// apart. Frightened and sickened penalties are status and contribute the negation of
/// their rank.
/// For each statistic, and for each of circumstance, item, and status, the highest bonus
/// and the worst penalty both apply. When those tie, the lowest sequence is reported,
/// then the lower condition id, and a missing condition id loses, then the lower source
/// id. Two statistics do not share that choice. Every untyped penalty is added with
/// checked addition. Overflow is [`ModifierError::Overflow`] and is not a saturated `i32`.
///
/// Player Core, Bonuses, Penalties, and Duplicate Effects.
pub(crate) fn total(modifiers: &[Modifier]) -> Result<ModifierTotal, ModifierError> {
    let mut bonuses = Vec::new();
    let mut penalties = Vec::new();
    for modifier in modifiers.iter().copied() {
        match modifier.kind {
            ModifierKind::Bonus(_) => bonuses.push(modifier),
            ModifierKind::Penalty(_) => penalties.push(modifier),
        }
    }
    let mut reduced = collapse_conditions(&bonuses)?;
    reduced.extend(collapse_conditions(&penalties)?);
    let keyed = collapse_keys(&reduced);
    finish(stack_types(&keyed))
}

/// DC equal to 10 plus `total`. The result is not clamped. The add is checked.
///
/// Player Core, "Getting a DC from a Modifier".
pub(crate) fn dc_from_total(total: i32) -> Result<i32, ModifierError> {
    10i32.checked_add(total).ok_or(ModifierError::Overflow)
}

fn collapse_conditions(modifiers: &[Modifier]) -> Result<Vec<Modifier>, ModifierError> {
    let mut plain = Vec::new();
    // The key uses the type after frightened and sickened penalties become status, so a
    // circumstance penalty and a status penalty from one condition do not share a group.
    let mut grouped: Vec<(ConditionId, Statistic, ModifierKind, Vec<Modifier>)> = Vec::new();
    for modifier in modifiers.iter().copied() {
        if valued_rank_is_zero(modifier) {
            continue;
        }
        match modifier.condition {
            None => plain.push(modifier),
            Some(condition) => {
                let kind = group_kind(modifier);
                match grouped.iter_mut().find(|(id, statistic, group_kind, _)| {
                    *id == condition.id && *statistic == modifier.statistic && *group_kind == kind
                }) {
                    Some((_, _, _, group)) => group.push(modifier),
                    None => grouped.push((condition.id, modifier.statistic, kind, vec![modifier])),
                }
            }
        }
    }
    for (_, _, _, group) in grouped {
        if let Some(modifier) = keep_condition(&group)? {
            plain.push(modifier);
        }
    }
    Ok(plain)
}

/// The worst penalty or the highest bonus wins. Frightened and sickened penalties compare
/// `-rank`, the condition value, before the rewrite. Any other condition compares the
/// modifier the caller wrote. A tie then keeps the highest rank, a rank over none, the
/// newest sequence, and the lowest source id. Equal source ids then keep the higher level,
/// a level over none, and the duplicate-key comparison decides what remains.
///
/// A frightened or sickened penalty then contributes one status penalty equal to the
/// negation of the kept rank. Construction rejects a bonus with those ids, so this
/// function does not rewrite one into a penalty. Any other ranked condition keeps its
/// declared type and value.
fn keep_condition(group: &[Modifier]) -> Result<Option<Modifier>, ModifierError> {
    let Some(mut best) = group.first().copied() else {
        return Ok(None);
    };
    for candidate in group.iter().copied().skip(1) {
        if condition_prefers(candidate, best)? {
            best = candidate;
        }
    }
    apply_ranked_penalty(best)
}

fn condition_prefers(candidate: Modifier, current: Modifier) -> Result<bool, ModifierError> {
    let candidate_value = grouped_value(candidate)?;
    let current_value = grouped_value(current)?;
    if candidate_value != current_value {
        return Ok(match candidate.kind {
            ModifierKind::Bonus(_) => candidate_value > current_value,
            ModifierKind::Penalty(_) => candidate_value < current_value,
        });
    }
    match (candidate.rank(), current.rank()) {
        (Some(candidate_rank), Some(current_rank)) if candidate_rank != current_rank => {
            return Ok(candidate_rank > current_rank);
        }
        (Some(_), None) => return Ok(true),
        (None, Some(_)) => return Ok(false),
        _ => {}
    }
    if candidate.sequence != current.sequence {
        return Ok(candidate.sequence > current.sequence);
    }
    let candidate_source = source_ord(candidate.source);
    let current_source = source_ord(current.source);
    if candidate_source != current_source {
        return Ok(candidate_source < current_source);
    }
    // Equal source ids. The higher level is an engineering tiebreak. What remains is the
    // duplicate-key comparison, which still sees the caller's value.
    match (candidate.level, current.level) {
        (Some(candidate_level), Some(current_level)) if candidate_level != current_level => {
            return Ok(candidate_level > current_level);
        }
        (Some(_), None) => return Ok(true),
        (None, Some(_)) => return Ok(false),
        _ => {}
    }
    Ok(key_prefers(candidate, current))
}

fn valued_rank_is_zero(modifier: Modifier) -> bool {
    derived_rank(modifier) == Some(0)
}

fn group_kind(modifier: Modifier) -> ModifierKind {
    if derived_rank(modifier).is_some() {
        ModifierKind::Penalty(PenaltyType::Status)
    } else {
        modifier.kind
    }
}

/// Frightened and sickened penalties are derived from the condition value, so the
/// comparison uses `-rank` even when the caller wrote a different penalty.
fn grouped_value(modifier: Modifier) -> Result<i32, ModifierError> {
    let Some(rank) = derived_rank(modifier) else {
        return Ok(modifier.value);
    };
    let rank = i32::try_from(rank).map_err(|_| ModifierError::Overflow)?;
    rank.checked_neg().ok_or(ModifierError::Overflow)
}

fn is_status_kind(kind: ModifierKind) -> bool {
    matches!(
        kind,
        ModifierKind::Bonus(BonusType::Status) | ModifierKind::Penalty(PenaltyType::Status)
    )
}

fn derived_rank(modifier: Modifier) -> Option<u32> {
    modifier.penalty_type()?;
    let condition = modifier.condition?;
    if condition.id == ConditionId::FRIGHTENED || condition.id == ConditionId::SICKENED {
        condition.rank
    } else {
        None
    }
}

fn apply_ranked_penalty(mut modifier: Modifier) -> Result<Option<Modifier>, ModifierError> {
    let Some(rank) = modifier.rank() else {
        return Ok(Some(modifier));
    };
    // Only a frightened or sickened penalty becomes a status penalty of -rank.
    // Construction rejects a bonus tagged with those ids, so it is not rewritten here.
    // Any other ranked condition, including rank 0, keeps the type and value the caller declared.
    let valued = modifier.condition.is_some_and(|condition| {
        condition.id == ConditionId::FRIGHTENED || condition.id == ConditionId::SICKENED
    });
    if !valued || modifier.penalty_type().is_none() {
        return Ok(Some(modifier));
    }
    // Rank 0 has ended, so frightened or sickened contributes nothing.
    if rank == 0 {
        return Ok(None);
    }
    let rank = i32::try_from(rank).map_err(|_| ModifierError::Overflow)?;
    modifier.value = rank.checked_neg().ok_or(ModifierError::Overflow)?;
    modifier.kind = ModifierKind::Penalty(PenaltyType::Status);
    Ok(Some(modifier))
}

/// One duplicate key contributes once.
///
/// The key is (source id, statistic, type, bonus or penalty) plus the condition id,
/// using the type after frightened and sickened penalties are rewritten. Unequal
/// condition ids do not share a key. A greater sequence still wins when its value is
/// lower. Rules 2266.
fn collapse_keys(modifiers: &[Modifier]) -> Vec<Modifier> {
    let mut kept: Vec<Modifier> = Vec::new();
    for modifier in modifiers.iter().copied() {
        match kept
            .iter_mut()
            .find(|existing| same_duplicate_key(**existing, modifier))
        {
            Some(existing) => {
                if key_prefers(modifier, *existing) {
                    *existing = modifier;
                }
            }
            None => kept.push(modifier),
        }
    }
    kept
}

fn same_duplicate_key(left: Modifier, right: Modifier) -> bool {
    left.source == right.source
        && left.statistic == right.statistic
        && left.kind == right.kind
        && left.condition_id() == right.condition_id()
}

/// Higher level, then a level over none, then higher rank, then a rank over none, then the
/// greater sequence. Ruling R-01 then keeps the higher bonus or the more negative penalty.
/// Duplicate Effects has no value step; Rules 2282 does, and this matches it. A lower
/// condition id breaks a remaining tie, and a missing condition id loses. The lower source
/// id is last. Both of those steps are engineering tiebreaks.
fn key_prefers(candidate: Modifier, current: Modifier) -> bool {
    match (candidate.level, current.level) {
        (Some(candidate_level), Some(current_level)) if candidate_level != current_level => {
            return candidate_level > current_level;
        }
        (Some(_), None) => return true,
        (None, Some(_)) => return false,
        _ => {}
    }
    match (candidate.rank(), current.rank()) {
        (Some(candidate_rank), Some(current_rank)) if candidate_rank != current_rank => {
            return candidate_rank > current_rank;
        }
        (Some(_), None) => return true,
        (None, Some(_)) => return false,
        _ => {}
    }
    if candidate.sequence != current.sequence {
        return candidate.sequence > current.sequence;
    }
    if candidate.value != current.value {
        return match candidate.kind {
            ModifierKind::Bonus(_) => candidate.value > current.value,
            ModifierKind::Penalty(_) => candidate.value < current.value,
        };
    }
    if let Some(preferred) = lower_condition_id(candidate, current) {
        return preferred;
    }
    lower_source(candidate, current)
}

/// `Some` when the condition ids differ. The lower id wins, and a missing id loses.
fn lower_condition_id(candidate: Modifier, current: Modifier) -> Option<bool> {
    match (candidate.condition_id(), current.condition_id()) {
        (Some(candidate_id), Some(current_id)) if candidate_id != current_id => {
            Some(candidate_id < current_id)
        }
        (Some(_), None) => Some(true),
        (None, Some(_)) => Some(false),
        _ => None,
    }
}

fn lower_source(candidate: Modifier, current: Modifier) -> bool {
    source_ord(candidate.source) < source_ord(current.source)
}

struct Buckets {
    circumstance_bonus: Option<Modifier>,
    item_bonus: Option<Modifier>,
    status_bonus: Option<Modifier>,
    circumstance_penalty: Option<Modifier>,
    item_penalty: Option<Modifier>,
    status_penalty: Option<Modifier>,
    untyped: Vec<Modifier>,
}

impl Buckets {
    fn new() -> Self {
        Self {
            circumstance_bonus: None,
            item_bonus: None,
            status_bonus: None,
            circumstance_penalty: None,
            item_penalty: None,
            status_penalty: None,
            untyped: Vec::new(),
        }
    }
}

struct ByStatistic {
    attack: Buckets,
    armor_class: Buckets,
    perception: Buckets,
    damage: Buckets,
    reflex: Buckets,
    fortitude: Buckets,
    will: Buckets,
}

fn stack_types(modifiers: &[Modifier]) -> Vec<Modifier> {
    let mut grouped = ByStatistic {
        attack: Buckets::new(),
        armor_class: Buckets::new(),
        perception: Buckets::new(),
        damage: Buckets::new(),
        reflex: Buckets::new(),
        fortitude: Buckets::new(),
        will: Buckets::new(),
    };
    for modifier in modifiers.iter().copied() {
        push_bucket(buckets_for(&mut grouped, modifier.statistic), modifier);
    }
    let mut applied = Vec::new();
    for buckets in [
        grouped.attack,
        grouped.armor_class,
        grouped.perception,
        grouped.damage,
        grouped.reflex,
        grouped.fortitude,
        grouped.will,
    ] {
        applied.extend(applied_from(buckets));
    }
    applied
}

fn buckets_for(grouped: &mut ByStatistic, statistic: Statistic) -> &mut Buckets {
    match statistic {
        Statistic::Attack => &mut grouped.attack,
        Statistic::ArmorClass => &mut grouped.armor_class,
        Statistic::Perception => &mut grouped.perception,
        Statistic::Damage => &mut grouped.damage,
        Statistic::Reflex => &mut grouped.reflex,
        Statistic::Fortitude => &mut grouped.fortitude,
        Statistic::Will => &mut grouped.will,
    }
}

fn push_bucket(buckets: &mut Buckets, modifier: Modifier) {
    match modifier.kind {
        ModifierKind::Bonus(BonusType::Circumstance) => {
            prefer_bonus(&mut buckets.circumstance_bonus, modifier);
        }
        ModifierKind::Bonus(BonusType::Item) => prefer_bonus(&mut buckets.item_bonus, modifier),
        ModifierKind::Bonus(BonusType::Status) => prefer_bonus(&mut buckets.status_bonus, modifier),
        ModifierKind::Penalty(PenaltyType::Circumstance) => {
            prefer_penalty(&mut buckets.circumstance_penalty, modifier);
        }
        ModifierKind::Penalty(PenaltyType::Item) => {
            prefer_penalty(&mut buckets.item_penalty, modifier);
        }
        ModifierKind::Penalty(PenaltyType::Status) => {
            prefer_penalty(&mut buckets.status_penalty, modifier);
        }
        ModifierKind::Penalty(PenaltyType::Untyped) => buckets.untyped.push(modifier),
    }
}

fn applied_from(buckets: Buckets) -> Vec<Modifier> {
    let mut applied = Vec::new();
    for modifier in [
        buckets.circumstance_bonus,
        buckets.item_bonus,
        buckets.status_bonus,
        buckets.circumstance_penalty,
        buckets.item_penalty,
        buckets.status_penalty,
    ]
    .into_iter()
    .flatten()
    {
        applied.push(modifier);
    }
    applied.extend(buckets.untyped);
    applied
}

fn prefer_bonus(slot: &mut Option<Modifier>, candidate: Modifier) {
    match *slot {
        Some(current) if !bonus_prefers(candidate, current) => {}
        _ => *slot = Some(candidate),
    }
}

fn prefer_penalty(slot: &mut Option<Modifier>, candidate: Modifier) {
    match *slot {
        Some(current) if !penalty_prefers(candidate, current) => {}
        _ => *slot = Some(candidate),
    }
}

/// Highest value wins. A tie reports the lowest sequence, then the lower condition id,
/// and a missing condition id loses, then the lower source id.
fn bonus_prefers(candidate: Modifier, current: Modifier) -> bool {
    if candidate.value != current.value {
        candidate.value > current.value
    } else {
        stack_tie(candidate, current)
    }
}

/// Lowest value wins. A tie reports the lowest sequence, then the lower condition id,
/// and a missing condition id loses, then the lower source id.
fn penalty_prefers(candidate: Modifier, current: Modifier) -> bool {
    if candidate.value != current.value {
        candidate.value < current.value
    } else {
        stack_tie(candidate, current)
    }
}

fn stack_tie(candidate: Modifier, current: Modifier) -> bool {
    if candidate.sequence != current.sequence {
        return candidate.sequence < current.sequence;
    }
    if let Some(preferred) = lower_condition_id(candidate, current) {
        return preferred;
    }
    lower_source(candidate, current)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StatisticTotals {
    attack: i32,
    armor_class: i32,
    perception: i32,
    damage: i32,
    reflex: i32,
    fortitude: i32,
    will: i32,
}

impl StatisticTotals {
    fn get(self, statistic: Statistic) -> i32 {
        match statistic {
            Statistic::Attack => self.attack,
            Statistic::ArmorClass => self.armor_class,
            Statistic::Perception => self.perception,
            Statistic::Damage => self.damage,
            Statistic::Reflex => self.reflex,
            Statistic::Fortitude => self.fortitude,
            Statistic::Will => self.will,
        }
    }
}

struct StatisticWide {
    attack: i64,
    armor_class: i64,
    perception: i64,
    damage: i64,
    reflex: i64,
    fortitude: i64,
    will: i64,
}

fn finish(mut applied: Vec<Modifier>) -> Result<ModifierTotal, ModifierError> {
    // Canonical order so insertion order cannot change the reported list.
    // Each statistic is summed in i64 because bonuses sort before penalties: an i32 add of
    // circumstance i32::MAX and item +1 overflows before a later status -5
    // brings that statistic's total back into range. Another statistic is not in the sum.
    applied.sort_by_key(sort_key);
    let mut wide = StatisticWide {
        attack: 0,
        armor_class: 0,
        perception: 0,
        damage: 0,
        reflex: 0,
        fortitude: 0,
        will: 0,
    };
    for modifier in &applied {
        add_statistic(&mut wide, modifier.statistic, modifier.value)?;
    }
    let totals = StatisticTotals {
        attack: i32::try_from(wide.attack).map_err(|_| ModifierError::Overflow)?,
        armor_class: i32::try_from(wide.armor_class).map_err(|_| ModifierError::Overflow)?,
        perception: i32::try_from(wide.perception).map_err(|_| ModifierError::Overflow)?,
        damage: i32::try_from(wide.damage).map_err(|_| ModifierError::Overflow)?,
        reflex: i32::try_from(wide.reflex).map_err(|_| ModifierError::Overflow)?,
        fortitude: i32::try_from(wide.fortitude).map_err(|_| ModifierError::Overflow)?,
        will: i32::try_from(wide.will).map_err(|_| ModifierError::Overflow)?,
    };
    Ok(ModifierTotal { totals, applied })
}

fn add_statistic(
    wide: &mut StatisticWide,
    statistic: Statistic,
    value: i32,
) -> Result<(), ModifierError> {
    let slot = match statistic {
        Statistic::Attack => &mut wide.attack,
        Statistic::ArmorClass => &mut wide.armor_class,
        Statistic::Perception => &mut wide.perception,
        Statistic::Damage => &mut wide.damage,
        Statistic::Reflex => &mut wide.reflex,
        Statistic::Fortitude => &mut wide.fortitude,
        Statistic::Will => &mut wide.will,
    };
    *slot = slot
        .checked_add(i64::from(value))
        .ok_or(ModifierError::Overflow)?;
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ReportOrder {
    kind: u8,
    value: i32,
    sequence: u32,
    // A missing level or condition id sorts after a present one. Level stays after
    // the value so the reported order does not put level ahead of the bonus or penalty.
    level: (u8, u32),
    rank: (u8, u32),
    condition: (u8, u32),
    source_kind: u8,
    source_id: u32,
    statistic: u8,
}

fn sort_key(modifier: &Modifier) -> ReportOrder {
    let (source_kind, source_id) = source_ord(modifier.source);
    ReportOrder {
        kind: kind_ord(modifier.kind),
        value: modifier.value,
        sequence: modifier.sequence,
        level: presence_ord(modifier.level),
        rank: presence_ord(modifier.rank()),
        condition: condition_sort(modifier.condition_id()),
        source_kind,
        source_id,
        statistic: statistic_ord(modifier.statistic),
    }
}

fn presence_ord(value: Option<u32>) -> (u8, u32) {
    match value {
        Some(value) => (0, value),
        None => (1, 0),
    }
}

fn condition_sort(id: Option<ConditionId>) -> (u8, u32) {
    match id {
        Some(id) => (0, id.0),
        None => (1, 0),
    }
}

fn source_ord(source: ModifierSource) -> (u8, u32) {
    match source {
        ModifierSource::Caller(id) => (0, id.0),
        ModifierSource::MultipleAttackPenalty => (1, 0),
        ModifierSource::Prone => (2, 0),
        ModifierSource::Aid => (3, 0),
        ModifierSource::Cover => (4, 0),
        ModifierSource::OffGuard => (5, 0),
        ModifierSource::UnconsciousArmorClass => (6, 0),
        ModifierSource::UnconsciousPerception => (7, 0),
        ModifierSource::RangePenalty => (8, 0),
        ModifierSource::AttackCircumstance => (9, 0),
        ModifierSource::UnconsciousReflex => (10, 0),
        ModifierSource::RaisedShield => (11, 0),
    }
}

fn statistic_ord(statistic: Statistic) -> u8 {
    match statistic {
        Statistic::Attack => 0,
        Statistic::ArmorClass => 1,
        Statistic::Perception => 2,
        Statistic::Damage => 3,
        Statistic::Reflex => 4,
        Statistic::Fortitude => 5,
        Statistic::Will => 6,
    }
}

fn kind_ord(kind: ModifierKind) -> u8 {
    match kind {
        ModifierKind::Bonus(BonusType::Circumstance) => 0,
        ModifierKind::Bonus(BonusType::Item) => 1,
        ModifierKind::Bonus(BonusType::Status) => 2,
        ModifierKind::Penalty(PenaltyType::Circumstance) => 3,
        ModifierKind::Penalty(PenaltyType::Item) => 4,
        ModifierKind::Penalty(PenaltyType::Status) => 5,
        ModifierKind::Penalty(PenaltyType::Untyped) => 6,
    }
}

const OFF_GUARD: ConditionId = ConditionId(1);

/// Cover granted to Armor Class.
///
/// Player Core, Cover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cover {
    /// Lesser cover, +1 circumstance. Player Core, Cover.
    Lesser,
    /// Standard cover, +2 circumstance. Player Core, Cover.
    Standard,
    /// Greater cover, +4 circumstance. Player Core, Cover.
    Greater,
}

impl Cover {
    fn bonus(self) -> i32 {
        match self {
            Cover::Lesser => 1,
            Cover::Standard => 2,
            Cover::Greater => 4,
        }
    }
}

/// Modifiers on one attack roll.
///
/// `multiple_attack_penalty` is the caller-supplied untyped penalty. Phase 1 passes the
/// result of `multiple_attack_penalty`. The argument is the whole penalty, so a later
/// change can pass the value frozen when an attack was readied. This function does not
/// implement the Ready action or a reaction attack, and it does not contain a penalty table.
/// Zero is omitted. Prone is a circumstance −2. An Aid result of 0 is omitted; any other
/// result is one circumstance modifier. This does not prepare, roll, or resolve Aid.
///
/// Player Core, "Multiple Attack Penalty", Prone, and Aid.
pub(crate) fn attack_modifiers(
    multiple_attack_penalty: i32,
    prone: bool,
    aid_circumstance: i32,
) -> Result<Vec<Modifier>, ModifierError> {
    let mut modifiers = Vec::new();
    push_untyped(
        &mut modifiers,
        multiple_attack_penalty,
        ModifierSource::MultipleAttackPenalty,
        1,
    )?;
    if prone {
        modifiers.push(Modifier::penalty_from(
            PenaltyType::Circumstance,
            -2,
            ModifierSource::Prone,
            2,
            None,
            Statistic::Attack,
        )?);
    }
    push_circumstance(&mut modifiers, aid_circumstance, ModifierSource::Aid, 3)?;
    Ok(modifiers)
}

/// Modifiers on Armor Class from this change. The shield bonus is not an argument.
///
/// Player Core, Cover, Off-Guard, and Unconscious.
pub(crate) fn ac_modifiers(
    cover: Option<Cover>,
    off_guard: bool,
    unconscious: bool,
) -> Result<Vec<Modifier>, ModifierError> {
    let mut modifiers = Vec::new();
    if let Some(cover) = cover {
        modifiers.push(Modifier::bonus_from(
            BonusType::Circumstance,
            cover.bonus(),
            ModifierSource::Cover,
            4,
            None,
            Statistic::ArmorClass,
        )?);
    }
    if off_guard {
        modifiers.push(
            Modifier::penalty_from(
                PenaltyType::Circumstance,
                -2,
                ModifierSource::OffGuard,
                5,
                None,
                Statistic::ArmorClass,
            )?
            .with_condition(OFF_GUARD, None)?,
        );
    }
    if unconscious {
        modifiers.push(
            Modifier::penalty_from(
                PenaltyType::Status,
                -4,
                ModifierSource::UnconsciousArmorClass,
                6,
                None,
                Statistic::ArmorClass,
            )?
            .with_condition(ConditionId::UNCONSCIOUS, None)?,
        );
    }
    Ok(modifiers)
}

/// Circumstance bonus from a raised shield. Callers cannot build this source.
///
/// Sequence 11, sort key `(11, 0)`, Armor Class, no condition. A bonus that is not
/// positive is [`ModifierError`].
///
/// Player Core, "Raise a Shield". Player Core, Bonuses.
pub(crate) fn raised_shield_modifier(bonus: i32) -> Result<Modifier, ModifierError> {
    Modifier::bonus_from(
        BonusType::Circumstance,
        bonus,
        ModifierSource::RaisedShield,
        11,
        None,
        Statistic::ArmorClass,
    )
}

/// Modifiers on Perception from this change. Unconscious is a status −4.
///
/// The slice is separate from Armor Class. Player Core, Unconscious.
pub(crate) fn perception_modifiers(unconscious: bool) -> Result<Vec<Modifier>, ModifierError> {
    let mut modifiers = Vec::new();
    if unconscious {
        modifiers.push(
            Modifier::penalty_from(
                PenaltyType::Status,
                -4,
                ModifierSource::UnconsciousPerception,
                7,
                None,
                Statistic::Perception,
            )?
            .with_condition(ConditionId::UNCONSCIOUS, None)?,
        );
    }
    Ok(modifiers)
}

/// Modifiers on Reflex from this change. Unconscious is a status −4.
///
/// The slice is separate from Armor Class and Perception. Player Core pg. 446, Unconscious.
pub(crate) fn reflex_modifiers(unconscious: bool) -> Result<Vec<Modifier>, ModifierError> {
    let mut modifiers = Vec::new();
    if unconscious {
        modifiers.push(
            Modifier::penalty_from(
                PenaltyType::Status,
                -4,
                ModifierSource::UnconsciousReflex,
                10,
                None,
                Statistic::Reflex,
            )?
            .with_condition(ConditionId::UNCONSCIOUS, None)?,
        );
    }
    Ok(modifiers)
}

fn push_untyped(
    modifiers: &mut Vec<Modifier>,
    value: i32,
    source: ModifierSource,
    sequence: u32,
) -> Result<(), ModifierError> {
    if value == 0 {
        return Ok(());
    }
    if value > 0 {
        return Err(ModifierError::UntypedBonus);
    }
    modifiers.push(Modifier::penalty_from(
        PenaltyType::Untyped,
        value,
        source,
        sequence,
        None,
        Statistic::Attack,
    )?);
    Ok(())
}

/// Internal terms of an attack check total.
///
/// The multiple attack penalty reuses [`ModifierSource::MultipleAttackPenalty`] and is
/// applied once. The range penalty and the circumstance penalty use reserved sources
/// that no caller can construct. A positive untyped term is a raw addend. Zero is omitted.
/// A reserved source does not skip same-type stacking.
///
/// Player Core, "Multiple Attack Penalty". Player Core, "Range Penalty". Player Core, Penalties.
pub(crate) struct AttackCheckParts {
    pub(crate) modifiers: Vec<Modifier>,
    pub(crate) raw_extra: i64,
}

enum AttackCheckTerm {
    MultipleAttackPenalty,
    Circumstance,
    RangePenalty,
}

pub(crate) fn attack_check_parts(
    map: i32,
    circumstance: i32,
    range_penalty: i32,
) -> AttackCheckParts {
    let mut parts = AttackCheckParts {
        modifiers: Vec::new(),
        raw_extra: 0,
    };
    push_attack_check_term(&mut parts, map, AttackCheckTerm::MultipleAttackPenalty, 1);
    push_attack_check_term(&mut parts, circumstance, AttackCheckTerm::Circumstance, 2);
    push_attack_check_term(&mut parts, range_penalty, AttackCheckTerm::RangePenalty, 3);
    parts
}

fn push_attack_check_term(
    parts: &mut AttackCheckParts,
    value: i32,
    term: AttackCheckTerm,
    sequence: u32,
) {
    if value == 0 {
        return;
    }
    if matches!(
        term,
        AttackCheckTerm::MultipleAttackPenalty | AttackCheckTerm::RangePenalty
    ) && value > 0
    {
        parts.raw_extra += i64::from(value);
        return;
    }
    let built = match term {
        AttackCheckTerm::MultipleAttackPenalty => Modifier::penalty_from(
            PenaltyType::Untyped,
            value,
            ModifierSource::MultipleAttackPenalty,
            sequence,
            None,
            Statistic::Attack,
        ),
        AttackCheckTerm::RangePenalty => Modifier::penalty_from(
            PenaltyType::Untyped,
            value,
            ModifierSource::RangePenalty,
            sequence,
            None,
            Statistic::Attack,
        ),
        AttackCheckTerm::Circumstance if value < 0 => Modifier::penalty_from(
            PenaltyType::Circumstance,
            value,
            ModifierSource::AttackCircumstance,
            sequence,
            None,
            Statistic::Attack,
        ),
        AttackCheckTerm::Circumstance => Modifier::bonus_from(
            BonusType::Circumstance,
            value,
            ModifierSource::AttackCircumstance,
            sequence,
            None,
            Statistic::Attack,
        ),
    };
    match built {
        Ok(modifier) => parts.modifiers.push(modifier),
        Err(_) => parts.raw_extra += i64::from(value),
    }
}

fn push_circumstance(
    modifiers: &mut Vec<Modifier>,
    value: i32,
    source: ModifierSource,
    sequence: u32,
) -> Result<(), ModifierError> {
    if value == 0 {
        return Ok(());
    }
    if value > 0 {
        modifiers.push(Modifier::bonus_from(
            BonusType::Circumstance,
            value,
            source,
            sequence,
            None,
            Statistic::Attack,
        )?);
    } else {
        modifiers.push(Modifier::penalty_from(
            PenaltyType::Circumstance,
            value,
            source,
            sequence,
            None,
            Statistic::Attack,
        )?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::{
        AttackTotalTerm, DegreeOfSuccess, MapInput, attack_check_total, degree_of_success,
        flat_check, multiple_attack_penalty,
    };
    use crate::dice::ScriptedDice;

    fn script() -> ScriptedDice {
        ScriptedDice::new([8])
    }

    fn bonus(
        ty: BonusType,
        value: i32,
        source: u32,
        sequence: u32,
        level: Option<u32>,
        statistic: Statistic,
    ) -> Modifier {
        Modifier::bonus(ty, value, SourceId(source), sequence, level, statistic).expect("bonus")
    }

    fn penalty(
        ty: PenaltyType,
        value: i32,
        source: u32,
        sequence: u32,
        level: Option<u32>,
        statistic: Statistic,
    ) -> Modifier {
        Modifier::penalty(ty, value, SourceId(source), sequence, level, statistic).expect("penalty")
    }

    fn summed(modifiers: &[Modifier]) -> ModifierTotal {
        total(modifiers).expect("total fits")
    }

    fn total_on(stacked: &ModifierTotal, statistic: Statistic) -> i32 {
        stacked.get(statistic)
    }

    fn orders_of_two(first: Modifier, second: Modifier) -> [[Modifier; 2]; 2] {
        [[first, second], [second, first]]
    }

    fn orders_of_three(first: Modifier, second: Modifier, third: Modifier) -> [[Modifier; 3]; 6] {
        [
            [first, second, third],
            [first, third, second],
            [second, first, third],
            [second, third, first],
            [third, first, second],
            [third, second, first],
        ]
    }

    fn for_each_order(modifiers: &mut [Modifier], visit: &mut dyn FnMut(&[Modifier])) {
        fn walk(modifiers: &mut [Modifier], start: usize, visit: &mut dyn FnMut(&[Modifier])) {
            if start >= modifiers.len() {
                visit(modifiers);
                return;
            }
            for index in start..modifiers.len() {
                modifiers.swap(start, index);
                walk(modifiers, start + 1, visit);
                modifiers.swap(start, index);
            }
        }
        walk(modifiers, 0, visit);
    }

    fn term(value: i32) -> AttackTotalTerm {
        AttackTotalTerm::new(value).expect("attack term fits")
    }

    fn second_non_agile_map() -> i32 {
        multiple_attack_penalty(MapInput {
            on_own_turn: true,
            prior_attack_trait_checks_this_turn: 1,
            current_weapon_agile: false,
            readied_attack_count: None,
        })
    }

    fn second_agile_map() -> i32 {
        multiple_attack_penalty(MapInput {
            on_own_turn: true,
            prior_attack_trait_checks_this_turn: 1,
            current_weapon_agile: true,
            readied_attack_count: None,
        })
    }

    /// Player Core, Penalties.
    #[test]
    fn an_untyped_bonus_is_rejected() {
        let error = Modifier::penalty(
            PenaltyType::Untyped,
            2,
            SourceId(1),
            1,
            None,
            Statistic::Attack,
        );
        assert_eq!(error, Err(ModifierError::UntypedBonus));
    }

    /// Player Core, Penalties.
    /// Player Core, "Condition Values".
    #[test]
    fn a_zero_modifier_is_rejected() {
        assert_eq!(
            Modifier::bonus(
                BonusType::Circumstance,
                0,
                SourceId(1),
                1,
                None,
                Statistic::Attack
            ),
            Err(ModifierError::Zero)
        );
        assert_eq!(
            Modifier::penalty(
                PenaltyType::Circumstance,
                0,
                SourceId(1),
                1,
                None,
                Statistic::Attack
            ),
            Err(ModifierError::Zero)
        );
    }

    /// Player Core, Penalties.
    #[test]
    fn a_negative_bonus_is_rejected() {
        assert_eq!(
            Modifier::bonus(
                BonusType::Circumstance,
                -1,
                SourceId(1),
                1,
                None,
                Statistic::Attack
            ),
            Err(ModifierError::Sign)
        );
    }

    /// Player Core, Penalties.
    #[test]
    fn a_positive_penalty_is_rejected() {
        assert_eq!(
            Modifier::penalty(
                PenaltyType::Circumstance,
                1,
                SourceId(1),
                1,
                None,
                Statistic::Attack
            ),
            Err(ModifierError::Sign)
        );
    }

    /// Player Core, Bonuses.
    #[test]
    fn an_item_bonus_is_accepted() {
        let dice = script();
        let stacked = summed(&[
            bonus(BonusType::Item, 2, 1, 1, None, Statistic::Attack),
            bonus(BonusType::Item, 1, 2, 2, None, Statistic::Attack),
        ]);
        assert_eq!(stacked.get(Statistic::Attack), 2);
        assert!(stacked.applied().iter().any(|modifier| {
            modifier.bonus_type() == Some(BonusType::Item) && modifier.value() == 2
        }));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Bonuses.
    /// Player Core, Penalties.
    ///
    /// Proficiency is excluded because AC, attack, Perception, and DCs are baked (DEV-01).
    #[test]
    fn proficiency_is_not_a_modifier_type() {
        let bonuses = [BonusType::Circumstance, BonusType::Item, BonusType::Status];
        let penalties = [
            PenaltyType::Circumstance,
            PenaltyType::Item,
            PenaltyType::Status,
            PenaltyType::Untyped,
        ];
        let bonus_names: Vec<_> = bonuses.iter().copied().map(bonus_name).collect();
        let penalty_names: Vec<_> = penalties.iter().copied().map(penalty_name).collect();
        assert_eq!(bonus_names, ["circumstance", "item", "status"]);
        assert_eq!(penalty_names, ["circumstance", "item", "status", "untyped"]);
        assert!(!bonus_names.contains(&"proficiency"));
        assert!(!penalty_names.contains(&"proficiency"));
    }

    fn bonus_name(ty: BonusType) -> &'static str {
        match ty {
            BonusType::Circumstance => "circumstance",
            BonusType::Item => "item",
            BonusType::Status => "status",
        }
    }

    fn penalty_name(ty: PenaltyType) -> &'static str {
        match ty {
            PenaltyType::Circumstance => "circumstance",
            PenaltyType::Item => "item",
            PenaltyType::Status => "status",
            PenaltyType::Untyped => "untyped",
        }
    }

    /// Player Core, Bonuses.
    #[test]
    fn two_circumstance_bonuses_keep_the_highest() {
        let dice = script();
        let stacked = summed(&[
            bonus(BonusType::Circumstance, 2, 1, 1, None, Statistic::Attack),
            bonus(BonusType::Circumstance, 1, 2, 2, None, Statistic::Attack),
        ]);
        assert_eq!(stacked.get(Statistic::Attack), 2);
        assert!(stacked.applied().iter().any(|modifier| {
            modifier.bonus_type() == Some(BonusType::Circumstance) && modifier.value() == 2
        }));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Penalties.
    #[test]
    fn plus_2_circumstance_with_minus_2_circumstance_equals_0() {
        let dice = script();
        let stacked = summed(&[
            bonus(BonusType::Circumstance, 2, 1, 1, None, Statistic::Attack),
            penalty(PenaltyType::Circumstance, -2, 2, 2, None, Statistic::Attack),
        ]);
        assert_eq!(stacked.applied().len(), 2);
        assert_eq!(stacked.get(Statistic::Attack), 0);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Penalties.
    #[test]
    fn status_plus_1_and_status_minus_2_total_minus_1() {
        let dice = script();
        let stacked = summed(&[
            bonus(BonusType::Status, 1, 1, 1, None, Statistic::Attack),
            penalty(PenaltyType::Status, -2, 2, 2, None, Statistic::Attack),
        ]);
        assert_eq!(stacked.applied().len(), 2);
        assert_eq!(stacked.get(Statistic::Attack), -1);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Frightened.
    /// Player Core, Sickened.
    /// Player Core, Penalties.
    #[test]
    fn frightened_2_and_sickened_1_equal_minus_2() {
        let dice = script();
        let frightened = penalty(PenaltyType::Status, -2, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(2))
            .expect("condition");
        let sickened = penalty(PenaltyType::Status, -1, 2, 2, None, Statistic::Attack)
            .with_condition(ConditionId::SICKENED, Some(1))
            .expect("condition");
        let stacked = summed(&[frightened, sickened]);
        assert_eq!(stacked.get(Statistic::Attack), -2);
        assert!(stacked.applied().iter().any(|modifier| {
            modifier.penalty_type() == Some(PenaltyType::Status) && modifier.value() == -2
        }));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Penalties.
    #[test]
    fn untyped_penalties_stack() {
        let dice = script();
        let stacked = summed(&[
            penalty(PenaltyType::Untyped, -2, 1, 1, None, Statistic::Attack),
            penalty(PenaltyType::Untyped, -3, 2, 2, None, Statistic::Attack),
        ]);
        assert_eq!(stacked.get(Statistic::Attack), -5);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 400, Penalties.
    #[test]
    fn two_item_penalties_on_one_statistic_apply_only_the_worst() {
        let dice = script();
        let lesser = penalty(PenaltyType::Item, -2, 1, 1, None, Statistic::Attack);
        let worse = penalty(PenaltyType::Item, -8, 2, 2, None, Statistic::Attack);
        for modifiers in orders_of_two(lesser, worse) {
            let stacked = summed(&modifiers);
            let item: Vec<_> = stacked
                .applied()
                .iter()
                .filter(|modifier| modifier.penalty_type() == Some(PenaltyType::Item))
                .collect();
            assert_eq!(item.len(), 1);
            assert_eq!(item[0].value(), -8);
            assert_eq!(stacked.get(Statistic::Attack), -8);
            assert_ne!(stacked.get(Statistic::Attack), -10);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Multiple Attack Penalty".
    /// Player Core, "Range Penalty".
    #[test]
    fn map_minus_5_plus_range_minus_2_equals_minus_7() {
        let dice = script();
        let map = second_non_agile_map();
        assert_eq!(map, -5);
        let mut modifiers = attack_modifiers(map, false, 0).expect("attack");
        modifiers.push(penalty(
            PenaltyType::Untyped,
            -2,
            20,
            9,
            None,
            Statistic::Attack,
        ));
        assert_eq!(summed(&modifiers).get(Statistic::Attack), -7);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Multiple Attack Penalty".
    /// Player Core, Penalties.
    ///
    /// Source 1 is a caller id. It cannot name the multiple attack penalty, so the newer
    /// untyped penalty stacks with it instead of replacing it.
    #[test]
    fn an_untyped_penalty_from_source_1_stacks_with_map() {
        let dice = script();
        let map = second_non_agile_map();
        assert_eq!(map, -5);
        let mut modifiers = attack_modifiers(map, false, 0).expect("attack");
        let caller = penalty(PenaltyType::Untyped, -2, 1, 9, None, Statistic::Attack);
        assert_eq!(caller.source(), Some(SourceId(1)));
        assert!(
            modifiers
                .iter()
                .all(|modifier| modifier.source() != Some(SourceId(1)))
        );
        modifiers.push(caller);
        let stacked = summed(&modifiers);
        assert_eq!(stacked.get(Statistic::Attack), -7);
        assert!(stacked.applied().iter().any(|modifier| {
            modifier.penalty_type() == Some(PenaltyType::Untyped) && modifier.value() == -5
        }));
        assert!(stacked.applied().iter().any(|modifier| {
            modifier.penalty_type() == Some(PenaltyType::Untyped) && modifier.value() == -2
        }));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Penalties.
    #[test]
    fn an_overflowing_modifier_sum_returns_an_error() {
        let dice = script();
        let result = total(&[
            penalty(
                PenaltyType::Untyped,
                i32::MIN,
                1,
                1,
                None,
                Statistic::Attack,
            ),
            penalty(PenaltyType::Untyped, -1, 2, 2, None, Statistic::Attack),
        ]);
        assert_eq!(result, Err(ModifierError::Overflow));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Bonuses.
    /// Player Core, Penalties.
    ///
    /// Circumstance `i32::MAX`, item +1, and status -5. The true total fits.
    #[test]
    fn circumstance_max_item_1_and_status_minus_5_fits() {
        let dice = script();
        let modifiers = [
            bonus(
                BonusType::Circumstance,
                i32::MAX,
                1,
                1,
                None,
                Statistic::Attack,
            ),
            bonus(BonusType::Item, 1, 2, 2, None, Statistic::Attack),
            penalty(PenaltyType::Status, -5, 3, 3, None, Statistic::Attack),
        ];
        let mut reversed = modifiers;
        reversed.reverse();
        assert_eq!(summed(&modifiers).get(Statistic::Attack), i32::MAX - 4);
        assert_eq!(summed(&reversed).get(Statistic::Attack), i32::MAX - 4);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Bonuses.
    /// Player Core, Penalties.
    #[test]
    fn the_same_modifiers_total_twice() {
        let dice = script();
        let modifiers = [
            bonus(BonusType::Circumstance, 2, 1, 1, None, Statistic::Attack),
            penalty(PenaltyType::Untyped, -3, 2, 2, None, Statistic::Attack),
        ];
        assert_eq!(summed(&modifiers), summed(&modifiers));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Bonuses.
    #[test]
    fn a_tied_highest_bonus_reports_the_lowest_sequence() {
        let dice = script();
        let stacked = summed(&[
            bonus(BonusType::Circumstance, 2, 1, 4, None, Statistic::Attack),
            bonus(BonusType::Circumstance, 2, 2, 1, None, Statistic::Attack),
        ]);
        let applied = stacked
            .applied()
            .iter()
            .find(|modifier| modifier.bonus_type() == Some(BonusType::Circumstance))
            .expect("circumstance bonus");
        assert_eq!(applied.sequence(), 1);
        assert_eq!(stacked.get(Statistic::Attack), 2);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Penalties.
    #[test]
    fn a_tied_worst_penalty_reports_the_lowest_sequence() {
        let dice = script();
        let stacked = summed(&[
            penalty(PenaltyType::Circumstance, -2, 1, 7, None, Statistic::Attack),
            penalty(PenaltyType::Circumstance, -2, 2, 3, None, Statistic::Attack),
        ]);
        let applied = stacked
            .applied()
            .iter()
            .find(|modifier| modifier.penalty_type() == Some(PenaltyType::Circumstance))
            .expect("circumstance penalty");
        assert_eq!(applied.sequence(), 3);
        assert_eq!(stacked.get(Statistic::Attack), -2);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Duplicate Effects".
    #[test]
    fn a_duplicate_same_source_untyped_penalty_counts_once() {
        let dice = script();
        let stacked = summed(&[
            penalty(PenaltyType::Untyped, -2, 1, 1, None, Statistic::Attack),
            penalty(PenaltyType::Untyped, -2, 1, 2, None, Statistic::Attack),
        ]);
        assert_eq!(stacked.applied().len(), 1);
        assert_eq!(stacked.applied()[0].value(), -2);
        assert_eq!(stacked.get(Statistic::Attack), -2);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Duplicate Effects".
    #[test]
    fn a_tied_source_keeps_the_newer_modifier() {
        let dice = script();
        let stacked = summed(&[
            bonus(BonusType::Circumstance, 2, 1, 1, Some(3), Statistic::Attack),
            bonus(BonusType::Circumstance, 2, 1, 4, Some(3), Statistic::Attack),
        ]);
        assert_eq!(stacked.applied().len(), 1);
        assert_eq!(stacked.applied()[0].sequence(), 4);
        assert_eq!(stacked.get(Statistic::Attack), 2);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Duplicate Effects".
    #[test]
    fn a_higher_level_beats_a_higher_value() {
        let dice = script();
        let stacked = summed(&[
            bonus(BonusType::Circumstance, 1, 1, 1, Some(2), Statistic::Attack),
            bonus(BonusType::Circumstance, 4, 1, 2, Some(1), Statistic::Attack),
        ]);
        assert_eq!(stacked.applied().len(), 1);
        assert_eq!(stacked.applied()[0].level(), Some(2));
        assert_eq!(stacked.applied()[0].value(), 1);
        assert_eq!(stacked.get(Statistic::Attack), 1);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Duplicate Effects".
    /// Rules 2266.
    #[test]
    fn a_newer_lower_value_wins() {
        let dice = script();
        let stacked = summed(&[
            penalty(PenaltyType::Untyped, -2, 1, 1, None, Statistic::Attack),
            penalty(PenaltyType::Untyped, -5, 1, 9, None, Statistic::Attack),
        ]);
        assert_eq!(stacked.applied().len(), 1);
        assert_eq!(stacked.applied()[0].sequence(), 9);
        assert_eq!(stacked.get(Statistic::Attack), -5);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 398, "Duplicate Effects".
    /// Player Core pg. 400, Penalties.
    ///
    /// Ruling R-01, an extension consistent with RAW via Rules 2282, not a deviation:
    /// an equal sequence keeps the higher bonus and the worse penalty.
    #[test]
    fn an_equal_sequence_keeps_the_higher_value() {
        let dice = script();
        let low_bonus = bonus(BonusType::Circumstance, 1, 1, 4, None, Statistic::Attack);
        let high_bonus = bonus(BonusType::Circumstance, 4, 1, 4, None, Statistic::Attack);
        for modifiers in [[low_bonus, high_bonus], [high_bonus, low_bonus]] {
            let stacked = summed(&modifiers);
            assert_eq!(stacked.applied().len(), 1);
            assert_eq!(stacked.applied()[0].value(), 4);
            assert_eq!(stacked.get(Statistic::Attack), 4);
        }
        let higher_penalty = penalty(PenaltyType::Untyped, -1, 2, 7, None, Statistic::Attack);
        let lower_penalty = penalty(PenaltyType::Untyped, -8, 2, 7, None, Statistic::Attack);
        for modifiers in [
            [higher_penalty, lower_penalty],
            [lower_penalty, higher_penalty],
        ] {
            let stacked = summed(&modifiers);
            assert_eq!(stacked.applied().len(), 1);
            assert_eq!(stacked.applied()[0].value(), -8);
            assert_eq!(stacked.get(Statistic::Attack), -8);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Redundant Conditions With Values".
    /// Player Core, Frightened.
    #[test]
    fn a_repeated_condition_keeps_its_highest_rank() {
        let dice = script();
        let frightened_1 = penalty(PenaltyType::Status, -1, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(1))
            .expect("condition");
        let frightened_2 = penalty(PenaltyType::Status, -2, 2, 2, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(2))
            .expect("condition");
        let stacked = summed(&[frightened_1, frightened_2]);
        let status: Vec<_> = stacked
            .applied()
            .iter()
            .filter(|modifier| modifier.penalty_type() == Some(PenaltyType::Status))
            .collect();
        assert_eq!(status.len(), 1);
        assert_eq!(status[0].rank(), Some(2));
        assert_eq!(status[0].value(), -2);
        assert!(status[0].rank() != Some(1));
        assert_eq!(stacked.get(Statistic::Attack), -2);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Duplicate Effects".
    /// Rules 2266. The same rank keeps the greater sequence. Value does not break that tie.
    #[test]
    fn a_same_rank_condition_keeps_the_newer_sequence() {
        let dice = script();
        let older = penalty(PenaltyType::Status, -9, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(2))
            .expect("condition");
        let newer = penalty(PenaltyType::Status, -1, 2, 9, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(2))
            .expect("condition");
        for modifiers in [[older, newer], [newer, older]] {
            let stacked = summed(&modifiers);
            let status: Vec<_> = stacked
                .applied()
                .iter()
                .filter(|modifier| modifier.penalty_type() == Some(PenaltyType::Status))
                .collect();
            assert_eq!(status.len(), 1);
            assert_eq!(status[0].sequence(), 9);
            assert_eq!(status[0].rank(), Some(2));
            assert_eq!(status[0].value(), -2);
            assert_eq!(stacked.get(Statistic::Attack), -2);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Redundant Conditions With Values".
    ///
    /// Frightened and sickened are the conditions that become `-rank`. Another ranked
    /// penalty keeps its declared type and value.
    #[test]
    fn a_ranked_penalty_keeps_its_declared_type() {
        let dice = script();
        let lower = penalty(PenaltyType::Circumstance, -4, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId(9), Some(1))
            .expect("condition");
        let higher = penalty(PenaltyType::Circumstance, -1, 2, 2, None, Statistic::Attack)
            .with_condition(ConditionId(9), Some(2))
            .expect("condition");
        let stacked = summed(&[lower, higher]);
        assert_eq!(stacked.applied().len(), 1);
        let kept = stacked.applied()[0];
        assert_eq!(kept.penalty_type(), Some(PenaltyType::Circumstance));
        assert_eq!(kept.value(), -4);
        assert_eq!(kept.rank(), Some(1));
        assert_eq!(stacked.get(Statistic::Attack), -4);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 444, Frightened.
    /// Player Core pg. 446, Sickened.
    /// Player Core pg. 442, "Redundant Conditions With Values".
    ///
    /// A frightened status bonus of +2 at rank 3 is not totaled as +2.
    #[test]
    fn a_frightened_bonus_keeps_its_declared_value() {
        let dice = script();
        let result = bonus(BonusType::Status, 2, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(3));
        assert_eq!(result, Err(ModifierError::InvalidValuedCondition));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 444, Frightened.
    /// Player Core pg. 446, Sickened.
    /// Player Core pg. 442, "Redundant Conditions With Values".
    #[test]
    fn frightened_bonus_rank_0_plus_2_is_rejected() {
        let dice = script();
        let result = bonus(BonusType::Status, 2, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(0));
        assert_eq!(result, Err(ModifierError::InvalidValuedCondition));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 444, Frightened.
    /// Player Core pg. 446, Sickened.
    /// Player Core pg. 442, "Redundant Conditions With Values".
    #[test]
    fn frightened_bonus_rank_2_plus_2_is_rejected() {
        let dice = script();
        let result = bonus(BonusType::Status, 2, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(2));
        assert_eq!(result, Err(ModifierError::InvalidValuedCondition));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 444, Frightened.
    /// Player Core pg. 446, Sickened.
    /// Player Core pg. 442, "Redundant Conditions With Values".
    #[test]
    fn sickened_bonus_rank_1_plus_1_is_rejected() {
        let dice = script();
        let result = bonus(BonusType::Status, 1, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::SICKENED, Some(1));
        assert_eq!(result, Err(ModifierError::InvalidValuedCondition));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 444, Frightened.
    /// Player Core pg. 442, "Redundant Conditions With Values".
    ///
    /// A frightened circumstance penalty of -7 at rank 3 is invalid. It is not rewritten
    /// into a status penalty of -3.
    #[test]
    fn a_frightened_circumstance_penalty_becomes_status_minus_rank() {
        let dice = script();
        let result = penalty(PenaltyType::Circumstance, -7, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(3));
        assert_eq!(result, Err(ModifierError::InvalidValuedCondition));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 444, Frightened.
    /// Player Core pg. 446, Sickened.
    /// Player Core pg. 442, "Redundant Conditions With Values".
    #[test]
    fn frightened_with_rank_none_is_rejected() {
        let dice = script();
        let result = penalty(PenaltyType::Status, -1, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, None);
        assert_eq!(result, Err(ModifierError::InvalidValuedCondition));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 444, Frightened.
    /// Player Core pg. 446, Sickened.
    /// Player Core pg. 442, "Redundant Conditions With Values".
    #[test]
    fn frightened_rank_2_declared_circumstance_is_rejected() {
        let dice = script();
        let result = penalty(PenaltyType::Circumstance, -2, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(2));
        assert_eq!(result, Err(ModifierError::InvalidValuedCondition));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 444, Frightened.
    /// Player Core pg. 446, Sickened.
    /// Player Core pg. 442, "Redundant Conditions With Values".
    #[test]
    fn sickened_with_no_rank_is_rejected() {
        let dice = script();
        let result = penalty(PenaltyType::Status, -1, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::SICKENED, None);
        assert_eq!(result, Err(ModifierError::InvalidValuedCondition));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 444, Frightened.
    /// Player Core pg. 446, Sickened.
    /// Player Core pg. 442, "Redundant Conditions With Values".
    ///
    /// The circumstance penalty has no rank, so the list is not totaled.
    #[test]
    fn the_qa_frightened_list_is_rejected() {
        let dice = script();
        let unranked = penalty(PenaltyType::Circumstance, -4, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, None);
        let ranked = penalty(PenaltyType::Status, -1, 2, 2, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(2));
        assert_eq!(unranked, Err(ModifierError::InvalidValuedCondition));
        assert!(ranked.is_ok());
        let mut totaled = false;
        if let (Ok(first), Ok(second)) = (unranked, ranked) {
            let _ = summed(&[first, second]);
            totaled = true;
        }
        assert!(!totaled);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 444, Frightened.
    /// Player Core pg. 446, Sickened.
    /// Player Core pg. 442, "Redundant Conditions With Values".
    #[test]
    fn two_status_frightened_rank_2_penalties_total_minus_2() {
        let dice = script();
        let first = penalty(PenaltyType::Status, -1, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(2))
            .expect("condition");
        let second = penalty(PenaltyType::Status, -1, 2, 2, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(2))
            .expect("condition");
        for modifiers in orders_of_two(first, second) {
            let stacked = summed(&modifiers);
            let status: Vec<_> = stacked
                .applied()
                .iter()
                .filter(|modifier| modifier.penalty_type() == Some(PenaltyType::Status))
                .collect();
            assert_eq!(status.len(), 1);
            assert_eq!(status[0].value(), -2);
            assert_eq!(stacked.get(Statistic::Attack), -2);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 444, Frightened.
    /// Player Core pg. 446, Sickened.
    /// Player Core pg. 442, "Redundant Conditions With Values".
    #[test]
    fn sickened_with_item_type_and_rank_1_is_rejected() {
        let dice = script();
        let result = penalty(PenaltyType::Item, -1, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::SICKENED, Some(1));
        assert_eq!(result, Err(ModifierError::InvalidValuedCondition));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 444, Frightened.
    /// Player Core pg. 446, Sickened.
    /// Player Core pg. 442, "Redundant Conditions With Values".
    #[test]
    fn frightened_untyped_with_rank_1_is_rejected() {
        let dice = script();
        let result = penalty(PenaltyType::Untyped, -1, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(1));
        assert_eq!(result, Err(ModifierError::InvalidValuedCondition));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Redundant Conditions".
    /// Player Core, Off-Guard.
    #[test]
    fn off_guard_from_two_causes_applies_once() {
        let dice = script();
        let prone = penalty(PenaltyType::Circumstance, -2, 1, 1, None, Statistic::Attack)
            .with_condition(OFF_GUARD, None)
            .expect("condition");
        let flanking = penalty(PenaltyType::Circumstance, -2, 2, 2, None, Statistic::Attack)
            .with_condition(OFF_GUARD, None)
            .expect("condition");
        let stacked = summed(&[prone, flanking]);
        let penalties: Vec<_> = stacked
            .applied()
            .iter()
            .filter(|modifier| modifier.penalty_type() == Some(PenaltyType::Circumstance))
            .collect();
        assert_eq!(penalties.len(), 1);
        assert_eq!(penalties[0].value(), -2);
        assert_eq!(stacked.get(Statistic::Attack), -2);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Getting a DC from a Modifier".
    #[test]
    fn a_negative_total() {
        let dice = script();
        let stacked = summed(&[
            bonus(BonusType::Circumstance, 1, 1, 1, None, Statistic::Attack),
            penalty(PenaltyType::Untyped, -3, 2, 2, None, Statistic::Attack),
            penalty(PenaltyType::Untyped, -4, 3, 3, None, Statistic::Attack),
        ]);
        assert_eq!(stacked.get(Statistic::Attack), -6);
        assert!(stacked.get(Statistic::Attack) < 0);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Getting a DC from a Modifier".
    #[test]
    fn a_dc_below_10() {
        let dice = script();
        assert_eq!(dc_from_total(-6).expect("dc fits"), 4);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Flat Checks".
    #[test]
    fn a_flat_check_ignores_modifiers() {
        let modifier_total = 9;
        let mut dice = ScriptedDice::new([10]);
        let roll = flat_check(&mut dice, 11);
        assert_eq!(roll.natural, Some(10));
        assert_eq!(degree_of_success(10, 10, 11), DegreeOfSuccess::Failure);
        assert_eq!(roll.degree, DegreeOfSuccess::Failure);
        assert_eq!(
            degree_of_success(10, 10 + modifier_total, 11),
            DegreeOfSuccess::Success
        );
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, Bonuses.
    /// Player Core, Penalties.
    #[test]
    fn insertion_order_does_not_change_the_total() {
        let dice = script();
        let status = bonus(BonusType::Status, 1, 1, 1, None, Statistic::Attack);
        let circumstance = penalty(PenaltyType::Circumstance, -2, 2, 2, None, Statistic::Attack);
        assert_eq!(summed(&[status, circumstance]).get(Statistic::Attack), -1);
        assert_eq!(
            summed(&[circumstance, status]),
            summed(&[status, circumstance])
        );
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Prone.
    /// Player Core, Off-Guard.
    #[test]
    fn a_prone_attacker_and_an_off_guard_defender_both_apply() {
        let dice = script();
        let attack = attack_modifiers(0, true, 0).expect("attack");
        let armor_class = ac_modifiers(None, true, false).expect("armor class");
        let attack_total = summed(&attack);
        let armor_total = summed(&armor_class);
        assert_eq!(attack_total.get(Statistic::Attack), -2);
        assert_eq!(armor_total.get(Statistic::ArmorClass), -2);
        assert!(attack_total.applied().iter().any(|modifier| {
            modifier.penalty_type() == Some(PenaltyType::Circumstance) && modifier.value() == -2
        }));
        assert!(armor_total.applied().iter().any(|modifier| {
            modifier.penalty_type() == Some(PenaltyType::Circumstance) && modifier.value() == -2
        }));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Multiple Attack Penalty".
    /// Player Core, Penalties.
    #[test]
    fn map_combines_with_circumstance_and_status() {
        let dice = script();
        let map = second_non_agile_map();
        assert_eq!(map, -5);
        let mut modifiers = attack_modifiers(map, false, 0).expect("attack");
        modifiers.push(penalty(
            PenaltyType::Circumstance,
            -2,
            21,
            8,
            None,
            Statistic::Attack,
        ));
        modifiers.push(penalty(
            PenaltyType::Status,
            -4,
            22,
            9,
            None,
            Statistic::Attack,
        ));
        assert_eq!(summed(&modifiers).get(Statistic::Attack), -11);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Multiple Attack Penalty".
    #[test]
    fn agile_map_stays_the_existing_untyped_penalty() {
        let dice = script();
        let map = second_agile_map();
        assert_eq!(map, -4);
        let mut modifiers = attack_modifiers(map, false, 0).expect("attack");
        modifiers.push(penalty(
            PenaltyType::Circumstance,
            -2,
            21,
            8,
            None,
            Statistic::Attack,
        ));
        assert_eq!(summed(&modifiers).get(Statistic::Attack), -6);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    #[test]
    fn an_aid_success_adds_plus_1_circumstance() {
        let dice = script();
        let modifiers = attack_modifiers(0, false, 1).expect("attack");
        let stacked = summed(&modifiers);
        assert_eq!(stacked.get(Statistic::Attack), 1);
        assert!(stacked.applied().iter().any(|modifier| {
            modifier.bonus_type() == Some(BonusType::Circumstance) && modifier.value() == 1
        }));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    #[test]
    fn an_aid_failure_adds_no_modifier() {
        let dice = script();
        let modifiers = attack_modifiers(0, false, 0).expect("attack");
        assert!(modifiers.is_empty());
        assert!(summed(&modifiers).applied().iter().all(|modifier| {
            modifier.bonus_type() != Some(BonusType::Circumstance)
                && modifier.penalty_type() != Some(PenaltyType::Circumstance)
        }));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Cover.
    #[test]
    fn cover_is_a_circumstance_bonus() {
        let dice = script();
        for (cover, expected) in [
            (Cover::Lesser, 1),
            (Cover::Standard, 2),
            (Cover::Greater, 4),
        ] {
            let stacked = summed(&ac_modifiers(Some(cover), false, false).expect("armor class"));
            assert_eq!(stacked.get(Statistic::ArmorClass), expected);
            assert!(stacked.applied().iter().any(|modifier| {
                modifier.bonus_type() == Some(BonusType::Circumstance)
                    && modifier.value() == expected
            }));
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Off-Guard.
    #[test]
    fn off_guard_is_minus_2_circumstance_to_armor_class() {
        let dice = script();
        let stacked = summed(&ac_modifiers(None, true, false).expect("armor class"));
        assert_eq!(stacked.get(Statistic::ArmorClass), -2);
        assert!(stacked.applied().iter().any(|modifier| {
            modifier.penalty_type() == Some(PenaltyType::Circumstance)
                && modifier.value() == -2
                && modifier.statistic() == Statistic::ArmorClass
        }));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Unconscious.
    #[test]
    fn unconscious_is_minus_4_status_to_armor_class_and_perception() {
        let dice = script();
        let armor_class = ac_modifiers(None, false, true).expect("armor class");
        let perception = perception_modifiers(true).expect("perception");
        let armor_total = summed(&armor_class);
        let perception_total = summed(&perception);
        assert_eq!(armor_total.get(Statistic::ArmorClass), -4);
        assert_eq!(perception_total.get(Statistic::Perception), -4);
        assert!(armor_total.applied().iter().any(|modifier| {
            modifier.penalty_type() == Some(PenaltyType::Status)
                && modifier.value() == -4
                && modifier.statistic() == Statistic::ArmorClass
        }));
        assert!(perception_total.applied().iter().any(|modifier| {
            modifier.penalty_type() == Some(PenaltyType::Status)
                && modifier.value() == -4
                && modifier.statistic() == Statistic::Perception
        }));
        assert_ne!(armor_class, perception);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Shield Statistics".
    ///
    /// Raise a Shield is out of scope. This total does not read a shield's AC bonus.
    #[test]
    fn a_shield_bonus_is_not_applied_here() {
        let dice = script();
        let shield_ac_bonus = 2;
        let modifiers = ac_modifiers(None, false, false).expect("armor class");
        let stacked = summed(&modifiers);
        assert_eq!(stacked.get(Statistic::ArmorClass), 0);
        assert!(
            stacked
                .applied()
                .iter()
                .all(|modifier| modifier.value() != shield_ac_bonus)
        );
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Damage Rolls".
    #[test]
    fn damage_keeps_the_highest_circumstance_bonus() {
        let dice = script();
        let stacked = summed(&[
            bonus(BonusType::Circumstance, 4, 1, 1, None, Statistic::Damage),
            bonus(BonusType::Circumstance, 1, 2, 2, None, Statistic::Damage),
        ]);
        assert_eq!(stacked.get(Statistic::Damage), 4);
        assert!(stacked.applied().iter().any(|modifier| {
            modifier.bonus_type() == Some(BonusType::Circumstance) && modifier.value() == 4
        }));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Bonuses.
    ///
    /// Damage has no Phase 1 producer, so that collection is empty.
    #[test]
    fn phase_1_produces_no_item_modifier() {
        let dice = script();
        let map = second_non_agile_map();
        let mut produced = attack_modifiers(map, true, 1).expect("attack");
        produced.extend(attack_modifiers(0, false, -1).expect("aid"));
        produced.extend(ac_modifiers(Some(Cover::Lesser), true, true).expect("lesser"));
        produced.extend(ac_modifiers(Some(Cover::Standard), false, false).expect("standard"));
        produced.extend(ac_modifiers(Some(Cover::Greater), false, false).expect("greater"));
        produced.extend(perception_modifiers(true).expect("perception"));
        produced.extend(reflex_modifiers(true).expect("reflex"));
        let damage: Vec<Modifier> = Vec::new();
        assert!(damage.is_empty());
        for modifier in produced.iter().chain(damage.iter()) {
            assert_ne!(modifier.bonus_type(), Some(BonusType::Item));
            assert_ne!(modifier.penalty_type(), Some(PenaltyType::Item));
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 398, "Duplicate Effects".
    /// Player Core pg. 400, Penalties.
    #[test]
    fn a_bonus_and_a_penalty_from_one_source_both_apply() {
        let dice = script();
        let bonus = bonus(BonusType::Status, 2, 7, 5, None, Statistic::Attack);
        let penalty = penalty(PenaltyType::Status, -1, 7, 5, None, Statistic::Attack);
        for modifiers in orders_of_two(bonus, penalty) {
            let stacked = summed(&modifiers);
            assert_eq!(stacked.applied().len(), 2);
            assert!(stacked.applied().iter().any(|modifier| {
                modifier.bonus_type() == Some(BonusType::Status) && modifier.value() == 2
            }));
            assert!(stacked.applied().iter().any(|modifier| {
                modifier.penalty_type() == Some(PenaltyType::Status) && modifier.value() == -1
            }));
            assert_eq!(stacked.get(Statistic::Attack), 1);
            assert_eq!(total_on(&stacked, Statistic::Attack), 1);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 398, "Duplicate Effects".
    /// Player Core pg. 400, Penalties.
    #[test]
    fn one_source_on_two_statistics_both_apply() {
        let dice = script();
        let attack = bonus(BonusType::Status, 2, 7, 5, None, Statistic::Attack);
        let armor_class = penalty(
            PenaltyType::Circumstance,
            -1,
            7,
            5,
            None,
            Statistic::ArmorClass,
        );
        for modifiers in orders_of_two(attack, armor_class) {
            let stacked = summed(&modifiers);
            assert_eq!(stacked.applied().len(), 2);
            assert!(stacked.applied().iter().any(|modifier| {
                modifier.statistic() == Statistic::Attack
                    && modifier.bonus_type() == Some(BonusType::Status)
                    && modifier.value() == 2
            }));
            assert!(stacked.applied().iter().any(|modifier| {
                modifier.statistic() == Statistic::ArmorClass
                    && modifier.penalty_type() == Some(PenaltyType::Circumstance)
                    && modifier.value() == -1
            }));
            assert_eq!(total_on(&stacked, Statistic::Attack), 2);
            assert_eq!(total_on(&stacked, Statistic::ArmorClass), -1);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 400, Penalties.
    /// Player Core pg. 442, "Redundant Conditions With Values".
    ///
    /// Condition 10 is a condition whose modifiers are not derived from its value.
    #[test]
    fn a_condition_bonus_does_not_hide_that_conditions_penalty() {
        let dice = script();
        let bonus = bonus(BonusType::Status, 2, 7, 4, None, Statistic::Attack)
            .with_condition(ConditionId(10), Some(3))
            .expect("condition");
        let penalty = penalty(PenaltyType::Status, -1, 7, 4, None, Statistic::Attack)
            .with_condition(ConditionId(10), Some(1))
            .expect("condition");
        for modifiers in orders_of_two(bonus, penalty) {
            let stacked = summed(&modifiers);
            assert!(stacked.applied().iter().any(|modifier| {
                modifier.bonus_type() == Some(BonusType::Status) && modifier.value() == 2
            }));
            assert!(stacked.applied().iter().any(|modifier| {
                modifier.penalty_type() == Some(PenaltyType::Status) && modifier.value() == -1
            }));
            assert_eq!(total_on(&stacked, Statistic::Attack), 1);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 442, "Redundant Conditions With Values".
    /// Player Core pg. 444, Frightened.
    #[test]
    fn frightened_2_on_attack_and_armor_class_both_survive() {
        let dice = script();
        let attack = penalty(PenaltyType::Status, -2, 7, 4, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(2))
            .expect("condition");
        let armor_class = penalty(PenaltyType::Status, -2, 7, 4, None, Statistic::ArmorClass)
            .with_condition(ConditionId::FRIGHTENED, Some(2))
            .expect("condition");
        for modifiers in orders_of_two(attack, armor_class) {
            let stacked = summed(&modifiers);
            assert_eq!(stacked.applied().len(), 2);
            assert!(stacked.applied().iter().any(|modifier| {
                modifier.statistic() == Statistic::Attack
                    && modifier.penalty_type() == Some(PenaltyType::Status)
                    && modifier.value() == -2
            }));
            assert!(stacked.applied().iter().any(|modifier| {
                modifier.statistic() == Statistic::ArmorClass
                    && modifier.penalty_type() == Some(PenaltyType::Status)
                    && modifier.value() == -2
            }));
            assert_eq!(total_on(&stacked, Statistic::Attack), -2);
            assert_eq!(total_on(&stacked, Statistic::ArmorClass), -2);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 398, "Duplicate Effects".
    ///
    /// Ruling R-01, an extension consistent with RAW via Rules 2282, not a deviation:
    /// an equal sequence keeps the higher bonus.
    #[test]
    fn an_equal_sequence_keeps_the_higher_bonus() {
        let dice = script();
        let low = bonus(BonusType::Circumstance, 1, 1, 4, None, Statistic::Attack);
        let high = bonus(BonusType::Circumstance, 4, 1, 4, None, Statistic::Attack);
        for modifiers in orders_of_two(low, high) {
            let stacked = summed(&modifiers);
            assert_eq!(stacked.applied().len(), 1);
            assert_eq!(stacked.applied()[0].value(), 4);
            assert_eq!(stacked.get(Statistic::Attack), 4);
            assert_eq!(total_on(&stacked, Statistic::Attack), 4);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 400, Penalties.
    ///
    /// Ruling R-01, an extension consistent with RAW via Rules 2282, not a deviation:
    /// an equal sequence keeps the worse penalty.
    #[test]
    fn an_equal_sequence_keeps_the_worse_penalty() {
        let dice = script();
        let mild = penalty(PenaltyType::Untyped, -1, 2, 7, None, Statistic::Attack);
        let worse = penalty(PenaltyType::Untyped, -8, 2, 7, None, Statistic::Attack);
        for modifiers in orders_of_two(mild, worse) {
            let stacked = summed(&modifiers);
            assert_eq!(stacked.applied().len(), 1);
            assert_eq!(stacked.applied()[0].value(), -8);
            assert_eq!(stacked.get(Statistic::Attack), -8);
            assert_eq!(total_on(&stacked, Statistic::Attack), -8);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 398, "Duplicate Effects".
    ///
    /// Substitute: rule, Duplicate Effects stops at level or rank then the newer effect;
    /// why not, it names no further identity when those and the value match; substitute,
    /// the lower condition id wins and a missing id loses; intent, every input order
    /// reports the same modifier.
    #[test]
    fn the_lower_condition_id_wins_an_equal_value() {
        let dice = script();
        let higher_id = bonus(BonusType::Circumstance, 2, 9, 5, Some(2), Statistic::Attack)
            .with_condition(ConditionId(8), Some(1))
            .expect("condition");
        let lower_id = bonus(BonusType::Circumstance, 2, 9, 5, Some(2), Statistic::Attack)
            .with_condition(ConditionId(5), Some(1))
            .expect("condition");
        for modifiers in orders_of_two(higher_id, lower_id) {
            let stacked = summed(&modifiers);
            assert_eq!(stacked.applied().len(), 1);
            assert_eq!(stacked.applied()[0].condition_id(), Some(ConditionId(5)));
            assert_eq!(stacked.get(Statistic::Attack), 2);
            assert_eq!(total_on(&stacked, Statistic::Attack), 2);
        }
        let present = bonus(BonusType::Circumstance, 2, 9, 5, None, Statistic::Attack)
            .with_condition(ConditionId(5), None)
            .expect("condition");
        let missing = bonus(BonusType::Circumstance, 2, 1, 5, None, Statistic::Attack);
        for modifiers in orders_of_two(present, missing) {
            let stacked = summed(&modifiers);
            assert_eq!(stacked.applied().len(), 1);
            assert_eq!(stacked.applied()[0].condition_id(), Some(ConditionId(5)));
            assert_eq!(stacked.applied()[0].source(), Some(SourceId(9)));
            assert_eq!(stacked.get(Statistic::Attack), 2);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 398, "Duplicate Effects".
    /// Player Core pg. 400, Penalties.
    #[test]
    fn equal_values_of_different_types_total_plus_5_in_every_order() {
        let dice = script();
        let status = bonus(BonusType::Status, 2, 7, 5, None, Statistic::Attack);
        let circumstance = bonus(BonusType::Circumstance, 2, 7, 5, None, Statistic::Attack);
        let higher = bonus(BonusType::Circumstance, 3, 8, 5, None, Statistic::Attack);
        let orders = orders_of_three(status, circumstance, higher);
        let first = summed(&orders[0]);
        for modifiers in orders {
            let stacked = summed(&modifiers);
            assert_eq!(stacked, first);
            assert!(
                stacked
                    .applied()
                    .iter()
                    .all(|modifier| modifier.statistic() == Statistic::Attack)
            );
            assert_eq!(stacked.get(Statistic::Attack), 5);
            assert_eq!(total_on(&stacked, Statistic::Attack), 5);
            assert!(stacked.applied().iter().any(|modifier| {
                modifier.bonus_type() == Some(BonusType::Status) && modifier.value() == 2
            }));
            assert!(stacked.applied().iter().any(|modifier| {
                modifier.bonus_type() == Some(BonusType::Circumstance) && modifier.value() == 3
            }));
            assert!(stacked.applied().iter().all(|modifier| {
                !(modifier.bonus_type() == Some(BonusType::Circumstance) && modifier.value() == 2)
            }));
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 406, "Damage Rolls".
    /// Player Core pg. 398, "Duplicate Effects".
    #[test]
    fn a_same_source_damage_modifier_and_attack_modifier_do_not_merge() {
        let dice = script();
        let damage = bonus(BonusType::Circumstance, 2, 4, 2, None, Statistic::Damage);
        let attack = bonus(BonusType::Circumstance, 3, 4, 2, None, Statistic::Attack);
        for modifiers in orders_of_two(damage, attack) {
            let stacked = summed(&modifiers);
            assert_eq!(stacked.applied().len(), 2);
            assert_eq!(total_on(&stacked, Statistic::Damage), 2);
            assert_eq!(total_on(&stacked, Statistic::Attack), 3);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 398, "Duplicate Effects".
    #[test]
    fn a_three_way_tie_is_the_same_in_all_six_orders() {
        let dice = script();
        let id_6 = bonus(
            BonusType::Circumstance,
            2,
            11,
            8,
            Some(1),
            Statistic::Attack,
        )
        .with_condition(ConditionId(6), Some(1))
        .expect("condition");
        let id_1 = bonus(
            BonusType::Circumstance,
            2,
            11,
            8,
            Some(1),
            Statistic::Attack,
        )
        .with_condition(ConditionId(1), Some(1))
        .expect("condition");
        let id_5 = bonus(
            BonusType::Circumstance,
            2,
            11,
            8,
            Some(1),
            Statistic::Attack,
        )
        .with_condition(ConditionId(5), Some(1))
        .expect("condition");
        let orders = orders_of_three(id_6, id_1, id_5);
        let first = summed(&orders[0]);
        for modifiers in orders {
            let stacked = summed(&modifiers);
            assert_eq!(stacked, first);
            assert_eq!(stacked.applied().len(), 1);
            assert_eq!(stacked.applied()[0].condition_id(), Some(ConditionId(1)));
            assert_eq!(stacked.get(Statistic::Attack), 2);
            assert_eq!(total_on(&stacked, Statistic::Attack), 2);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Duplicate Effects.
    /// Player Core, "Damage Rolls".
    ///
    /// One source's circumstance +2 on each statistic stays on that statistic.
    #[test]
    fn one_source_on_four_statistics_does_not_total_8() {
        let dice = script();
        let modifiers = [
            bonus(BonusType::Circumstance, 2, 1, 1, None, Statistic::Attack),
            bonus(
                BonusType::Circumstance,
                2,
                1,
                1,
                None,
                Statistic::ArmorClass,
            ),
            bonus(
                BonusType::Circumstance,
                2,
                1,
                1,
                None,
                Statistic::Perception,
            ),
            bonus(BonusType::Circumstance, 2, 1, 1, None, Statistic::Damage),
        ];
        let stacked = summed(&modifiers);
        assert_eq!(stacked.applied().len(), 4);
        assert_eq!(stacked.get(Statistic::Attack), 2);
        assert_eq!(stacked.get(Statistic::ArmorClass), 2);
        assert_eq!(stacked.get(Statistic::Perception), 2);
        assert_eq!(stacked.get(Statistic::Damage), 2);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Duplicate Effects.
    ///
    /// Two maximum bonuses on different statistics each fit. They are not one sum.
    #[test]
    fn two_max_values_on_different_statistics_do_not_overflow() {
        let dice = script();
        let stacked = total(&[
            bonus(
                BonusType::Circumstance,
                i32::MAX,
                1,
                1,
                None,
                Statistic::Attack,
            ),
            bonus(
                BonusType::Circumstance,
                i32::MAX,
                1,
                2,
                None,
                Statistic::Damage,
            ),
        ])
        .expect("each statistic fits");
        assert_eq!(stacked.get(Statistic::Attack), i32::MAX);
        assert_eq!(stacked.get(Statistic::Damage), i32::MAX);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 442, "Redundant Conditions With Values".
    #[test]
    fn a_same_condition_keeps_the_worse_penalty_in_both_orders() {
        let dice = script();
        let mild = penalty(PenaltyType::Circumstance, -1, 1, 5, None, Statistic::Attack)
            .with_condition(ConditionId(10), Some(1))
            .expect("condition");
        let worse = penalty(PenaltyType::Circumstance, -4, 2, 5, None, Statistic::Attack)
            .with_condition(ConditionId(10), Some(1))
            .expect("condition");
        for modifiers in orders_of_two(mild, worse) {
            let stacked = summed(&modifiers);
            let applied: Vec<_> = stacked
                .applied()
                .iter()
                .filter(|modifier| modifier.penalty_type() == Some(PenaltyType::Circumstance))
                .collect();
            assert_eq!(applied.len(), 1);
            assert_eq!(applied[0].value(), -4);
            assert_eq!(stacked.get(Statistic::Attack), -4);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 442, "Redundant Conditions With Values".
    /// Player Core pg. 444, Frightened.
    ///
    /// The rewrite to `-rank` happens before the value tie, so the declared `-1` is not
    /// the applied value.
    #[test]
    fn two_frightened_penalties_report_the_lower_source_in_both_orders() {
        let dice = script();
        let source_1 = penalty(PenaltyType::Status, -1, 1, 5, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(2))
            .expect("condition");
        let source_2 = penalty(PenaltyType::Status, -1, 2, 5, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(2))
            .expect("condition");
        for modifiers in orders_of_two(source_1, source_2) {
            let stacked = summed(&modifiers);
            let status: Vec<_> = stacked
                .applied()
                .iter()
                .filter(|modifier| modifier.penalty_type() == Some(PenaltyType::Status))
                .collect();
            assert_eq!(status.len(), 1);
            assert_eq!(status[0].source(), Some(SourceId(1)));
            assert_eq!(status[0].value(), -2);
            assert_eq!(stacked.get(Statistic::Attack), -2);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 442, "Redundant Conditions With Values".
    ///
    /// The order is the worst penalty or the highest bonus. For a condition whose modifier
    /// is derived from its value (frightened, sickened: −value), that is the highest
    /// condition value (Player Core p.442, Rules 2455). Otherwise the larger modifier wins
    /// (our ruling, see the non-derived requirement). Then rank, sequence, and source id
    /// are engineering tiebreaks with no rules effect.
    #[test]
    fn an_older_higher_condition_value_beats_a_newer_lower_one() {
        let dice = script();
        let older = penalty(PenaltyType::Status, -3, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(3))
            .expect("condition");
        let newer = penalty(PenaltyType::Status, -1, 2, 2, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(1))
            .expect("condition");
        for modifiers in orders_of_two(older, newer) {
            let stacked = summed(&modifiers);
            let status: Vec<_> = stacked
                .applied()
                .iter()
                .filter(|modifier| modifier.penalty_type() == Some(PenaltyType::Status))
                .collect();
            assert_eq!(status.len(), 1);
            assert_eq!(status[0].value(), -3);
            assert_eq!(stacked.get(Statistic::Attack), -3);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 442, "Redundant Conditions With Values".
    /// Player Core pg. 444, Frightened.
    /// Player Core pg. 446, Sickened.
    #[test]
    fn frightened_and_sickened_stay_present_while_the_worse_penalty_applies() {
        let dice = script();
        let frightened = penalty(PenaltyType::Status, -3, 4, 1, Some(1), Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(3))
            .expect("condition");
        let sickened = penalty(PenaltyType::Status, -1, 4, 2, Some(5), Statistic::Attack)
            .with_condition(ConditionId::SICKENED, Some(1))
            .expect("condition");
        let mut list = vec![frightened, sickened];
        let stacked = summed(&list);
        let status: Vec<_> = stacked
            .applied()
            .iter()
            .filter(|modifier| modifier.penalty_type() == Some(PenaltyType::Status))
            .collect();
        assert_eq!(status.len(), 1);
        assert_eq!(status[0].value(), -3);
        assert_eq!(stacked.get(Statistic::Attack), -3);
        assert!(list.iter().any(|modifier| {
            modifier.condition_id() == Some(ConditionId::FRIGHTENED) && modifier.rank() == Some(3)
        }));
        assert!(list.iter().any(|modifier| {
            modifier.condition_id() == Some(ConditionId::SICKENED) && modifier.rank() == Some(1)
        }));
        let index = list
            .iter()
            .position(|modifier| modifier.condition_id() == Some(ConditionId::FRIGHTENED))
            .expect("frightened stays in the caller's list");
        list[index] = penalty(PenaltyType::Status, -2, 4, 1, Some(1), Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(2))
            .expect("condition");
        let stacked = summed(&list);
        let status: Vec<_> = stacked
            .applied()
            .iter()
            .filter(|modifier| modifier.penalty_type() == Some(PenaltyType::Status))
            .collect();
        assert_eq!(status.len(), 1);
        assert_eq!(status[0].value(), -2);
        assert_eq!(stacked.get(Statistic::Attack), -2);
        assert!(list.iter().any(|modifier| {
            modifier.condition_id() == Some(ConditionId::SICKENED) && modifier.rank() == Some(1)
        }));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 400, Penalties.
    ///
    /// This is our extension, consistent with Rules 2282 (Player Core pg. 400), not the
    /// text of Rules 2455, and it is a ruling, not a deviation.
    #[test]
    fn a_non_derived_penalty_keeps_the_larger_modifier() {
        let dice = script();
        let larger = penalty(PenaltyType::Circumstance, -4, 1, 5, None, Statistic::Attack)
            .with_condition(ConditionId(10), Some(1))
            .expect("condition");
        let smaller = penalty(PenaltyType::Circumstance, -1, 2, 5, None, Statistic::Attack)
            .with_condition(ConditionId(10), Some(2))
            .expect("condition");
        for modifiers in orders_of_two(larger, smaller) {
            let stacked = summed(&modifiers);
            let applied: Vec<_> = stacked
                .applied()
                .iter()
                .filter(|modifier| modifier.penalty_type() == Some(PenaltyType::Circumstance))
                .collect();
            assert_eq!(applied.len(), 1);
            assert_eq!(applied[0].value(), -4);
            assert_eq!(applied[0].rank(), Some(1));
            assert_eq!(stacked.get(Statistic::Attack), -4);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 442, "Redundant Conditions With Values".
    #[test]
    fn frightened_1_3_and_2_give_minus_3_in_all_six_orders() {
        let dice = script();
        let frightened_1 = penalty(PenaltyType::Status, -1, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(1))
            .expect("condition");
        let frightened_3 = penalty(PenaltyType::Status, -3, 2, 2, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(3))
            .expect("condition");
        let frightened_2 = penalty(PenaltyType::Status, -2, 3, 3, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(2))
            .expect("condition");
        for modifiers in orders_of_three(frightened_1, frightened_3, frightened_2) {
            let stacked = summed(&modifiers);
            let status: Vec<_> = stacked
                .applied()
                .iter()
                .filter(|modifier| modifier.penalty_type() == Some(PenaltyType::Status))
                .collect();
            assert_eq!(status.len(), 1);
            assert_eq!(status[0].value(), -3);
            assert_eq!(status[0].sequence(), 2);
            assert_eq!(stacked.get(Statistic::Attack), -3);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 442, "Redundant Conditions With Values".
    #[test]
    fn removing_frightened_2_leaves_frightened_1() {
        let dice = script();
        let frightened_2 = penalty(PenaltyType::Status, -2, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(2))
            .expect("condition");
        let frightened_1 = penalty(PenaltyType::Status, -1, 2, 2, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(1))
            .expect("condition");
        let mut list = vec![frightened_2, frightened_1];
        list.retain(|modifier| modifier.rank() != Some(2));
        let stacked = summed(&list);
        let status: Vec<_> = stacked
            .applied()
            .iter()
            .filter(|modifier| modifier.penalty_type() == Some(PenaltyType::Status))
            .collect();
        assert_eq!(status.len(), 1);
        assert_eq!(status[0].value(), -1);
        assert_eq!(status[0].rank(), Some(1));
        assert_eq!(status[0].condition_id(), Some(ConditionId::FRIGHTENED));
        assert_eq!(stacked.get(Statistic::Attack), -1);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 444, Frightened.
    /// Player Core pg. 446, Sickened.
    #[test]
    fn frightened_0_plus_sickened_1_gives_minus_1() {
        let dice = script();
        let frightened = penalty(PenaltyType::Status, -1, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(0))
            .expect("condition");
        let sickened = penalty(PenaltyType::Status, -1, 2, 2, None, Statistic::Attack)
            .with_condition(ConditionId::SICKENED, Some(1))
            .expect("condition");
        let stacked = summed(&[frightened, sickened]);
        assert_eq!(stacked.get(Statistic::Attack), -1);
        assert!(stacked.applied().iter().any(|modifier| {
            modifier.condition_id() == Some(ConditionId::SICKENED) && modifier.rank() == Some(1)
        }));
        assert!(
            stacked
                .applied()
                .iter()
                .all(|modifier| modifier.condition_id() != Some(ConditionId::FRIGHTENED))
        );
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 444, Frightened.
    /// Player Core pg. 446, Sickened.
    #[test]
    fn frightened_0_alone_gives_total_0() {
        let dice = script();
        let frightened = penalty(PenaltyType::Status, -1, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(0))
            .expect("condition");
        let stacked = summed(&[frightened]);
        assert_eq!(stacked.get(Statistic::Attack), 0);
        assert!(stacked.applied().is_empty());
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 442, "Redundant Conditions With Values".
    /// Player Core pg. 444, Frightened.
    /// Player Core pg. 446, Sickened.
    #[test]
    fn removing_frightened_leaves_sickened_1() {
        let dice = script();
        let frightened = penalty(PenaltyType::Status, -3, 4, 1, Some(1), Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(3))
            .expect("condition");
        let sickened = penalty(PenaltyType::Status, -1, 4, 2, Some(5), Statistic::Attack)
            .with_condition(ConditionId::SICKENED, Some(1))
            .expect("condition");
        let mut list = vec![frightened, sickened];
        let stacked = summed(&list);
        assert_eq!(stacked.get(Statistic::Attack), -3);
        list.retain(|modifier| modifier.condition_id() != Some(ConditionId::FRIGHTENED));
        let stacked = summed(&list);
        let status: Vec<_> = stacked
            .applied()
            .iter()
            .filter(|modifier| modifier.penalty_type() == Some(PenaltyType::Status))
            .collect();
        assert_eq!(status.len(), 1);
        assert_eq!(status[0].value(), -1);
        assert_eq!(stacked.get(Statistic::Attack), -1);
        assert!(list.iter().any(|modifier| {
            modifier.condition_id() == Some(ConditionId::SICKENED) && modifier.rank() == Some(1)
        }));
        assert!(
            list.iter()
                .all(|modifier| modifier.condition_id() != Some(ConditionId::FRIGHTENED))
        );
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 400, Penalties.
    #[test]
    fn a_condition_bonus_and_penalty_both_apply() {
        let dice = script();
        let bonus = bonus(BonusType::Circumstance, 2, 1, 5, None, Statistic::Attack)
            .with_condition(ConditionId(10), Some(1))
            .expect("condition");
        let penalty = penalty(PenaltyType::Circumstance, -1, 1, 5, None, Statistic::Attack)
            .with_condition(ConditionId(10), Some(1))
            .expect("condition");
        for modifiers in orders_of_two(bonus, penalty) {
            let stacked = summed(&modifiers);
            assert_eq!(stacked.applied().len(), 2);
            assert!(stacked.applied().iter().any(|modifier| {
                modifier.bonus_type() == Some(BonusType::Circumstance) && modifier.value() == 2
            }));
            assert!(stacked.applied().iter().any(|modifier| {
                modifier.penalty_type() == Some(PenaltyType::Circumstance) && modifier.value() == -1
            }));
            assert_eq!(stacked.get(Statistic::Attack), 1);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 400, Bonuses.
    /// Player Core pg. 398, "Duplicate Effects".
    ///
    /// Substitute: rule, Duplicate Effects and same-type stacking name no further instance
    /// once the bonus or penalty is chosen; why not, they are silent when those match;
    /// substitute, the lower source id wins; intent, every input order reports the same
    /// modifier and the same total.
    #[test]
    fn a_tied_circumstance_bonus_reports_the_lower_source_in_both_orders() {
        let dice = script();
        let source_1 = bonus(BonusType::Circumstance, 2, 1, 3, None, Statistic::Attack);
        let source_2 = bonus(BonusType::Circumstance, 2, 2, 3, None, Statistic::Attack);
        for modifiers in orders_of_two(source_1, source_2) {
            let stacked = summed(&modifiers);
            let applied: Vec<_> = stacked
                .applied()
                .iter()
                .filter(|modifier| modifier.bonus_type() == Some(BonusType::Circumstance))
                .collect();
            assert_eq!(applied.len(), 1);
            assert_eq!(applied[0].source(), Some(SourceId(1)));
            assert_eq!(stacked.get(Statistic::Attack), 2);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 398, "Duplicate Effects".
    #[test]
    fn every_input_order_gives_an_identical_result() {
        let dice = script();
        let mut modifiers = [
            penalty(PenaltyType::Circumstance, -1, 1, 5, None, Statistic::Attack)
                .with_condition(ConditionId(10), Some(1))
                .expect("condition"),
            penalty(PenaltyType::Circumstance, -4, 2, 5, None, Statistic::Attack)
                .with_condition(ConditionId(10), Some(1))
                .expect("condition"),
            penalty(PenaltyType::Status, -1, 1, 5, None, Statistic::Attack)
                .with_condition(ConditionId::FRIGHTENED, Some(2))
                .expect("condition"),
            penalty(PenaltyType::Status, -1, 2, 5, None, Statistic::Attack)
                .with_condition(ConditionId::FRIGHTENED, Some(2))
                .expect("condition"),
            bonus(BonusType::Circumstance, 2, 1, 3, None, Statistic::Attack),
            bonus(BonusType::Circumstance, 2, 2, 3, None, Statistic::Attack),
            penalty(PenaltyType::Status, -3, 4, 1, Some(1), Statistic::Attack)
                .with_condition(ConditionId::FRIGHTENED, Some(3))
                .expect("condition"),
            penalty(PenaltyType::Status, -1, 4, 2, Some(5), Statistic::Attack)
                .with_condition(ConditionId::SICKENED, Some(1))
                .expect("condition"),
        ];
        let first = summed(&modifiers);
        for_each_order(&mut modifiers, &mut |order| {
            assert_eq!(summed(order), first);
        });
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 400, Bonuses.
    ///
    /// Sequence wins before source id.
    #[test]
    fn the_lowest_sequence_beats_the_lower_source_id() {
        let dice = script();
        let earlier = bonus(BonusType::Circumstance, 2, 2, 1, None, Statistic::Attack);
        let later = bonus(BonusType::Circumstance, 2, 1, 4, None, Statistic::Attack);
        for modifiers in orders_of_two(earlier, later) {
            let stacked = summed(&modifiers);
            let applied: Vec<_> = stacked
                .applied()
                .iter()
                .filter(|modifier| modifier.bonus_type() == Some(BonusType::Circumstance))
                .collect();
            assert_eq!(applied.len(), 1);
            assert_eq!(applied[0].source(), Some(SourceId(2)));
            assert_eq!(applied[0].sequence(), 1);
            assert_eq!(stacked.get(Statistic::Attack), 2);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 400, Penalties.
    #[test]
    fn a_circumstance_and_status_tie_totals_minus_5_in_every_order() {
        let dice = script();
        let circumstance = penalty(PenaltyType::Circumstance, -2, 1, 5, None, Statistic::Attack)
            .with_condition(ConditionId(10), Some(1))
            .expect("condition");
        let status = penalty(PenaltyType::Status, -2, 1, 5, None, Statistic::Attack)
            .with_condition(ConditionId(10), Some(1))
            .expect("condition");
        let worse = penalty(PenaltyType::Status, -3, 2, 1, None, Statistic::Attack);
        let orders = orders_of_three(circumstance, status, worse);
        let first = summed(&orders[0]);
        for modifiers in orders {
            let stacked = summed(&modifiers);
            assert_eq!(stacked, first);
            assert!(stacked.applied().iter().any(|modifier| {
                modifier.penalty_type() == Some(PenaltyType::Circumstance) && modifier.value() == -2
            }));
            assert!(stacked.applied().iter().any(|modifier| {
                modifier.penalty_type() == Some(PenaltyType::Status) && modifier.value() == -3
            }));
            assert!(stacked.applied().iter().all(|modifier| {
                !(modifier.penalty_type() == Some(PenaltyType::Status) && modifier.value() == -2)
            }));
            assert_eq!(stacked.get(Statistic::Attack), -5);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 398, "Duplicate Effects".
    ///
    /// The higher level is an engineering tiebreak with no rules effect.
    #[test]
    fn a_same_type_tie_keeps_the_higher_level() {
        let dice = script();
        let lower = penalty(PenaltyType::Status, -2, 1, 5, Some(1), Statistic::Attack)
            .with_condition(ConditionId(10), Some(1))
            .expect("condition");
        let higher = penalty(PenaltyType::Status, -2, 1, 5, Some(4), Statistic::Attack)
            .with_condition(ConditionId(10), Some(1))
            .expect("condition");
        for modifiers in orders_of_two(lower, higher) {
            let stacked = summed(&modifiers);
            let status: Vec<_> = stacked
                .applied()
                .iter()
                .filter(|modifier| modifier.penalty_type() == Some(PenaltyType::Status))
                .collect();
            assert_eq!(status.len(), 1);
            assert_eq!(status[0].level(), Some(4));
            assert_eq!(stacked.get(Statistic::Attack), -2);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 400, Penalties.
    #[test]
    fn a_circumstance_and_item_tie_totals_minus_5_in_every_order() {
        let dice = script();
        let circumstance = penalty(PenaltyType::Circumstance, -2, 1, 5, None, Statistic::Attack)
            .with_condition(ConditionId(10), Some(1))
            .expect("condition");
        let item = penalty(PenaltyType::Item, -2, 1, 5, None, Statistic::Attack)
            .with_condition(ConditionId(10), Some(1))
            .expect("condition");
        let worse = penalty(PenaltyType::Circumstance, -3, 9, 1, None, Statistic::Attack);
        let orders = orders_of_three(circumstance, item, worse);
        let first = summed(&orders[0]);
        for modifiers in orders {
            let stacked = summed(&modifiers);
            assert_eq!(stacked, first);
            assert!(stacked.applied().iter().any(|modifier| {
                modifier.penalty_type() == Some(PenaltyType::Item)
                    && modifier.value() == -2
                    && modifier.source() == Some(SourceId(1))
                    && modifier.condition_id() == Some(ConditionId(10))
            }));
            assert!(stacked.applied().iter().any(|modifier| {
                modifier.penalty_type() == Some(PenaltyType::Circumstance)
                    && modifier.value() == -3
                    && modifier.source() == Some(SourceId(9))
            }));
            assert!(stacked.applied().iter().all(|modifier| {
                !(modifier.penalty_type() == Some(PenaltyType::Circumstance)
                    && modifier.value() == -2)
            }));
            assert_eq!(stacked.get(Statistic::Attack), -5);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 442, "Redundant Conditions With Values".
    /// Player Core pg. 444, Frightened.
    #[test]
    fn a_frightened_level_tie_keeps_level_3() {
        let dice = script();
        let lower = penalty(PenaltyType::Status, -1, 1, 5, Some(1), Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(2))
            .expect("condition");
        let higher = penalty(PenaltyType::Status, -1, 1, 5, Some(3), Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(2))
            .expect("condition");
        for modifiers in orders_of_two(lower, higher) {
            let stacked = summed(&modifiers);
            let status: Vec<_> = stacked
                .applied()
                .iter()
                .filter(|modifier| modifier.penalty_type() == Some(PenaltyType::Status))
                .collect();
            assert_eq!(status.len(), 1);
            assert_eq!(status[0].level(), Some(3));
            assert_eq!(status[0].value(), -2);
            assert_eq!(stacked.get(Statistic::Attack), -2);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 442, "Redundant Conditions With Values".
    /// Player Core pg. 398, "Duplicate Effects".
    #[test]
    fn two_identical_condition_penalties_apply_once() {
        let dice = script();
        let penalty = penalty(PenaltyType::Status, -2, 1, 5, Some(1), Statistic::Attack)
            .with_condition(ConditionId(10), Some(1))
            .expect("condition");
        for modifiers in orders_of_two(penalty, penalty) {
            let stacked = summed(&modifiers);
            assert_eq!(stacked.applied().len(), 1);
            assert_eq!(stacked.get(Statistic::Attack), -2);
        }
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 402, "Multiple Attack Penalty".
    /// Player Core pg. 403, "Range Penalty".
    #[test]
    fn a_second_attack_with_no_caller_map_gets_minus_5_once() {
        let dice = script();
        let map = second_non_agile_map();
        assert_eq!(map, -5);
        let total =
            attack_check_total(10, term(0), term(map), term(0), term(0), &[]).expect("total fits");
        assert_eq!(total, 5);
        assert_ne!(total, 0);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 400, Penalties.
    #[test]
    fn an_internal_circumstance_penalty_and_a_caller_circumstance_penalty_total_minus_3() {
        let dice = script();
        let caller = penalty(PenaltyType::Circumstance, -3, 1, 4, None, Statistic::Attack);
        let total = attack_check_total(10, term(0), term(0), term(-2), term(0), &[caller])
            .expect("total fits");
        assert_eq!(total, 7);
        assert_ne!(total, 5);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 400, Penalties.
    /// Player Core pg. 402, "Multiple Attack Penalty".
    /// Player Core pg. 403, "Range Penalty".
    /// Player Core pg. 398, Duplicate Effects.
    ///
    /// Substitute: rule, Duplicate Effects keys a repeated effect on its source; why not,
    /// these internal terms are not caller effects; substitute, each internal term uses a
    /// reserved source no caller can construct; intent, caller source 1 cannot collapse an
    /// internal term.
    #[test]
    fn a_caller_source_1_does_not_collapse_an_internal_term() {
        let dice = script();
        let caller = penalty(PenaltyType::Untyped, -3, 1, 4, None, Statistic::Attack);
        assert_eq!(caller.source(), Some(SourceId(1)));
        let total = attack_check_total(10, term(0), term(-5), term(-2), term(-2), &[caller])
            .expect("total fits");
        assert_eq!(total, -2);
        assert_ne!(total, 1);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Penalties.
    #[test]
    fn an_empty_caller_list_keeps_the_stacked_total() {
        let dice = script();
        let total =
            attack_check_total(12, term(9), term(-5), term(-2), term(0), &[]).expect("total fits");
        assert_eq!(total, 14);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Penalties.
    #[test]
    fn a_caller_untyped_penalty_stacks_with_the_multiple_attack_penalty() {
        let dice = script();
        let caller = penalty(PenaltyType::Untyped, -2, 1, 1, None, Statistic::Attack);
        let total = attack_check_total(12, term(9), term(-5), term(0), term(0), &[caller])
            .expect("total fits");
        assert_eq!(total, 14);
        assert_ne!(total, 16);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Penalties.
    #[test]
    fn two_caller_circumstance_bonuses_keep_the_higher() {
        let dice = script();
        let higher = bonus(BonusType::Circumstance, 4, 1, 1, None, Statistic::Attack);
        let lower = bonus(BonusType::Circumstance, 1, 2, 2, None, Statistic::Attack);
        let total = attack_check_total(10, term(0), term(0), term(0), term(0), &[higher, lower])
            .expect("total fits");
        assert_eq!(total, 14);
        assert_ne!(total, 15);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Penalties.
    #[test]
    fn a_caller_circumstance_bonus_and_an_internal_circumstance_penalty_both_apply() {
        let dice = script();
        let caller = bonus(BonusType::Circumstance, 2, 1, 1, None, Statistic::Attack);
        let total = attack_check_total(10, term(0), term(0), term(-2), term(0), &[caller])
            .expect("total fits");
        assert_eq!(total, 10);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Penalties.
    #[test]
    fn an_armor_class_modifier_does_not_change_the_attack_total() {
        let dice = script();
        let caller = penalty(
            PenaltyType::Circumstance,
            -2,
            1,
            1,
            None,
            Statistic::ArmorClass,
        );
        let total = attack_check_total(10, term(0), term(0), term(0), term(0), &[caller])
            .expect("total fits");
        assert_eq!(total, 10);
        assert_ne!(total, 8);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Penalties.
    #[test]
    fn an_overflowing_caller_list_returns_an_error() {
        let dice = script();
        let first = penalty(
            PenaltyType::Untyped,
            i32::MIN,
            1,
            1,
            None,
            Statistic::Attack,
        );
        let second = penalty(
            PenaltyType::Untyped,
            i32::MIN,
            2,
            2,
            None,
            Statistic::Attack,
        );
        let result = attack_check_total(12, term(0), term(0), term(0), term(0), &[first, second]);
        assert_eq!(result, Err(ModifierError::Overflow));
        assert_ne!(result, Ok(12));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Penalties.
    #[test]
    fn the_same_caller_list_totals_twice() {
        let dice = script();
        let caller = penalty(PenaltyType::Circumstance, -2, 3, 1, None, Statistic::Attack);
        let list = [caller];
        let first =
            attack_check_total(10, term(4), term(-5), term(0), term(0), &list).expect("total fits");
        let second =
            attack_check_total(10, term(4), term(-5), term(0), term(0), &list).expect("total fits");
        assert_eq!(first, second);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 400, Penalties.
    /// Rules 2282.
    #[test]
    fn a_condition_10_circumstance_penalty_does_not_add_to_an_unconditioned_one() {
        let dice = script();
        let conditioned = penalty(PenaltyType::Circumstance, -2, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId(10), Some(1))
            .expect("condition");
        let plain = penalty(PenaltyType::Circumstance, -3, 2, 2, None, Statistic::Attack);
        let stacked = summed(&[conditioned, plain]);
        let circumstance: Vec<_> = stacked
            .applied()
            .iter()
            .filter(|modifier| modifier.penalty_type() == Some(PenaltyType::Circumstance))
            .collect();
        assert_eq!(circumstance.len(), 1);
        assert_eq!(stacked.get(Statistic::Attack), -3);
        assert_ne!(stacked.get(Statistic::Attack), -5);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 442, "Redundant Conditions With Values".
    /// Player Core pg. 444, Frightened.
    /// Player Core pg. 400, Bonuses and Penalties.
    #[test]
    fn permuted_sets_match_the_stacking_oracle() {
        let dice = script();
        let seed = 0x0D1A_7E5B_u64;
        let sets = generated_sets(seed);
        assert_eq!(sets, generated_sets(seed));
        assert!(sets.iter().any(|set| is_untyped_exact_duplicate(set)));
        assert!(sets.iter().any(|set| is_conditioned_exact_duplicate(set)));
        assert!(sets.iter().any(|set| both_levels_absent(set)));
        let mut reserved = 0usize;
        let mut frightened_ranks = [0usize; 3];
        let mut sickened_ranks = [0usize; 3];
        let mut built = 0usize;
        let mut circumstance_pairs = 0usize;
        let mut item_pairs = 0usize;
        let mut status_pairs = 0usize;
        let mut untyped_pairs = 0usize;
        let mut circumstance_bonus_pairs = 0usize;
        let mut item_bonus_pairs = 0usize;
        let mut status_bonus_pairs = 0usize;
        let mut sickened_sets = 0usize;
        for set in &sets {
            assert!(set.len() <= 4);
            if two_on_one_statistic(set, |modifier| {
                modifier.penalty_type() == Some(PenaltyType::Circumstance)
            }) {
                circumstance_pairs += 1;
            }
            if two_on_one_statistic(set, |modifier| {
                modifier.penalty_type() == Some(PenaltyType::Item)
            }) {
                item_pairs += 1;
            }
            if two_on_one_statistic(set, |modifier| {
                modifier.penalty_type() == Some(PenaltyType::Status)
            }) {
                status_pairs += 1;
            }
            if two_on_one_statistic(set, |modifier| {
                modifier.penalty_type() == Some(PenaltyType::Untyped)
            }) {
                untyped_pairs += 1;
            }
            if two_on_one_statistic(set, |modifier| {
                modifier.bonus_type() == Some(BonusType::Circumstance)
            }) {
                circumstance_bonus_pairs += 1;
            }
            if two_on_one_statistic(set, |modifier| {
                modifier.bonus_type() == Some(BonusType::Item)
            }) {
                item_bonus_pairs += 1;
            }
            if two_on_one_statistic(set, |modifier| {
                modifier.bonus_type() == Some(BonusType::Status)
            }) {
                status_bonus_pairs += 1;
            }
            if set
                .iter()
                .any(|modifier| modifier.condition_id() == Some(ConditionId::SICKENED))
            {
                sickened_sets += 1;
            }
            for modifier in set {
                built += 1;
                assert_ne!(modifier.value(), 0);
                if modifier.bonus_type().is_some() {
                    assert_ne!(modifier.penalty_type(), Some(PenaltyType::Untyped));
                }
                if modifier.source().is_none() {
                    reserved += 1;
                }
                let valued_bonus = modifier.bonus_type().is_some()
                    && matches!(
                        modifier.condition_id(),
                        Some(ConditionId::FRIGHTENED | ConditionId::SICKENED)
                    );
                assert!(
                    !valued_bonus,
                    "generator drew a frightened or sickened bonus"
                );
                if modifier.condition_id() == Some(ConditionId::FRIGHTENED) {
                    assert_eq!(modifier.penalty_type(), Some(PenaltyType::Status));
                    let rank = modifier.rank().expect("frightened rank");
                    assert!(rank <= 2);
                    frightened_ranks[rank as usize] += 1;
                }
                if modifier.condition_id() == Some(ConditionId::SICKENED) {
                    assert_eq!(modifier.penalty_type(), Some(PenaltyType::Status));
                    let rank = modifier.rank().expect("sickened rank");
                    assert!(rank <= 2);
                    sickened_ranks[rank as usize] += 1;
                }
            }
        }
        assert!(reserved >= 1);
        assert!(frightened_ranks.iter().all(|count| *count >= 1));
        assert!(sickened_ranks.iter().all(|count| *count >= 1));
        let set_count = sets.len();
        println!(
            "property-test seed=0x0D1A_7E5B sets={set_count} modifiers={built} reserved={reserved} frightened_ranks={frightened_ranks:?} sickened_ranks={sickened_ranks:?} pairs circumstance={circumstance_pairs} item={item_pairs} status={status_pairs} untyped={untyped_pairs} bonus circumstance={circumstance_bonus_pairs} item={item_bonus_pairs} status={status_bonus_pairs} sickened_sets={sickened_sets}"
        );
        assert!(
            circumstance_pairs >= 50,
            "circumstance penalty pairs {circumstance_pairs}"
        );
        assert!(item_pairs >= 50, "item penalty pairs {item_pairs}");
        assert!(status_pairs >= 50, "status penalty pairs {status_pairs}");
        assert!(untyped_pairs >= 50, "untyped penalty pairs {untyped_pairs}");
        assert!(
            circumstance_bonus_pairs >= 50,
            "circumstance bonus pairs {circumstance_bonus_pairs}"
        );
        assert!(
            item_bonus_pairs >= 50,
            "item bonus pairs {item_bonus_pairs}"
        );
        assert!(
            status_bonus_pairs >= 50,
            "status bonus pairs {status_bonus_pairs}"
        );
        assert!(sickened_sets >= 50, "sickened sets {sickened_sets}");
        for set in &sets {
            let mut modifiers = set.clone();
            let first = summed(&modifiers);
            for_each_order(&mut modifiers, &mut |order| {
                let stacked = summed(order);
                assert_eq!(stacked, first);
                for statistic in [
                    Statistic::Attack,
                    Statistic::ArmorClass,
                    Statistic::Perception,
                    Statistic::Damage,
                    Statistic::Reflex,
                    Statistic::Fortitude,
                    Statistic::Will,
                ] {
                    assert_eq!(stacked.get(statistic), oracle_total(order, statistic));
                }
            });
        }
        assert_eq!(dice.remaining(), 1);
    }

    fn two_on_one_statistic(set: &[Modifier], matches: impl Fn(Modifier) -> bool) -> bool {
        let mut attack = 0usize;
        let mut armor_class = 0usize;
        let mut perception = 0usize;
        let mut damage = 0usize;
        let mut reflex = 0usize;
        let mut fortitude = 0usize;
        let mut will = 0usize;
        for modifier in set.iter().copied() {
            if !matches(modifier) {
                continue;
            }
            let count = match modifier.statistic() {
                Statistic::Attack => &mut attack,
                Statistic::ArmorClass => &mut armor_class,
                Statistic::Perception => &mut perception,
                Statistic::Damage => &mut damage,
                Statistic::Reflex => &mut reflex,
                Statistic::Fortitude => &mut fortitude,
                Statistic::Will => &mut will,
            };
            *count += 1;
            if *count >= 2 {
                return true;
            }
        }
        false
    }

    fn generated_sets(seed: u64) -> Vec<Vec<Modifier>> {
        let mut rng = Lcg(seed);
        let mut sets = vec![
            untyped_exact_duplicates(),
            both_levels_absent_set(),
            conditioned_exact_duplicates(),
        ];
        // Item, circumstance, and untyped pairs, and each bonus type, are uncommon
        // in a set of at most four. Keep drawing from this seed so each count can clear 50.
        for _ in 0..16_000 {
            let count = rng.below(5) as usize;
            sets.push((0..count).map(|_| random_modifier(&mut rng)).collect());
        }
        sets
    }

    fn untyped_exact_duplicates() -> Vec<Modifier> {
        let penalty = penalty(PenaltyType::Untyped, -2, 4, 3, None, Statistic::Attack);
        vec![penalty, penalty]
    }

    fn both_levels_absent_set() -> Vec<Modifier> {
        vec![
            penalty(PenaltyType::Status, -1, 1, 5, None, Statistic::Attack)
                .with_condition(ConditionId::FRIGHTENED, Some(2))
                .expect("condition"),
            penalty(PenaltyType::Status, -4, 1, 5, None, Statistic::Attack)
                .with_condition(ConditionId::FRIGHTENED, Some(2))
                .expect("condition"),
        ]
    }

    fn conditioned_exact_duplicates() -> Vec<Modifier> {
        let modifier = penalty(
            PenaltyType::Circumstance,
            -2,
            4,
            3,
            Some(2),
            Statistic::Attack,
        )
        .with_condition(ConditionId(10), Some(1))
        .expect("condition");
        vec![modifier, modifier]
    }

    fn is_untyped_exact_duplicate(set: &[Modifier]) -> bool {
        set.len() == 2
            && set[0] == set[1]
            && set[0].condition_id().is_none()
            && set[0].penalty_type() == Some(PenaltyType::Untyped)
    }

    fn is_conditioned_exact_duplicate(set: &[Modifier]) -> bool {
        set.len() == 2 && set[0] == set[1] && set[0].condition_id().is_some()
    }

    fn both_levels_absent(set: &[Modifier]) -> bool {
        set.len() == 2 && set.iter().all(|modifier| modifier.level().is_none())
    }

    fn random_modifier(rng: &mut Lcg) -> Modifier {
        let statistic = if rng.below(2) == 0 {
            Statistic::Attack
        } else {
            Statistic::ArmorClass
        };
        let source = random_source(rng);
        let sequence = 1 + rng.below(5) as u32;
        let level = match rng.below(3) {
            0 => None,
            _ => Some(1 + rng.below(4) as u32),
        };
        let magnitude = 1 + rng.below(5) as i32;
        match rng.below(5) {
            0 => frightened_status_penalty(rng, magnitude, source, sequence, level, statistic),
            1 => sickened_status_penalty(rng, magnitude, source, sequence, level, statistic),
            2 => conditioned_other(rng, magnitude, source, sequence, level, statistic),
            _ => unconditioned(rng, magnitude, source, sequence, level, statistic),
        }
    }

    fn sickened_status_penalty(
        rng: &mut Lcg,
        magnitude: i32,
        source: ModifierSource,
        sequence: u32,
        level: Option<u32>,
        statistic: Statistic,
    ) -> Modifier {
        let base = Modifier::penalty_from(
            PenaltyType::Status,
            -magnitude,
            source,
            sequence,
            level,
            statistic,
        )
        .expect("penalty");
        let rank = Some(rng.below(3) as u32);
        built_ok(base.with_condition(ConditionId::SICKENED, rank))
    }

    fn random_source(rng: &mut Lcg) -> ModifierSource {
        match rng.below(12) {
            0..=5 => ModifierSource::Caller(SourceId(1 + rng.below(6) as u32)),
            6 => ModifierSource::MultipleAttackPenalty,
            7 => ModifierSource::RangePenalty,
            8 => ModifierSource::AttackCircumstance,
            9 => ModifierSource::Prone,
            10 => ModifierSource::Aid,
            _ => ModifierSource::Cover,
        }
    }

    fn frightened_status_penalty(
        rng: &mut Lcg,
        magnitude: i32,
        source: ModifierSource,
        sequence: u32,
        level: Option<u32>,
        statistic: Statistic,
    ) -> Modifier {
        let base = Modifier::penalty_from(
            PenaltyType::Status,
            -magnitude,
            source,
            sequence,
            level,
            statistic,
        )
        .expect("penalty");
        let rank = Some(rng.below(3) as u32);
        built_ok(base.with_condition(ConditionId::FRIGHTENED, rank))
    }

    fn conditioned_other(
        rng: &mut Lcg,
        magnitude: i32,
        source: ModifierSource,
        sequence: u32,
        level: Option<u32>,
        statistic: Statistic,
    ) -> Modifier {
        let base = unconditioned(rng, magnitude, source, sequence, level, statistic);
        let rank = Some(1 + rng.below(3) as u32);
        built_ok(base.with_condition(ConditionId(10), rank))
    }

    fn unconditioned(
        rng: &mut Lcg,
        magnitude: i32,
        source: ModifierSource,
        sequence: u32,
        level: Option<u32>,
        statistic: Statistic,
    ) -> Modifier {
        if rng.below(2) == 0 {
            let ty = match rng.below(3) {
                0 => BonusType::Circumstance,
                1 => BonusType::Item,
                _ => BonusType::Status,
            };
            Modifier::bonus_from(ty, magnitude, source, sequence, level, statistic).expect("bonus")
        } else {
            let ty = match rng.below(4) {
                0 => PenaltyType::Circumstance,
                1 => PenaltyType::Item,
                2 => PenaltyType::Status,
                _ => PenaltyType::Untyped,
            };
            Modifier::penalty_from(ty, -magnitude, source, sequence, level, statistic)
                .expect("penalty")
        }
    }

    fn built_ok(result: Result<Modifier, ModifierError>) -> Modifier {
        assert!(result.is_ok(), "generated modifier builds without error");
        result.expect("generated modifier builds")
    }

    /// Frightened and sickened penalties use status -rank, and rank 0 is dropped. The
    /// generator never draws a frightened or sickened bonus, so this oracle has no branch
    /// that keeps one. (a) Group by condition. (b) Collapse the duplicate key. (c) Best
    /// bonus, worst penalty, and every untyped penalty.
    /// This table is the test's own source order. It does not call the production helpers.
    fn oracle_total(modifiers: &[Modifier], statistic: Statistic) -> i32 {
        let mut sum = 0i64;
        for modifier in stack_best_and_worst(&collapse_duplicate_keys(group_conditions(modifiers)))
        {
            if modifier.statistic() == statistic {
                sum += i64::from(modifier.value());
            }
        }
        i32::try_from(sum).expect("oracle fits")
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    struct KindKey {
        bonus: bool,
        ord: u8,
    }

    struct ConditionGroup {
        id: ConditionId,
        statistic: Statistic,
        kind: KindKey,
        modifiers: Vec<Modifier>,
    }

    fn group_conditions(modifiers: &[Modifier]) -> Vec<Modifier> {
        let mut plain = Vec::new();
        let mut groups: Vec<ConditionGroup> = Vec::new();
        for modifier in modifiers.iter().copied() {
            if valued_penalty_rank(modifier) == Some(0) {
                continue;
            }
            let Some(id) = modifier.condition_id() else {
                plain.push(modifier);
                continue;
            };
            let kind = kind_key(modifier);
            match groups.iter_mut().find(|group| {
                group.id == id && group.statistic == modifier.statistic() && group.kind == kind
            }) {
                Some(group) => group.modifiers.push(modifier),
                None => groups.push(ConditionGroup {
                    id,
                    statistic: modifier.statistic(),
                    kind,
                    modifiers: vec![modifier],
                }),
            }
        }
        for group in groups {
            let mut best = group.modifiers[0];
            for candidate in group.modifiers.into_iter().skip(1) {
                if condition_group_wins(candidate, best) {
                    best = candidate;
                }
            }
            if let Some(rewritten) = rewrite_valued_penalty(best) {
                plain.push(rewritten);
            }
        }
        plain
    }

    fn kind_key(modifier: Modifier) -> KindKey {
        match modifier.bonus_type() {
            Some(BonusType::Circumstance) => KindKey {
                bonus: true,
                ord: 0,
            },
            Some(BonusType::Item) => KindKey {
                bonus: true,
                ord: 1,
            },
            Some(BonusType::Status) => KindKey {
                bonus: true,
                ord: 2,
            },
            None => match modifier.penalty_type() {
                Some(PenaltyType::Circumstance) => KindKey {
                    bonus: false,
                    ord: 0,
                },
                Some(PenaltyType::Item) => KindKey {
                    bonus: false,
                    ord: 1,
                },
                Some(PenaltyType::Status) => KindKey {
                    bonus: false,
                    ord: 2,
                },
                Some(PenaltyType::Untyped) => KindKey {
                    bonus: false,
                    ord: 3,
                },
                None => KindKey {
                    bonus: false,
                    ord: 4,
                },
            },
        }
    }

    fn valued_penalty_rank(modifier: Modifier) -> Option<u32> {
        modifier.penalty_type()?;
        let id = modifier.condition_id()?;
        if id == ConditionId::FRIGHTENED || id == ConditionId::SICKENED {
            modifier.rank()
        } else {
            None
        }
    }

    fn grouped_penalty_value(modifier: Modifier) -> i32 {
        match valued_penalty_rank(modifier) {
            Some(rank) => -i32::try_from(rank).expect("rank fits"),
            None => modifier.value(),
        }
    }

    fn rewrite_valued_penalty(mut modifier: Modifier) -> Option<Modifier> {
        let Some(rank) = valued_penalty_rank(modifier) else {
            return Some(modifier);
        };
        if rank == 0 {
            return None;
        }
        let rank = i32::try_from(rank).expect("rank fits");
        modifier.value = rank.checked_neg().expect("negation fits");
        modifier.kind = ModifierKind::Penalty(PenaltyType::Status);
        Some(modifier)
    }

    fn condition_group_wins(candidate: Modifier, current: Modifier) -> bool {
        let candidate_value = grouped_penalty_value(candidate);
        let current_value = grouped_penalty_value(current);
        if candidate_value != current_value {
            return if candidate.bonus_type().is_some() {
                candidate_value > current_value
            } else {
                candidate_value < current_value
            };
        }
        match (candidate.rank(), current.rank()) {
            (Some(candidate_rank), Some(current_rank)) if candidate_rank != current_rank => {
                return candidate_rank > current_rank;
            }
            (Some(_), None) => return true,
            (None, Some(_)) => return false,
            _ => {}
        }
        if candidate.sequence() != current.sequence() {
            return candidate.sequence() > current.sequence();
        }
        let candidate_source = caller_then_reserved(candidate.source);
        let current_source = caller_then_reserved(current.source);
        if candidate_source != current_source {
            return candidate_source < current_source;
        }
        match (candidate.level(), current.level()) {
            (Some(candidate_level), Some(current_level)) if candidate_level != current_level => {
                return candidate_level > current_level;
            }
            (Some(_), None) => return true,
            (None, Some(_)) => return false,
            _ => {}
        }
        duplicate_key_wins(candidate, current)
    }

    fn collapse_duplicate_keys(modifiers: Vec<Modifier>) -> Vec<Modifier> {
        let mut kept = Vec::new();
        for modifier in modifiers {
            match kept
                .iter_mut()
                .find(|existing| duplicate_key_matches(**existing, modifier))
            {
                Some(existing) => {
                    if duplicate_key_wins(modifier, *existing) {
                        *existing = modifier;
                    }
                }
                None => kept.push(modifier),
            }
        }
        kept
    }

    fn duplicate_key_matches(left: Modifier, right: Modifier) -> bool {
        left.source == right.source
            && left.statistic() == right.statistic()
            && left.bonus_type() == right.bonus_type()
            && left.penalty_type() == right.penalty_type()
            && left.condition_id() == right.condition_id()
    }

    fn duplicate_key_wins(candidate: Modifier, current: Modifier) -> bool {
        match (candidate.level(), current.level()) {
            (Some(candidate_level), Some(current_level)) if candidate_level != current_level => {
                return candidate_level > current_level;
            }
            (Some(_), None) => return true,
            (None, Some(_)) => return false,
            _ => {}
        }
        match (candidate.rank(), current.rank()) {
            (Some(candidate_rank), Some(current_rank)) if candidate_rank != current_rank => {
                return candidate_rank > current_rank;
            }
            (Some(_), None) => return true,
            (None, Some(_)) => return false,
            _ => {}
        }
        if candidate.sequence() != current.sequence() {
            return candidate.sequence() > current.sequence();
        }
        if candidate.value() != current.value() {
            return if candidate.bonus_type().is_some() {
                candidate.value() > current.value()
            } else {
                candidate.value() < current.value()
            };
        }
        match (candidate.condition_id(), current.condition_id()) {
            (Some(candidate_id), Some(current_id)) if candidate_id != current_id => {
                return candidate_id < current_id;
            }
            (Some(_), None) => return true,
            (None, Some(_)) => return false,
            _ => {}
        }
        caller_then_reserved(candidate.source) < caller_then_reserved(current.source)
    }

    fn stack_best_and_worst(modifiers: &[Modifier]) -> Vec<Modifier> {
        let mut selected = Vec::new();
        for statistic in [
            Statistic::Attack,
            Statistic::ArmorClass,
            Statistic::Perception,
            Statistic::Damage,
            Statistic::Reflex,
            Statistic::Fortitude,
            Statistic::Will,
        ] {
            let on_statistic: Vec<_> = modifiers
                .iter()
                .copied()
                .filter(|modifier| modifier.statistic() == statistic)
                .collect();
            selected.extend(stack_one_statistic(&on_statistic));
        }
        selected
    }

    fn stack_one_statistic(modifiers: &[Modifier]) -> Vec<Modifier> {
        let mut best_bonus = [None, None, None];
        let mut worst_penalty = [None, None, None];
        let mut untyped = Vec::new();
        for modifier in modifiers.iter().copied() {
            match modifier.bonus_type() {
                Some(BonusType::Circumstance) => {
                    prefer_higher(&mut best_bonus[0], modifier);
                }
                Some(BonusType::Item) => prefer_higher(&mut best_bonus[1], modifier),
                Some(BonusType::Status) => prefer_higher(&mut best_bonus[2], modifier),
                None => match modifier.penalty_type() {
                    Some(PenaltyType::Circumstance) => {
                        prefer_lower(&mut worst_penalty[0], modifier);
                    }
                    Some(PenaltyType::Item) => prefer_lower(&mut worst_penalty[1], modifier),
                    Some(PenaltyType::Status) => prefer_lower(&mut worst_penalty[2], modifier),
                    Some(PenaltyType::Untyped) => untyped.push(modifier),
                    None => {}
                },
            }
        }
        let mut applied = Vec::new();
        for modifier in best_bonus.into_iter().chain(worst_penalty).flatten() {
            applied.push(modifier);
        }
        applied.extend(untyped);
        applied
    }

    fn prefer_higher(slot: &mut Option<Modifier>, candidate: Modifier) {
        match *slot {
            Some(current) if !higher_bonus_wins(candidate, current) => {}
            _ => *slot = Some(candidate),
        }
    }

    fn prefer_lower(slot: &mut Option<Modifier>, candidate: Modifier) {
        match *slot {
            Some(current) if !lower_penalty_wins(candidate, current) => {}
            _ => *slot = Some(candidate),
        }
    }

    fn higher_bonus_wins(candidate: Modifier, current: Modifier) -> bool {
        if candidate.value() != current.value() {
            candidate.value() > current.value()
        } else {
            equal_value_order(candidate, current)
        }
    }

    fn lower_penalty_wins(candidate: Modifier, current: Modifier) -> bool {
        if candidate.value() != current.value() {
            candidate.value() < current.value()
        } else {
            equal_value_order(candidate, current)
        }
    }

    fn equal_value_order(candidate: Modifier, current: Modifier) -> bool {
        if candidate.sequence() != current.sequence() {
            return candidate.sequence() < current.sequence();
        }
        match (candidate.condition_id(), current.condition_id()) {
            (Some(candidate_id), Some(current_id)) if candidate_id != current_id => {
                return candidate_id < current_id;
            }
            (Some(_), None) => return true,
            (None, Some(_)) => return false,
            _ => {}
        }
        caller_then_reserved(candidate.source) < caller_then_reserved(current.source)
    }

    /// Caller ids first, by number, then the reserved variants in a fixed order.
    fn caller_then_reserved(source: ModifierSource) -> (u8, u32) {
        match source {
            ModifierSource::Caller(id) => (0, id.0),
            ModifierSource::MultipleAttackPenalty => (1, 0),
            ModifierSource::Prone => (2, 0),
            ModifierSource::Aid => (3, 0),
            ModifierSource::Cover => (4, 0),
            ModifierSource::OffGuard => (5, 0),
            ModifierSource::UnconsciousArmorClass => (6, 0),
            ModifierSource::UnconsciousPerception => (7, 0),
            ModifierSource::RangePenalty => (8, 0),
            ModifierSource::AttackCircumstance => (9, 0),
            ModifierSource::UnconsciousReflex => (10, 0),
            ModifierSource::RaisedShield => (11, 0),
        }
    }

    /// Player Core pg. 446, Unconscious.
    #[test]
    fn a_conscious_creature_adds_no_reflex_penalty() {
        let dice = script();
        let stacked = summed(&reflex_modifiers(false).expect("reflex"));
        assert_eq!(stacked.get(Statistic::Reflex), 0);
        assert!(stacked.applied().is_empty());
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 446, Unconscious.
    #[test]
    fn reflex_and_armor_class_both_keep_the_penalty() {
        let dice = script();
        let reflex = summed(&reflex_modifiers(true).expect("reflex"));
        let armor_class = summed(&ac_modifiers(None, true, true).expect("armor class"));
        assert!(reflex.applied().iter().any(|modifier| {
            modifier.penalty_type() == Some(PenaltyType::Status)
                && modifier.value() == -4
                && modifier.sequence() == 10
                && modifier.statistic() == Statistic::Reflex
        }));
        assert!(armor_class.applied().iter().any(|modifier| {
            modifier.penalty_type() == Some(PenaltyType::Status)
                && modifier.value() == -4
                && modifier.sequence() == 6
                && modifier.statistic() == Statistic::ArmorClass
        }));
        assert_eq!(reflex.get(Statistic::Reflex), -4);
        assert_eq!(armor_class.get(Statistic::ArmorClass), -6);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 446, Unconscious.
    #[test]
    fn unconscious_is_minus_4_status_to_reflex() {
        let dice = script();
        let stacked = summed(&reflex_modifiers(true).expect("reflex"));
        assert_eq!(stacked.get(Statistic::Reflex), -4);
        assert!(stacked.applied().iter().any(|modifier| {
            modifier.penalty_type() == Some(PenaltyType::Status)
                && modifier.value() == -4
                && modifier.condition_id() == Some(ConditionId::UNCONSCIOUS)
                && modifier.sequence() == 10
        }));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 446, Unconscious.
    #[test]
    fn a_ranked_unconscious_modifier_is_rejected() {
        let dice = script();
        let penalty = penalty(PenaltyType::Status, -4, 1, 1, None, Statistic::Reflex);
        assert_eq!(
            penalty
                .with_condition(ConditionId::UNCONSCIOUS, Some(1))
                .expect_err("rank"),
            ModifierError::InvalidValuedCondition
        );
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 446, Unconscious.
    #[test]
    fn a_non_status_unconscious_modifier_is_rejected() {
        let dice = script();
        let penalty = penalty(PenaltyType::Circumstance, -4, 1, 1, None, Statistic::Reflex);
        assert_eq!(
            penalty
                .with_condition(ConditionId::UNCONSCIOUS, None)
                .expect_err("type"),
            ModifierError::InvalidValuedCondition
        );
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 446, Unconscious.
    #[test]
    fn an_unconscious_value_other_than_minus_4_is_rejected() {
        let dice = script();
        let penalty = penalty(PenaltyType::Status, -3, 1, 1, None, Statistic::Reflex);
        assert_eq!(
            penalty
                .with_condition(ConditionId::UNCONSCIOUS, None)
                .expect_err("value"),
            ModifierError::InvalidValuedCondition
        );
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 446, Unconscious.
    #[test]
    fn an_unconscious_bonus_is_rejected() {
        let dice = script();
        let bonus = bonus(BonusType::Status, 4, 1, 1, None, Statistic::Reflex);
        assert_eq!(
            bonus
                .with_condition(ConditionId::UNCONSCIOUS, None)
                .expect_err("bonus"),
            ModifierError::InvalidValuedCondition
        );
        assert_eq!(dice.remaining(), 1);
    }

    fn frightened_on(rank: u32, statistic: Statistic) -> Modifier {
        penalty(
            PenaltyType::Status,
            -i32::try_from(rank).expect("rank"),
            3,
            1,
            None,
            statistic,
        )
        .with_condition(ConditionId::FRIGHTENED, Some(rank))
        .expect("frightened")
    }

    fn sickened_on(rank: u32, statistic: Statistic) -> Modifier {
        penalty(
            PenaltyType::Status,
            -i32::try_from(rank).expect("rank"),
            4,
            2,
            None,
            statistic,
        )
        .with_condition(ConditionId::SICKENED, Some(rank))
        .expect("sickened")
    }

    /// Player Core pg. 446, Unconscious. Ruling R-09.
    #[test]
    fn frightened_2_and_unconscious_total_minus_6_on_armor_class() {
        let dice = script();
        let mut modifiers = ac_modifiers(None, true, true).expect("armor class");
        modifiers.push(frightened_on(2, Statistic::ArmorClass));
        let stacked = summed(&modifiers);
        assert_eq!(stacked.get(Statistic::ArmorClass), -6);
        assert!(stacked.applied().iter().any(|modifier| {
            modifier.penalty_type() == Some(PenaltyType::Status) && modifier.value() == -4
        }));
        assert!(stacked.applied().iter().any(|modifier| {
            modifier.penalty_type() == Some(PenaltyType::Circumstance) && modifier.value() == -2
        }));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 446, Unconscious. Ruling R-09.
    #[test]
    fn frightened_5_and_unconscious_total_minus_7_on_armor_class() {
        let dice = script();
        let mut modifiers = ac_modifiers(None, true, true).expect("armor class");
        modifiers.push(frightened_on(5, Statistic::ArmorClass));
        let stacked = summed(&modifiers);
        assert_eq!(stacked.get(Statistic::ArmorClass), -7);
        assert!(stacked.applied().iter().any(|modifier| {
            modifier.penalty_type() == Some(PenaltyType::Status) && modifier.value() == -5
        }));
        assert!(stacked.applied().iter().any(|modifier| {
            modifier.penalty_type() == Some(PenaltyType::Circumstance) && modifier.value() == -2
        }));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 446, Unconscious. Ruling R-09.
    #[test]
    fn frightened_3_sickened_1_and_unconscious_total_minus_4_on_reflex() {
        let dice = script();
        let unconscious = reflex_modifiers(true).expect("reflex");
        let modifiers = [
            frightened_on(3, Statistic::Reflex),
            sickened_on(1, Statistic::Reflex),
            unconscious[0],
        ];
        let stacked = summed(&modifiers);
        assert_eq!(stacked.get(Statistic::Reflex), -4);
        assert!(stacked.applied().iter().any(|modifier| {
            modifier.penalty_type() == Some(PenaltyType::Status) && modifier.value() == -4
        }));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 446, Unconscious. Ruling R-09.
    #[test]
    fn unconscious_and_a_status_bonus_total_minus_3_on_reflex() {
        let dice = script();
        let mut modifiers = reflex_modifiers(true).expect("reflex");
        modifiers.push(bonus(BonusType::Status, 1, 8, 3, None, Statistic::Reflex));
        let stacked = summed(&modifiers);
        assert_eq!(stacked.get(Statistic::Reflex), -3);
        assert!(stacked.applied().iter().any(|modifier| {
            modifier.penalty_type() == Some(PenaltyType::Status) && modifier.value() == -4
        }));
        assert!(stacked.applied().iter().any(|modifier| {
            modifier.bonus_type() == Some(BonusType::Status) && modifier.value() == 1
        }));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 446, Unconscious. Ruling R-09.
    #[test]
    fn frightened_4_tied_with_unconscious_totals_minus_4_on_reflex() {
        let dice = script();
        let mut modifiers = reflex_modifiers(true).expect("reflex");
        modifiers.push(frightened_on(4, Statistic::Reflex));
        let stacked = summed(&modifiers);
        assert_eq!(stacked.get(Statistic::Reflex), -4);
        assert!(stacked.applied().iter().any(|modifier| {
            modifier.penalty_type() == Some(PenaltyType::Status) && modifier.value() == -4
        }));
        assert_eq!(
            stacked
                .applied()
                .iter()
                .filter(|modifier| modifier.penalty_type() == Some(PenaltyType::Status))
                .count(),
            1
        );
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 404, Saving Throws.
    #[test]
    fn fortitude_and_will_do_not_share_a_stack() {
        let fortitude = bonus(BonusType::Circumstance, 2, 1, 1, None, Statistic::Fortitude);
        let will = bonus(BonusType::Circumstance, 1, 2, 1, None, Statistic::Will);
        let stacked = summed(&[fortitude, will]);
        assert_eq!(stacked.get(Statistic::Fortitude), 2);
        assert_eq!(stacked.get(Statistic::Will), 1);
    }

    /// Player Core pg. 404, Saving Throws. Player Core pg. 446, Unconscious.
    #[test]
    fn unconscious_adds_nothing_to_fortitude() {
        let fortitude = bonus(BonusType::Circumstance, 1, 1, 1, None, Statistic::Fortitude);
        let stacked = summed(&[fortitude]);
        assert_eq!(stacked.get(Statistic::Fortitude), 1);
        assert!(
            stacked
                .applied()
                .iter()
                .all(|modifier| modifier.penalty_type() != Some(PenaltyType::Status))
        );
    }

    /// Player Core pg. 404, Saving Throws. Player Core pg. 446, Unconscious.
    #[test]
    fn unconscious_adds_nothing_to_will() {
        let will = bonus(BonusType::Circumstance, 1, 1, 1, None, Statistic::Will);
        let stacked = summed(&[will]);
        assert_eq!(stacked.get(Statistic::Will), 1);
        assert!(
            stacked
                .applied()
                .iter()
                .all(|modifier| modifier.penalty_type() != Some(PenaltyType::Status))
        );
    }

    /// Player Core pg. 446, Unconscious.
    #[test]
    fn reflex_keeps_the_unconscious_penalty() {
        let modifiers = reflex_modifiers(true).expect("reflex");
        let stacked = summed(&modifiers);
        assert_eq!(stacked.get(Statistic::Reflex), -4);
        assert!(stacked.applied().iter().any(|modifier| {
            modifier.penalty_type() == Some(PenaltyType::Status)
                && modifier.value() == -4
                && modifier.sequence() == 10
                && modifier.statistic() == Statistic::Reflex
        }));
        assert_eq!(stacked.get(Statistic::Fortitude), 0);
    }

    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0
        }

        fn below(&mut self, bound: u64) -> u64 {
            // The low bits of this multiplier repeat on a short period, so `% bound` barely mixes.
            (self.next() >> 33) % bound
        }
    }
}
