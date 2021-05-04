use crossbeam_channel::Sender;
use std::path::{Path, PathBuf};
use std::env::current_dir;
use std::fs::read_to_string;
use ModelEvent::*;

pub struct RootModel {
    model_sender: Sender<ModelEvent>,
    file_name: Option<String>,
    file_content: Option<String>,
    error: Option<Box<dyn ToString>>,
}

pub enum ModelEvent {
    FileName(String),
    FileContent,
    Error(String),
}

impl RootModel {
    pub fn new(model_sender: Sender<ModelEvent>) -> Self {
        RootModel {
            model_sender,
            file_name: None,
            file_content: None,
            error: None
        }
    }

    fn emit_event(&self, event: ModelEvent) {
        self.model_sender.send(event).unwrap();
    }

    pub fn file_name(&self) -> Option<&str> {
        self.file_name.as_ref().map(|s|&s[..])
    }

    pub fn set_file_name(&mut self, value: String) {
        // TODO: check equality
        // if self.file_name != value {
        self.file_name = Some(value);
        self.emit_event(FileName(self.file_name.as_ref().unwrap().to_owned()));
        self.load_file();
        // }
    }

    pub fn file_content(&self) -> Option<&str> {
        self.file_content.as_ref().map(|s| &s[..])
    }

    fn set_file_content(&mut self, content: String) {
        self.file_content = Some(content);
        self.emit_event(FileContent);
    }

    pub fn error(&self) -> Option<String> {
        self.error.as_ref().map(|t| t.to_string())
    }

    fn set_error(&mut self, err: Box<dyn ToString>) {
        self.error.replace(err);
        self.emit_event(Error(self.error.as_ref().unwrap().to_string()));
    }

    fn load_file(&mut self) {
        if let Some(path) = self.resolve_file_name() {
            let content = read_to_string(path);
            match content {
                Ok(content) => self.set_file_content(content),
                Err(err) => self.set_error(Box::new(err))
            }
        }
    }

    fn resolve_file_name(&self) -> Option<PathBuf> {
        self.file_name.as_ref().map(|fname| {
            let p = Path::new(fname);
            if !p.is_absolute() {
                let mut buf = current_dir().unwrap();
                buf.push(p);
                buf
            } else {
                let mut buf = PathBuf::new();
                buf.push(p);
                buf
            }
        })
    }
}
