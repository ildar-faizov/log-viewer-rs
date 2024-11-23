use cursive::event::{Event, EventResult};
use logv_macro::define_action;

use crate::model::model::RootModel;

#[define_action]
fn select_all(model: &mut RootModel, _event: &Event) -> EventResult {
    model.select_all();
    EventResult::Consumed(None)
}