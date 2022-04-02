use cursive::event::EventResult;
use cursive::event::Key::End;
use crate::actions::action::Action;
use crate::{Event, RootModel};
use crate::model::Dimension;

pub struct LineEndAction {}

impl LineEndAction {
    pub fn new() -> Self {
        LineEndAction {}
    }
}

impl Action for LineEndAction {
    fn description(&self) -> &str {
        "Go to line end"
    }

    fn hotkeys(&self) -> Vec<Event> {
        vec![Event::Key(End)]
    }

    fn perform_action(&self, model: &mut RootModel, _event: &Event) -> EventResult {
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
}