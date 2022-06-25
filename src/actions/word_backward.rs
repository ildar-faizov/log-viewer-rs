use cursive::event::{Event, EventResult, Key};
use crate::actions::action::Action;
use crate::model::model::RootModel;
use crate::model::cursor_shift::CursorShift;

pub struct WordBackwardAction {}

impl WordBackwardAction {
    pub fn new() -> Self {
        WordBackwardAction {}
    }
}

impl Action for WordBackwardAction {

    fn description(&self) -> &str {
        "Move cursor to the beginning of the word"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Ctrl(Key::Left)]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.move_cursor(CursorShift::token_backward(), false);
        EventResult::Consumed(None)
    }
}