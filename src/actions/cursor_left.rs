use cursive::event::{EventResult, Key};
use crate::actions::action::Action;
use crate::{Event, RootModel};
use crate::model::CursorShift;

pub struct CursorLeftAction {}

impl CursorLeftAction {
    pub fn new() -> Self {
        CursorLeftAction {}
    }
}

impl Action for CursorLeftAction {
    fn description(&self) -> &str {
        "Move cursor left"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Key(Key::Left)]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.move_cursor(CursorShift::left(), false);
        EventResult::Consumed(None)
    }
}