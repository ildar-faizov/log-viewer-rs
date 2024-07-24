use std::collections::HashMap;
use std::rc::Rc;
use std::vec::IntoIter;

use cursive::event::Event;

use crate::actions::action::Action;
use crate::profiles::Profile;

pub struct ActionRegistry {
    registry: HashMap<Event, Rc<Action>>,
}

impl ActionRegistry {
    pub fn new(profile: &Profile) -> Self {
        let mut registry = HashMap::new();
        for action_description in profile.actions() {
            let action = Rc::new(Action::from(action_description));
            for x in &action.hotkeys() {
                registry.insert(x.clone(), Rc::clone(&action));
            }
        }
        Self {
            registry
        }
    }

    pub fn lookup_by_key(&self, event: &Event) -> Option<&Rc<Action>> {
        self.registry.get(event)
    }
}

impl IntoIterator for &ActionRegistry {
    type Item = Rc<Action>;
    type IntoIter = IntoIter<Rc<Action>>;

    fn into_iter(self) -> Self::IntoIter {
        let v: Vec<Rc<Action>> = self.registry.values().cloned().collect();
        v.into_iter()
    }
}