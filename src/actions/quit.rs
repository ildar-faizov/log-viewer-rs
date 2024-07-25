use cursive::event::{Event, EventResult};
use logv_macro::define_action;

use crate::model::model::RootModel;

#[define_action]
fn quit(model: &mut RootModel, _event: &Event) -> EventResult {
    model.quit();
    EventResult::Consumed(None)
}