use cursive::event::EventResult;
use crate::actions::action::Action;
use crate::{Event, RootModel};

pub struct QuitAction {}

impl QuitAction {
    pub fn new() -> Self {
        QuitAction {}
    }
}

impl Action for QuitAction {
    fn description(&self) -> &str {
        "Quit"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Char('q')]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.quit();
        EventResult::Consumed(None)
    }
}