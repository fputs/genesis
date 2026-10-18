//! Seeded encounter sequences for Stride, Step, Crawl, Leap, Drop Prone, Stand, Strike, Seek,
//! Point Out, Take Cover, Interact, Release, Raise a Shield, Shield Block, Reactive Strike,
//! cover, difficult terrain, and the dying track.

use rules::{
    advance_if_turn_done, declare_action, end_turn, legal_actions, prepare_combatant,
    reactors_for_left_square, resolve_pending_shield_block, start_encounter, Action, ClassFeature,
    CombatantId, CreatureTemplate, DamageFormula, Degree, Detection, Encounter, Event, Gear, Grid,
    GridPos, Illegal, Outcome, ScriptedDice, Shield, Side, StrikeTemplate, Terrain,
};

fn strike(name: &str, attack: i32, dice: u8, faces: u8, bonus: i32) -> StrikeTemplate {
    StrikeTemplate {
        name: name.into(),
        attack_bonus: attack,
        damage: DamageFormula { dice, faces, bonus },
        reach_feet: 5,
    }
}

fn fighter(name: &str) -> CreatureTemplate {
    CreatureTemplate {
        name: name.into(),
        level: 1,
        hit_points: 20,
        armor_class: 18,
        perception: 5,
        speed_feet: 25,
        strike: strike("longsword", 7, 1, 8, 4),
        side: Side::Party,
        features: Vec::new(),
        shield: None,
        shield_block: false,
        stealth_dc: None,
    }
}

fn guard() -> CreatureTemplate {
    CreatureTemplate {
        name: "Hall Guard".into(),
        level: 1,
        hit_points: 18,
        armor_class: 14,
        perception: 3,
        speed_feet: 25,
        strike: strike("spear", 6, 1, 6, 2),
        side: Side::Enemy,
        features: Vec::new(),
        shield: None,
        shield_block: false,
        stealth_dc: None,
    }
}

fn steel_shield() -> Shield {
    Shield {
        hardness: 5,
        hit_points: 20,
        max_hit_points: 20,
        broken_threshold: 10,
        raised: false,
    }
}

fn shielded(name: &str, shield_block: bool) -> CreatureTemplate {
    let mut creature = fighter(name);
    creature.shield = Some(steel_shield());
    creature.shield_block = shield_block;
    creature
}

fn end_until(encounter: &mut Encounter, id: CombatantId, dice: &mut ScriptedDice) {
    for _ in 0..8 {
        if encounter.active() == Some(id) {
            return;
        }
        end_turn(encounter, dice).unwrap();
    }
    panic!("did not reach {id}");
}

fn party() -> [CreatureTemplate; 4] {
    [
        fighter("Sera Voss"),
        fighter("Bram Holt"),
        fighter("Quill Ames"),
        fighter("Doss Kade"),
    ]
}

fn open(width: i32, height: i32) -> Grid {
    Grid::new(width, height, [])
}

/// Player Core, "Initiative": each participant rolls Perception once when the encounter starts.
#[test]
fn initiative_orders_perception_totals_and_breaks_ties_by_modifier() {
    // Roll order is the four party slots, then the enemy.
    // Totals: 10+5=15, 5+5=10, 15+5=20, 1+5=6, 12+3=15.
    let mut dice = ScriptedDice::new([10, 5, 15, 1, 12]);
    let encounter = start_encounter(
        &party(),
        &[guard()],
        &[
            GridPos::new(0, 0),
            GridPos::new(1, 0),
            GridPos::new(2, 0),
            GridPos::new(3, 0),
        ],
        &[GridPos::new(5, 0)],
        open(8, 4),
        &mut dice,
    )
    .unwrap();

    assert_eq!(encounter.party().len(), 4);
    assert_eq!(
        encounter.initiative(),
        &[
            CombatantId(2), // 20
            CombatantId(0), // 15, Perception +5
            CombatantId(4), // 15, Perception +3
            CombatantId(1), // 10
            CombatantId(3), // 6
        ]
    );
    assert_eq!(encounter.active(), Some(CombatantId(2)));
    assert_eq!(encounter.round(), 1);
    assert_eq!(encounter.actions_remaining(), 3);
    assert_eq!(dice.remaining(), 0);
    assert!(encounter.party().contains(&CombatantId(3)));
}

#[test]
fn preparation_copies_fixture_numbers() {
    let prepared = prepare_combatant(&fighter("Sera Voss"));
    assert_eq!(prepared.armor_class, 18);
    assert_eq!(prepared.speed_feet, 25);
    assert_eq!(prepared.perception, 5);
    assert_eq!(prepared.strike.attack_bonus, 7);
    assert_eq!(prepared.strike.damage.faces, 8);
    assert_eq!(prepared.hit_points, 20);
}

/// Player Core, "Stride": one action moves up to Speed.
/// Player Core, "Movement": diagonal squares cost 5, then 10, alternating.
#[test]
fn stride_spends_one_action_and_respects_speed_including_diagonals() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[guard()],
        &[
            GridPos::new(0, 0),
            GridPos::new(0, 2),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(0, 6)],
        open(12, 12),
        &mut dice,
    )
    .unwrap();
    // All totals tie at 6 except the enemy at 4, so slot 0 acts first.
    assert_eq!(encounter.active(), Some(CombatantId(0)));

    let legal = legal_actions(&encounter, CombatantId(0));
    assert!(legal.contains(&Action::Stride {
        destination: GridPos::new(5, 0)
    }));
    assert!(!legal.contains(&Action::Stride {
        destination: GridPos::new(6, 0)
    }));
    assert!(legal.contains(&Action::Stride {
        destination: GridPos::new(3, 3)
    }));
    assert!(!legal.contains(&Action::Stride {
        destination: GridPos::new(4, 4)
    }));

    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Stride {
            destination: GridPos::new(3, 3),
        },
        &mut dice,
    )
    .unwrap();
    assert_eq!(
        events,
        vec![Event::Moved {
            id: CombatantId(0),
            from: GridPos::new(0, 0),
            to: GridPos::new(3, 3),
            feet: 20,
            actions_remaining: 2,
        }]
    );
    assert_eq!(encounter.strikes_this_turn(), 0);
}

#[test]
fn three_strides_spend_the_turn() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[guard()],
        &[
            GridPos::new(0, 1),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
            GridPos::new(0, 5),
        ],
        &[GridPos::new(0, 7)],
        open(10, 10),
        &mut dice,
    )
    .unwrap();

    for step in 1..=3 {
        declare_action(
            &mut encounter,
            CombatantId(0),
            Action::Stride {
                destination: GridPos::new(step, 1),
            },
            &mut dice,
        )
        .unwrap();
    }
    assert_eq!(encounter.actions_remaining(), 0);
    let err = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Stride {
            destination: GridPos::new(4, 1),
        },
        &mut dice,
    )
    .unwrap_err();
    assert_eq!(err, Illegal::NotLegal);
}

/// Player Core, "Step": one action moves 5 feet when Speed is at least 10.
#[test]
fn step_moves_five_feet_and_spends_one_action() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[guard()],
        &[
            GridPos::new(0, 0),
            GridPos::new(0, 1),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(0, 6)],
        Grid::new(12, 12, [GridPos::new(1, 1)]),
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(0)));

    let legal = legal_actions(&encounter, CombatantId(0));
    assert!(legal.contains(&Action::Step {
        destination: GridPos::new(1, 0)
    }));
    assert!(!legal.contains(&Action::Step {
        destination: GridPos::new(0, 1)
    }));
    assert!(!legal.contains(&Action::Step {
        destination: GridPos::new(1, 1)
    }));

    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Step {
            destination: GridPos::new(1, 0),
        },
        &mut dice,
    )
    .unwrap();
    assert_eq!(
        events,
        vec![Event::Stepped {
            id: CombatantId(0),
            from: GridPos::new(0, 0),
            to: GridPos::new(1, 0),
            feet: 5,
            actions_remaining: 2,
        }]
    );
    assert_eq!(
        encounter.combatant(CombatantId(0)).unwrap().position,
        GridPos::new(1, 0)
    );
    assert_eq!(encounter.actions_remaining(), 2);
    assert_eq!(encounter.strikes_this_turn(), 0);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, "Step": Speed below 10 cannot Step, and a Step is not a two-square move.
#[test]
fn step_rejects_speed_below_ten_and_a_two_square_destination() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[guard()],
        &[
            GridPos::new(0, 0),
            GridPos::new(0, 2),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(0, 6)],
        open(12, 12),
        &mut dice,
    )
    .unwrap();
    assert!(
        !legal_actions(&encounter, CombatantId(0)).contains(&Action::Step {
            destination: GridPos::new(2, 0)
        })
    );
    let err = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Step {
            destination: GridPos::new(2, 0),
        },
        &mut dice,
    )
    .unwrap_err();
    assert_eq!(err, Illegal::NotLegal);
    assert_eq!(encounter.actions_remaining(), 3);
    assert_eq!(
        encounter.combatant(CombatantId(0)).unwrap().position,
        GridPos::new(0, 0)
    );

    let mut slow = party();
    slow[0].speed_feet = 5;
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &slow,
        &[guard()],
        &[
            GridPos::new(0, 0),
            GridPos::new(0, 2),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(0, 6)],
        open(12, 12),
        &mut dice,
    )
    .unwrap();
    let legal = legal_actions(&encounter, CombatantId(0));
    assert!(legal
        .iter()
        .all(|action| !matches!(action, Action::Step { .. })));
    assert!(legal.contains(&Action::Stride {
        destination: GridPos::new(1, 0)
    }));
    let err = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Step {
            destination: GridPos::new(1, 0),
        },
        &mut dice,
    )
    .unwrap_err();
    assert_eq!(err, Illegal::NotLegal);
}

#[test]
fn enemies_block_passage_and_overlapping_spawns_are_rejected() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let overlap = start_encounter(
        &party(),
        &[guard()],
        &[
            GridPos::new(0, 0),
            GridPos::new(0, 0),
            GridPos::new(0, 0),
            GridPos::new(0, 0),
        ],
        &[GridPos::new(1, 0)],
        Grid::new(3, 1, []),
        &mut dice,
    );
    assert!(overlap.is_err(), "overlapping party spawns are rejected");

    // Corridor along y = 1. Walls seal the north and south until past the enemy,
    // so the only route to (3, 1) steps on the enemy.
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let corridor = Grid::new(
        5,
        3,
        [
            GridPos::new(0, 0),
            GridPos::new(1, 0),
            GridPos::new(2, 0),
            GridPos::new(0, 2),
            GridPos::new(1, 2),
            GridPos::new(2, 2),
        ],
    );
    let encounter = start_encounter(
        &party(),
        &[guard()],
        &[
            GridPos::new(0, 1),
            GridPos::new(3, 0),
            GridPos::new(4, 0),
            GridPos::new(4, 2),
        ],
        &[GridPos::new(2, 1)],
        corridor,
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(0)));
    let legal = legal_actions(&encounter, CombatantId(0));
    assert!(legal.contains(&Action::Stride {
        destination: GridPos::new(1, 1)
    }));
    assert!(!legal.contains(&Action::Stride {
        destination: GridPos::new(3, 1)
    }));
    assert!(!legal.contains(&Action::Stride {
        destination: GridPos::new(2, 1)
    }));
}

/// Player Core, "Strike": d20 + attack bonus vs Armor Class, damage on a success, double on a critical success.
#[test]
fn strike_hits_misses_and_doubles_damage_on_a_critical_success() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[guard()],
        &[
            GridPos::new(1, 1),
            GridPos::new(0, 0),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(2, 1)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(0)));
    assert!(
        legal_actions(&encounter, CombatantId(0)).contains(&Action::Strike {
            target: CombatantId(4)
        })
    );

    // 10 + 7 = 17 vs AC 14 success. Damage die 4 + 4 = 8. 18 - 8 = 10.
    let mut dice = ScriptedDice::new([10, 4]);
    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    assert_eq!(events.len(), 1);
    match &events[0] {
        Event::StrikeResolved {
            degree,
            damage,
            map,
            total,
            target_hit_points,
            ..
        } => {
            assert_eq!(*degree, Degree::Success);
            assert_eq!(*damage, 8);
            assert_eq!(*map, 0);
            assert_eq!(*total, 17);
            assert_eq!(*target_hit_points, 10);
        }
        other => panic!("unexpected {other:?}"),
    }

    // Natural 20: 20 + 7 = 27 vs AC 14 is a critical success (DC + 10 = 24). Die 4 + 4 = 8, doubled to 16.
    let mut dice = ScriptedDice::new([20, 4]);
    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    match &events[0] {
        Event::StrikeResolved {
            degree,
            damage,
            map,
            target_hit_points,
            ..
        } => {
            assert_eq!(*degree, Degree::CriticalSuccess);
            assert_eq!(*damage, 16);
            assert_eq!(*map, -5);
            assert_eq!(*target_hit_points, 0);
        }
        other => panic!("unexpected {other:?}"),
    }
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::CombatantDowned { id } if *id == CombatantId(4))));
    let guard = encounter.combatant(CombatantId(4)).unwrap();
    assert_eq!(guard.hit_points, 0);
    assert_eq!(guard.dying, 2);
    assert_eq!(guard.wounded, 1);
    assert!(guard.prone);
    assert!(!guard.dead);
    assert!(encounter.outcome().is_none());
    assert_eq!(encounter.active(), Some(CombatantId(0)));
}

