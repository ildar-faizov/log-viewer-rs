use std::cell::Cell;
use crate::background_process::signal::Signal;
use crossbeam_channel::{Receiver, SendError, Sender};
use uuid::Uuid;

pub struct TaskContext<M, R> {
    interrupted: Cell<bool>,
    sender: Sender<Signal<M, R>>,
    ri: Receiver<bool>,
    id: Uuid,
    last_reported_progress: Cell<u8>,
}

impl<M, R> TaskContext<M, R> {
    pub fn new(sender: Sender<Signal<M, R>>, ri: Receiver<bool>, id: Uuid) -> Self {
        TaskContext {
            interrupted: Cell::new(false),
            sender,
            ri,
            id,
            last_reported_progress: Cell::new(0),
        }
    }

    pub fn send_message(&self, msg: M) -> Result<(), SendError<Signal<M, R>>> {
        self.sender.send(Signal::Custom(msg))
    }

    pub fn update_progress(&self, progress: u8) {
        let last_reported_progress = self.last_reported_progress.replace(progress);
        if last_reported_progress != progress {
            self.sender
                .send(Signal::Progress(progress))
                .expect("Failed to update progress");
        }
    }

    pub fn interrupted(&self) -> bool {
        let r = self.interrupted.get() || self.ri.try_recv().unwrap_or_default();
        self.interrupted.set(r);
        r
    }

    #[allow(dead_code)]
    pub fn get_id(&self) -> &Uuid {
        &self.id
    }
}
