use cursive::event::{EventResult, Key};
use crate::actions::action::Action;
use crate::{Event, RootModel};
use crate::model::CursorShift;

pub struct ShiftRightAction {}

impl ShiftRightAction {
    pub fn new() -> Self {
        ShiftRightAction {}
    }
}

impl Action for ShiftRightAction {
    fn description(&self) -> &str {
        "Move cursor right with shift"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Shift(Key::Right)]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.move_cursor(CursorShift::right(), true);
        EventResult::Consumed(None)
    }
}