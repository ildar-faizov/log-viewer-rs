use cursive::event::{Event, EventResult};
use logv_macro::define_action;

use crate::model::model::RootModel;

#[define_action]
fn file_end(model: &mut RootModel, _event: &Event) -> EventResult {
    if model.move_cursor_to_end() {
        EventResult::Consumed(None)
    } else {
        EventResult::Ignored
    }
}