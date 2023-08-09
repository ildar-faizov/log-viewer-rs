use crate::background_process::background_process_handler::BackgroundProcessHandler;
use crate::background_process::run_in_background::RunInBackground;
use crate::background_process::signal::Signal;
use crate::background_process::task_context::TaskContext;
use crate::model::model::RootModel;
use crate::shared::Shared;
use crossbeam_channel::Receiver;
use std::any::Any;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Default)]
pub struct BackgroundProcessRegistry {
    processes: HashMap<Uuid, Box<dyn HandleSignals>>,
}

trait HandleSignals: Any {
    fn handle_signals(&mut self, root_model: Shared<RootModel>) -> bool;
}

impl BackgroundProcessRegistry {
    pub fn new() -> Self {
        BackgroundProcessRegistry::default()
    }

    pub fn handle_events_from_background(&mut self, root_model: Shared<RootModel>) {
        let mut finished_ids = vec![];
        for (id, b) in self.processes.iter_mut() {
            let finished = b.handle_signals(root_model.clone());
            if finished {
                finished_ids.push(id.clone());
            }
        }
        for id in finished_ids {
            self.unregister(&id);
        }
    }

    fn register<M, R, L>(&mut self, id: Uuid, bgp: BackgroundProcessData<M, R, L>)
    where
        M: Send + 'static,
        R: Send + 'static,
        L: FnMut(&mut RootModel, Result<R, M>) + 'static,
    {
        self.processes.insert(id, Box::new(bgp));
    }

    fn unregister(&mut self, id: &Uuid) {
        self.processes.remove(id);
    }
}

impl RunInBackground for BackgroundProcessRegistry {
    fn run_in_background<M, T, R, L>(&mut self, task: T, listener: L) -> BackgroundProcessHandler
    where
        M: Send + 'static,
        R: Send + 'static,
        T: FnOnce(&TaskContext<M, R>) -> R,
        T: Send + 'static,
        L: FnMut(&mut RootModel, Result<R, M>) + 'static,
    {
        let id = Uuid::new_v4();
        let (sender, receiver) = crossbeam_channel::unbounded();
        let (sender_interrupt, receiver_interrupt) = crossbeam_channel::unbounded();
        let bgd = BackgroundProcessData::new(receiver, listener);
        self.register(id, bgd);

        std::thread::spawn(move || {
            let task_context = TaskContext::new(sender.clone(), receiver_interrupt);
            let result = task(&task_context);
            sender
                .send(Signal::Complete(result))
                .expect("Failed to send result");
        });

        BackgroundProcessHandler::new(sender_interrupt)
    }
}

pub struct BackgroundProcessData<M, R, L>
where
    M: Send + 'static,
    R: Send + 'static,
    L: FnMut(&mut RootModel, Result<R, M>) + 'static,
{
    receiver: Receiver<Signal<M, R>>,
    listener: L,
}

impl<M, R, L> BackgroundProcessData<M, R, L>
where
    M: Send + 'static,
    R: Send + 'static,
    L: FnMut(&mut RootModel, Result<R, M>) + 'static,
{
    pub fn new(receiver: Receiver<Signal<M, R>>, listener: L) -> Self {
        BackgroundProcessData { receiver, listener }
    }
}

impl<M, R, L> HandleSignals for BackgroundProcessData<M, R, L>
where
    M: Send + 'static,
    R: Send + 'static,
    L: FnMut(&mut RootModel, Result<R, M>) + 'static,
{
    fn handle_signals(&mut self, root_model: Shared<RootModel>) -> bool {
        let listener = &mut self.listener;
        for signal in self.receiver.try_iter() {
            match signal {
                Signal::Progress(p) => (),
                Signal::Custom(msg) => listener(&mut *root_model.get_mut_ref(), Err(msg)),
                Signal::Complete(result) => {
                    listener(&mut *root_model.get_mut_ref(), Ok(result));
                    return true;
                }
            }
        }
        false
    }
}
