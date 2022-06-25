use cursive::event::{Event, EventResult, Key};
use crate::actions::action::Action;
use crate::model::cursor_shift::CursorShift;
use crate::model::model::RootModel;

pub struct ShiftDownAction {}

impl ShiftDownAction {
    pub fn new() -> Self {
        ShiftDownAction {}
    }
}

impl Action for ShiftDownAction {
    fn description(&self) -> &str {
        "Move cursor down with shift"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Shift(Key::Down)]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.move_cursor(CursorShift::down(), true);
        EventResult::Consumed(None)
    }
}