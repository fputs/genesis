//! Seeded encounter sequences for the Stride, Step, and Strike subset.

use rules::{
    advance_if_turn_done, declare_action, end_turn, legal_actions, prepare_combatant,
    start_encounter, Action, CombatantId, CreatureTemplate, DamageFormula, Degree, Event, Grid,
    GridPos, Illegal, Outcome, ScriptedDice, Side, StrikeTemplate,
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
    }
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
    assert_eq!(encounter.outcome(), Some(Outcome::Victory));
    assert!(encounter.active().is_none());
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
    assert!(events.iter().any(|event| matches!(
        event,
        Event::CombatantDowned { id } if *id == CombatantId(0)
    )));
    assert_eq!(encounter.combatant(CombatantId(0)).unwrap().hit_points, 0);
    assert_eq!(encounter.party()[0], CombatantId(0));
    assert!(encounter.outcome().is_none());

    let events = end_turn(&mut encounter).unwrap();
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
    let started = advance_if_turn_done(&mut encounter);
    assert!(started.iter().any(|event| matches!(
        event,
        Event::TurnStarted { id, .. } if *id == CombatantId(3)
    )));
    end_turn(&mut encounter).unwrap();
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

    // Separate fight: one Strike drops the enemy.
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
    let mut dice = ScriptedDice::new([15, 8]);
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
    assert_eq!(encounter.actions_remaining(), 2);
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
    let events = end_turn(&mut encounter).unwrap();
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