/// Player Core, "Multiple Attack Penalty".
#[test]
fn later_strikes_take_map_and_a_miss_does_not_roll_damage_dice() {
    let mut enemy = guard();
    enemy.hit_points = 80;
    enemy.armor_class = 18;
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[enemy],
        &[
            GridPos::new(1, 1),
            GridPos::new(0, 0),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(2, 1)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();

    // First: 11 + 7 = 18 vs 18 success. Damage 1 + 4 = 5. MAP still 0.
    // Second: 11 + 7 - 5 = 13 vs 18 failure. Damage dice are not rolled.
    // Third: 20 + 7 - 10 = 17 vs 18 failure, natural 20 improves it to a success. Damage 2 + 4 = 6.
    let mut dice = ScriptedDice::new([11, 1, 11, 20, 2]);
    let first = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    match &first[0] {
        Event::StrikeResolved {
            map,
            degree,
            damage,
            ..
        } => {
            assert_eq!(*map, 0);
            assert_eq!(*degree, Degree::Success);
            assert_eq!(*damage, 5);
        }
        other => panic!("unexpected {other:?}"),
    }

    let second = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    match &second[0] {
        Event::StrikeResolved {
            map,
            degree,
            damage,
            total,
            ..
        } => {
            assert_eq!(*map, -5);
            assert_eq!(*degree, Degree::Failure);
            assert_eq!(*damage, 0);
            assert_eq!(*total, 13);
        }
        other => panic!("unexpected {other:?}"),
    }
    assert_eq!(dice.remaining(), 2, "a miss does not consume damage dice");

    let third = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    match &third[0] {
        Event::StrikeResolved {
            map,
            degree,
            damage,
            total,
            ..
        } => {
            assert_eq!(*map, -10);
            assert_eq!(*degree, Degree::Success);
            assert_eq!(*total, 17);
            assert_eq!(*damage, 6);
        }
        other => panic!("unexpected {other:?}"),
    }
    assert_eq!(encounter.actions_remaining(), 0);
    assert_eq!(dice.remaining(), 0);
}

#[test]
fn strike_out_of_reach_and_another_creatures_turn_are_illegal() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[guard()],
        &[
            GridPos::new(0, 0),
            GridPos::new(0, 2),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(4, 0)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    let err = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap_err();
    assert_eq!(err, Illegal::NotLegal);
    let err = declare_action(
        &mut encounter,
        CombatantId(1),
        Action::Stride {
            destination: GridPos::new(1, 2),
        },
        &mut dice,
    )
    .unwrap_err();
    assert_eq!(err, Illegal::NotYourTurn);
}

#[test]
fn downed_party_slot_remains_and_is_skipped() {
    let mut enemy = guard();
    enemy.strike = strike("glaive", 12, 1, 4, 30);
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 20]);
    let mut encounter = start_encounter(
        &party(),
        &[enemy],
        &[
            GridPos::new(2, 1),
            GridPos::new(0, 0),
            GridPos::new(0, 4),
            GridPos::new(0, 5),
        ],
        &[GridPos::new(2, 2)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(4)));

    let mut dice = ScriptedDice::new([10, 1, 11]);
    let events = declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Strike {
            target: CombatantId(0),
        },
        &mut dice,
    )
    .unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        Event::CombatantDowned { id } if *id == CombatantId(0)
    )));
    assert_eq!(encounter.combatant(CombatantId(0)).unwrap().hit_points, 0);
    assert_eq!(encounter.party()[0], CombatantId(0));
    assert!(encounter.outcome().is_none());

    let events = end_turn(&mut encounter, &mut dice).unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        Event::TurnStarted { id, .. } if *id == CombatantId(1)
    )));
    assert_ne!(encounter.active(), Some(CombatantId(0)));
    assert_eq!(encounter.party().len(), 4);
}

#[test]
fn party_wipe_ends_in_defeat_and_enemy_defeat_ends_in_victory() {
    let mut enemy = guard();
    enemy.strike = strike("glaive", 20, 1, 4, 40);
    enemy.hit_points = 10;
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 16]);
    let mut encounter = start_encounter(
        &party(),
        &[enemy],
        &[
            GridPos::new(1, 2),
            GridPos::new(2, 1),
            GridPos::new(3, 2),
            GridPos::new(2, 3),
        ],
        &[GridPos::new(2, 2)],
        open(6, 6),
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(4)));

    // Three killing blows, then the surviving PC passes, then the enemy drops the last PC.
    let mut dice = ScriptedDice::new([12, 1, 12, 1, 12, 1, 12, 1]);
    for target in [CombatantId(0), CombatantId(1), CombatantId(2)] {
        declare_action(
            &mut encounter,
            CombatantId(4),
            Action::Strike { target },
            &mut dice,
        )
        .unwrap();
    }
    assert!(encounter.outcome().is_none());
    let started = advance_if_turn_done(&mut encounter, &mut dice);
    assert!(started.iter().any(|event| matches!(
        event,
        Event::TurnStarted { id, .. } if *id == CombatantId(3)
    )));
    end_turn(&mut encounter, &mut dice).unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(4)));
    let events = declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Strike {
            target: CombatantId(3),
        },
        &mut dice,
    )
    .unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        Event::EncounterEnded {
            outcome: Outcome::Defeat
        }
    )));
    assert_eq!(encounter.party().len(), 4);
    assert!(encounter
        .party()
        .iter()
        .all(|id| encounter.combatant(*id).unwrap().hit_points == 0));

    // Separate fight: dropping the enemy to 0 HP does not end it. Death does.
    let mut frail = guard();
    frail.hit_points = 10;
    let mut dice = ScriptedDice::new([20, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[frail],
        &[
            GridPos::new(1, 1),
            GridPos::new(0, 0),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(2, 1)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    let mut dice = ScriptedDice::new([15, 8, 10, 1, 16, 1]);
    declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    assert!(encounter.outcome().is_none());
    assert!(!encounter.combatant(CombatantId(4)).unwrap().dead);
    declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    assert!(encounter.outcome().is_none());
    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        Event::EncounterEnded {
            outcome: Outcome::Victory
        }
    )));
    assert!(encounter.combatant(CombatantId(4)).unwrap().dead);
}

#[test]
fn ending_the_turn_early_advances_and_resets_the_action_economy() {
    let mut dice = ScriptedDice::new([15, 10, 5, 2, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[guard()],
        &[
            GridPos::new(0, 0),
            GridPos::new(1, 0),
            GridPos::new(2, 0),
            GridPos::new(3, 0),
        ],
        &[GridPos::new(5, 0)],
        open(8, 3),
        &mut dice,
    )
    .unwrap();
    let first = encounter.active().unwrap();
    declare_action(
        &mut encounter,
        first,
        Action::Stride {
            destination: GridPos::new(0, 1),
        },
        &mut dice,
    )
    .ok();
    // The first actor might not be able to move to (0, 1) if they did not start at x=0.
    // End the turn from whoever is active.
    let active = encounter.active().unwrap();
    let events = end_turn(&mut encounter, &mut dice).unwrap();
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::TurnStarted { .. })));
    assert_ne!(encounter.active(), Some(active));
    assert_eq!(encounter.actions_remaining(), 3);
    assert_eq!(encounter.strikes_this_turn(), 0);
}

#[test]
fn a_move_does_not_increase_map() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[guard()],
        &[
            GridPos::new(0, 1),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
            GridPos::new(0, 5),
        ],
        &[GridPos::new(3, 1)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Stride {
            destination: GridPos::new(2, 1),
        },
        &mut dice,
    )
    .unwrap();
    let mut dice = ScriptedDice::new([10, 4]);
    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    match &events[0] {
        Event::StrikeResolved { map, .. } => assert_eq!(*map, 0),
        other => panic!("unexpected {other:?}"),
    }
}

/// Player Core, reactions: one reaction, regained at the start of your turn.
#[test]
fn reaction_returns_at_the_start_of_your_turn() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[guard()],
        &[
            GridPos::new(0, 0),
            GridPos::new(1, 0),
            GridPos::new(2, 0),
            GridPos::new(3, 0),
        ],
        &[GridPos::new(5, 0)],
        open(8, 4),
        &mut dice,
    )
    .unwrap();

    assert_eq!(encounter.active(), Some(CombatantId(0)));
    for combatant in encounter.combatants() {
        assert!(combatant.is_conscious());
        assert_eq!(encounter.reaction_ready(combatant.id), Some(true));
    }

    encounter.spend_reaction(CombatantId(0)).unwrap();
    encounter.spend_reaction(CombatantId(2)).unwrap();
    assert_eq!(encounter.reaction_ready(CombatantId(0)), Some(false));
    assert_eq!(encounter.reaction_ready(CombatantId(2)), Some(false));
    assert_eq!(
        encounter.spend_reaction(CombatantId(0)),
        Err(Illegal::NoReaction)
    );

    let events = end_turn(&mut encounter, &mut dice).unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        Event::TurnStarted { id, .. } if *id == CombatantId(1)
    )));
    assert_eq!(encounter.reaction_ready(CombatantId(0)), Some(false));
    assert_eq!(encounter.reaction_ready(CombatantId(1)), Some(true));
    assert_eq!(encounter.reaction_ready(CombatantId(2)), Some(false));

    let events = end_turn(&mut encounter, &mut dice).unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        Event::TurnStarted { id, .. } if *id == CombatantId(2)
    )));
    assert_eq!(encounter.reaction_ready(CombatantId(2)), Some(true));
    assert_eq!(encounter.reaction_ready(CombatantId(0)), Some(false));

    while encounter.active() != Some(CombatantId(0)) {
        end_turn(&mut encounter, &mut dice).unwrap();
    }
    assert_eq!(encounter.round(), 2);
    assert_eq!(encounter.actions_remaining(), 3);
    assert_eq!(encounter.reaction_ready(CombatantId(0)), Some(true));
}

/// Stride, Step, and Strike spend actions and leave the reaction flag set.
#[test]
fn stride_step_and_strike_do_not_spend_the_reaction() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[guard()],
        &[
            GridPos::new(1, 1),
            GridPos::new(0, 0),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(2, 1)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(0)));

    declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Stride {
            destination: GridPos::new(1, 0),
        },
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.actions_remaining(), 2);
    assert_eq!(encounter.reaction_ready(CombatantId(0)), Some(true));

    declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Step {
            destination: GridPos::new(2, 0),
        },
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.actions_remaining(), 1);
    assert_eq!(encounter.strikes_this_turn(), 0);
    assert_eq!(encounter.reaction_ready(CombatantId(0)), Some(true));

    let mut strike_dice = ScriptedDice::new([10, 4]);
    declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut strike_dice,
    )
    .unwrap();
    assert_eq!(encounter.actions_remaining(), 0);
    assert_eq!(encounter.strikes_this_turn(), 1);
    for combatant in encounter.combatants() {
        assert_eq!(encounter.reaction_ready(combatant.id), Some(true));
    }
}

fn reactive_party() -> [CreatureTemplate; 4] {
    let mut party = party();
    party[0].features.push(ClassFeature::ReactiveStrike);
    party
}

