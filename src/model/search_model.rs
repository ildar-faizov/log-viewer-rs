use crossbeam_channel::Sender;
use crate::model::model::ModelEvent;

#[derive(Debug)]
pub struct SearchModel {
    model_sender: Sender<ModelEvent>,
    visible: bool,
    pattern: String,
    is_regexp: bool,
}

impl SearchModel {
    pub fn new(model_sender: Sender<ModelEvent>) -> Self {
        SearchModel {
            model_sender,
            visible: false,
            pattern: String::new(),
            is_regexp: false,
        }
    }

    pub fn set_visible(&mut self, visible: bool) {
        if self.visible != visible {
            self.visible = visible;
            self.emit_event(ModelEvent::Search(visible));
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_pattern<T: Into<String>>(&mut self, pattern: T) {
        self.pattern = pattern.into();
    }

    pub fn get_pattern(&self) -> &str {
        self.pattern.as_str()
    }

    pub fn start_search(&mut self) {
        // TODO here the action begins
        log::info!("Search to be implemented: {:?}", self.pattern);
    }

    fn emit_event(&self, evt: ModelEvent) {
        let msg = format!("Failed to send event: {:?}", evt);
        self.model_sender.send(evt)
            .expect(msg.as_str());
    }
}