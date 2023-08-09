use crate::actions::action::Action;
use crate::model::model::RootModel;
use cursive::event::{Event, EventResult};

#[derive(Default)]
pub struct TestBGPAction {}

impl Action for TestBGPAction {
    fn description(&self) -> &str {
        "Start test background process"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::CtrlChar('t')]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.start_test_bgp();
        EventResult::Ignored
    }
}
