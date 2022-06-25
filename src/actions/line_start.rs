use cursive::event::{Event, EventResult};
use cursive::event::Key::Home;
use crate::actions::action::Action;
use crate::model::dimension::Dimension;
use crate::model::model::RootModel;

pub struct LineStartAction {}

impl LineStartAction {
    pub fn new() -> Self {
        LineStartAction {}
    }
}

impl Action for LineStartAction {
    fn description(&self) -> &str {
        "Go to line start"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Key(Home)]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
        match model.get_cursor_on_screen() {
            Some(Dimension { height: h, width: _ }) => {
                let p = model.data()
                    .and_then(|data| data.lines.get(h.as_usize()))
                    .map(|line| line.start);
                if let Some(p) = p {
                    model.move_cursor_to_offset(p, false);
                    EventResult::Consumed(None)
                } else {
                    EventResult::Ignored
                }
            }
            _ => EventResult::Ignored
        }
    }
}