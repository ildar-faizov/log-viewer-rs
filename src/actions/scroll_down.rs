use cursive::event::{Event, EventResult};
use fluent_integer::Integer;
use logv_macro::define_action;
use num_traits::One;

use crate::model::model::RootModel;

#[define_action]
fn scroll_down(model: &mut RootModel, _event: &Event) -> EventResult {
    model.scroll(Integer::one());
    EventResult::Consumed(None)
}