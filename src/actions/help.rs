use cursive::event::{Event, EventResult};
use crate::actions::action::Action;
use crate::model::model::RootModel;

#[derive(Default)]
pub struct HelpAction {}

impl Action for HelpAction {
    fn description(&self) -> &str {
        "Help"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Char('?')]
    }

    fn perform_action(&self, model: &mut RootModel, event: &Event) -> EventResult {
        model.get_help_model().set_open(true);
        EventResult::Ignored
    }
}