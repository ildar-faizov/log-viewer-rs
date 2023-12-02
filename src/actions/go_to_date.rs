use cursive::event::{Event, EventResult};
use crate::actions::action::Action;
use crate::model::model::RootModel;

#[derive(Default, Debug)]
pub struct GoToDateAction {}

impl Action for GoToDateAction {
    fn description(&self) -> &str {
        "Go to date"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::CtrlChar('d')]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        let go_to_date_model = &mut *model.get_go_to_date_model();
        go_to_date_model.set_is_open(true);
        EventResult::Consumed(None)
    }
}