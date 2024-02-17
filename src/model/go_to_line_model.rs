use crate::data_source::BUFFER_SIZE;
use crate::model::model::{ModelEvent, RootModel};
use crate::shared::Shared;
use anyhow::anyhow;
use crossbeam_channel::Sender;
use fluent_integer::Integer;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;
use crate::background_process::run_in_background::RunInBackground;
use crate::background_process::signal::Signal;
use crate::model::abstract_go_to_model::{AbstractGoToModel, GoToError};

pub struct GoToLineModel<R: RunInBackground> {
    go_to_model: AbstractGoToModel<R>,
    value: String,
}

impl<R: RunInBackground> GoToLineModel<R> {
    pub fn new(
        model_sender: Sender<ModelEvent>,
        background_process_registry: Shared<R>,
    ) -> Self {
        let go_to_model = AbstractGoToModel::new(
            model_sender,
            background_process_registry,
            Box::new(ModelEvent::GoToOpen),
        );
        GoToLineModel {
            go_to_model,
            value: String::new(),
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

    pub fn submit(&mut self, file_name: &str) -> Result<(), anyhow::Error> {
        let line = self.value.parse::<u64>()?;
        if line < 1 {
            return Err(anyhow!("Line number must not be less than 1"));
        }
        let file = PathBuf::from_str(file_name)?;
        self.go_to_model.submit(
            |root_model: &mut RootModel, pid: Uuid, msg: Signal<(), Result<Integer, GoToError>>| {
                let m = &mut root_model.get_go_to_line_model().go_to_model;
                m.handle_result(pid, msg)
            },
            move |ctx| {
                let total = std::fs::metadata(file.as_path())
                    .map(|m| m.len())
                    .unwrap_or(u64::MAX);
                let mut progress = 0_u8;
                let mut reader = BufReader::new(File::open(file)?);
                let mut offset = 0;
                let mut buf = [0_u8; BUFFER_SIZE];
                let mut line = line - 1;
                loop {
                    if line == 0 {
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
                        if ctx.interrupted() {
                            return Err(GoToError::Cancelled);
                        }

                        if *ch == b'\n' {
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

            }
        );
        Ok(())
    }
}
