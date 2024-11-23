use crossbeam_channel::Sender;
use uuid::Uuid;

#[derive(Clone)]
pub struct BackgroundProcessHandler {
    sender_interrupt: Sender<bool>,
    id: Uuid,
    title: String,
    description: String,
}

impl BackgroundProcessHandler {
    pub fn new(sender_interrupt: Sender<bool>, id: Uuid, title: String, description: String) -> Self {
        BackgroundProcessHandler {
            sender_interrupt,
            id,
            title,
            description,
        }
    }

    pub fn interrupt(&self) -> bool {
        self.sender_interrupt.send(true).is_ok()
    }

    pub fn get_id(&self) -> &Uuid {
        &self.id
    }

    pub fn get_title(&self) -> &str {
        self.title.as_str()
    }

    pub fn get_description(&self) -> &str {
        self.description.as_str()
    }
}
