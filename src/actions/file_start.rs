use cursive::event::{Event, EventResult};
use fluent_integer::Integer;
use logv_macro::define_action;
use num_traits::Zero;

use crate::model::model::RootModel;

#[define_action]
fn file_start(model: &mut RootModel, _event: &Event) -> EventResult {
    model.move_cursor_to_offset(Integer::zero(), false);
    EventResult::Consumed(None)
}