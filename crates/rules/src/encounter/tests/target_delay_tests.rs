use super::{begin, id, pc};
use crate::encounter::TurnSlot;

fn open(encounter: &mut super::super::Encounter, value: u32) {
    assert_eq!(encounter.start_turn().expect("start"), id(value));
}

fn close(encounter: &mut super::super::Encounter) {
    let mut unused = crate::dice::ScriptedDice::new([]);
    encounter.end_turn(&mut unused).expect("end");
}

fn slot(encounter: &super::super::Encounter, value: u32) -> TurnSlot {
    encounter.turn_slot(id(value)).expect("slot")
}

/// Player Core, Delay. Player Core, "Getting Knocked Out".
///
/// `TargetDelay` follows the slot. `TurnState` is only the round's wrap, so a
/// pending wrap and a delayed target can both be true.
#[test]
fn target_delay_follows_the_slot_apart_from_the_wrap() {
    let (mut encounter, _) = begin(
        vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18), pc(3, 0, 20, 18)],
        &[15, 10, 5],
    );
    assert_eq!(
        encounter.delayed_target(id(1)).expect("ordinary"),
        super::super::TargetDelay::NotDelayed
    );
    open(&mut encounter, 1);
    encounter
        .delay(id(1), &mut crate::dice::ScriptedDice::new([]))
        .expect("delay 1");
    open(&mut encounter, 2);
    encounter
        .delay(id(2), &mut crate::dice::ScriptedDice::new([]))
        .expect("delay 2");
    open(&mut encounter, 3);
    close(&mut encounter);
    assert_eq!(encounter.round(), 1);
    assert_eq!(encounter.turn_state, super::super::TurnState::PendingWrap);
    assert_eq!(
        encounter.delayed_target(id(1)).expect("delayed"),
        super::super::TargetDelay::Delayed
    );
    assert_eq!(
        encounter.delayed_target(id(2)).expect("delayed"),
        super::super::TargetDelay::Delayed
    );
    assert_eq!(
        encounter.delayed_target(id(3)).expect("acted"),
        super::super::TargetDelay::NotDelayed
    );
    encounter
        .return_from_delay(&[id(2), id(1)])
        .expect("return");
    assert_eq!(slot(&encounter, 1), TurnSlot::Returned);
    assert_eq!(slot(&encounter, 2), TurnSlot::SkipStartHook);
    assert_eq!(
        encounter.delayed_target(id(1)).expect("returned"),
        super::super::TargetDelay::NotDelayed
    );
    assert_eq!(
        encounter.delayed_target(id(2)).expect("later"),
        super::super::TargetDelay::NotDelayed
    );
    assert_eq!(encounter.turn_state, super::super::TurnState::Continuing);
}
