use anyhow::{anyhow, bail};
use cursive::event::{Event, EventResult, Key};
use itertools::Itertools;
use phf::phf_map;

use crate::actions::action_impl::ActionImpl;
use crate::actions::action_impl_registry::REGISTRY;
use crate::actions::event_display::EventSliceDisplay;
use crate::model::model::RootModel;
use crate::profiles::ActionDescription;

#[derive(Clone)]
pub struct Action {
    id: String,
    description: String,
    hotkeys: Vec<Event>,
    action_impl: &'static ActionImpl,
}

/// UI action
impl Action {
    /// User-friendly description of the action
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// List of events that trigger the action
    pub fn hotkeys(&self) -> Vec<Event> {
        self.hotkeys.clone()
    }

    /// Actually performs action.
    /// The method is intended to mutate model if necessary and return a result
    /// indicating whether model state is changed
    pub fn perform_action(&self, model: &mut RootModel, event: &Event) -> EventResult {
        (self.action_impl.action_impl)(model, event)
    }

    pub fn print_hotkeys(&self) -> String {
        let hotkeys = self.hotkeys();
        format!("{}", EventSliceDisplay::new(&hotkeys))
    }
}

impl From<&ActionDescription> for Action {
    fn from(value: &ActionDescription) -> Self {
        let hotkeys = value
            .hotkeys()
            .iter()
            .map(|i| {
                parse_hotkey(i).unwrap()
            }).collect();
        let action_impl = REGISTRY.iter()
            .find(|action_impl| action_impl.id == value.id())
            .expect(format!("Implementation for id {} not found", value.id()).as_str());
        Self {
            id: value.id().to_string(),
            description: value.description().unwrap_or_default().to_string(),
            hotkeys,
            action_impl
        }
    }
}

const KEYS: phf::Map<&'static str, Key> = phf_map! {
    "ENTER" => Key::Enter,
    "TAB" => Key::Tab,
    "BACKSPACE" => Key::Backspace,
    "ESC" => Key::Esc,
    "LEFT" => Key::Left,
    "RIGHT" => Key::Right,
    "UP" => Key::Up,
    "DOWN" => Key::Down,
    "INS" => Key::Ins,
    "DEL" => Key::Del,
    "HOME" => Key::Home,
    "END" => Key::End,
    "PAGEUP" => Key::PageUp,
    "PAGEDOWN" => Key::PageDown,
};

/// Parses hotkey from profile to Event
fn parse_hotkey(s: &String) -> anyhow::Result<Event> {
    let split = s.split("+");
    let mut is_ctrl = false;
    let mut is_shift = false;
    let mut is_alt = false;
    let mut ch = None;
    let mut key = None;
    for p in split {
        if "ctrl".eq_ignore_ascii_case(p) {
            is_ctrl = true;
        } else if "shift".eq_ignore_ascii_case(p) {
            is_shift = true;
        } else if "alt".eq_ignore_ascii_case(p) {
            is_alt = true;
        } else if p.len() == 1 {
            ch = p.chars().next();
        } else {
            key = KEYS.get(p.to_ascii_uppercase().as_str());
            if key.is_none() {
                bail!("Failed to parse {}", s);
            }
        }
    }
    if ch.is_some() && key.is_some() {
        bail!("Failed to parse {}. Cannot have char and key at the same time", s);
    }
    if is_shift && ch.is_some() {
        is_shift = false;
        ch = ch.map(|c| c.to_ascii_uppercase());
    }

    if ch.is_some() {
        let ch = ch.unwrap();
        match (is_ctrl, is_alt) {
            (true, true) => Err(anyhow!("Failed to parse {}. Ctrl+Alt+<char> is not supported.", s)),
            (true, false) => Ok(Event::CtrlChar(ch)),
            (false, true) => Ok(Event::AltChar(ch)),
            (false, false) => Ok(Event::Char(ch)),
        }
    } else if key.is_some() {
        let key = *key.unwrap();
        match (is_ctrl, is_alt, is_shift) {
            (true, true, true) => Err(anyhow!("Failed to parse {}. Combination is not supported.", s)),
            (true, true, false) => Ok(Event::CtrlAlt(key)),
            (true, false, true) => Ok(Event::CtrlShift(key)),
            (true, false, false) => Ok(Event::Ctrl(key)),
            (false, true, true) => Ok(Event::AltShift(key)),
            (false, true, false) => Ok(Event::Alt(key)),
            (false, false, true) => Ok(Event::Shift(key)),
            (false, false, false) => Ok(Event::Key(key)),
        }
    } else {
        bail!("Failed to parse {}. Either char or key must be present", s);
    }
}

#[cfg(test)]
mod tests {
    use cursive::event::{Event, Key};
    use paste::paste;
    use spectral::prelude::*;

    use crate::actions::action::parse_hotkey;

    macro_rules! test_parse_hotkey {
        ($name: ident, $input: expr => $expected: expr) => {
            paste! {
                #[test]
                fn [< test_parse_hotkey_ $name >]() {
                    let input = $input.to_string();
                    let result = parse_hotkey(&input);
                    assert_that!(result).is_ok_containing($expected);
                }
            }
        };
    }
    test_parse_hotkey!(single_char, "o" => Event::Char('o'));
    test_parse_hotkey!(ctrl_char, "Ctrl+o" => Event::CtrlChar('o'));
    test_parse_hotkey!(alt_char, "Alt+l" => Event::AltChar('l'));
    test_parse_hotkey!(shift_char, "Shift+d" => Event::Char('D'));
    test_parse_hotkey!(ctrl_shift_char, "Ctrl+Shift+d" => Event::CtrlChar('D'));
    test_parse_hotkey!(single_key_up, "UP" => Event::Key(Key::Up));
    test_parse_hotkey!(single_key_pgdown, "PageDown" => Event::Key(Key::PageDown));
    test_parse_hotkey!(ctrl_key, "Ctrl+Home" => Event::Ctrl(Key::Home));
    test_parse_hotkey!(shift_key, "Shift+Left" => Event::Shift(Key::Left));
}