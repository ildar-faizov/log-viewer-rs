use cursive::event::{Event, EventResult};

use crate::model::model::RootModel;

pub struct ActionImpl {
    pub id: &'static str,
    pub action_impl: fn(model: &mut RootModel, event: &Event) -> EventResult,
}