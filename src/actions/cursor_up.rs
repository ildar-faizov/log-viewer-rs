use cursive::event::{Event, EventResult, Key};
use crate::actions::action::Action;
use crate::model::cursor_shift::CursorShift;
use crate::model::model::RootModel;

pub struct CursorUpAction {}

impl CursorUpAction {
    pub fn new() -> Self {
        CursorUpAction {}
    }
}

impl Action for CursorUpAction {
    fn description(&self) -> &str {
        "Move cursor up"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Key(Key::Up)]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.move_cursor(CursorShift::up(), false);
        EventResult::Consumed(None)
    }
}