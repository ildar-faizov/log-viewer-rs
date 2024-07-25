use cursive::event::{Event, EventResult};
use logv_macro::define_action;

use crate::data_source::Direction;
use crate::model::model::RootModel;

#[define_action]
fn search_prev(model: &mut RootModel, _event: &Event) -> EventResult {
    model.get_current_search().as_mut().map(|s| s.search(Direction::Backward));
    EventResult::Ignored
}