/// Which side of the encounter a creature fights for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Party,
    Enemy,
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
}

/// Modifiers gathered before derived stats are written.
///
/// Fixture preparation leaves every field at zero.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Synthetics {
    pub armor_class: i32,
    pub attack_bonus: i32,
    pub perception: i32,
    pub speed_feet: i32,
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
}

/// Collects synthetic modifiers. Templates contribute none, so the accumulator stays zero.
pub fn apply_effects(_defn: &CreatureTemplate, synthetics: &mut Synthetics) {
    let _ = synthetics;
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
    }
}
