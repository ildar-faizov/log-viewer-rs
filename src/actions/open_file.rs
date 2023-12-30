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
        let current_file = model.resolve_file_name();
        let open_file_model = &mut *model.get_open_file_model();
        open_file_model.set_open(true);
        if let Some(current_file) = current_file {
            if let Some(location) = current_file.parent() {
                open_file_model.set_current_location(location.to_path_buf());
            }
            let file_name = current_file.file_name().and_then(|f| f.to_str());
            open_file_model.set_current_file(file_name);
        } else {
            log::warn!("Could not resolve current file name");
            open_file_model.set_current_location(std::env::current_dir().expect("Current dir not set"));
            open_file_model.set_current_file(None);
        }
        EventResult::Consumed(None)
    }
}