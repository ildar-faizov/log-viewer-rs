use cursive::event::{EventResult, Key};
use crate::actions::action::Action;
use crate::{Event, RootModel};
use crate::model::CursorShift;

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