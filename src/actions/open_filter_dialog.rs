use cursive::event::{Event, EventResult};
use crate::actions::action::Action;
use crate::model::model::RootModel;

#[derive(Default)]
pub struct OpenFilterDialogAction {}

impl Action for OpenFilterDialogAction {
    fn description(&self) -> &str {
        "Open filter dialog (grep)"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::CtrlChar('y')]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.get_filter_dialog_model().set_open(true);
        EventResult::Consumed(None)
    }
}