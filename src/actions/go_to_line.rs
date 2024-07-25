use cursive::event::{Event, EventResult};
use logv_macro::define_action;

use crate::model::model::RootModel;

#[define_action]
fn go_to_line(model: &mut RootModel, _event: &Event) -> EventResult {
    let go_to_model = &mut *model.get_go_to_line_model();
    go_to_model.set_is_open(true);
    EventResult::Consumed(None)
}
