use cursive::event::{Event, EventResult};
use crate::actions::action::Action;
use crate::model::cursor_shift::CursorShift;
use crate::model::model::RootModel;

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

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.move_cursor(CursorShift::token_forward(), true);
        EventResult::Consumed(None)
    }
}