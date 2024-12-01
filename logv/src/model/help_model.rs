use crossbeam_channel::Sender;

use crate::actions::action::Action;
use crate::actions::action_registry::ActionRegistry;
use crate::model::escape_handler::{CompoundEscapeHandler, EscapeHandlerManager, EscapeHandlerResult};
use crate::model::model::{ModelEvent, RootModel};
use crate::model::model::ModelEvent::HelpEvent;
use crate::shared::Shared;
use crate::utils::event_emitter::EventEmitter;

#[derive(Debug)]
pub struct HelpModel {
    is_open: bool,
    filter: String,
    actions: Vec<ActionDescription>,
    filtered_actions: Vec<ActionDescription>,
    model_sender: Sender<ModelEvent>,
    escape_handler_manager: EscapeHandlerManager,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ActionDescription {
    pub hotkeys: String,
    pub description: String,
}

#[derive(Debug)]
pub enum HelpModelEvent {
    Show,
    Hide,
    ListUpdated,
}

impl HelpModel {
    pub fn new(
        model_sender: Sender<ModelEvent>,
        action_registry: &ActionRegistry,
        escape_handler: Shared<CompoundEscapeHandler>,
    ) -> Self {
        let actions = action_registry
            .into_iter()
            .map(|a| ActionDescription::from(&*a))
            .collect();
        Self {
            is_open: false,
            filter: String::new(),
            actions,
            filtered_actions: Vec::default(),
            model_sender,
            escape_handler_manager: EscapeHandlerManager::new(escape_handler, Self::on_esc),
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn set_open(&mut self, is_open: bool) {
        if self.is_open != is_open {
            self.is_open = is_open;
            self.escape_handler_manager.toggle(is_open);
            let event = if is_open {
                self.filter_items();
                HelpModelEvent::Show
            } else {
                HelpModelEvent::Hide
            };
            self.emit_event(event);
        }
    }

    pub fn get_filter(&self) -> &str {
        self.filter.as_str()
    }

    pub fn set_filter<F: ToString>(&mut self, filter: F) {
        let s = filter.to_string();
        if self.filter != s {
            self.filter = s;
            self.filter_items();
            self.emit_event(HelpModelEvent::ListUpdated);
        }
    }

    pub fn get_filtered_actions(&self) -> Vec<ActionDescription> {
        self.filtered_actions.clone()
    }

    fn filter_items(&mut self) {
        let filter = self.create_filter();
        self.filtered_actions = self.actions
            .iter()
            .filter(|a| filter(a))
            .map(Clone::clone)
            .collect();
    }

    fn create_filter(&self) -> Box<Predicate<ActionDescription>> {
        let f = self.filter.trim().to_string().to_lowercase();
        if f.is_empty() {
            Box::new(|_a: &ActionDescription| true)
        } else {
            Box::new(move |a: &ActionDescription| a.description.to_lowercase().contains(f.as_str()))
        }
    }

    fn on_esc(root_model: &mut RootModel) -> EscapeHandlerResult {
        let help_model = &mut *root_model.get_help_model();
        if help_model.is_open() {
            help_model.set_open(false);
            EscapeHandlerResult::Dismiss
        } else {
            EscapeHandlerResult::Ignore
        }
    }
}

impl EventEmitter<HelpModelEvent> for HelpModel {
    fn emit_event(&self, evt: HelpModelEvent) {
        self.model_sender.emit_event(HelpEvent(evt));
    }
}

impl From<&Action> for ActionDescription {
    fn from(value: &Action) -> Self {
        ActionDescription {
            hotkeys: value.print_hotkeys(),
            description: value.description().to_string(),
        }
    }
}

type Predicate<T> = dyn Fn(&T) -> bool;
