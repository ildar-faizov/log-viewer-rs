use cursive::event::{Event, EventResult};
use logv_macro::define_action;

use crate::model::model::RootModel;

#[define_action]
fn copy(model: &mut RootModel, _event: &Event) -> EventResult {
    if let Some(content) = model.get_selected_content() {
        terminal_clipboard::set_string(content).unwrap();
    }
    EventResult::Ignored
}