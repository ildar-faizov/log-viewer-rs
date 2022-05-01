use cursive::event::{EventResult, Key};
use fluent_integer::Integer;
use crate::actions::action::Action;
use crate::{Event, RootModel};
use crate::model::CursorShift;

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
        model.move_cursor(CursorShift::word_forward(), false);
        EventResult::Consumed(None)
    }
}