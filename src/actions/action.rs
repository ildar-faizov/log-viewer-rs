use cursive::event::{Event, EventResult};
use crate::RootModel;

/// Generic trait for any UI action
pub trait Action {
    /// Factory method
    // fn new() -> Self
    //     where Self: Sized;

    /// User-friendly description of the action
    fn description(&self) -> &str;

    /// List of events that trigger the action
    fn hotkeys(&self) -> Vec<Event>;

    /// Actually performs action.
    /// The method is intended to mutate model if necessary and return a result
    /// indicating whether model state is changed
    fn perform_action(&self, model: &mut RootModel, event: &Event) -> EventResult;
}