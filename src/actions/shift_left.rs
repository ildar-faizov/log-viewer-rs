use cursive::event::{Event, EventResult, Key};
use crate::actions::action::Action;
use crate::model::cursor_shift::CursorShift;
use crate::model::model::RootModel;

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