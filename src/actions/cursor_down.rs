use cursive::event::{Event, EventResult, Key};
use crate::actions::action::Action;
use crate::model::cursor_shift::CursorShift;
use crate::model::model::RootModel;

pub struct CursorDownAction {}

impl CursorDownAction {
    pub fn new() -> Self {
        CursorDownAction {}
    }
}

impl Action for CursorDownAction {
    fn description(&self) -> &str {
        "Move cursor down"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Key(Key::Down)]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.move_cursor(CursorShift::down(), false);
        EventResult::Consumed(None)
    }
}