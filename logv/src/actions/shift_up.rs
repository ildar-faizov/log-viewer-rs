use cursive::event::{Event, EventResult};
use logv_macro::define_action;

use crate::model::cursor_shift::CursorShift;
use crate::model::model::RootModel;

#[define_action]
fn select_up(model: &mut RootModel, _event: &Event) -> EventResult {
    model.move_cursor(CursorShift::up(), true);
    EventResult::Consumed(None)
}