/// Player Core, Reactive Strike: leaving a square in reach provokes at multiple attack penalty 0.
/// Damage applies on a success and doubles on a critical success. The reaction does not change the strike count.
#[test]
fn reactive_strike_ignores_map() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 16, 2, 10, 4]);
    let mut encounter = start_encounter(
        &reactive_party(),
        &[guard()],
        &[
            GridPos::new(1, 1),
            GridPos::new(0, 0),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(2, 1)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(4)));
    assert_eq!(
        encounter.combatant(CombatantId(0)).unwrap().features,
        vec![ClassFeature::ReactiveStrike]
    );
    assert!(encounter
        .combatant(CombatantId(4))
        .unwrap()
        .features
        .is_empty());
    assert_eq!(
        reactors_for_left_square(&encounter, CombatantId(4), GridPos::new(2, 1)),
        vec![CombatantId(0)]
    );

    // The guard's Strike spends the first attack. A later Strike would take −5.
    let strike = declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Strike {
            target: CombatantId(0),
        },
        &mut dice,
    )
    .unwrap();
    assert!(strike
        .iter()
        .all(|event| matches!(event, Event::StrikeResolved { .. })));
    assert_eq!(encounter.strikes_this_turn(), 1);

    // 10 + 7 + 0 = 17 vs AC 14 is a success. With −5 the same d20 would miss. Damage 4 + 4 = 8.
    let events = declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Stride {
            destination: GridPos::new(4, 1),
        },
        &mut dice,
    )
    .unwrap();
    assert_eq!(
        events,
        vec![
            Event::ReactiveStrike {
                attacker: CombatantId(0),
                target: CombatantId(4),
                natural: 10,
                attack_bonus: 7,
                map: 0,
                total: 17,
                armor_class: 14,
                degree: Degree::Success,
                damage: 8,
                target_hit_points: 10,
            },
            Event::Moved {
                id: CombatantId(4),
                from: GridPos::new(2, 1),
                to: GridPos::new(4, 1),
                feet: 10,
                actions_remaining: 1,
            },
        ]
    );
    assert_eq!(encounter.strikes_this_turn(), 1);
    assert_eq!(encounter.reaction_ready(CombatantId(0)), Some(false));
    assert_eq!(encounter.reaction_ready(CombatantId(4)), Some(true));
    assert_eq!(
        encounter.combatant(CombatantId(4)).unwrap().position,
        GridPos::new(4, 1)
    );
    assert_eq!(dice.remaining(), 0);

    // 18 + 7 + 0 = 25 vs AC 14 is a critical success, so 8 damage doubles to 16.
    // 18 + 7 − 5 = 20 would only be a success.
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 16, 2, 18, 4]);
    let mut encounter = start_encounter(
        &reactive_party(),
        &[guard()],
        &[
            GridPos::new(1, 1),
            GridPos::new(0, 0),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(2, 1)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Strike {
            target: CombatantId(0),
        },
        &mut dice,
    )
    .unwrap();
    let events = declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Stride {
            destination: GridPos::new(4, 1),
        },
        &mut dice,
    )
    .unwrap();
    match &events[0] {
        Event::ReactiveStrike {
            map,
            degree,
            damage,
            total,
            target_hit_points,
            ..
        } => {
            assert_eq!(*map, 0);
            assert_eq!(*degree, Degree::CriticalSuccess);
            assert_eq!(*damage, 16);
            assert_eq!(*total, 25);
            assert_eq!(*target_hit_points, 2);
        }
        other => panic!("unexpected {other:?}"),
    }
    assert!(matches!(events[1], Event::Moved { .. }));
    assert_eq!(encounter.strikes_this_turn(), 1);
    assert_eq!(dice.remaining(), 0);

    // A fighter does not Reactive Strike an ally who leaves their reach.
    let mut allies = party();
    allies[0].features.push(ClassFeature::ReactiveStrike);
    allies[1].features.push(ClassFeature::ReactiveStrike);
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &allies,
        &[guard()],
        &[
            GridPos::new(0, 0),
            GridPos::new(1, 0),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(0, 6)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(0)));
    assert!(reactors_for_left_square(&encounter, CombatantId(0), GridPos::new(0, 0)).is_empty());
    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Stride {
            destination: GridPos::new(2, 0),
        },
        &mut dice,
    )
    .unwrap();
    assert!(events
        .iter()
        .all(|event| matches!(event, Event::Moved { .. })));
    assert_eq!(encounter.reaction_ready(CombatantId(1)), Some(true));
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, Step: a Step does not trigger Reactive Strike.
#[test]
fn step_does_not_provoke() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 16]);
    let mut encounter = start_encounter(
        &reactive_party(),
        &[guard()],
        &[
            GridPos::new(1, 1),
            GridPos::new(0, 0),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(2, 1)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(4)));
    assert_eq!(
        reactors_for_left_square(&encounter, CombatantId(4), GridPos::new(2, 1)),
        vec![CombatantId(0)]
    );

    let events = declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Step {
            destination: GridPos::new(3, 1),
        },
        &mut dice,
    )
    .unwrap();
    assert_eq!(
        events,
        vec![Event::Stepped {
            id: CombatantId(4),
            from: GridPos::new(2, 1),
            to: GridPos::new(3, 1),
            feet: 5,
            actions_remaining: 2,
        }]
    );
    assert_eq!(encounter.reaction_ready(CombatantId(0)), Some(true));
    assert_eq!(encounter.strikes_this_turn(), 0);
    assert_eq!(dice.remaining(), 0);
    assert_eq!(
        encounter.combatant(CombatantId(4)).unwrap().position,
        GridPos::new(3, 1)
    );
}

fn adjacent(enemy: CreatureTemplate, dice: &mut ScriptedDice) -> rules::Encounter {
    start_encounter(
        &party(),
        &[enemy],
        &[
            GridPos::new(1, 1),
            GridPos::new(0, 0),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(2, 1)],
        open(8, 8),
        dice,
    )
    .unwrap()
}

/// Player Core, knocked out: a hit from positive HP to 0 sets dying 1 plus wounded, then wounded increases.
#[test]
fn a_hit_that_drops_a_creature_sets_dying_and_wounded() {
    let mut enemy = guard();
    enemy.hit_points = 8;
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = adjacent(enemy, &mut dice);
    assert_eq!(encounter.active(), Some(CombatantId(0)));

    // 10 + 7 = 17 vs AC 14 success. Damage 4 + 4 = 8.
    let mut dice = ScriptedDice::new([10, 4]);
    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        Event::CombatantDowned { id } if *id == CombatantId(4)
    )));
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::CombatantDied { .. })));
    let guard = encounter.combatant(CombatantId(4)).unwrap();
    assert_eq!(guard.hit_points, 0);
    assert!(guard.is_unconscious());
    assert!(!guard.is_conscious());
    assert!(guard.prone);
    assert_eq!(guard.dying, 1);
    assert_eq!(guard.wounded, 1);
    assert!(!guard.dead);
    assert!(encounter.outcome().is_none());
}

/// Player Core, knocked out: a critical success that drops a creature sets dying 2 plus wounded.
#[test]
fn a_critical_hit_that_drops_a_creature_sets_dying_two_plus_wounded() {
    let mut enemy = guard();
    enemy.hit_points = 12;
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = adjacent(enemy, &mut dice);

    // Natural 20 is a critical success. Damage 4 + 4 = 8, doubled to 16.
    let mut dice = ScriptedDice::new([20, 4]);
    declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    let guard = encounter.combatant(CombatantId(4)).unwrap();
    assert_eq!(guard.hit_points, 0);
    assert_eq!(guard.dying, 2);
    assert_eq!(guard.wounded, 1);
    assert!(guard.prone);
    assert!(!guard.dead);
    assert!(encounter.outcome().is_none());
}

/// Wounded from an earlier drop is added the next time the creature gains dying.
#[test]
fn wounded_increases_the_next_dying_value() {
    let mut enemy = guard();
    enemy.hit_points = 8;
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = adjacent(enemy, &mut dice);

    // Drop: dying 1, wounded 1. Recovery 11 vs DC 11 is a success, so dying falls to 0.
    // The later hit is a critical success against the unconscious creature: dying 2 + wounded 1.
    let mut dice = ScriptedDice::new([10, 4, 11, 12, 3]);
    declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.combatant(CombatantId(4)).unwrap().dying, 1);
    assert_eq!(encounter.combatant(CombatantId(4)).unwrap().wounded, 1);

    while encounter.round() == 1 || encounter.active() != Some(CombatantId(0)) {
        end_turn(&mut encounter, &mut dice).unwrap();
    }
    let stable = encounter.combatant(CombatantId(4)).unwrap();
    assert_eq!(stable.dying, 0);
    assert_eq!(stable.wounded, 1);
    assert_eq!(stable.hit_points, 0);
    assert!(stable.prone);
    assert!(stable.is_unconscious());
    assert!(!stable.dead);

    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    match &events[0] {
        Event::StrikeResolved { degree, .. } => assert_eq!(*degree, Degree::CriticalSuccess),
        other => panic!("unexpected {other:?}"),
    }
    let guard = encounter.combatant(CombatantId(4)).unwrap();
    assert_eq!(guard.dying, 3);
    assert_eq!(guard.wounded, 2);
    assert!(!guard.dead);
    assert!(encounter.outcome().is_none());
}

/// Damage while dying increases dying and does not increase wounded. A success becomes a critical success.
#[test]
fn damage_while_dying_increases_dying_without_wounded() {
    let mut enemy = guard();
    enemy.hit_points = 8;
    enemy.armor_class = 18;
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = adjacent(enemy, &mut dice);

    // 11 + 7 = 18 vs AC 18 success. Damage 4 + 4 = 8. Dying 1, wounded 1.
    // 10 + 7 - 5 = 12 vs off-guard AC 16 would miss. The penalty is −2 once, so AC 16, not 14.
    // Use a roll that hits the penalized AC: 14 + 7 - 5 = 16 vs AC 16 success, which upgrades to a critical success.
    let mut dice = ScriptedDice::new([11, 4, 14, 1]);
    declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    match &events[0] {
        Event::StrikeResolved {
            degree,
            armor_class,
            damage,
            ..
        } => {
            assert_eq!(*degree, Degree::CriticalSuccess);
            assert_eq!(*armor_class, 16);
            assert_eq!(*damage, 10);
        }
        other => panic!("unexpected {other:?}"),
    }
    let guard = encounter.combatant(CombatantId(4)).unwrap();
    assert_eq!(guard.dying, 3);
    assert_eq!(guard.wounded, 1);
    assert!(!guard.dead);
}

/// Dying 4 kills. The corpse stays in its square and the party still has four slots.
#[test]
fn dying_four_kills_and_a_dead_party_slot_remains() {
    let mut enemy = guard();
    enemy.hit_points = 30;
    enemy.strike = strike("glaive", 12, 1, 4, 20);
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 20]);
    let mut encounter = adjacent(enemy, &mut dice);
    assert_eq!(encounter.active(), Some(CombatantId(4)));

    // 10 + 12 = 22 vs AC 18 success. Damage 1 + 20 = 21. Dying 1, wounded 1.
    // A success against unconscious becomes a critical success, so later hits add 2 dying.
    // 10 + 12 − 5 = 17 vs off-guard AC 16. Damage doubled to 42. Dying 3.
    // 16 + 12 − 10 = 18 vs AC 16. Dying 5, which kills.
    let mut dice = ScriptedDice::new([10, 1, 10, 1, 16, 1]);
    declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Strike {
            target: CombatantId(0),
        },
        &mut dice,
    )
    .unwrap();
    assert!(encounter.outcome().is_none());
    assert_eq!(encounter.combatant(CombatantId(0)).unwrap().dying, 1);
    assert_eq!(encounter.party().len(), 4);
    assert_eq!(encounter.party()[0], CombatantId(0));

    declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Strike {
            target: CombatantId(0),
        },
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.combatant(CombatantId(0)).unwrap().dying, 3);
    assert_eq!(encounter.combatant(CombatantId(0)).unwrap().wounded, 1);
    assert!(encounter.outcome().is_none());

    let events = declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Strike {
            target: CombatantId(0),
        },
        &mut dice,
    )
    .unwrap();
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::CombatantDied { id } if *id == CombatantId(0))));
    let slain = encounter.combatant(CombatantId(0)).unwrap();
    assert!(slain.dead);
    assert_eq!(slain.dying, 0);
    assert_eq!(slain.hit_points, 0);
    assert_eq!(slain.position, GridPos::new(1, 1));
    assert_eq!(encounter.party().len(), 4);
    assert!(encounter.party().contains(&CombatantId(0)));
    assert!(encounter.outcome().is_none());
    assert!(
        !legal_actions(&encounter, CombatantId(4)).contains(&Action::Strike {
            target: CombatantId(0)
        })
    );
    let err = declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Strike {
            target: CombatantId(0),
        },
        &mut dice,
    )
    .unwrap_err();
    assert_eq!(err, Illegal::NotLegal);
}

