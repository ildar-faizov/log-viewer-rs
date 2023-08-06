use cursive::event::{Event, EventResult};
use cursive::event::Event::Char;
use crate::actions::action::Action;
use crate::data_source::Direction;
use crate::model::model::RootModel;

#[derive(Default)]
pub struct SearchPrevAction {}

impl Action for SearchPrevAction {
    fn description(&self) -> &str {
        "Search previous occurrence"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Char('N')]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.get_current_search().as_mut().map(|s| s.search(Direction::Backward));
        EventResult::Ignored
    }
}