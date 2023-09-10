use crate::background_process::background_process_handler::BackgroundProcessHandler;
use crate::background_process::background_process_registry::BackgroundProcessRegistry;
use crate::background_process::run_in_background::RunInBackground;
use crate::data_source::BUFFER_SIZE;
use crate::model::model::ModelEvent;
use crate::search::searcher::Occurrence;
use crate::shared::Shared;
use crate::utils::event_emitter::EventEmitter;
use anyhow::anyhow;
use crossbeam_channel::Sender;
use fluent_integer::Integer;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use uuid::Uuid;

pub struct GoToLineModel {
    model_sender: Sender<ModelEvent>,
    background_process_registry: Shared<BackgroundProcessRegistry>,
    current_process: Option<BackgroundProcessHandler>,
    is_open: bool,
    value: String,
}

impl GoToLineModel {
    pub fn new(
        model_sender: Sender<ModelEvent>,
        background_process_registry: Shared<BackgroundProcessRegistry>,
    ) -> Self {
        GoToLineModel {
            model_sender,
            background_process_registry,
            current_process: None,
            is_open: false,
            value: String::new(),
        }
    }

    pub fn set_is_open(&mut self, is_open: bool) {
        if self.is_open != is_open {
            self.is_open = is_open;
            self.model_sender
                .emit_event(ModelEvent::GoToOpen(self.is_open));
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, value: &str) {
        self.value = value.to_string()
    }

    pub fn submit(&mut self, file_name: &str) -> Result<(), anyhow::Error> {
        let line = self.value.parse::<u64>()?;
        if line < 1 {
            return Err(anyhow!("Line number must not be less than 1"));
        }
        self.set_is_open(false);
        self.go_to_line(line - 1, PathBuf::from(file_name.to_string()));
        Ok(())
    }

    pub fn go_to_line(&mut self, mut line: u64, file: PathBuf) {
        let registry = &mut *self.background_process_registry.get_mut_ref();
        let handler = registry
            .background_process_builder::<(), _, Result<Integer, GoToLineError>, _>()
            .with_task(move |ctx| {
                let total = std::fs::metadata(file.as_path())
                    .map(|m| m.len())
                    .unwrap_or(u64::MAX);
                let mut progress = 0_u8;
                let mut reader = BufReader::new(File::open(file)?);
                let mut offset = 0;
                let mut buf = [0_u8; BUFFER_SIZE];
                loop {
                    if line == 0 {
                        return Ok(offset.into());
                    }
                    if ctx.interrupted() {
                        return Err(GoToLineError::Cancelled);
                    }
                    let bytes_read = reader.read(&mut buf)?;
                    if bytes_read == 0 {
                        return Err(GoToLineError::NotReachable);
                    }
                    for ch in &buf[0..bytes_read] {
                        if ctx.interrupted() {
                            return Err(GoToLineError::Cancelled);
                        }

                        if *ch == '\n' as u8 {
                            line -= 1;
                        }
                        offset += 1;

                        let new_progress = (offset * 100 / total) as u8;
                        if new_progress > progress {
                            ctx.update_progress(new_progress);
                            progress = new_progress;
                        }

                        if line == 0 {
                            return Ok(offset.into());
                        }
                    }
                }
            })
            .with_listener(move |model, msg, id| {
                let handle_result = {
                    let m = &mut *model.get_go_to_line_model();
                    m.handle_result(id.clone(), msg)
                };
                if let Err(err) = handle_result {
                    model.set_error(Box::new(err))
                }
            })
            .run();
        if let Some(previous) = self.current_process.replace(handler) {
            log::info!(
                "Interrupting GoToLine process {} due to new request",
                previous.get_id()
            );
            previous.interrupt();
        }
    }

    fn handle_result(
        &mut self,
        pid: Uuid,
        msg: Result<Result<Integer, GoToLineError>, ()>,
    ) -> Result<(), GoToLineError> {
        let h = self.current_process.take();
        if let Some(h) = h {
            if *h.get_id() == pid {
                match msg {
                    Ok(Ok(p)) => self
                        .model_sender
                        .emit_event(ModelEvent::Search(Ok(Occurrence::new(p, p)))),
                    Ok(Err(GoToLineError::Cancelled)) => {
                        log::info!("{:?}", GoToLineError::Cancelled)
                    }
                    Ok(Err(err)) => {
                        return Err(err);
                    }
                    Err(_) => (), // TODO: show progress
                };
            } else {
                log::trace!("Result of GoToLine {} has been ignored, because different process {} is started", pid, h.get_id());
            }
        }
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GoToLineError {
    #[error("IO error: {0:?}")]
    IO(#[from] std::io::Error),
    #[error("Line not found, EOF reached")]
    NotReachable,
    #[error("Operation is cancelled")]
    Cancelled,
}