/// Player Core, recovery check: DC 10 + dying, one step per degree.
#[test]
fn recovery_checks_cover_each_degree() {
    // Dying 2. DC 12.
    // Natural 20: success improved to a critical success, dying 0.
    // Natural 12: success, dying 1.
    // Natural 11: failure, dying 3.
    // Natural 1: critical failure, dying 4, dead.
    let cases = [
        (20, Degree::CriticalSuccess, 0, false),
        (12, Degree::Success, 1, false),
        (11, Degree::Failure, 3, false),
        (1, Degree::CriticalFailure, 0, true),
    ];
    for (roll, degree, dying, dead) in cases {
        let mut enemy = guard();
        enemy.hit_points = 16;
        let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
        let mut encounter = adjacent(enemy, &mut dice);
        // Natural 20, damage 6 + 4 = 10 doubled to 20. Dying 2, wounded 1.
        let mut dice = ScriptedDice::new([20, 6, roll]);
        declare_action(
            &mut encounter,
            CombatantId(0),
            Action::Strike {
                target: CombatantId(4),
            },
            &mut dice,
        )
        .unwrap();
        assert_eq!(encounter.combatant(CombatantId(4)).unwrap().dying, 2);

        let mut events = Vec::new();
        for _ in 0..6 {
            events = end_turn(&mut encounter, &mut dice).unwrap();
            if events.iter().any(
                |event| matches!(event, Event::RecoveryCheck { id, .. } if *id == CombatantId(4)),
            ) {
                break;
            }
        }
        assert!(
            events.iter().any(|event| matches!(
                event,
                Event::RecoveryCheck {
                    id,
                    natural,
                    dc,
                    degree: rolled,
                    dying: now,
                } if *id == CombatantId(4)
                    && *natural == roll
                    && *dc == 12
                    && *rolled == degree
                    && *now == dying
            )),
            "recovery for roll {roll} missing in {events:?}"
        );
        let guard = encounter.combatant(CombatantId(4)).unwrap();
        assert_eq!(guard.dying, dying);
        assert_eq!(guard.dead, dead);
        assert_eq!(guard.wounded, 1);
        assert!(guard.prone);
        assert_eq!(guard.hit_points, 0);
        if !dead {
            assert!(guard.is_unconscious());
            assert!(!guard.is_conscious());
            assert!(legal_actions(&encounter, CombatantId(4)).is_empty());
        }
    }
}

/// Player Core, massive damage: one hit of at least twice maximum HP kills, and dying is not applied.
#[test]
fn massive_damage_kills_without_dying() {
    let mut enemy = guard();
    enemy.strike = strike("glaive", 12, 1, 4, 40);
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 20]);
    let mut encounter = adjacent(enemy, &mut dice);
    assert_eq!(encounter.active(), Some(CombatantId(4)));

    let mut dice = ScriptedDice::new([10, 1]);
    let events = declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Strike {
            target: CombatantId(0),
        },
        &mut dice,
    )
    .unwrap();
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::CombatantDied { id } if *id == CombatantId(0))));
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::CombatantDowned { .. })));
    let slain = encounter.combatant(CombatantId(0)).unwrap();
    assert!(slain.dead);
    assert_eq!(slain.dying, 0);
    assert_eq!(slain.wounded, 0);
    assert_eq!(slain.hit_points, 0);
    assert_eq!(encounter.party().len(), 4);
    assert!(encounter.outcome().is_none());
}

/// Off-guard is −2 for a flanking attacker and not for an attacker outside that pair.
#[test]
fn flanking_off_guard_applies_only_to_the_flanking_attacker() {
    let mut enemy = guard();
    enemy.hit_points = 80;
    enemy.armor_class = 18;
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[enemy],
        &[
            GridPos::new(5, 4),
            GridPos::new(4, 5),
            GridPos::new(6, 5),
            GridPos::new(0, 0),
        ],
        &[GridPos::new(5, 5)],
        open(10, 10),
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(0)));

    // South attacker is not opposite either ally. 11 + 7 = 18 vs AC 18.
    let mut dice = ScriptedDice::new([11, 1]);
    let south = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    match &south[0] {
        Event::StrikeResolved { armor_class, .. } => assert_eq!(*armor_class, 18),
        other => panic!("unexpected {other:?}"),
    }

    end_turn(&mut encounter, &mut dice).unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(1)));
    // West attacker, with the east ally, flanks. AC 16.
    let mut dice = ScriptedDice::new([11, 1]);
    let west = declare_action(
        &mut encounter,
        CombatantId(1),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    match &west[0] {
        Event::StrikeResolved { armor_class, .. } => assert_eq!(*armor_class, 16),
        other => panic!("unexpected {other:?}"),
    }
}

/// An unconscious creature has no legal Stride or Strike.
#[test]
fn an_unconscious_creature_cannot_stride_or_strike() {
    let mut mover = fighter("Sera Voss");
    mover.hit_points = 20;
    let mut reactor = guard();
    reactor.features.push(ClassFeature::ReactiveStrike);
    reactor.strike.attack_bonus = 12;
    reactor.strike.damage.bonus = 20;
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &[
            mover,
            fighter("Bram Holt"),
            fighter("Quill Ames"),
            fighter("Doss Kade"),
        ],
        &[reactor],
        &[
            GridPos::new(1, 1),
            GridPos::new(0, 0),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(2, 1)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(0)));

    // 10 + 12 = 22 vs AC 18 success. Damage 1 + 20 = 21. Twice maximum HP is 40, so this is dying, not death.
    let mut dice = ScriptedDice::new([10, 1]);
    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Stride {
            destination: GridPos::new(1, 3),
        },
        &mut dice,
    )
    .unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        Event::CombatantDowned { id } if *id == CombatantId(0)
    )));
    let downed = encounter.combatant(CombatantId(0)).unwrap();
    assert!(downed.is_unconscious());
    assert!(downed.prone);
    assert_eq!(downed.dying, 1);
    assert!(!downed.dead);
    assert_eq!(encounter.active(), Some(CombatantId(0)));
    let legal = legal_actions(&encounter, CombatantId(0));
    assert!(legal.iter().all(|action| !matches!(
        action,
        Action::Stride { .. } | Action::Strike { .. } | Action::Step { .. }
    )));
    assert!(legal.is_empty());
    let err = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap_err();
    assert_eq!(err, Illegal::NotLegal);
    let err = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Stride {
            destination: GridPos::new(1, 4),
        },
        &mut dice,
    )
    .unwrap_err();
    assert_eq!(err, Illegal::NotLegal);
}

/// A Reactive Strike that drops the mover ends the Stride on the square they were leaving.
#[test]
fn downed_mover_stops_on_the_left_square() {
    let mut reactor = fighter("Sera Voss");
    reactor.features.push(ClassFeature::ReactiveStrike);
    reactor.strike.reach_feet = 10;
    let mut second = fighter("Bram Holt");
    second.features.push(ClassFeature::ReactiveStrike);
    let mut mover = guard();
    mover.hit_points = 8;
    let mut witness = guard();
    witness.name = "Witness".into();

    let blocked: Vec<GridPos> = (0..8)
        .flat_map(|x| [GridPos::new(x, 0), GridPos::new(x, 2)])
        .filter(|pos| *pos != GridPos::new(1, 2))
        .collect();
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 18, 1, 10, 4]);
    let mut encounter = start_encounter(
        &[reactor, second, fighter("Quill Ames"), fighter("Doss Kade")],
        &[mover, witness],
        &[
            GridPos::new(0, 1),
            GridPos::new(1, 2),
            GridPos::new(5, 1),
            GridPos::new(6, 1),
        ],
        &[GridPos::new(3, 1), GridPos::new(7, 1)],
        Grid::new(8, 3, blocked),
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(4)));
    assert_eq!(
        reactors_for_left_square(&encounter, CombatantId(4), GridPos::new(3, 1)),
        Vec::<CombatantId>::new()
    );
    assert_eq!(
        reactors_for_left_square(&encounter, CombatantId(4), GridPos::new(2, 1)),
        vec![CombatantId(0), CombatantId(1)]
    );

    let events = declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Stride {
            destination: GridPos::new(1, 1),
        },
        &mut dice,
    )
    .unwrap();
    assert_eq!(
        events,
        vec![
            Event::ReactiveStrike {
                attacker: CombatantId(0),
                target: CombatantId(4),
                natural: 10,
                attack_bonus: 7,
                map: 0,
                total: 17,
                armor_class: 14,
                degree: Degree::Success,
                damage: 8,
                target_hit_points: 0,
            },
            Event::CombatantDowned { id: CombatantId(4) },
        ]
    );
    assert_eq!(
        encounter.combatant(CombatantId(4)).unwrap().position,
        GridPos::new(2, 1)
    );
    assert_eq!(encounter.combatant(CombatantId(4)).unwrap().hit_points, 0);
    assert_eq!(encounter.actions_remaining(), 2);
    assert_eq!(encounter.strikes_this_turn(), 0);
    assert_eq!(encounter.reaction_ready(CombatantId(0)), Some(false));
    assert_eq!(encounter.reaction_ready(CombatantId(1)), Some(true));
    assert!(encounter.outcome().is_none());
    assert_eq!(dice.remaining(), 0);
}

fn place(sera: CreatureTemplate, foe: CreatureTemplate, dice: &mut ScriptedDice) -> Encounter {
    start_encounter(
        &[
            sera,
            fighter("Bram Holt"),
            fighter("Quill Ames"),
            fighter("Doss Kade"),
        ],
        &[foe],
        &[
            GridPos::new(1, 1),
            GridPos::new(0, 0),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(2, 1)],
        open(8, 8),
        dice,
    )
    .unwrap()
}

/// Player Core, Raise a Shield: one action, +2 circumstance bonus to AC.
#[test]
fn raise_shield_spends_one_action_and_adds_two_ac() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1, 2]);
    let mut encounter = place(shielded("Sera Voss", true), guard(), &mut dice);
    assert_eq!(encounter.active(), Some(CombatantId(0)));
    assert!(legal_actions(&encounter, CombatantId(0)).contains(&Action::RaiseShield));
    let shield = encounter
        .combatant(CombatantId(0))
        .unwrap()
        .shield
        .as_ref()
        .unwrap();
    assert_eq!(shield.hardness, 5);
    assert!(!shield.raised);

    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::RaiseShield,
        &mut dice,
    )
    .unwrap();
    assert_eq!(
        events,
        vec![Event::ShieldRaised {
            id: CombatantId(0),
            actions_remaining: 2,
        }]
    );
    assert_eq!(encounter.actions_remaining(), 2);
    assert!(
        encounter
            .combatant(CombatantId(0))
            .unwrap()
            .shield
            .as_ref()
            .unwrap()
            .raised
    );

    end_until(&mut encounter, CombatantId(4), &mut dice);
    let events = declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Strike {
            target: CombatantId(0),
        },
        &mut dice,
    )
    .unwrap();
    assert!(encounter.pending_shield_block().is_none());
    match events[0] {
        Event::StrikeResolved {
            armor_class,
            damage,
            ..
        } => {
            assert_eq!(armor_class, 20);
            assert_eq!(damage, 0);
        }
        ref other => panic!("expected the strike, got {other:?}"),
    }
}

/// The circumstance bonus lasts until the start of that creature's next turn.
#[test]
fn raised_shield_ends_at_the_next_turn() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1, 2, 2]);
    let mut encounter = place(shielded("Sera Voss", true), guard(), &mut dice);
    declare_action(
        &mut encounter,
        CombatantId(0),
        Action::RaiseShield,
        &mut dice,
    )
    .unwrap();
    end_until(&mut encounter, CombatantId(4), &mut dice);
    let events = declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Strike {
            target: CombatantId(0),
        },
        &mut dice,
    )
    .unwrap();
    assert!(matches!(
        events[0],
        Event::StrikeResolved {
            armor_class: 20,
            ..
        }
    ));

    end_turn(&mut encounter, &mut dice).unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(0)));
    assert_eq!(encounter.round(), 2);
    assert!(
        !encounter
            .combatant(CombatantId(0))
            .unwrap()
            .shield
            .as_ref()
            .unwrap()
            .raised
    );

    end_until(&mut encounter, CombatantId(4), &mut dice);
    let events = declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Strike {
            target: CombatantId(0),
        },
        &mut dice,
    )
    .unwrap();
    assert!(matches!(
        events[0],
        Event::StrikeResolved {
            armor_class: 18,
            ..
        }
    ));
}

