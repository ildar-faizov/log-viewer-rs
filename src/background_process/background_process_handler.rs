use crossbeam_channel::Sender;
use uuid::Uuid;

pub struct BackgroundProcessHandler {
    sender_interrupt: Sender<bool>,
    id: Uuid,
}

impl BackgroundProcessHandler {
    pub fn new(sender_interrupt: Sender<bool>, id: Uuid) -> Self {
        BackgroundProcessHandler { sender_interrupt, id }
    }

    pub fn interrupt(&self) -> bool {
        self.sender_interrupt.send(true).is_ok()
    }

    pub fn get_id(&self) -> &Uuid {
        &self.id
    }
}
