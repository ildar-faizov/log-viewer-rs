use copypasta::{ClipboardContext, ClipboardProvider};
use cursive::event::{Event, EventResult};
use logv_macro::define_action;

use crate::model::model::RootModel;

#[define_action]
fn copy(model: &mut RootModel, _event: &Event) -> EventResult {
    if let Some(content) = model.get_selected_content() {
        let mut ctx = ClipboardContext::new().unwrap();
        let result = ctx.set_contents(content);
        if let Err(e) = result {
            model.set_error(Box::new(format!("Failed to copy: {:?}", e)));
        }
    }
    EventResult::Ignored
}