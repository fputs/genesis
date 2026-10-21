//! Circumstance, item, status, and untyped modifiers.
//!
//! Callers supply sequence numbers. Newer means a greater sequence, not a clock.
//!
//! Player Core, Bonuses and Penalties.

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
    /// Frightened. A penalty with this id and a rank becomes one status penalty of `-rank`.
    /// A bonus with this id keeps the type and value the caller declared.
    ///
    /// Player Core, Frightened.
    pub const FRIGHTENED: Self = Self(2);

    /// Sickened. A penalty with this id and a rank becomes one status penalty of `-rank`.
    /// A bonus with this id keeps the type and value the caller declared.
    ///
    /// Player Core, Sickened.
    pub const SICKENED: Self = Self(3);
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

/// One bonus or penalty. The value is greater than 0 for a bonus and less than 0 for a penalty.
///
/// Player Core, Bonuses and Penalties.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Modifier {
    source: SourceId,
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
    /// Rank is not the algebraic value of the penalty.
    ///
    /// Player Core, "Redundant Conditions With Values" and "Condition Values".
    pub fn with_condition(mut self, id: ConditionId, rank: Option<u32>) -> Self {
        self.condition = Some(Condition { id, rank });
        self
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

    /// The effect this modifier came from.
    pub fn source(self) -> SourceId {
        self.source
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
pub struct ModifierTotal {
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

    /// The modifiers that were applied. A tied bonus or penalty is the one with the lowest sequence.
    pub fn applied(&self) -> &[Modifier] {
        &self.applied
    }
}

/// Splits bonuses from penalties, reduces repeated conditions per statistic, collapses
/// the duplicate key, then stacks what remains on each statistic.
///
/// The key is (source id, statistic, type, bonus or penalty). The type is the type after
/// frightened and sickened penalties become status. For each statistic, and for each of
/// circumstance, item, and status, the highest bonus and the worst penalty both apply.
/// When different sources tie, the lowest sequence is the one reported. Two statistics do
/// not share that choice. Every untyped penalty is added with checked addition. Overflow
/// is [`ModifierError::Overflow`] and is not a saturated `i32`.
///
/// Player Core, Bonuses, Penalties, and Duplicate Effects.
pub fn total(modifiers: &[Modifier]) -> Result<ModifierTotal, ModifierError> {
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
pub fn dc_from_total(total: i32) -> Result<i32, ModifierError> {
    10i32.checked_add(total).ok_or(ModifierError::Overflow)
}

fn collapse_conditions(modifiers: &[Modifier]) -> Result<Vec<Modifier>, ModifierError> {
    let mut plain = Vec::new();
    let mut grouped: Vec<(ConditionId, Statistic, Vec<Modifier>)> = Vec::new();
    for modifier in modifiers.iter().copied() {
        match modifier.condition {
            None => plain.push(modifier),
            Some(condition) => {
                match grouped.iter_mut().find(|(id, statistic, _)| {
                    *id == condition.id && *statistic == modifier.statistic
                }) {
                    Some((_, _, group)) => group.push(modifier),
                    None => grouped.push((condition.id, modifier.statistic, vec![modifier])),
                }
            }
        }
    }
    for (_, _, group) in grouped {
        if let Some(modifier) = keep_condition(&group)? {
            plain.push(modifier);
        }
    }
    Ok(plain)
}

/// Highest rank wins. The same rank keeps the newer sequence, which is the greater one.
/// A condition with no rank is kept once, and that tie uses the same newer sequence.
///
/// Rank is the condition's number, so frightened 2 beats frightened 1 even though -1 is
/// algebraically greater than -2. A frightened or sickened penalty then contributes one
/// status penalty equal to the negation of that rank. A bonus with those ids, and any other
/// ranked condition, keeps its declared type and value. Value does not break a same-rank tie.
fn keep_condition(group: &[Modifier]) -> Result<Option<Modifier>, ModifierError> {
    let Some(mut best) = group.first().copied() else {
        return Ok(None);
    };
    for candidate in group.iter().copied().skip(1) {
        if condition_prefers(candidate, best) {
            best = candidate;
        }
    }
    apply_ranked_penalty(best)
}

fn condition_prefers(candidate: Modifier, current: Modifier) -> bool {
    match (candidate.rank(), current.rank()) {
        (Some(candidate_rank), Some(current_rank)) if candidate_rank != current_rank => {
            candidate_rank > current_rank
        }
        (Some(_), None) => true,
        (None, Some(_)) => false,
        // Same rank, or neither has one: the newer effect wins. Newer is the greater sequence.
        _ => candidate.sequence > current.sequence,
    }
}

fn apply_ranked_penalty(mut modifier: Modifier) -> Result<Option<Modifier>, ModifierError> {
    let Some(rank) = modifier.rank() else {
        return Ok(Some(modifier));
    };
    // Only a frightened or sickened penalty becomes a status penalty of -rank.
    // A bonus tagged with those ids keeps its declared type and value. Any other
    // ranked condition, including rank 0, keeps the type and value the caller declared.
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
/// The key is (source id, statistic, type, bonus or penalty), using the type after
/// frightened and sickened penalties are rewritten. A greater sequence still wins when
/// its value is lower. Rules 2266.
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
    left.source == right.source && left.statistic == right.statistic && left.kind == right.kind
}

/// Higher level, then a level over none, then higher rank, then a rank over none, then the
/// greater sequence. Ruling R-01 then keeps the higher bonus or the more negative penalty.
/// Duplicate Effects has no value step; Rules 2282 does, and this matches it. A lower
/// condition id breaks a remaining tie. A missing condition id loses. Otherwise the
/// incumbent stays, so input order cannot change the modifier.
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
    match (candidate.condition_id(), current.condition_id()) {
        (Some(candidate_id), Some(current_id)) if candidate_id != current_id => {
            candidate_id < current_id
        }
        (Some(_), None) => true,
        _ => false,
    }
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
}

fn stack_types(modifiers: &[Modifier]) -> Vec<Modifier> {
    let mut grouped = ByStatistic {
        attack: Buckets::new(),
        armor_class: Buckets::new(),
        perception: Buckets::new(),
        damage: Buckets::new(),
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

/// Highest value wins. A tie reports the lowest sequence, not the newer effect.
fn bonus_prefers(candidate: Modifier, current: Modifier) -> bool {
    if candidate.value != current.value {
        candidate.value > current.value
    } else {
        candidate.sequence < current.sequence
    }
}

/// Lowest value wins. A tie reports the lowest sequence.
fn penalty_prefers(candidate: Modifier, current: Modifier) -> bool {
    if candidate.value != current.value {
        candidate.value < current.value
    } else {
        candidate.sequence < current.sequence
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StatisticTotals {
    attack: i32,
    armor_class: i32,
    perception: i32,
    damage: i32,
}

impl StatisticTotals {
    fn get(self, statistic: Statistic) -> i32 {
        match statistic {
            Statistic::Attack => self.attack,
            Statistic::ArmorClass => self.armor_class,
            Statistic::Perception => self.perception,
            Statistic::Damage => self.damage,
        }
    }
}

struct StatisticWide {
    attack: i64,
    armor_class: i64,
    perception: i64,
    damage: i64,
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
    };
    for modifier in &applied {
        add_statistic(&mut wide, modifier.statistic, modifier.value)?;
    }
    let totals = StatisticTotals {
        attack: i32::try_from(wide.attack).map_err(|_| ModifierError::Overflow)?,
        armor_class: i32::try_from(wide.armor_class).map_err(|_| ModifierError::Overflow)?,
        perception: i32::try_from(wide.perception).map_err(|_| ModifierError::Overflow)?,
        damage: i32::try_from(wide.damage).map_err(|_| ModifierError::Overflow)?,
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
    };
    *slot = slot
        .checked_add(i64::from(value))
        .ok_or(ModifierError::Overflow)?;
    Ok(())
}

fn sort_key(modifier: &Modifier) -> (u8, i32, u32, u32, u8) {
    (
        kind_ord(modifier.kind),
        modifier.value,
        modifier.sequence,
        modifier.source.0,
        statistic_ord(modifier.statistic),
    )
}

fn statistic_ord(statistic: Statistic) -> u8 {
    match statistic {
        Statistic::Attack => 0,
        Statistic::ArmorClass => 1,
        Statistic::Perception => 2,
        Statistic::Damage => 3,
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

const MAP_SOURCE: SourceId = SourceId(1);
const PRONE_SOURCE: SourceId = SourceId(2);
const AID_SOURCE: SourceId = SourceId(3);
const COVER_SOURCE: SourceId = SourceId(4);
const OFF_GUARD_SOURCE: SourceId = SourceId(5);
const UNCONSCIOUS_AC_SOURCE: SourceId = SourceId(6);
const UNCONSCIOUS_PERCEPTION_SOURCE: SourceId = SourceId(7);

const OFF_GUARD: ConditionId = ConditionId(1);

/// Cover granted to Armor Class.
///
/// Player Core, Cover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cover {
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
pub fn attack_modifiers(
    multiple_attack_penalty: i32,
    prone: bool,
    aid_circumstance: i32,
) -> Result<Vec<Modifier>, ModifierError> {
    let mut modifiers = Vec::new();
    push_untyped(&mut modifiers, multiple_attack_penalty, MAP_SOURCE, 1)?;
    if prone {
        modifiers.push(Modifier::penalty(
            PenaltyType::Circumstance,
            -2,
            PRONE_SOURCE,
            2,
            None,
            Statistic::Attack,
        )?);
    }
    push_circumstance(&mut modifiers, aid_circumstance, AID_SOURCE, 3)?;
    Ok(modifiers)
}

/// Modifiers on Armor Class from this change. The shield bonus is not an argument.
///
/// Player Core, Cover, Off-Guard, and Unconscious.
pub fn ac_modifiers(
    cover: Option<Cover>,
    off_guard: bool,
    unconscious: bool,
) -> Result<Vec<Modifier>, ModifierError> {
    let mut modifiers = Vec::new();
    if let Some(cover) = cover {
        modifiers.push(Modifier::bonus(
            BonusType::Circumstance,
            cover.bonus(),
            COVER_SOURCE,
            4,
            None,
            Statistic::ArmorClass,
        )?);
    }
    if off_guard {
        modifiers.push(
            Modifier::penalty(
                PenaltyType::Circumstance,
                -2,
                OFF_GUARD_SOURCE,
                5,
                None,
                Statistic::ArmorClass,
            )?
            .with_condition(OFF_GUARD, None),
        );
    }
    if unconscious {
        modifiers.push(Modifier::penalty(
            PenaltyType::Status,
            -4,
            UNCONSCIOUS_AC_SOURCE,
            6,
            None,
            Statistic::ArmorClass,
        )?);
    }
    Ok(modifiers)
}

/// Modifiers on Perception from this change. Unconscious is a status −4.
///
/// The slice is separate from Armor Class. Player Core, Unconscious.
pub fn perception_modifiers(unconscious: bool) -> Result<Vec<Modifier>, ModifierError> {
    let mut modifiers = Vec::new();
    if unconscious {
        modifiers.push(Modifier::penalty(
            PenaltyType::Status,
            -4,
            UNCONSCIOUS_PERCEPTION_SOURCE,
            7,
            None,
            Statistic::Perception,
        )?);
    }
    Ok(modifiers)
}

fn push_untyped(
    modifiers: &mut Vec<Modifier>,
    value: i32,
    source: SourceId,
    sequence: u32,
) -> Result<(), ModifierError> {
    if value == 0 {
        return Ok(());
    }
    if value > 0 {
        return Err(ModifierError::UntypedBonus);
    }
    modifiers.push(Modifier::penalty(
        PenaltyType::Untyped,
        value,
        source,
        sequence,
        None,
        Statistic::Attack,
    )?);
    Ok(())
}

fn push_circumstance(
    modifiers: &mut Vec<Modifier>,
    value: i32,
    source: SourceId,
    sequence: u32,
) -> Result<(), ModifierError> {
    if value == 0 {
        return Ok(());
    }
    if value > 0 {
        modifiers.push(Modifier::bonus(
            BonusType::Circumstance,
            value,
            source,
            sequence,
            None,
            Statistic::Attack,
        )?);
    } else {
        modifiers.push(Modifier::penalty(
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
        DegreeOfSuccess, MapInput, degree_of_success, flat_check, multiple_attack_penalty,
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

    fn second_non_agile_map() -> i32 {
        multiple_attack_penalty(MapInput {
            on_own_turn: true,
            prior_attack_trait_checks_this_turn: 1,
            current_weapon_agile: false,
        })
    }

    fn second_agile_map() -> i32 {
        multiple_attack_penalty(MapInput {
            on_own_turn: true,
            prior_attack_trait_checks_this_turn: 1,
            current_weapon_agile: true,
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
            .with_condition(ConditionId::FRIGHTENED, Some(2));
        let sickened = penalty(PenaltyType::Status, -1, 2, 2, None, Statistic::Attack)
            .with_condition(ConditionId::SICKENED, Some(1));
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
            .with_condition(ConditionId::FRIGHTENED, Some(1));
        let frightened_2 = penalty(PenaltyType::Status, -2, 2, 2, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(2));
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
            .with_condition(ConditionId::FRIGHTENED, Some(2));
        let newer = penalty(PenaltyType::Status, -1, 2, 9, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(2));
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
            .with_condition(ConditionId(9), Some(1));
        let higher = penalty(PenaltyType::Circumstance, -1, 2, 2, None, Statistic::Attack)
            .with_condition(ConditionId(9), Some(2));
        let stacked = summed(&[lower, higher]);
        assert_eq!(stacked.applied().len(), 1);
        let kept = stacked.applied()[0];
        assert_eq!(kept.penalty_type(), Some(PenaltyType::Circumstance));
        assert_eq!(kept.value(), -1);
        assert_eq!(kept.rank(), Some(2));
        assert_eq!(stacked.get(Statistic::Attack), -1);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Frightened.
    ///
    /// The rank rewrite applies to penalties. A status bonus tagged frightened stays +2.
    #[test]
    fn a_frightened_bonus_keeps_its_declared_value() {
        let dice = script();
        let modifier = bonus(BonusType::Status, 2, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(3));
        let stacked = summed(&[modifier]);
        assert_eq!(stacked.applied().len(), 1);
        let kept = stacked.applied()[0];
        assert_eq!(kept.bonus_type(), Some(BonusType::Status));
        assert_eq!(kept.penalty_type(), None);
        assert_eq!(kept.value(), 2);
        assert_eq!(kept.rank(), Some(3));
        assert_eq!(stacked.get(Statistic::Attack), 2);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Frightened.
    ///
    /// A frightened circumstance penalty of -7 at rank 3 becomes one status penalty of -3.
    #[test]
    fn a_frightened_circumstance_penalty_becomes_status_minus_rank() {
        let dice = script();
        let modifier = penalty(PenaltyType::Circumstance, -7, 1, 1, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(3));
        let stacked = summed(&[modifier]);
        assert_eq!(stacked.applied().len(), 1);
        let kept = stacked.applied()[0];
        assert_eq!(kept.penalty_type(), Some(PenaltyType::Status));
        assert_eq!(kept.bonus_type(), None);
        assert_eq!(kept.value(), -3);
        assert_eq!(kept.rank(), Some(3));
        assert_eq!(stacked.get(Statistic::Attack), -3);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Redundant Conditions".
    /// Player Core, Off-Guard.
    #[test]
    fn off_guard_from_two_causes_applies_once() {
        let dice = script();
        let prone = penalty(PenaltyType::Circumstance, -2, 1, 1, None, Statistic::Attack)
            .with_condition(OFF_GUARD, None);
        let flanking = penalty(PenaltyType::Circumstance, -2, 2, 2, None, Statistic::Attack)
            .with_condition(OFF_GUARD, None);
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

    /// Player Core pg. 444, Frightened.
    /// Player Core pg. 442, "Redundant Conditions With Values".
    ///
    /// The case is synthetic: frightened never grants a bonus, and this is a robustness check.
    #[test]
    fn a_frightened_bonus_does_not_hide_a_frightened_penalty() {
        let dice = script();
        let bonus = bonus(BonusType::Status, 2, 7, 4, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(3));
        let penalty = penalty(PenaltyType::Status, -1, 7, 4, None, Statistic::Attack)
            .with_condition(ConditionId::FRIGHTENED, Some(1));
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
            .with_condition(ConditionId::FRIGHTENED, Some(2));
        let armor_class = penalty(PenaltyType::Status, -2, 7, 4, None, Statistic::ArmorClass)
            .with_condition(ConditionId::FRIGHTENED, Some(2));
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
            .with_condition(ConditionId(8), Some(1));
        let lower_id = bonus(BonusType::Circumstance, 2, 9, 5, Some(2), Statistic::Attack)
            .with_condition(ConditionId(3), Some(1));
        for modifiers in orders_of_two(higher_id, lower_id) {
            let stacked = summed(&modifiers);
            assert_eq!(stacked.applied().len(), 1);
            assert_eq!(stacked.applied()[0].condition_id(), Some(ConditionId(3)));
            assert_eq!(stacked.get(Statistic::Attack), 2);
            assert_eq!(total_on(&stacked, Statistic::Attack), 2);
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
        .with_condition(ConditionId(6), Some(1));
        let id_1 = bonus(
            BonusType::Circumstance,
            2,
            11,
            8,
            Some(1),
            Statistic::Attack,
        )
        .with_condition(ConditionId(1), Some(1));
        let id_4 = bonus(
            BonusType::Circumstance,
            2,
            11,
            8,
            Some(1),
            Statistic::Attack,
        )
        .with_condition(ConditionId(4), Some(1));
        let orders = orders_of_three(id_6, id_1, id_4);
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
}
