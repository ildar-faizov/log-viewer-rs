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
    fn handle_signals(&mut self, root_model: Shared<RootModel>, id: &Uuid) -> bool;
}

impl BackgroundProcessRegistry {
    pub fn new() -> Self {
        BackgroundProcessRegistry::default()
    }

    pub fn handle_events_from_background(&mut self, root_model: Shared<RootModel>) {
        let mut finished_ids = vec![];
        for (id, b) in self.processes.iter_mut() {
            let finished = b.handle_signals(root_model.clone(), id);
            if finished {
                finished_ids.push(*id);
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
        L: FnMut(&mut RootModel, Signal<M, R>, &Uuid) + 'static,
    {
        self.processes.insert(id, Box::new(bgp));
    }

    fn unregister(&mut self, id: &Uuid) {
        self.processes.remove(id);
    }
}

impl RunInBackground for BackgroundProcessRegistry {
    fn run_in_background<T1, T2, M, T, R, L>(
        &mut self,
        title: T1,
        description: T2,
        task: T,
        listener: L
    ) -> BackgroundProcessHandler
    where
        T1: ToString,
        T2: ToString,
        M: Send + 'static,
        R: Send + 'static,
        T: FnOnce(&mut TaskContext<M, R>) -> R,
        T: Send + 'static,
        L: FnMut(&mut RootModel, Signal<M, R>, &Uuid) + 'static,
    {
        let id = Uuid::new_v4();
        let (sender, receiver) = crossbeam_channel::unbounded();
        let (sender_interrupt, receiver_interrupt) = crossbeam_channel::unbounded();
        let bgd = BackgroundProcessData::new(receiver, listener);
        self.register(id, bgd);

        std::thread::spawn(move || {
            let mut task_context = TaskContext::new(sender.clone(), receiver_interrupt, id);
            let result = task(&mut task_context);
            sender
                .send(Signal::Complete(result))
                .expect("Failed to send result");
        });

        BackgroundProcessHandler::new(sender_interrupt, id, title.to_string(), description.to_string())
    }
}

pub struct BackgroundProcessData<M, R, L>
where
    M: Send + 'static,
    R: Send + 'static,
    L: FnMut(&mut RootModel, Signal<M, R>, &Uuid) + 'static,
{
    receiver: Receiver<Signal<M, R>>,
    listener: L,
}

impl<M, R, L> BackgroundProcessData<M, R, L>
where
    M: Send + 'static,
    R: Send + 'static,
    L: FnMut(&mut RootModel, Signal<M, R>, &Uuid) + 'static,
{
    pub fn new(receiver: Receiver<Signal<M, R>>, listener: L) -> Self {
        BackgroundProcessData { receiver, listener }
    }
}

impl<M, R, L> HandleSignals for BackgroundProcessData<M, R, L>
where
    M: Send + 'static,
    R: Send + 'static,
    L: FnMut(&mut RootModel, Signal<M, R>, &Uuid) + 'static,
{
    fn handle_signals(&mut self, root_model: Shared<RootModel>, id: &Uuid) -> bool {
        let listener = &mut self.listener;
        for signal in self.receiver.try_iter() {
            let complete = is_complete(&signal);
            listener(&mut root_model.get_mut_ref(), signal, id);
            if complete {
                return true;
            }
        }
        false
    }
}

fn is_complete<M, R>(signal: &Signal<M, R>) -> bool
    where
        M: Send + 'static,
        R: Send + 'static,
{
    match signal {
        Signal::Custom(_) => false,
        Signal::Progress(_) => false,
        Signal::Complete(_) => true,
    }
}