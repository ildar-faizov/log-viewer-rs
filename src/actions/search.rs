use cursive::event::{Event, EventResult};
use crate::actions::action::Action;
use crate::model::model::RootModel;

#[derive(Default)]
pub struct SearchAction {

}

impl SearchAction {

}

impl Action for SearchAction {
    fn description(&self) -> &str {
        "Search"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::CtrlChar('f')]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.get_search_model().set_visible(true);
        EventResult::Ignored // TODO: is it correct?
    }
}