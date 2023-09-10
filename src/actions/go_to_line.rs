use crate::actions::action::Action;
use crate::model::model::RootModel;
use cursive::event::{Event, EventResult};

#[derive(Default)]
pub struct GoToLineAction {}

impl Action for GoToLineAction {
    fn description(&self) -> &str {
        "Go to line"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::CtrlChar('g')]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        let go_to_model = &mut *model.get_go_to_line_model();
        go_to_model.set_is_open(true);
        EventResult::Consumed(None)
    }
}
