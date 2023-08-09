use crate::background_process::signal::Signal;
use crossbeam_channel::{Receiver, SendError, Sender};

pub struct TaskContext<M, R> {
    pub interrupted: bool,
    pub sender: Sender<Signal<M, R>>,
    ri: Receiver<bool>,
}

impl<M, R> TaskContext<M, R> {
    pub fn new(sender: Sender<Signal<M, R>>, ri: Receiver<bool>) -> Self {
        TaskContext {
            interrupted: false,
            sender,
            ri,
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
}
