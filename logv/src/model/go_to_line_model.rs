use crate::background_process::run_in_background::RunInBackground;
use crate::data_source::line_registry::{LineRegistry, LineRegistryImpl};
use crate::data_source::reader_factory::ReaderFactory;
use crate::data_source::BUFFER_SIZE;
use crate::model::abstract_go_to_model::{AbstractGoToModel, GoToError};
use crate::model::escape_handler::{CompoundEscapeHandler, EscapeHandlerResult};
use crate::model::model::{ModelEvent, RootModel};
use crate::shared::Shared;
use anyhow::bail;
use crossbeam_channel::Sender;
use fluent_integer::Integer;
use std::io::{BufReader, Read};
use std::sync::Arc;

pub struct GoToLineModel<R: RunInBackground> {
    go_to_model: AbstractGoToModel<R>,
    value: String,
    line_registry: Option<Arc<LineRegistryImpl>>,
    warning: Option<String>,
}

impl<R: RunInBackground> GoToLineModel<R> {
    pub fn new(
        model_sender: Sender<ModelEvent>,
        background_process_registry: Shared<R>,
        escape_handler: Shared<CompoundEscapeHandler>,
    ) -> Self {
        let go_to_model = AbstractGoToModel::new(
            model_sender,
            background_process_registry,
            Box::new(ModelEvent::GoToOpen),
            escape_handler,
            Self::on_esc,
        );
        GoToLineModel {
            go_to_model,
            value: String::new(),
            line_registry: None,
            warning: None,
        }
    }

    pub fn set_is_open(&mut self, is_open: bool) {
        self.go_to_model.set_is_open(is_open)
    }

    pub fn is_open(&self) -> bool {
        self.go_to_model.is_open()
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, value: &str) {
        self.value = value.to_string()
    }

    pub fn set_line_registry(&mut self, line_registry: Option<Arc<LineRegistryImpl>>) {
        self.line_registry = line_registry;
    }

    pub fn set_warning(&mut self, warning: Option<impl ToString>) {
        self.warning = warning.map(|arg| arg.to_string());
    }

    pub fn get_warning(&self) -> Option<&str> {
        self.warning.as_ref().map(String::as_str)
    }

    pub fn submit(
        &mut self,
        reader_factory: Box<dyn ReaderFactory>,
        total: Option<Integer>
    ) -> Result<(), anyhow::Error> {
        let line = self.value.parse::<u64>()?;
        if line < 1 {
            bail!("Line number must not be less than 1")
        }

        if let Some(line_registry) = &self.line_registry {
            let offset = line_registry.find_offset_by_line_number(line - 1);
            if let Ok(offset) = offset {
                self.set_is_open(false);
                self.go_to_model.complete(Ok(offset))?;
                return Ok(());
            }
        }
        self.go_to_model.submit(
            |root_model, pid, msg| {
                let m = &mut root_model.get_go_to_line_model().go_to_model;
                m.handle_result(pid, msg)
            },
            move |ctx| {
                let mut progress = 0_u8;
                let mut reader = BufReader::new(reader_factory.new_reader()?);
                let mut offset = 0;
                let mut buf = [0_u8; BUFFER_SIZE];
                let mut line = line - 1;
                loop {
                    if line == 0 {
                        ctx.update_progress(100);
                        return Ok(offset.into());
                    }
                    if ctx.interrupted() {
                        return Err(GoToError::Cancelled);
                    }
                    let bytes_read = reader.read(&mut buf)?;
                    if bytes_read == 0 {
                        return Err(GoToError::NotReachable);
                    }
                    for ch in &buf[0..bytes_read] {
                        if *ch == b'\n' {
                            line -= 1;
                        }
                        offset += 1;

                        if line == 0 {
                            ctx.update_progress(100);
                            return Ok(offset.into());
                        }
                    }

                    let new_progress = match total {
                        None => 50_u8,
                        Some(total) => (offset * 100 / total.as_u64()) as u8,
                    };
                    if new_progress > progress {
                        ctx.update_progress(new_progress);
                        progress = new_progress;
                    }
                }
            }
        );
        Ok(())
    }

    fn on_esc(root_model: &mut RootModel) -> EscapeHandlerResult {
        let me = &mut *root_model.get_go_to_line_model();
        if me.is_open() {
            me.set_is_open(false);
            EscapeHandlerResult::Dismiss
        } else {
            EscapeHandlerResult::Ignore
        }
    }
}
