use cursive::event::{EventResult, Key};
use fluent_integer::Integer;
use crate::actions::action::Action;
use crate::{Event, RootModel};

pub struct ScrollUpAction {

}

impl ScrollUpAction {
    pub fn new() -> Self {
        ScrollUpAction {}
    }
}

impl Action for ScrollUpAction {

    fn description(&self) -> &str {
        "Scroll one line above"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Ctrl(Key::Up)]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.scroll(Integer::from(-1));
        EventResult::Consumed(None)
    }
}