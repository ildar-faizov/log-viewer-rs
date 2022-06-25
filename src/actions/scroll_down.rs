use cursive::event::{Event, EventResult, Key};
use num_traits::One;
use fluent_integer::Integer;
use crate::actions::action::Action;
use crate::model::model::RootModel;

pub struct ScrollDownAction {

}

impl ScrollDownAction {
    pub fn new() -> Self {
        ScrollDownAction {}
    }
}

impl Action for ScrollDownAction {

    fn description(&self) -> &str {
        "Scroll one line below"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Ctrl(Key::Down)]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.scroll(Integer::one());
        EventResult::Consumed(None)
    }
}