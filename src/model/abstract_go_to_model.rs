use crossbeam_channel::Sender;
use uuid::Uuid;
use fluent_integer::Integer;
use crate::background_process::background_process_handler::BackgroundProcessHandler;
use crate::background_process::background_process_registry::BackgroundProcessRegistry;
use crate::background_process::run_in_background::RunInBackground;
use crate::background_process::signal::Signal;
use crate::background_process::task_context::TaskContext;
use crate::model::model::{ModelEvent, RootModel};
use crate::search::searcher::Occurrence;
use crate::shared::Shared;
use crate::utils::event_emitter::EventEmitter;

pub struct AbstractGoToModel
{
    model_sender: Sender<ModelEvent>,
    background_process_registry: Shared<BackgroundProcessRegistry>,
    current_process: Option<BackgroundProcessHandler>,
    is_open: bool,
    open_event_producer: Box<dyn Fn(bool) -> ModelEvent>,
}

impl AbstractGoToModel
{
    pub fn new(
        model_sender: Sender<ModelEvent>,
        background_process_registry: Shared<BackgroundProcessRegistry>,
        open_event_producer: Box<dyn Fn(bool) -> ModelEvent>,
    ) -> Self {
        AbstractGoToModel {
            model_sender,
            background_process_registry,
            current_process: None,
            is_open: false,
            open_event_producer,
        }
    }

    pub fn set_is_open(&mut self, is_open: bool) {
        if self.is_open != is_open {
            self.is_open = is_open;
            let evt = (*self.open_event_producer)(self.is_open);
            self.model_sender.emit_event(evt);
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn submit<F, G>(&mut self, result_handler: G, task: F)
        where
            F: FnOnce(&mut TaskContext<(), GoToResult>) -> GoToResult,
            F: Send + 'static,
            G: Fn(&mut RootModel, Uuid, Signal<(), GoToResult>) -> Result<(), GoToError>,
            G: Send + 'static,
    {
        self.set_is_open(false);

        let registry = &mut *self.background_process_registry.get_mut_ref();
        let handler = registry
            .background_process_builder::<(), _, Result<Integer, GoToError>, _>()
            .with_task(task)
            .with_listener(move |model, msg, id| {
                let handle_result = result_handler(&mut *model, *id, msg);
                if let Err(err) = handle_result {
                    model.set_error(Box::new(err))
                }
            })
            .run();
        if let Some(previous) = self.current_process.replace(handler) {
            log::info!(
                "Interrupting GoTo process {} due to new request",
                previous.get_id()
            );
            previous.interrupt();
        }
    }

    pub fn handle_result(
        &mut self,
        pid: Uuid,
        msg: Signal<(), GoToResult>,
    ) -> Result<(), GoToError> {
        let h = self.current_process.take();
        if let Some(h) = h {
            if *h.get_id() == pid {
                match msg {
                    Signal::Complete(Ok(p)) => self
                        .model_sender
                        .emit_event(ModelEvent::Search(Ok(Occurrence::new(p, p)))),
                    Signal::Complete(Err(GoToError::Cancelled)) => {
                        log::info!("{:?}", GoToError::Cancelled)
                    }
                    Signal::Complete(Err(err)) => {
                        return Err(err);
                    }
                    _ => (), // TODO: show progress
                };
            } else {
                log::trace!("Result of GoToLine {} has been ignored, because different process {} is started", pid, h.get_id());
            }
        }
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GoToError {
    #[error("IO error: {0:?}")]
    IO(#[from] std::io::Error),
    #[error("Line not found, EOF reached")]
    NotReachable,
    #[error("Operation is cancelled")]
    Cancelled,
}

pub type GoToResult = Result<Integer, GoToError>;