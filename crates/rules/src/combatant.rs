/// Which side of the encounter a creature fights for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Party,
    Enemy,
}

/// One class feature. This encounter resolves Reactive Strike and no other.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClassFeature {
    ReactiveStrike,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DamageFormula {
    pub dice: u8,
    pub faces: u8,
    pub bonus: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrikeTemplate {
    pub name: String,
    pub attack_bonus: i32,
    pub damage: DamageFormula,
    pub reach_feet: u32,
}

/// Numbers a fixture provides before preparation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreatureTemplate {
    pub name: String,
    pub level: u8,
    pub hit_points: i32,
    pub armor_class: i32,
    pub perception: i32,
    pub speed_feet: u32,
    pub strike: StrikeTemplate,
    pub side: Side,
    pub features: Vec<ClassFeature>,
}

/// Modifiers gathered before derived stats are written.
///
/// Fixture preparation leaves the numeric fields at zero. [`apply_effects`] copies class features.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Synthetics {
    pub armor_class: i32,
    pub attack_bonus: i32,
    pub perception: i32,
    pub speed_feet: i32,
    pub features: Vec<ClassFeature>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedStrike {
    pub name: String,
    pub attack_bonus: i32,
    pub damage: DamageFormula,
    pub reach_feet: u32,
}

/// Stats after the preparation phases.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedCombatant {
    pub name: String,
    pub level: u8,
    pub hit_points: i32,
    pub armor_class: i32,
    pub perception: i32,
    pub speed_feet: u32,
    pub strike: PreparedStrike,
    pub side: Side,
    pub features: Vec<ClassFeature>,
}

/// Copies class features onto `synthetics`. Numeric modifiers stay at zero.
pub fn apply_effects(defn: &CreatureTemplate, synthetics: &mut Synthetics) {
    synthetics.features.clone_from(&defn.features);
}

/// Runs preparation phases and returns derived combat stats.
pub fn prepare_combatant(defn: &CreatureTemplate) -> PreparedCombatant {
    let mut synthetics = Synthetics::default();
    apply_effects(defn, &mut synthetics);
    derive(defn, &synthetics)
}

fn derive(defn: &CreatureTemplate, synthetics: &Synthetics) -> PreparedCombatant {
    let speed = defn.speed_feet as i32 + synthetics.speed_feet;
    PreparedCombatant {
        name: defn.name.clone(),
        level: defn.level,
        hit_points: defn.hit_points,
        armor_class: defn.armor_class + synthetics.armor_class,
        perception: defn.perception + synthetics.perception,
        speed_feet: speed.max(0) as u32,
        strike: PreparedStrike {
            name: defn.strike.name.clone(),
            attack_bonus: defn.strike.attack_bonus + synthetics.attack_bonus,
            damage: defn.strike.damage.clone(),
            reach_feet: defn.strike.reach_feet,
        },
        side: defn.side,
        features: synthetics.features.clone(),
    }
}
