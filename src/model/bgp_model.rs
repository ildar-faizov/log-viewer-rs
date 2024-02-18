use crossbeam_channel::Sender;
use linked_hash_map::LinkedHashMap;
use log::Level;
use uuid::Uuid;
use crate::background_process::background_process_handler::BackgroundProcessHandler;
use crate::background_process::background_process_registry::BackgroundProcessRegistry;
use crate::background_process::run_in_background::RunInBackground;
use crate::background_process::signal::Signal;
use crate::background_process::task_context::TaskContext;
use crate::model::model::{ModelEvent, RootModel};
use crate::shared::Shared;
use crate::utils::event_emitter::EventEmitter;
use crate::utils::measure_l;

pub struct BGPModel {
    sender: Sender<ModelEvent>,
    background_process_registry: Shared<BackgroundProcessRegistry>,
    processes: LinkedHashMap<Uuid, ProcessDescriptor>,
    count: usize,
    overall_progress: u8,
}

#[derive(Debug)]
pub enum BGPModelEvent {
    CountUpdated,
    OverallProgressUpdated,
}

struct ProcessDescriptor {
    progress: u8, // 0-100
    #[allow(dead_code)]
    handler: BackgroundProcessHandler,
}

impl BGPModel {
    pub fn new(
        sender: Sender<ModelEvent>,
        background_process_registry: Shared<BackgroundProcessRegistry>
    ) -> Self {
        Self {
            sender,
            background_process_registry,
            processes: LinkedHashMap::new(),
            count: 0,
            overall_progress: 0,
        }
    }

    pub fn get_number(&self) -> usize {
        self.count
    }

    pub fn get_overall_progress(&self) -> u8 {
        self.overall_progress
    }

    fn update_progress(&mut self, id: &Uuid, p: u8) {
        let Some(pd) = self.processes.get_mut(id)
            else { return; };
        pd.progress = p;
        self.recalculate_overall_progress();
    }

    fn complete(&mut self, id: &Uuid) {
        self.processes.remove(id);
        self.recalculate_overall_progress();
    }

    fn recalculate_overall_progress(&mut self) {
        if self.count != self.processes.len() {
            self.count = self.processes.len();
            self.emit_event(BGPModelEvent::CountUpdated);
        }

        let n = self.processes.len() as u64;
        let p = if n > 0 {
            (self.processes.values().map(|h| h.progress as u64).sum::<u64>().div_ceil(n)) as u8
        } else {
            0
        };
        if self.overall_progress != p {
            self.overall_progress = p;
            self.emit_event(BGPModelEvent::OverallProgressUpdated);
        }
    }

    fn emit_event(&self, evt: BGPModelEvent) {
        self.sender.emit_event(ModelEvent::BGPEvent(evt))
    }
}

impl RunInBackground for BGPModel {
    fn run_in_background<T1, T2, M, T, R, L>(
        &mut self,
        title: T1,
        description: T2,
        task: T,
        mut listener: L
    ) -> BackgroundProcessHandler
        where
            T1: ToString,
            T2: ToString,
            M: Send + 'static,
            R: Send + 'static,
            T: FnOnce(&mut TaskContext<M, R>) -> R,
            T: Send + 'static,
            L: FnMut(&mut RootModel, Signal<M, R>, &Uuid) + 'static
    {
        let descr = description.to_string();
        let listener_wrapper = move |root_model: &mut RootModel, signal: Signal<M, R>, id: &Uuid| {
            match &signal {
                Signal::Custom(_) => {}
                Signal::Progress(p) => {
                    let this = &mut *root_model.get_bgp_model();
                    this.update_progress(id, *p);
                }
                Signal::Complete(_) => {
                    let this = &mut *root_model.get_bgp_model();
                    this.complete(id);
                }
            }
            listener(root_model, signal, id);
        };
        let task_wrapper = move |ctx: &mut TaskContext<M, R>| {
            measure_l(Level::Info, descr.as_str(), move || task(ctx))
        };
        let handle = {
            let registry = &mut *self.background_process_registry.get_mut_ref();
            registry.run_in_background(title, description, task_wrapper, listener_wrapper)
        };
        let result = handle.clone();
        self.processes.insert(*handle.get_id(), ProcessDescriptor::new(handle));
        self.recalculate_overall_progress();
        result
    }
}

impl ProcessDescriptor {
    fn new(handler: BackgroundProcessHandler) -> Self {
        ProcessDescriptor {
            progress: 0,
            handler,
        }
    }
}