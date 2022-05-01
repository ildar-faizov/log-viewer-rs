use cursive::event::{EventResult, Key};
use fluent_integer::Integer;
use crate::actions::action::Action;
use crate::{Event, RootModel};
use crate::model::CursorShift;

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
        model.move_cursor(CursorShift::word_backward(), false);
        EventResult::Consumed(None)
    }
}