#[test]
fn broken_shield_cannot_be_raised() {
    for hit_points in [10, 0] {
        let mut sera = shielded("Sera Voss", true);
        sera.shield.as_mut().unwrap().hit_points = hit_points;
        let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
        let mut encounter = place(sera, guard(), &mut dice);
        assert!(!legal_actions(&encounter, CombatantId(0)).contains(&Action::RaiseShield));
        let err = declare_action(
            &mut encounter,
            CombatantId(0),
            Action::RaiseShield,
            &mut dice,
        )
        .unwrap_err();
        assert_eq!(err, Illegal::NotLegal);
        assert!(
            !encounter
                .combatant(CombatantId(0))
                .unwrap()
                .shield
                .as_ref()
                .unwrap()
                .raised
        );
    }
}

/// Hardness 5 against 12 damage. Creature and shield each take 7.
#[test]
fn shield_block_reduces_by_hardness_and_splits_the_rest() {
    let mut foe = guard();
    foe.strike.attack_bonus = 10;
    foe.strike.damage = DamageFormula {
        dice: 1,
        faces: 8,
        bonus: 4,
    };
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1, 10, 8]);
    let mut encounter = place(shielded("Sera Voss", true), foe, &mut dice);
    declare_action(
        &mut encounter,
        CombatantId(0),
        Action::RaiseShield,
        &mut dice,
    )
    .unwrap();
    end_until(&mut encounter, CombatantId(4), &mut dice);

    let events = declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Strike {
            target: CombatantId(0),
        },
        &mut dice,
    )
    .unwrap();
    assert!(events.is_empty());
    assert_eq!(encounter.pending_shield_block().unwrap().damage, 12);
    let events = resolve_pending_shield_block(&mut encounter, true, &mut dice).unwrap();
    assert_eq!(
        events[0],
        Event::ShieldBlock {
            defender: CombatantId(0),
            prevented: 5,
            damage: 7,
            shield_hit_points: 13,
            broken: false,
            destroyed: false,
        }
    );
    assert!(matches!(
        events[1],
        Event::StrikeResolved {
            damage: 7,
            target_hit_points: 13,
            armor_class: 20,
            ..
        }
    ));
    let sera = encounter.combatant(CombatantId(0)).unwrap();
    assert_eq!(sera.hit_points, 13);
    let shield = sera.shield.as_ref().unwrap();
    assert_eq!(shield.hit_points, 13);
    assert!(!shield.is_broken());
    assert_eq!(encounter.reaction_ready(CombatantId(0)), Some(false));
    assert!(encounter.pending_shield_block().is_none());
}

/// Remaining damage that lands on the broken threshold breaks the shield.
#[test]
fn shield_breaks_when_hit_points_reach_the_threshold() {
    let mut foe = guard();
    foe.strike.attack_bonus = 11;
    foe.strike.damage = DamageFormula {
        dice: 1,
        faces: 10,
        bonus: 5,
    };
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1, 9, 10, 9]);
    let mut encounter = place(shielded("Sera Voss", true), foe, &mut dice);
    declare_action(
        &mut encounter,
        CombatantId(0),
        Action::RaiseShield,
        &mut dice,
    )
    .unwrap();
    end_until(&mut encounter, CombatantId(4), &mut dice);

    declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Strike {
            target: CombatantId(0),
        },
        &mut dice,
    )
    .unwrap();
    let events = resolve_pending_shield_block(&mut encounter, true, &mut dice).unwrap();
    assert_eq!(
        events[0],
        Event::ShieldBlock {
            defender: CombatantId(0),
            prevented: 5,
            damage: 10,
            shield_hit_points: 10,
            broken: true,
            destroyed: false,
        }
    );
    let shield = encounter
        .combatant(CombatantId(0))
        .unwrap()
        .shield
        .as_ref()
        .unwrap();
    assert_eq!(shield.hit_points, shield.broken_threshold);
    assert!(shield.is_broken());
    assert!(!shield.is_destroyed());

    let events = declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Strike {
            target: CombatantId(0),
        },
        &mut dice,
    )
    .unwrap();
    assert!(encounter.pending_shield_block().is_none());
    assert!(matches!(
        events[0],
        Event::StrikeResolved {
            armor_class: 18,
            ..
        }
    ));
    assert_eq!(
        encounter
            .combatant(CombatantId(0))
            .unwrap()
            .shield
            .as_ref()
            .unwrap()
            .hit_points,
        10
    );

    end_until(&mut encounter, CombatantId(0), &mut dice);
    assert!(!legal_actions(&encounter, CombatantId(0)).contains(&Action::RaiseShield));
}

/// Shield Block and Reactive Strike spend one reaction. The second try in the round fails.
#[test]
fn shield_block_spends_the_reaction() {
    let mut sera = shielded("Sera Voss", true);
    sera.features.push(ClassFeature::ReactiveStrike);
    let mut foe = guard();
    foe.strike.attack_bonus = 16;
    foe.strike.damage = DamageFormula {
        dice: 1,
        faces: 8,
        bonus: 4,
    };
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1, 10, 8, 10, 8]);
    let mut encounter = place(sera, foe, &mut dice);
    declare_action(
        &mut encounter,
        CombatantId(0),
        Action::RaiseShield,
        &mut dice,
    )
    .unwrap();
    end_until(&mut encounter, CombatantId(4), &mut dice);

    declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Strike {
            target: CombatantId(0),
        },
        &mut dice,
    )
    .unwrap();
    resolve_pending_shield_block(&mut encounter, true, &mut dice).unwrap();
    assert_eq!(encounter.combatant(CombatantId(0)).unwrap().hit_points, 13);
    assert_eq!(
        encounter
            .combatant(CombatantId(0))
            .unwrap()
            .shield
            .as_ref()
            .unwrap()
            .hit_points,
        13
    );
    assert_eq!(
        encounter.spend_reaction(CombatantId(0)),
        Err(Illegal::NoReaction)
    );

    let events = declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Strike {
            target: CombatantId(0),
        },
        &mut dice,
    )
    .unwrap();
    assert!(encounter.pending_shield_block().is_none());
    assert!(matches!(
        events[0],
        Event::StrikeResolved {
            damage: 12,
            target_hit_points: 1,
            ..
        }
    ));
    assert_eq!(
        encounter
            .combatant(CombatantId(0))
            .unwrap()
            .shield
            .as_ref()
            .unwrap()
            .hit_points,
        13
    );

    assert!(reactors_for_left_square(&encounter, CombatantId(4), GridPos::new(2, 1)).is_empty());
    let events = declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Stride {
            destination: GridPos::new(5, 1),
        },
        &mut dice,
    )
    .unwrap();
    assert!(events
        .iter()
        .all(|event| !matches!(event, Event::ReactiveStrike { .. })));
    assert!(matches!(events[0], Event::Moved { .. }));
}

#[test]
fn fighter_without_shield_block_cannot_block() {
    let mut foe = guard();
    foe.strike.attack_bonus = 10;
    foe.strike.damage = DamageFormula {
        dice: 1,
        faces: 8,
        bonus: 4,
    };
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1, 10, 8]);
    let mut encounter = place(shielded("Sera Voss", false), foe, &mut dice);
    declare_action(
        &mut encounter,
        CombatantId(0),
        Action::RaiseShield,
        &mut dice,
    )
    .unwrap();
    end_until(&mut encounter, CombatantId(4), &mut dice);

    let events = declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Strike {
            target: CombatantId(0),
        },
        &mut dice,
    )
    .unwrap();
    assert!(encounter.pending_shield_block().is_none());
    assert!(matches!(
        events[0],
        Event::StrikeResolved {
            armor_class: 20,
            damage: 12,
            target_hit_points: 8,
            ..
        }
    ));
    let sera = encounter.combatant(CombatantId(0)).unwrap();
    assert!(!sera.shield_block);
    assert!(sera.features.is_empty());
    assert_eq!(sera.shield.as_ref().unwrap().hit_points, 20);
    assert_eq!(encounter.reaction_ready(CombatantId(0)), Some(true));
    assert_eq!(
        resolve_pending_shield_block(&mut encounter, true, &mut dice),
        Err(Illegal::NotLegal)
    );
}

fn corner_party() -> [GridPos; 4] {
    [
        GridPos::new(2, 2),
        GridPos::new(6, 6),
        GridPos::new(6, 7),
        GridPos::new(7, 6),
    ]
}

/// Player Core, Drop Prone: one action, no movement, and it does not provoke.
#[test]
fn drop_prone_sets_prone_without_moving_or_provoking() {
    let mut reactor = guard();
    reactor.features.push(ClassFeature::ReactiveStrike);
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[reactor],
        &corner_party(),
        &[GridPos::new(3, 2)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(0)));
    let start = encounter.combatant(CombatantId(0)).unwrap().position;
    assert!(legal_actions(&encounter, CombatantId(0)).contains(&Action::DropProne));
    assert!(!legal_actions(&encounter, CombatantId(0)).contains(&Action::Stand));

    let events =
        declare_action(&mut encounter, CombatantId(0), Action::DropProne, &mut dice).unwrap();
    assert_eq!(
        events,
        vec![Event::DroppedProne {
            id: CombatantId(0),
            actions_remaining: 2,
        }]
    );
    let actor = encounter.combatant(CombatantId(0)).unwrap();
    assert!(actor.prone);
    assert!(actor.is_conscious());
    assert_eq!(actor.position, start);
    assert_eq!(encounter.reaction_ready(CombatantId(4)), Some(true));
    assert!(!legal_actions(&encounter, CombatantId(0)).contains(&Action::DropProne));
    assert_eq!(
        declare_action(&mut encounter, CombatantId(0), Action::DropProne, &mut dice).unwrap_err(),
        Illegal::NotLegal
    );
    assert_eq!(dice.remaining(), 0);
}

