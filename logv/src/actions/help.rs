use cursive::event::{Event, EventResult};
use logv_macro::define_action;

use crate::model::model::RootModel;

#[define_action]
fn help(model: &mut RootModel, _event: &Event) -> EventResult {
    model.get_help_model().set_open(true);
    EventResult::Ignored
}