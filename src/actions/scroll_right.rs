use cursive::event::{EventResult, Key};
use crate::actions::action::Action;
use crate::{Event, RootModel};

pub struct ScrollRightAction {}

impl ScrollRightAction {
    pub fn new() -> Self {
        ScrollRightAction {}
    }
}

impl Action for ScrollRightAction {

    fn description(&self) -> &str {
        "Scroll one symbol right"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Ctrl(Key::Right)]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        let horizontal_scroll = model.get_horizontal_scroll();
        if model.set_horizontal_scroll(horizontal_scroll + 1) {
            EventResult::Consumed(None)
        } else {
            EventResult::Ignored
        }
    }
}