/// A prone attacker's −2 and a −5 multiple attack penalty are circumstance penalties.
/// The more severe one applies. Off-guard still lowers the target's Armor Class.
#[test]
fn prone_attack_penalty_does_not_stack_with_map() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[guard()],
        &[
            GridPos::new(1, 1),
            GridPos::new(0, 6),
            GridPos::new(0, 7),
            GridPos::new(7, 0),
        ],
        &[GridPos::new(2, 1)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    declare_action(&mut encounter, CombatantId(0), Action::DropProne, &mut dice).unwrap();
    end_until(&mut encounter, CombatantId(4), &mut dice);
    declare_action(&mut encounter, CombatantId(4), Action::DropProne, &mut dice).unwrap();
    end_until(&mut encounter, CombatantId(0), &mut dice);
    assert!(encounter.combatant(CombatantId(0)).unwrap().prone);
    assert!(encounter.combatant(CombatantId(4)).unwrap().prone);
    assert!(encounter.combatant(CombatantId(4)).unwrap().is_conscious());

    // 6 + 7 − 2 = 11 vs off-guard AC 12. A standing roll would be 13.
    let mut dice = ScriptedDice::new([6, 8]);
    let first = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    match &first[0] {
        Event::StrikeResolved {
            map,
            total,
            armor_class,
            degree,
            damage,
            ..
        } => {
            assert_eq!(*map, -2);
            assert_eq!(*total, 11);
            assert_eq!(*armor_class, 12);
            assert_eq!(*degree, Degree::Failure);
            assert_eq!(*damage, 0);
        }
        other => panic!("unexpected {other:?}"),
    }

    // 8 + 7 − 5 = 10. Stacking the prone penalty would make this 8.
    let second = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    match &second[0] {
        Event::StrikeResolved {
            map,
            total,
            armor_class,
            ..
        } => {
            assert_eq!(*map, -5);
            assert_eq!(*total, 10);
            assert_eq!(*armor_class, 12);
        }
        other => panic!("unexpected {other:?}"),
    }
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, Stand: one action, clears prone, no movement, and it does not provoke.
#[test]
fn stand_clears_prone_without_moving_or_provoking() {
    let mut reactor = guard();
    reactor.features.push(ClassFeature::ReactiveStrike);
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[reactor],
        &corner_party(),
        &[GridPos::new(3, 2)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    assert!(!legal_actions(&encounter, CombatantId(0)).contains(&Action::Stand));
    declare_action(&mut encounter, CombatantId(0), Action::DropProne, &mut dice).unwrap();
    let start = encounter.combatant(CombatantId(0)).unwrap().position;

    let events = declare_action(&mut encounter, CombatantId(0), Action::Stand, &mut dice).unwrap();
    assert_eq!(
        events,
        vec![Event::Stood {
            id: CombatantId(0),
            actions_remaining: 1,
        }]
    );
    let actor = encounter.combatant(CombatantId(0)).unwrap();
    assert!(!actor.prone);
    assert_eq!(actor.position, start);
    assert_eq!(encounter.reaction_ready(CombatantId(4)), Some(true));
    assert!(
        legal_actions(&encounter, CombatantId(0)).contains(&Action::Stride {
            destination: GridPos::new(2, 3)
        })
    );
    assert!(!legal_actions(&encounter, CombatantId(0)).contains(&Action::Stand));
    assert_eq!(
        declare_action(&mut encounter, CombatantId(0), Action::Stand, &mut dice).unwrap_err(),
        Illegal::NotLegal
    );
}

/// While prone, Stride, Step, and Leap are illegal. Crawl and Stand are the movement options.
#[test]
fn prone_creature_cannot_stride_step_or_leap() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[guard()],
        &corner_party(),
        &[GridPos::new(7, 7)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    declare_action(&mut encounter, CombatantId(0), Action::DropProne, &mut dice).unwrap();
    let legal = legal_actions(&encounter, CombatantId(0));
    assert!(legal.iter().all(|action| !matches!(
        action,
        Action::Stride { .. } | Action::Step { .. } | Action::Leap { .. }
    )));
    assert!(legal
        .iter()
        .any(|action| matches!(action, Action::Crawl { .. })));
    assert!(legal.contains(&Action::Stand));
    assert_eq!(
        declare_action(
            &mut encounter,
            CombatantId(0),
            Action::Stride {
                destination: GridPos::new(3, 2)
            },
            &mut dice,
        )
        .unwrap_err(),
        Illegal::NotLegal
    );
    assert_eq!(
        declare_action(
            &mut encounter,
            CombatantId(0),
            Action::Step {
                destination: GridPos::new(3, 2)
            },
            &mut dice,
        )
        .unwrap_err(),
        Illegal::NotLegal
    );
    assert_eq!(
        declare_action(
            &mut encounter,
            CombatantId(0),
            Action::Leap {
                destination: GridPos::new(4, 2)
            },
            &mut dice,
        )
        .unwrap_err(),
        Illegal::NotLegal
    );
}

/// Player Core, Crawl: one square that costs 5 feet. A 10-foot diagonal, and any square above 5, is out of reach.
#[test]
fn crawl_moves_five_feet_and_refuses_a_costlier_square() {
    let mut grid = open(8, 8);
    grid.set_terrain(GridPos::new(2, 3), Terrain::Difficult);
    grid.set_terrain(GridPos::new(2, 1), Terrain::Greater);
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[guard()],
        &corner_party(),
        &[GridPos::new(7, 7)],
        grid,
        &mut dice,
    )
    .unwrap();
    declare_action(&mut encounter, CombatantId(0), Action::DropProne, &mut dice).unwrap();
    let legal = legal_actions(&encounter, CombatantId(0));
    assert!(legal.contains(&Action::Crawl {
        destination: GridPos::new(3, 2)
    }));
    assert!(legal.contains(&Action::Crawl {
        destination: GridPos::new(3, 3)
    }));
    assert!(!legal.contains(&Action::Crawl {
        destination: GridPos::new(4, 2)
    }));
    assert!(!legal.contains(&Action::Crawl {
        destination: GridPos::new(4, 4)
    }));
    assert!(!legal.contains(&Action::Crawl {
        destination: GridPos::new(2, 3)
    }));
    assert!(!legal.contains(&Action::Crawl {
        destination: GridPos::new(2, 1)
    }));

    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Crawl {
            destination: GridPos::new(3, 2),
        },
        &mut dice,
    )
    .unwrap();
    assert_eq!(
        events,
        vec![Event::Crawled {
            id: CombatantId(0),
            from: GridPos::new(2, 2),
            to: GridPos::new(3, 2),
            feet: 5,
            actions_remaining: 1,
        }]
    );
    let actor = encounter.combatant(CombatantId(0)).unwrap();
    assert!(actor.prone);
    assert_eq!(actor.position, GridPos::new(3, 2));
}

/// Crawl leaves the start square, so it uses the same Reactive Strike path as Stride.
#[test]
fn crawl_provokes_reactive_strike() {
    let mut reactor = guard();
    reactor.features.push(ClassFeature::ReactiveStrike);
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[reactor],
        &[
            GridPos::new(1, 1),
            GridPos::new(0, 6),
            GridPos::new(0, 7),
            GridPos::new(7, 0),
        ],
        &[GridPos::new(2, 1)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    declare_action(&mut encounter, CombatantId(0), Action::DropProne, &mut dice).unwrap();
    assert_eq!(encounter.reaction_ready(CombatantId(4)), Some(true));

    let mut dice = ScriptedDice::new([1]);
    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Crawl {
            destination: GridPos::new(1, 2),
        },
        &mut dice,
    )
    .unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        Event::ReactiveStrike {
            attacker: CombatantId(4),
            target: CombatantId(0),
            map: 0,
            ..
        }
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Crawled {
            id: CombatantId(0),
            to: GridPos { x: 1, y: 2 },
            feet: 5,
            ..
        }
    )));
    assert_eq!(
        encounter.combatant(CombatantId(0)).unwrap().position,
        GridPos::new(1, 2)
    );
    assert!(encounter.combatant(CombatantId(0)).unwrap().prone);
    assert_eq!(encounter.reaction_ready(CombatantId(4)), Some(false));
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, Leap: Speed 25 reaches 10 feet and not 15. The leap can land on difficult terrain and does not cross a creature.
#[test]
fn leap_reaches_ten_feet_at_speed_25() {
    let mut grid = open(8, 8);
    grid.set_terrain(GridPos::new(1, 2), Terrain::Difficult);
    grid.set_terrain(GridPos::new(2, 1), Terrain::Greater);
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[guard()],
        &corner_party(),
        &[GridPos::new(3, 2)],
        grid,
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.combatant(CombatantId(0)).unwrap().speed_feet, 25);
    let legal = legal_actions(&encounter, CombatantId(0));
    assert!(legal.contains(&Action::Leap {
        destination: GridPos::new(2, 4)
    }));
    assert!(legal.contains(&Action::Leap {
        destination: GridPos::new(1, 2)
    }));
    assert!(!legal.contains(&Action::Leap {
        destination: GridPos::new(4, 2)
    }));
    assert!(!legal.contains(&Action::Leap {
        destination: GridPos::new(2, 5)
    }));
    assert!(!legal.contains(&Action::Leap {
        destination: GridPos::new(2, 1)
    }));

    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Leap {
            destination: GridPos::new(2, 4),
        },
        &mut dice,
    )
    .unwrap();
    assert_eq!(
        events,
        vec![Event::Leapt {
            id: CombatantId(0),
            from: GridPos::new(2, 2),
            to: GridPos::new(2, 4),
            feet: 10,
            actions_remaining: 2,
        }]
    );
    let actor = encounter.combatant(CombatantId(0)).unwrap();
    assert_eq!(actor.position, GridPos::new(2, 4));
    assert!(!actor.prone);

    let mut fast = fighter("Sera Voss");
    fast.speed_feet = 30;
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let encounter = start_encounter(
        &[
            fast,
            fighter("Bram Holt"),
            fighter("Quill Ames"),
            fighter("Doss Kade"),
        ],
        &[guard()],
        &[
            GridPos::new(0, 0),
            GridPos::new(0, 6),
            GridPos::new(0, 7),
            GridPos::new(7, 0),
        ],
        &[GridPos::new(7, 7)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    let legal = legal_actions(&encounter, CombatantId(0));
    assert!(legal.contains(&Action::Leap {
        destination: GridPos::new(3, 0)
    }));
    assert!(legal.contains(&Action::Leap {
        destination: GridPos::new(2, 2)
    }));
    assert!(!legal.contains(&Action::Leap {
        destination: GridPos::new(4, 0)
    }));
}

/// Speed below 15 cannot Leap. Speed 15 leaps 10 feet and not 15.
#[test]
fn leap_is_illegal_below_speed_15() {
    let mut slow = fighter("Sera Voss");
    slow.speed_feet = 10;
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &[
            slow,
            fighter("Bram Holt"),
            fighter("Quill Ames"),
            fighter("Doss Kade"),
        ],
        &[guard()],
        &corner_party(),
        &[GridPos::new(7, 7)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    assert!(legal_actions(&encounter, CombatantId(0))
        .iter()
        .all(|action| !matches!(action, Action::Leap { .. })));
    assert_eq!(
        declare_action(
            &mut encounter,
            CombatantId(0),
            Action::Leap {
                destination: GridPos::new(3, 2)
            },
            &mut dice,
        )
        .unwrap_err(),
        Illegal::NotLegal
    );

    let mut edged = fighter("Sera Voss");
    edged.speed_feet = 15;
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let encounter = start_encounter(
        &[
            edged,
            fighter("Bram Holt"),
            fighter("Quill Ames"),
            fighter("Doss Kade"),
        ],
        &[guard()],
        &[
            GridPos::new(0, 0),
            GridPos::new(0, 6),
            GridPos::new(0, 7),
            GridPos::new(7, 0),
        ],
        &[GridPos::new(7, 7)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    let legal = legal_actions(&encounter, CombatantId(0));
    assert!(legal.contains(&Action::Leap {
        destination: GridPos::new(2, 0)
    }));
    assert!(!legal.contains(&Action::Leap {
        destination: GridPos::new(3, 0)
    }));
}

/// Player Core, Step: the destination cannot be difficult or greater difficult terrain.
#[test]
fn step_refuses_difficult_and_greater_terrain() {
    let mut grid = open(8, 8);
    grid.set_terrain(GridPos::new(3, 2), Terrain::Difficult);
    grid.set_terrain(GridPos::new(2, 3), Terrain::Greater);
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[guard()],
        &corner_party(),
        &[GridPos::new(7, 7)],
        grid,
        &mut dice,
    )
    .unwrap();
    let legal = legal_actions(&encounter, CombatantId(0));
    assert!(!legal.contains(&Action::Step {
        destination: GridPos::new(3, 2)
    }));
    assert!(!legal.contains(&Action::Step {
        destination: GridPos::new(2, 3)
    }));
    assert!(legal.contains(&Action::Step {
        destination: GridPos::new(1, 2)
    }));
    assert_eq!(
        declare_action(
            &mut encounter,
            CombatantId(0),
            Action::Step {
                destination: GridPos::new(3, 2)
            },
            &mut dice,
        )
        .unwrap_err(),
        Illegal::NotLegal
    );
}

fn corridor(mark: GridPos, terrain: Terrain) -> Grid {
    let blocked: Vec<GridPos> = (0..12)
        .flat_map(|x| [GridPos::new(x, 0), GridPos::new(x, 2)])
        .collect();
    let mut grid = Grid::new(12, 3, blocked);
    grid.set_terrain(mark, terrain);
    grid
}

/// Entering difficult terrain costs 5 extra feet, so a Stride stops short of the square Speed would otherwise reach.
#[test]
fn difficult_terrain_stops_a_stride_short() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[guard()],
        &[
            GridPos::new(0, 1),
            GridPos::new(9, 1),
            GridPos::new(10, 1),
            GridPos::new(11, 1),
        ],
        &[GridPos::new(8, 1)],
        corridor(GridPos::new(3, 1), Terrain::Difficult),
        &mut dice,
    )
    .unwrap();
    let legal = legal_actions(&encounter, CombatantId(0));
    assert!(legal.contains(&Action::Stride {
        destination: GridPos::new(3, 1)
    }));
    assert!(legal.contains(&Action::Stride {
        destination: GridPos::new(4, 1)
    }));
    assert!(!legal.contains(&Action::Stride {
        destination: GridPos::new(5, 1)
    }));
    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Stride {
            destination: GridPos::new(4, 1),
        },
        &mut dice,
    )
    .unwrap();
    assert_eq!(
        events,
        vec![Event::Moved {
            id: CombatantId(0),
            from: GridPos::new(0, 1),
            to: GridPos::new(4, 1),
            feet: 25,
            actions_remaining: 2,
        }]
    );

    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let encounter = start_encounter(
        &party(),
        &[guard()],
        &[
            GridPos::new(0, 1),
            GridPos::new(9, 1),
            GridPos::new(10, 1),
            GridPos::new(11, 1),
        ],
        &[GridPos::new(8, 1)],
        corridor(GridPos::new(1, 1), Terrain::Greater),
        &mut dice,
    )
    .unwrap();
    let legal = legal_actions(&encounter, CombatantId(0));
    assert!(legal.contains(&Action::Stride {
        destination: GridPos::new(3, 1)
    }));
    assert!(!legal.contains(&Action::Stride {
        destination: GridPos::new(4, 1)
    }));
}

