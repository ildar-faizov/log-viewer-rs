use cursive::event::EventResult;
use crate::actions::action::Action;
use crate::{Event, RootModel};
use crate::Event::CtrlChar;

pub struct CopyAction {}

impl CopyAction {
    pub fn new() -> Self {
        CopyAction {}
    }
}

impl Action for CopyAction {
    fn description(&self) -> &str {
        "Copy selected text"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![CtrlChar('c')]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        if let Some(content) = model.get_selected_content() {
            terminal_clipboard::set_string(content).unwrap();
        }
        EventResult::Ignored
    }
}