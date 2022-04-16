use std::collections::HashMap;
use std::rc::Rc;
use cursive::event::Event;
use crate::actions::action::Action;
use crate::actions::copy::CopyAction;
use crate::actions::cursor_down::CursorDownAction;
use crate::actions::cursor_left::CursorLeftAction;
use crate::actions::cursor_right::CursorRightAction;
use crate::actions::cursor_up::CursorUpAction;
use crate::actions::file_end::FileEndAction;
use crate::actions::file_start::FileStartAction;
use crate::actions::line_end::LineEndAction;
use crate::actions::line_start::LineStartAction;
use crate::actions::pgdown::PgDownAction;
use crate::actions::pgup::PgUpAction;
use crate::actions::quit::QuitAction;
use crate::actions::scroll_down::ScrollDownAction;
use crate::actions::scroll_left::ScrollLeftAction;
use crate::actions::scroll_right::ScrollRightAction;
use crate::actions::scroll_up::ScrollUpAction;
use crate::actions::select_all::SelectAllAction;
use crate::actions::select_word_forward::SelectWordForwardAction;
use crate::actions::select_word_backward::SelectWordBackwardAction;
use crate::actions::shift_down::ShiftDownAction;
use crate::actions::shift_left::ShiftLeftAction;
use crate::actions::shift_right::ShiftRightAction;
use crate::actions::shift_up::ShiftUpAction;

pub fn action_registry() -> HashMap<Event, Rc<dyn Action + 'static>> {
    let mut map = HashMap::new();
    for a in plain_action_registry() {
        for hotkey in a.hotkeys() {
            map.insert(hotkey, Rc::clone(&a));
        }
    }
    map
}

fn plain_action_registry() -> Vec<Rc<dyn Action + 'static>> {
    vec![
        Rc::new(ScrollDownAction::new()),
        Rc::new(ScrollUpAction::new()),
        Rc::new(ScrollLeftAction::new()),
        Rc::new(ScrollRightAction::new()),

        Rc::new(CursorDownAction::new()),
        Rc::new(CursorUpAction::new()),
        Rc::new(CursorLeftAction::new()),
        Rc::new(CursorRightAction::new()),

        Rc::new(LineStartAction::new()),
        Rc::new(LineEndAction::new()),

        Rc::new(FileStartAction::new()),
        Rc::new(FileEndAction::new()),

        Rc::new(ShiftDownAction::new()),
        Rc::new(ShiftUpAction::new()),
        Rc::new(ShiftLeftAction::new()),
        Rc::new(ShiftRightAction::new()),

        Rc::new(PgUpAction::new()),
        Rc::new(PgDownAction::new()),

        Rc::new(SelectAllAction::new()),

        Rc::new(SelectWordForwardAction::new()),
        Rc::new(SelectWordBackwardAction::new()),

        Rc::new(CopyAction::new()),

        Rc::new(QuitAction::new()),
    ]
}