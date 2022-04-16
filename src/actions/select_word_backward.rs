use cursive::event::{EventResult, Key};
use cursive::event::Key::Right;
use fluent_integer::Integer;
use crate::actions::action::Action;
use crate::{Event, RootModel};
use crate::Event::Shift;
use crate::model::CursorShift;

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
        model.move_cursor(CursorShift::Word(Integer::from(-1)), true);
        EventResult::Consumed(None)
    }
}