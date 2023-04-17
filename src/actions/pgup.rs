use cursive::event::{Event, EventResult, Key};
use crate::actions::action::Action;
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
        if model.scroll(-h) {
            let p = model.data()
                .and_then(|data| data.lines.first())
                .map(|line| line.start);
            if let Some(p) = p {
                model.move_cursor_to_offset(p, false);
            }
        } else {
            let p = model.data()
                .and_then(|data| data.lines.first())
                .map(|line| line.start);
            if let Some(p) = p {
                model.move_cursor_to_offset(p, false);
            }
        }
        EventResult::Consumed(None)
    }
}