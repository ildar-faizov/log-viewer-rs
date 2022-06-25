use cursive::event::{Event, EventResult};
use cursive::event::Key::End;
use crate::actions::action::Action;
use crate::model::model::RootModel;

pub struct FileEndAction {}

impl FileEndAction {
    pub fn new() -> Self {
        FileEndAction {}
    }
}

impl Action for FileEndAction {
    fn description(&self) -> &str {
        "Go to file end"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Ctrl(End)]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        if model.move_cursor_to_end() {
            EventResult::Consumed(None)
        } else {
            EventResult::Ignored
        }
    }
}