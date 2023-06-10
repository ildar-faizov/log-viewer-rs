use cursive::event::{Event, EventResult, Key};
use crate::actions::action::Action;
use crate::model::cursor_shift::CursorShift;
use crate::model::model::RootModel;

pub struct PgUpAction {}

impl PgUpAction {
    pub fn new() -> Self {
        PgUpAction {}
    }
}

impl Action for PgUpAction {
    fn description(&self) -> &str {
        "Scroll one page up"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Key(Key::PageUp)]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        let h = model.get_viewport_height();
        model.move_cursor(CursorShift::Y(-h), false);
        EventResult::Consumed(None)
    }
}