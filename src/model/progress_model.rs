use crossbeam_channel::Sender;
use log::Level;
use uuid::Uuid;
use crate::background_process::background_process_registry::BackgroundProcessRegistry;
use crate::background_process::run_in_background::RunInBackground;
use crate::background_process::signal::Signal;
use crate::background_process::task_context::TaskContext;
use crate::model::model::{ModelEvent, RootModel};
use crate::shared::Shared;
use crate::utils::event_emitter::EventEmitter;
use crate::utils::measure_l;

pub struct ProgressModel {
    sender: Sender<ModelEvent>,
    background_process_registry: Shared<BackgroundProcessRegistry>,
    is_open: bool,
    title: String,
    description: String,
    progress: u8, // 0 - 100
}

#[derive(Debug)]
pub enum ProgressModelEvent {
    Toggle,
    TitleUpdated,
    DescriptionUpdated,
    ProgressUpdated,
}

impl ProgressModel {
    pub fn new(sender: Sender<ModelEvent>, background_process_registry: Shared<BackgroundProcessRegistry>) -> Self {
        Self {
            sender,
            background_process_registry,
            is_open: false,
            title: String::new(),
            description: String::new(),
            progress: 0,
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn set_open(&mut self, is_open: bool) {
        if self.is_open != is_open {
            self.is_open = is_open;
            self.emit_event(ProgressModelEvent::Toggle);
        }
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
        self.emit_event(ProgressModelEvent::TitleUpdated);
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn set_description(&mut self, description: String) {
        self.description = description;
        self.emit_event(ProgressModelEvent::DescriptionUpdated);
    }

    pub fn get_progress(&self) -> u8 {
        self.progress
    }

    pub fn set_progress(&mut self, progress: u8) {
        if self.progress != progress {
            self.progress = progress;
            self.emit_event(ProgressModelEvent::ProgressUpdated);
        }
    }

    pub fn run<M, T, R, L, S1, S2>(&mut self, title: S1, description: S2, task: T, mut listener: L)
        where
            M: Send + 'static,
            R: Send + 'static,
            T: FnOnce(&mut TaskContext<M, R>) -> R,
            T: Send + 'static,
            L: FnMut(&mut RootModel, Signal<M, R>, &Uuid) + 'static,
            S1: ToString,
            S2: ToString,
    {
        let description = description.to_string();
        self.set_title(title.to_string());
        self.set_description(description.clone());
        self.set_open(true);

        let listener_wrapper = move |root_model: &mut RootModel, signal: Signal<M, R>, id: &Uuid| {
            match &signal {
                Signal::Custom(_) => {}
                Signal::Progress(p) => {
                    let this = &mut *root_model.get_progress_model();
                    this.set_progress(*p);
                }
                Signal::Complete(_) => {
                    let this = &mut *root_model.get_progress_model();
                    this.set_open(false);
                }
            }
            listener(root_model, signal, id);
        };
        let descr = description.clone();
        let task_wrapper = move |ctx: &mut TaskContext<M, R>| {
            measure_l(Level::Info, descr.as_str(), move || task(ctx))
        };
        let registry = &mut *self.background_process_registry.get_mut_ref();
        let _handle = registry.run_in_background(title.to_string(), description.to_string(), task_wrapper, listener_wrapper);
        // TODO: capture handle and allow to cancel process


    }

}

impl EventEmitter<ProgressModelEvent> for ProgressModel {
    fn emit_event(&self, evt: ProgressModelEvent) {
        self.sender.emit_event(ModelEvent::ProgressEvent(evt));
    }
}