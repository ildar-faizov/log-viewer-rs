use crate::background_process::background_process_handler::BackgroundProcessHandler;
use crate::background_process::run_in_background::RunInBackground;
use crate::background_process::signal::Signal;
use crate::background_process::task_context::TaskContext;
use crate::model::model::{ModelEvent, RootModel};
use crate::search::searcher::Occurrence;
use crate::shared::Shared;
use crate::utils::event_emitter::EventEmitter;
use crossbeam_channel::Sender;
use fluent_integer::Integer;
use uuid::Uuid;
use crate::model::escape_handler::{CompoundEscapeHandler, EscapeHandlerManager, EscapeHandlerResult};

pub struct AbstractGoToModel<R: RunInBackground>
{
    model_sender: Sender<ModelEvent>,
    background_process_registry: Shared<R>,
    current_process: Option<BackgroundProcessHandler>,
    is_open: bool,
    open_event_producer: Box<dyn Fn(bool) -> ModelEvent>,
    escape_handler_manager: EscapeHandlerManager,
}

impl<R: RunInBackground> AbstractGoToModel<R>
{
    pub fn new(
        model_sender: Sender<ModelEvent>,
        background_process_registry: Shared<R>,
        open_event_producer: Box<dyn Fn(bool) -> ModelEvent>,
        escape_handler: Shared<CompoundEscapeHandler>,
        on_esc: fn(&mut RootModel) -> EscapeHandlerResult,
    ) -> Self {
        AbstractGoToModel {
            model_sender,
            background_process_registry,
            current_process: None,
            is_open: false,
            open_event_producer,
            escape_handler_manager: EscapeHandlerManager::new(escape_handler, on_esc),
        }
    }

    pub fn set_is_open(&mut self, is_open: bool) {
        if self.is_open != is_open {
            self.is_open = is_open;
            self.escape_handler_manager.toggle(is_open);
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
            G: Fn(&mut RootModel, Uuid, GoToResult) -> Result<(), GoToError>,
            G: Send + 'static,
    {
        self.set_is_open(false);

        let registry = &mut *self.background_process_registry.get_mut_ref();
        let handler = registry
            .background_process_builder::<(), _, Result<Integer, GoToError>, _>()
            .with_title("Go to")
            .with_description("Go to")
            .with_task(task)
            .with_listener(move |model, msg, id| {
                if let Signal::Complete(r) = msg {
                    let handle_result = result_handler(&mut *model, *id, r);
                    if let Err(err) = handle_result {
                        model.set_error(Box::new(err))
                    }
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
        result: GoToResult,
    ) -> Result<(), GoToError> {
        let h = self.current_process.take();
        if let Some(h) = h {
            if *h.get_id() == pid {
                return self.complete(result);
            } else {
                log::trace!("Result of GoToLine {} has been ignored, because different process {} is started", pid, h.get_id());
            }
        }
        Ok(())
    }

    pub fn complete(&self, result: Result<Integer, GoToError>) -> Result<(), GoToError> {
        match result {
            Ok(p) => {
                self.model_sender.emit_event(ModelEvent::Search(Ok(Occurrence::new(p, p))));
                Ok(())
            },
            Err(GoToError::Cancelled) => {
                log::info!("{:?}", GoToError::Cancelled);
                Ok(())
            },
            err => err.map(|_| ()),
        }
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
    #[error("Total source length is unknown")]
    #[allow(dead_code)]
    LengthUnknown,
}

pub type GoToResult = Result<Integer, GoToError>;