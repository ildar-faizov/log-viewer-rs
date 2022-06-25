use cursive::event::{Event, EventResult};
use crate::actions::action::Action;
use crate::model::cursor_shift::CursorShift;
use crate::model::model::RootModel;

pub struct SelectWordBackwardAction{}

impl SelectWordBackwardAction {
    pub fn new() -> Self {
        SelectWordBackwardAction {}
    }
}

impl Action for SelectWordBackwardAction {
    fn description(&self) -> &str {
        "Select word backward"
    }

    fn hotkeys(&self) -> Vec<Event> {
        // Unfortunately Shift+Ctrl+<Arrow> does not work in terminals
        vec![Event::CtrlChar('b')]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.move_cursor(CursorShift::token_backward(), true);
        EventResult::Consumed(None)
    }
}