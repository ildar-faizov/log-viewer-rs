use cursive::event::{Event, EventResult};
use cursive::event::Event::Char;
use crate::actions::action::Action;
use crate::model::model::RootModel;

#[derive(Default, Debug, Clone)]
pub struct ApplicationMetricsAction {}

impl Action for ApplicationMetricsAction {
    fn description(&self) -> &str {
        "Show application metrics"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Char('M')]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        model.get_metrics_model().set_open(true);
        EventResult::Consumed(None)
    }
}