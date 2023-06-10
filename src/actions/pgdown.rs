use cursive::event::{Event, EventResult, Key};
use crate::actions::action::Action;
use crate::model::cursor_shift::CursorShift;
use crate::model::model::RootModel;

pub struct PgDownAction {}

impl PgDownAction {
    pub fn new() -> Self {
        PgDownAction {}
    }
}

impl Action for PgDownAction {
    fn description(&self) -> &str {
        "Scroll one page down"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Key(Key::PageDown)]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        let h = model.get_viewport_height();
        model.move_cursor(CursorShift::Y(h), false);
        EventResult::Consumed(None)
    }
}