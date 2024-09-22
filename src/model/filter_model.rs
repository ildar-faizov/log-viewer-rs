use crossbeam_channel::Sender;
use crate::model::model::ModelEvent;
use crate::utils::event_emitter::EventEmitter;

#[derive(Debug)]
pub enum FilterDialogModelEvent {
    VisibilityChanged(bool),
}

pub struct FilterDialogModel {
    model_sender: Sender<ModelEvent>,
    is_open: bool,
    pattern: String,
    is_regexp: bool,
    neighbourhood: String,
}

impl FilterDialogModel {
    pub fn new(model_sender: Sender<ModelEvent>) -> Self {
        FilterDialogModel {
            model_sender,
            is_open: false,
            pattern: String::new(),
            is_regexp: false,
            neighbourhood: 0.to_string(),
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn set_open(&mut self, is_open: bool) {
        if self.is_open != is_open {
            self.is_open = is_open;
            self.emit_event(FilterDialogModelEvent::VisibilityChanged(is_open));
        }
    }

    pub fn get_pattern(&self) -> &str {
        &self.pattern
    }

    pub fn set_pattern(&mut self, pattern: impl ToString) {
        self.pattern = pattern.to_string();
    }

    pub fn is_regexp(&self) -> bool {
        self.is_regexp
    }

    pub fn get_neighbourhood(&self) -> &str {
        &self.neighbourhood
    }

    pub fn set_neighbourhood(&mut self, neighbourhood: impl ToString) {
        self.neighbourhood = neighbourhood.to_string();
    }

    fn emit_event(&self, evt: FilterDialogModelEvent) {
        self.model_sender.emit_event(ModelEvent::FilterEvent(evt));
    }
}