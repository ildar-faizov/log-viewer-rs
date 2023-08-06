use cursive::event::{Event, EventResult};
use cursive::event::Event::Char;
use crate::actions::action::Action;
use crate::data_source::Direction;
use crate::model::model::RootModel;

#[derive(Default)]
pub struct SearchNextAction {}

impl Action for SearchNextAction {
    fn description(&self) -> &str {
        "Search next occurrence"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Char('n')]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.get_current_search().as_mut().map(|s| s.search(Direction::Forward));
        EventResult::Ignored
    }
}