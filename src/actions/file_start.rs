use cursive::event::{Event, EventResult};
use cursive::event::Key::Home;
use num_traits::Zero;
use fluent_integer::Integer;
use crate::actions::action::Action;
use crate::model::model::RootModel;

pub struct FileStartAction {}

impl FileStartAction {
    pub fn new() -> Self {
        FileStartAction {}
    }
}

impl Action for FileStartAction {
    fn description(&self) -> &str {
        "Go to file start"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Ctrl(Home)]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.move_cursor_to_offset(Integer::zero(), false);
        EventResult::Consumed(None)
    }
}