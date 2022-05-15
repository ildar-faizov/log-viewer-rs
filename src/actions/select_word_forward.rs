use cursive::event::{EventResult, Key};
use cursive::event::Key::Right;
use crate::actions::action::Action;
use crate::{Event, RootModel};
use crate::Event::Shift;
use crate::model::CursorShift;

pub struct SelectWordForwardAction {}

impl SelectWordForwardAction {
    pub fn new() -> Self {
        SelectWordForwardAction {}
    }
}

impl Action for SelectWordForwardAction {
    fn description(&self) -> &str {
        "Select word forward"
    }

    fn hotkeys(&self) -> Vec<Event> {
        // Unfortunately Shift+Ctrl+<Arrow> does not work in terminals
        vec![Event::CtrlChar('d')]
    }

    fn perform_action(&self, model: &mut RootModel, event: &Event) -> EventResult {
        model.move_cursor(CursorShift::token_forward(), true);
        EventResult::Consumed(None)
    }
}