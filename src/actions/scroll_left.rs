use cursive::event::{EventResult, Key};
use crate::actions::action::Action;
use crate::{Event, RootModel};

pub struct ScrollLeftAction {}

impl ScrollLeftAction {
    pub fn new() -> Self {
        ScrollLeftAction {}
    }
}

impl Action for ScrollLeftAction {

    fn description(&self) -> &str {
        "Scroll one symbol left"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Ctrl(Key::Left)]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        let horizontal_scroll = model.get_horizontal_scroll();
        if horizontal_scroll > 0 {
            model.set_horizontal_scroll(horizontal_scroll - 1);
        }
        EventResult::Consumed(None)
    }
}