use cursive::event::{Event, EventResult};
use logv_macro::define_action;

use crate::model::dimension::Dimension;
use crate::model::model::RootModel;

#[define_action]
fn line_end(model: &mut RootModel, _event: &Event) -> EventResult {
    match model.get_cursor_on_screen() {
        Some(Dimension { height: h, width: _ }) => {
            let p = model.data()
                .and_then(|data| data.lines.get(h.as_usize()))
                .map(|line| line.end);
            if let Some(p) = p {
                model.move_cursor_to_offset(p - 1, false);
                EventResult::Consumed(None)
            } else {
                EventResult::Ignored
            }
        }
        _ => EventResult::Ignored
    }
}