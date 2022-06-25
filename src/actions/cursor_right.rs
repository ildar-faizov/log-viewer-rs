use cursive::event::{Event, EventResult, Key};
use crate::actions::action::Action;
use crate::model::cursor_shift::CursorShift;
use crate::model::model::RootModel;

pub struct CursorRightAction {}

impl CursorRightAction {
    pub fn new() -> Self {
        CursorRightAction {}
    }
}

impl Action for CursorRightAction {
    fn description(&self) -> &str {
        "Move cursor right"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Key(Key::Right)]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.move_cursor(CursorShift::right(), false);
        EventResult::Consumed(None)
    }
}