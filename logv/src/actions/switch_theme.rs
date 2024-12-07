use crate::model::model::RootModel;
use cursive::event::{Event, EventResult};
use logv_macro::define_action;

#[define_action]
fn switch_theme(model: &mut RootModel, _event: &Event) -> EventResult {
    model.trigger_theme_switch();
    EventResult::Consumed(None)
}
