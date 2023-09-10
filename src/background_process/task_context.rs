use crate::background_process::signal::Signal;
use crossbeam_channel::{Receiver, SendError, Sender};
use uuid::Uuid;

pub struct TaskContext<M, R> {
    interrupted: bool,
    pub sender: Sender<Signal<M, R>>,
    ri: Receiver<bool>,
    id: Uuid,
}

impl<M, R> TaskContext<M, R> {
    pub fn new(sender: Sender<Signal<M, R>>, ri: Receiver<bool>, id: Uuid) -> Self {
        TaskContext {
            interrupted: false,
            sender,
            ri,
            id,
        }
    }

    pub fn send_message(&self, msg: M) -> Result<(), SendError<Signal<M, R>>> {
        self.sender.send(Signal::Custom(msg))
    }

    pub fn update_progress(&self, progress: u8) {
        self.sender
            .send(Signal::Progress(progress))
            .expect("Failed to update progress");
    }

    pub fn interrupted(&mut self) -> bool {
        let r = self.interrupted || self.ri.try_recv().unwrap_or_default();
        self.interrupted = r;
        r
    }

    #[allow(dead_code)]
    pub fn get_id(&self) -> &Uuid {
        &self.id
    }
}
