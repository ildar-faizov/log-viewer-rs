use crossbeam_channel::{SendError, Sender};

pub struct BackgroundProcessHandler {
    sender_interrupt: Sender<bool>,
}

impl BackgroundProcessHandler {
    pub fn new(sender_interrupt: Sender<bool>) -> Self {
        BackgroundProcessHandler { sender_interrupt }
    }

    pub fn interrupt(self) -> Result<(), SendError<bool>> {
        self.sender_interrupt.send(true)
    }
}
