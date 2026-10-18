//! Seeded encounter sequences for Stride, Step, Strike, Reactive Strike, and the dying track.

use rules::{
    advance_if_turn_done, declare_action, end_turn, legal_actions, prepare_combatant,
    reactors_for_left_square, start_encounter, Action, ClassFeature, CombatantId, CreatureTemplate,
    DamageFormula, Degree, Event, Grid, GridPos, Illegal, Outcome, ScriptedDice, Side,
    StrikeTemplate,
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
