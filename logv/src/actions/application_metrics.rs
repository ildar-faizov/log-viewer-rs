use cursive::event::{Event, EventResult};
use logv_macro::define_action;

use crate::model::model::RootModel;

#[define_action]
fn application_metrics(model: &mut RootModel, _event: &Event) -> EventResult {
    model.get_metrics_model().set_open(true);
    EventResult::Consumed(None)
}