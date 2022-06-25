use cursive::event::{Event, EventResult, Key};
use crate::actions::action::Action;
use crate::model::cursor_shift::CursorShift;
use crate::model::model::RootModel;

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