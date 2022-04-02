use cursive::event::{EventResult, Key};
use crate::actions::action::Action;
use crate::{Event, RootModel};
use crate::model::CursorShift;

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