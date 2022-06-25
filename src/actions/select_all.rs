use cursive::event::{Event, EventResult};
use crate::actions::action::Action;
use crate::model::model::RootModel;

pub struct SelectAllAction {}

impl SelectAllAction {
    pub fn new() -> Self {
        SelectAllAction {}
    }
}

impl Action for SelectAllAction {
    fn description(&self) -> &str {
        "Select all"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::CtrlChar('a')]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.select_all();
        EventResult::Consumed(None)
    }
}