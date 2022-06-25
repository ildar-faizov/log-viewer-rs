use cursive::event::{Event, EventResult, Key};
use crate::actions::action::Action;
use crate::model::model::RootModel;
use crate::model::cursor_shift::CursorShift;

pub struct WordForwardAction {}

impl WordForwardAction {
    pub fn new() -> Self {
        WordForwardAction {}
    }
}

impl Action for WordForwardAction {

    fn description(&self) -> &str {
        "Move cursor to the end of the word"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Ctrl(Key::Right)]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.move_cursor(CursorShift::token_forward(), false);
        EventResult::Consumed(None)
    }
}