fn hidden_foe(dc: i32) -> CreatureTemplate {
    let mut foe = guard();
    foe.stealth_dc = Some(dc);
    foe
}

fn long_reach(name: &str, feet: u32) -> CreatureTemplate {
    let mut creature = fighter(name);
    creature.strike.reach_feet = feet;
    creature
}

fn strike_armor_class(events: &[Event]) -> i32 {
    events
        .iter()
        .find_map(|event| match event {
            Event::StrikeResolved { armor_class, .. } => Some(*armor_class),
            _ => None,
        })
        .expect("strike")
}

/// Player Core, Seek: success against a hidden creature makes them observed.
#[test]
fn seek_success_turns_hidden_into_observed() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1, 10]);
    let mut encounter = start_encounter(
        &party(),
        &[hidden_foe(15)],
        &[
            GridPos::new(0, 0),
            GridPos::new(0, 2),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(3, 0)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(0)));
    assert_eq!(
        encounter.set_detection(CombatantId(0), CombatantId(1), Some(Detection::Hidden)),
        Err(Illegal::NotLegal)
    );
    encounter
        .set_detection(CombatantId(0), CombatantId(4), Some(Detection::Hidden))
        .unwrap();
    assert!(legal_actions(&encounter, CombatantId(0)).contains(&Action::Seek));

    let events = declare_action(&mut encounter, CombatantId(0), Action::Seek, &mut dice).unwrap();
    assert_eq!(
        events,
        vec![Event::Sought {
            id: CombatantId(0),
            rolls: vec![rules::SeekRoll {
                target: CombatantId(4),
                natural: 10,
                total: 15,
                dc: 15,
                degree: Degree::Success,
            }],
            actions_remaining: 2,
        }]
    );
    assert_eq!(encounter.detection(CombatantId(0), CombatantId(4)), None);
    assert_eq!(encounter.actions_remaining(), 2);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, Seek: failure and critical failure leave the target undetected.
#[test]
fn seek_failure_leaves_undetected() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1, 4]);
    let mut encounter = start_encounter(
        &party(),
        &[hidden_foe(15)],
        &[
            GridPos::new(0, 0),
            GridPos::new(0, 2),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(4, 0)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    encounter
        .set_detection(CombatantId(0), CombatantId(4), Some(Detection::Undetected))
        .unwrap();
    declare_action(&mut encounter, CombatantId(0), Action::Seek, &mut dice).unwrap();
    // 4 + Perception 5 = 9 against DC 15 is a failure.
    assert_eq!(
        encounter.detection(CombatantId(0), CombatantId(4)),
        Some(Detection::Undetected)
    );
    assert_eq!(dice.remaining(), 0);

    // Total 6 would succeed against DC 6. A natural 1 worsens it to a failure.
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[hidden_foe(6)],
        &[
            GridPos::new(0, 0),
            GridPos::new(0, 2),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(4, 0)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    encounter
        .set_detection(CombatantId(0), CombatantId(4), Some(Detection::Undetected))
        .unwrap();
    declare_action(&mut encounter, CombatantId(0), Action::Seek, &mut dice).unwrap();
    assert_eq!(
        encounter.detection(CombatantId(0), CombatantId(4)),
        Some(Detection::Undetected)
    );

    // Total 25 against DC 30 is a failure. A natural 20 improves it to a success.
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1, 20]);
    let mut encounter = start_encounter(
        &party(),
        &[hidden_foe(30)],
        &[
            GridPos::new(0, 0),
            GridPos::new(0, 2),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(4, 0)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    encounter
        .set_detection(CombatantId(0), CombatantId(4), Some(Detection::Undetected))
        .unwrap();
    declare_action(&mut encounter, CombatantId(0), Action::Seek, &mut dice).unwrap();
    assert_eq!(
        encounter.detection(CombatantId(0), CombatantId(4)),
        Some(Detection::Hidden)
    );
}

/// Player Core, Point Out: allies who had the target undetected now have them hidden.
#[test]
fn point_out_turns_undetected_into_hidden_for_an_ally_only() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &party(),
        &[hidden_foe(15)],
        &[
            GridPos::new(0, 0),
            GridPos::new(1, 0),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(3, 0)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    let enemy = CombatantId(4);
    encounter
        .set_detection(CombatantId(1), enemy, Some(Detection::Undetected))
        .unwrap();
    encounter
        .set_detection(CombatantId(3), enemy, Some(Detection::Hidden))
        .unwrap();
    encounter
        .set_detection(CombatantId(0), enemy, Some(Detection::Concealed))
        .unwrap();
    assert!(
        !legal_actions(&encounter, CombatantId(0)).contains(&Action::PointOut { target: enemy })
    );
    encounter
        .set_detection(CombatantId(0), enemy, None)
        .unwrap();
    assert!(legal_actions(&encounter, CombatantId(0)).contains(&Action::PointOut { target: enemy }));

    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::PointOut { target: enemy },
        &mut dice,
    )
    .unwrap();
    assert_eq!(
        events,
        vec![Event::PointedOut {
            id: CombatantId(0),
            target: enemy,
            actions_remaining: 2,
        }]
    );
    assert_eq!(encounter.detection(CombatantId(0), enemy), None);
    assert_eq!(
        encounter.detection(CombatantId(1), enemy),
        Some(Detection::Hidden)
    );
    assert_eq!(encounter.detection(CombatantId(2), enemy), None);
    assert_eq!(
        encounter.detection(CombatantId(3), enemy),
        Some(Detection::Hidden)
    );
    assert_eq!(dice.remaining(), 0);
}

/// Player Core: a hidden Strike rolls a DC 11 flat check first. Failure spends no attack.
#[test]
fn failed_hidden_flat_check_spends_no_attack() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1, 10]);
    let mut encounter = start_encounter(
        &party(),
        &[hidden_foe(15)],
        &[
            GridPos::new(1, 1),
            GridPos::new(0, 0),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(2, 1)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    let enemy = CombatantId(4);
    encounter
        .set_detection(CombatantId(0), enemy, Some(Detection::Undetected))
        .unwrap();
    assert!(!legal_actions(&encounter, CombatantId(0)).contains(&Action::Strike { target: enemy }));
    encounter
        .set_detection(CombatantId(0), enemy, Some(Detection::Hidden))
        .unwrap();
    assert!(legal_actions(&encounter, CombatantId(0)).contains(&Action::Strike { target: enemy }));

    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike { target: enemy },
        &mut dice,
    )
    .unwrap();
    assert_eq!(
        events,
        vec![Event::FlatCheckFailed {
            attacker: CombatantId(0),
            target: enemy,
            natural: 10,
            dc: 11,
            actions_remaining: 2,
        }]
    );
    assert_eq!(encounter.strikes_this_turn(), 0);
    assert_eq!(encounter.actions_remaining(), 2);
    assert_eq!(encounter.combatant(enemy).unwrap().hit_points, 18);
    assert_eq!(dice.remaining(), 0);

    let mut dice = ScriptedDice::new([11, 10, 4]);
    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike { target: enemy },
        &mut dice,
    )
    .unwrap();
    assert!(matches!(
        events[0],
        Event::StrikeResolved {
            natural: 10,
            map: 0,
            total: 17,
            armor_class: 14,
            degree: Degree::Success,
            damage: 8,
            ..
        }
    ));
    assert_eq!(encounter.strikes_this_turn(), 1);
    assert_eq!(dice.remaining(), 0);
}

/// A creature between the attacker and the target is lesser cover, +1 circumstance bonus to AC.
#[test]
fn lesser_cover_from_a_creature_adds_one_ac() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1, 10, 4]);
    let mut encounter = start_encounter(
        &[
            long_reach("Sera Voss", 15),
            fighter("Bram Holt"),
            fighter("Quill Ames"),
            fighter("Doss Kade"),
        ],
        &[guard()],
        &[
            GridPos::new(0, 0),
            GridPos::new(1, 0),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(2, 0)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    assert_eq!(strike_armor_class(&events), 15);
}

/// An unwalkable square on the segment is standard cover, +2. A creature on that segment does not add more.
#[test]
fn standard_cover_from_a_wall_adds_two_ac() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1, 10, 4]);
    let mut encounter = start_encounter(
        &[
            long_reach("Sera Voss", 15),
            fighter("Bram Holt"),
            fighter("Quill Ames"),
            fighter("Doss Kade"),
        ],
        &[guard()],
        &[
            GridPos::new(0, 0),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
            GridPos::new(0, 5),
        ],
        &[GridPos::new(2, 0)],
        Grid::new(8, 8, [GridPos::new(1, 0)]),
        &mut dice,
    )
    .unwrap();
    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    assert_eq!(strike_armor_class(&events), 16);

    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1, 10, 4]);
    let mut encounter = start_encounter(
        &[
            long_reach("Sera Voss", 20),
            fighter("Bram Holt"),
            fighter("Quill Ames"),
            fighter("Doss Kade"),
        ],
        &[guard()],
        &[
            GridPos::new(0, 0),
            GridPos::new(2, 0),
            GridPos::new(0, 4),
            GridPos::new(0, 5),
        ],
        &[GridPos::new(3, 0)],
        Grid::new(8, 8, [GridPos::new(1, 0)]),
        &mut dice,
    )
    .unwrap();
    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    assert_eq!(strike_armor_class(&events), 16);
}

/// Take Cover improves standard cover to greater cover, +4.
#[test]
fn take_cover_upgrades_standard_cover() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 20, 16, 4]);
    let mut encounter = start_encounter(
        &[
            long_reach("Sera Voss", 15),
            fighter("Bram Holt"),
            fighter("Quill Ames"),
            fighter("Doss Kade"),
        ],
        &[guard()],
        &[
            GridPos::new(0, 0),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
            GridPos::new(0, 5),
        ],
        &[GridPos::new(2, 0)],
        Grid::new(8, 8, [GridPos::new(1, 0)]),
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(4)));
    assert!(legal_actions(&encounter, CombatantId(4)).contains(&Action::TakeCover));
    let events =
        declare_action(&mut encounter, CombatantId(4), Action::TakeCover, &mut dice).unwrap();
    assert_eq!(
        events,
        vec![Event::TookCover {
            id: CombatantId(4),
            actions_remaining: 2,
        }]
    );
    assert!(encounter.combatant(CombatantId(4)).unwrap().taking_cover);
    end_turn(&mut encounter, &mut dice).unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(0)));
    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Strike {
            target: CombatantId(4),
        },
        &mut dice,
    )
    .unwrap();
    assert_eq!(strike_armor_class(&events), 18);
    assert!(encounter.combatant(CombatantId(4)).unwrap().taking_cover);
}

/// A raised shield and cover are circumstance bonuses. The strike uses the larger.
#[test]
fn shield_bonus_does_not_stack_with_cover() {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1, 2]);
    let mut foe = guard();
    foe.strike.reach_feet = 15;
    let mut encounter = start_encounter(
        &[
            shielded("Sera Voss", true),
            fighter("Bram Holt"),
            fighter("Quill Ames"),
            fighter("Doss Kade"),
        ],
        &[foe],
        &[
            GridPos::new(2, 0),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
            GridPos::new(0, 5),
        ],
        &[GridPos::new(0, 0)],
        Grid::new(8, 8, [GridPos::new(1, 0)]),
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(0)));
    declare_action(
        &mut encounter,
        CombatantId(0),
        Action::RaiseShield,
        &mut dice,
    )
    .unwrap();
    end_until(&mut encounter, CombatantId(4), &mut dice);
    let events = declare_action(
        &mut encounter,
        CombatantId(4),
        Action::Strike {
            target: CombatantId(0),
        },
        &mut dice,
    )
    .unwrap();
    assert_eq!(strike_armor_class(&events), 20);
    assert_eq!(dice.remaining(), 0);
}

fn guard_in_reach(party: [CreatureTemplate; 4]) -> Encounter {
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let encounter = start_encounter(
        &party,
        &[guard()],
        &[
            GridPos::new(0, 0),
            GridPos::new(0, 1),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(1, 0)],
        open(10, 8),
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(0)));
    assert_eq!(dice.remaining(), 0);
    encounter
}

