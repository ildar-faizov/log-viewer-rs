use cursive::event::{EventResult, Key};
use crate::actions::action::Action;
use crate::{Event, RootModel};
use crate::model::CursorShift;

pub struct ShiftLeftAction {}

impl ShiftLeftAction {
    pub fn new() -> Self {
        ShiftLeftAction {}
    }
}

impl Action for ShiftLeftAction {
    fn description(&self) -> &str {
        "Move cursor left with shift"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Shift(Key::Left)]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.move_cursor(CursorShift::left(), true);
        EventResult::Consumed(None)
    }
}