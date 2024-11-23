use cursive::event::{Event, EventResult};
use fluent_integer::Integer;
use logv_macro::define_action;

use crate::model::model::RootModel;

#[define_action]
fn scroll_up(model: &mut RootModel, _event: &Event) -> EventResult {
    model.scroll(Integer::from(-1));
    EventResult::Consumed(None)
}