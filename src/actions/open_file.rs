use cursive::event::{Event, EventResult};
use cursive::event::Event::CtrlChar;
use crate::actions::action::Action;
use crate::model::model::RootModel;

pub struct OpenFileAction {}

impl OpenFileAction {
    pub fn new() -> Self {
        OpenFileAction {}
    }
}

impl Action for OpenFileAction {
    fn description(&self) -> &str {
        "Open File"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![CtrlChar('o')]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        let mut open_file_model = &mut *model.get_open_file_model();
        open_file_model.set_open(true);
        EventResult::Consumed(None)
    }
}