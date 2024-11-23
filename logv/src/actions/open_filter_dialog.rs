use cursive::event::{Event, EventResult};
use logv_macro::define_action;

use crate::model::model::RootModel;

#[define_action]
fn open_filter_dialog(model: &mut RootModel, _event: &Event) -> EventResult {
    model.get_filter_dialog_model().set_open(true);
    EventResult::Consumed(None)
}