use cursive::event::{Event, EventResult};
use logv_macro::define_action;

use crate::model::cursor_shift::CursorShift;
use crate::model::model::RootModel;

#[define_action]
fn page_down(model: &mut RootModel, _event: &Event) -> EventResult {
    let h = model.get_viewport_height();
    model.move_cursor(CursorShift::Y(h), false);
    EventResult::Consumed(None)
}