/// Player Core, Release and Interact. Release does not spend an action. Interact spends one.
/// The shield is picked up before a weapon. Strike needs the weapon in hand.
#[test]
fn release_is_free_and_interact_returns_the_shield() {
    let mut encounter = guard_in_reach([
        shielded("Sera Voss", true),
        fighter("Bram Holt"),
        fighter("Quill Ames"),
        fighter("Doss Kade"),
    ]);
    let pos = GridPos::new(0, 0);
    let mut dice = ScriptedDice::new([]);
    assert!(
        legal_actions(&encounter, CombatantId(0)).contains(&Action::Release { item: Gear::Shield })
    );
    assert!(
        legal_actions(&encounter, CombatantId(0)).contains(&Action::Strike {
            target: CombatantId(4)
        })
    );
    assert!(!legal_actions(&encounter, CombatantId(0)).contains(&Action::Interact));

    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Release { item: Gear::Shield },
        &mut dice,
    )
    .unwrap();
    assert_eq!(
        events,
        vec![Event::Released {
            id: CombatantId(0),
            item: Gear::Shield,
            actions_remaining: 3,
        }]
    );
    assert_eq!(encounter.actions_remaining(), 3);
    assert!(encounter
        .combatant(CombatantId(0))
        .unwrap()
        .shield
        .is_none());
    assert!(encounter.dropped_shield(pos).is_some());
    assert!(!legal_actions(&encounter, CombatantId(0)).contains(&Action::RaiseShield));

    declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Release { item: Gear::Weapon },
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.actions_remaining(), 3);
    assert!(!encounter.combatant(CombatantId(0)).unwrap().holding_weapon);
    assert!(encounter.dropped_weapon(pos));
    assert!(legal_actions(&encounter, CombatantId(0))
        .iter()
        .all(|action| !matches!(action, Action::Strike { .. })));
    assert_eq!(
        declare_action(
            &mut encounter,
            CombatantId(0),
            Action::Strike {
                target: CombatantId(4),
            },
            &mut dice,
        )
        .unwrap_err(),
        Illegal::NotLegal
    );
    assert!(!legal_actions(&encounter, CombatantId(0))
        .contains(&Action::Release { item: Gear::Shield }));
    assert!(!legal_actions(&encounter, CombatantId(0))
        .contains(&Action::Release { item: Gear::Weapon }));

    let events =
        declare_action(&mut encounter, CombatantId(0), Action::Interact, &mut dice).unwrap();
    assert_eq!(
        events,
        vec![Event::PickedUp {
            id: CombatantId(0),
            item: Gear::Shield,
            actions_remaining: 2,
        }]
    );
    assert_eq!(encounter.actions_remaining(), 2);
    assert!(encounter
        .combatant(CombatantId(0))
        .unwrap()
        .shield
        .is_some());
    assert!(encounter.dropped_shield(pos).is_none());
    assert!(encounter.dropped_weapon(pos));
    assert!(!encounter.combatant(CombatantId(0)).unwrap().holding_weapon);
    assert!(legal_actions(&encounter, CombatantId(0)).contains(&Action::RaiseShield));

    let events =
        declare_action(&mut encounter, CombatantId(0), Action::Interact, &mut dice).unwrap();
    assert_eq!(
        events,
        vec![Event::PickedUp {
            id: CombatantId(0),
            item: Gear::Weapon,
            actions_remaining: 1,
        }]
    );
    assert!(encounter.combatant(CombatantId(0)).unwrap().holding_weapon);
    assert!(!encounter.dropped_weapon(pos));
    assert!(encounter.dropped_shield(pos).is_none());
    assert!(
        legal_actions(&encounter, CombatantId(0)).contains(&Action::Strike {
            target: CombatantId(4)
        })
    );
}

/// A square that already holds a shield rejects another, and Interact will not take the weapon under it.
#[test]
fn a_square_rejects_a_second_shield_and_keeps_shield_first() {
    let mut encounter = guard_in_reach([
        shielded("Sera Voss", false),
        shielded("Bram Holt", false),
        fighter("Quill Ames"),
        fighter("Doss Kade"),
    ]);
    let mut dice = ScriptedDice::new([]);
    declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Release { item: Gear::Shield },
        &mut dice,
    )
    .unwrap();
    declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Stride {
            destination: GridPos::new(0, 2),
        },
        &mut dice,
    )
    .unwrap();
    end_turn(&mut encounter, &mut dice).unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(1)));
    declare_action(
        &mut encounter,
        CombatantId(1),
        Action::Stride {
            destination: GridPos::new(0, 0),
        },
        &mut dice,
    )
    .unwrap();
    let square = GridPos::new(0, 0);
    assert!(encounter.dropped_shield(square).is_some());
    assert!(encounter
        .combatant(CombatantId(1))
        .unwrap()
        .shield
        .is_some());
    assert!(!legal_actions(&encounter, CombatantId(1))
        .contains(&Action::Release { item: Gear::Shield }));
    assert_eq!(
        declare_action(
            &mut encounter,
            CombatantId(1),
            Action::Release { item: Gear::Shield },
            &mut dice,
        )
        .unwrap_err(),
        Illegal::NotLegal
    );

    declare_action(
        &mut encounter,
        CombatantId(1),
        Action::Release { item: Gear::Weapon },
        &mut dice,
    )
    .unwrap();
    assert!(encounter.dropped_weapon(square));
    assert!(!encounter.combatant(CombatantId(1)).unwrap().holding_weapon);
    assert!(!legal_actions(&encounter, CombatantId(1)).contains(&Action::Interact));
    assert!(encounter.dropped_shield(square).is_some());
    assert!(encounter.dropped_weapon(square));
}

fn shield_dropped_beside_reactor(mut reactor: CreatureTemplate) -> Encounter {
    reactor.features.push(ClassFeature::ReactiveStrike);
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &[
            shielded("Sera Voss", false),
            fighter("Bram Holt"),
            fighter("Quill Ames"),
            fighter("Doss Kade"),
        ],
        &[reactor],
        &[
            GridPos::new(1, 1),
            GridPos::new(0, 0),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(2, 1)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(0)));
    encounter.spend_reaction(CombatantId(4)).unwrap();
    let dropped = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Release { item: Gear::Shield },
        &mut dice,
    )
    .unwrap();
    assert!(dropped
        .iter()
        .all(|event| !matches!(event, Event::ReactiveStrike { .. })));
    assert_eq!(encounter.actions_remaining(), 3);
    end_turn(&mut encounter, &mut dice).unwrap();
    end_until(&mut encounter, CombatantId(0), &mut dice);
    assert_eq!(encounter.actions_remaining(), 3);
    assert_eq!(encounter.reaction_ready(CombatantId(4)), Some(true));
    assert!(encounter.dropped_shield(GridPos::new(1, 1)).is_some());
    assert!(encounter
        .combatant(CombatantId(0))
        .unwrap()
        .shield
        .is_none());
    assert_eq!(
        reactors_for_left_square(&encounter, CombatantId(0), GridPos::new(1, 1)),
        vec![CombatantId(4)]
    );
    encounter
}

/// Player Core, Reactive Strike: a manipulate action in reach draws one strike at penalty 0.
/// A normal hit does not disrupt Interact.
#[test]
fn reactive_strike_on_interact_lets_a_normal_hit_finish() {
    let mut encounter = shield_dropped_beside_reactor(guard());
    // 12 + 6 = 18 vs AC 18 is a success. Damage 1 + 2 = 3. With a −5 penalty the same d20 would miss.
    let mut dice = ScriptedDice::new([12, 1]);
    let events =
        declare_action(&mut encounter, CombatantId(0), Action::Interact, &mut dice).unwrap();
    assert_eq!(encounter.strikes_this_turn(), 0);
    assert_eq!(encounter.reaction_ready(CombatantId(4)), Some(false));
    assert_eq!(encounter.actions_remaining(), 2);
    assert!(encounter
        .combatant(CombatantId(0))
        .unwrap()
        .shield
        .is_some());
    assert!(encounter.dropped_shield(GridPos::new(1, 1)).is_none());
    assert_eq!(
        events,
        vec![
            Event::ReactiveStrike {
                attacker: CombatantId(4),
                target: CombatantId(0),
                natural: 12,
                attack_bonus: 6,
                map: 0,
                total: 18,
                armor_class: 18,
                degree: Degree::Success,
                damage: 3,
                target_hit_points: 17,
            },
            Event::PickedUp {
                id: CombatantId(0),
                item: Gear::Shield,
                actions_remaining: 2,
            },
        ]
    );
    assert_eq!(dice.remaining(), 0);
}

/// A critical success disrupts Interact. The shield stays where it was, and the action is spent.
#[test]
fn critical_reactive_strike_leaves_the_shield_unmoved() {
    let mut encounter = shield_dropped_beside_reactor(guard());
    // Natural 20: 26 vs AC 18 is a success, improved to a critical success. Damage 3 doubles to 6.
    let mut dice = ScriptedDice::new([20, 1]);
    let events =
        declare_action(&mut encounter, CombatantId(0), Action::Interact, &mut dice).unwrap();
    assert_eq!(encounter.actions_remaining(), 2);
    assert_eq!(encounter.strikes_this_turn(), 0);
    assert!(encounter
        .combatant(CombatantId(0))
        .unwrap()
        .shield
        .is_none());
    assert!(encounter.dropped_shield(GridPos::new(1, 1)).is_some());
    assert!(events
        .iter()
        .all(|event| !matches!(event, Event::PickedUp { .. })));
    match &events[0] {
        Event::ReactiveStrike {
            map,
            degree,
            damage,
            target_hit_points,
            ..
        } => {
            assert_eq!(*map, 0);
            assert_eq!(*degree, Degree::CriticalSuccess);
            assert_eq!(*damage, 6);
            assert_eq!(*target_hit_points, 14);
        }
        other => panic!("unexpected {other:?}"),
    }
    assert_eq!(events.len(), 1);
    assert_eq!(dice.remaining(), 0);
}

/// Release is manipulate. A critical Reactive Strike does not drop the item and does not spend an action.
#[test]
fn critical_reactive_strike_does_not_release_the_shield() {
    let mut reactor = guard();
    reactor.features.push(ClassFeature::ReactiveStrike);
    let mut dice = ScriptedDice::new([1, 1, 1, 1, 1]);
    let mut encounter = start_encounter(
        &[
            shielded("Sera Voss", false),
            fighter("Bram Holt"),
            fighter("Quill Ames"),
            fighter("Doss Kade"),
        ],
        &[reactor],
        &[
            GridPos::new(1, 1),
            GridPos::new(0, 0),
            GridPos::new(0, 3),
            GridPos::new(0, 4),
        ],
        &[GridPos::new(2, 1)],
        open(8, 8),
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.active(), Some(CombatantId(0)));
    let mut dice = ScriptedDice::new([20, 1]);
    let events = declare_action(
        &mut encounter,
        CombatantId(0),
        Action::Release { item: Gear::Shield },
        &mut dice,
    )
    .unwrap();
    assert_eq!(encounter.actions_remaining(), 3);
    assert!(encounter
        .combatant(CombatantId(0))
        .unwrap()
        .shield
        .is_some());
    assert!(encounter.dropped_shield(GridPos::new(1, 1)).is_none());
    assert!(events
        .iter()
        .all(|event| !matches!(event, Event::Released { .. })));
    match &events[0] {
        Event::ReactiveStrike { degree, map, .. } => {
            assert_eq!(*degree, Degree::CriticalSuccess);
            assert_eq!(*map, 0);
        }
        other => panic!("unexpected {other:?}"),
    }
    assert_eq!(encounter.strikes_this_turn(), 0);
    assert_eq!(dice.remaining(), 0);
}

/// Dropping the actor to 0 HP cancels Interact even when the Reactive Strike is not a critical hit.
#[test]
fn dropping_to_zero_cancels_interact() {
    let mut reactor = guard();
    reactor.strike.damage.bonus = 19;
    let mut encounter = shield_dropped_beside_reactor(reactor);
    // 12 + 6 = 18 vs AC 18 is a success. Damage 1 + 19 = 20, short of twice maximum HP.
    let mut dice = ScriptedDice::new([12, 1]);
    let events =
        declare_action(&mut encounter, CombatantId(0), Action::Interact, &mut dice).unwrap();
    let sera = encounter.combatant(CombatantId(0)).unwrap();
    assert!(sera.is_unconscious());
    assert!(sera.shield.is_none());
    assert!(encounter.dropped_shield(GridPos::new(1, 1)).is_some());
    assert_eq!(encounter.actions_remaining(), 2);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::ReactiveStrike {
            degree: Degree::Success,
            target_hit_points: 0,
            ..
        }
    )));
    assert!(events
        .iter()
        .all(|event| !matches!(event, Event::PickedUp { .. })));
    assert_eq!(dice.remaining(), 0);
}
