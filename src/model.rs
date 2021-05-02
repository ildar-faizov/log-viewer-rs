use crossbeam_channel::Sender;

pub struct RootModel {
    model_sender: Sender<ModelEvent>,
    file_name: Option<String>,
}

pub enum ModelEvent {
    FileName(String)
}

impl RootModel {
    pub fn new(model_sender: Sender<ModelEvent>) -> Self {
        RootModel {
            model_sender,
            file_name: None
        }
    }

    fn emit_event(&self, event: ModelEvent) {
        self.model_sender.send(event);
    }

    pub fn file_name(&self) -> Option<String> {
        self.file_name.clone()
    }

    pub fn set_file_name(&mut self, value: String) {
        // TODO: check equality
        // if self.file_name != value {
            self.file_name = Some(value);
            self.emit_event(ModelEvent::FileName(self.file_name.as_ref().unwrap().to_owned()));
        // }
    }
}
