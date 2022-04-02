use cursive::event::{EventResult, Key};
use crate::actions::action::Action;
use crate::{Event, RootModel};
use crate::model::CursorShift;

pub struct ShiftUpAction {}

impl ShiftUpAction {
    pub fn new() -> Self {
        ShiftUpAction {}
    }
}

impl Action for ShiftUpAction {
    fn description(&self) -> &str {
        "Move cursor up with shift"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Shift(Key::Up)]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.move_cursor(CursorShift::up(), true);
        EventResult::Consumed(None)
    }
}