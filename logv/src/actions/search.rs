use cursive::event::{Event, EventResult};
use logv_macro::define_action;

use crate::model::model::RootModel;

#[define_action]
fn search(model: &mut RootModel, _event: &Event) -> EventResult {
    model.get_search_model().set_visible(true);
    EventResult::Ignored // TODO: is it correct?
}