use std::cmp::Ordering;
use std::fmt::Display;
use std::fs::{DirEntry, Metadata};
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use anyhow::anyhow;
use crossbeam_channel::Sender;
use itertools::Itertools;
use crate::model::model::ModelEvent;
use crate::utils::event_emitter::EventEmitter;

const GO_UP: &str = "..";

pub struct OpenFileModel {
    sender: Sender<ModelEvent>,
    is_open: bool,
    current_location: PathBuf,
    files: Vec<DirEntry0>,
    current_file: Option<String>,
    entry_info: Option<EntryInfo>,
}

#[derive(Debug)]
pub enum OpenFileModelEvent {
    LocationUpdated,
    FilesUpdated,
    EntryInfoUpdated,
    Error(anyhow::Error),
}

#[derive(PartialEq, Eq, Clone)]
pub enum DirEntry0 {
    Up, // ..
    Folder(String),
    File(String),
}

#[derive(Debug, Clone)]
pub struct EntryInfo {
    pub size: u64,
    pub created_at: Option<SystemTime>,
    pub modified_at: Option<SystemTime>,
    // todo other properties
}

impl OpenFileModel {
    pub fn new(sender: Sender<ModelEvent>) -> Self {
        OpenFileModel {
            sender,
            is_open: false,
            current_location: PathBuf::new(),
            files: vec![],
            current_file: None,
            entry_info: None,
        }
    }

    pub fn set_open(&mut self, is_open: bool) {
        if self.is_open != is_open {
            self.is_open = is_open;
            self.sender.emit_event(ModelEvent::OpenFileDialog(is_open));
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn get_current_location(&self) -> &Path {
        self.current_location.as_path()
    }

    pub fn set_current_location(&mut self, current_location: PathBuf) {
        std::fs::canonicalize(current_location)
            .map(|loc| self.set_canonical_current_location(loc))
            .unwrap_or_else(|e| log::error!("{:?}", e));
    }

    pub fn get_files(&self) -> &[DirEntry0] {
        &self.files[..]
    }

    pub fn set_current_file(&mut self, current_file: Option<&str>) {
        self.current_file = current_file.map(|s| String::from(s));
        self.update_entry_info();
    }

    pub fn get_entry_info(&self) -> Option<&EntryInfo> {
        self.entry_info.as_ref()
    }

    pub fn submit_current_file(&mut self) {
        let current_file = self.current_file.clone();
        match &current_file {
            Some(file_name) => {
                if file_name == GO_UP {
                    self.step_up();
                } else {
                    let mut target = self.current_location.clone();
                    target.push(file_name);
                    match std::fs::metadata(&target) {
                        Ok(metadata) => {
                            if metadata.is_dir() {
                                self.set_current_location(target);
                            } else {
                                self.set_open(false);
                                self.sender.emit_event(ModelEvent::OpenFile(target.to_str().unwrap().to_string()));
                            }
                        }
                        Err(err) => self.emit_event(OpenFileModelEvent::Error(anyhow::Error::from(err)))
                    }
                }
            },
            None => {
                self.sender.emit_event(ModelEvent::Error(Some("No file chosen".to_string())));
            }
        }

    }

    fn set_canonical_current_location(&mut self, current_location: PathBuf) {
        if self.current_location == current_location {
            return;
        }
        match Self::list_files(&current_location) {
            Ok(files) => {
                self.current_location = current_location;
                self.set_current_file(None);
                self.emit_event(OpenFileModelEvent::LocationUpdated);
                self.files = files;
                self.emit_event(OpenFileModelEvent::FilesUpdated);
            }
            Err(err) =>
                self.emit_event(OpenFileModelEvent::Error(anyhow!(err))),
        }
    }

    fn step_up(&mut self) {
        match self.current_location.parent() {
            Some(p) => {
                self.set_current_location(p.to_path_buf());
            },
            None => {
                log::warn!("Cannot perform step up tree. Already in parent");
            }
        }
    }

    fn update_entry_info(&mut self) {
        self.entry_info = self.path_to_file()
            .map(std::fs::metadata)
            .and_then(Result::ok)
            .map(|metadata| EntryInfo::from(&metadata));
        self.emit_event(OpenFileModelEvent::EntryInfoUpdated);
    }

    fn path_to_file(&self) -> Option<PathBuf> {
        self.current_file.as_ref()
            .map(|file_name| {
                let mut p = self.current_location.clone();
                p.push(file_name);
                p
            })
    }

    fn emit_event(&self, evt: OpenFileModelEvent) {
        self.sender.emit_event(ModelEvent::OpenFileModelEventWrapper(evt))
    }

    fn list_files(location: &PathBuf) -> io::Result<Vec<DirEntry0>> {
        std::fs::read_dir(&location).map(|read_dir| {
            let mut files: Vec<DirEntry0> = vec![];
            if location.as_path().parent().is_some() {
                files.push(DirEntry0::Up);
            }
            read_dir
                .filter_map(Result::ok)
                .filter_map(|e| DirEntry0::try_from(&e).ok())
                .sorted()
                .for_each(|item| files.push(item));

            files
        })
    }
}

impl TryFrom<&DirEntry> for DirEntry0 {
    type Error = anyhow::Error;

    fn try_from(value: &DirEntry) -> Result<Self, Self::Error> {
        let file_type = value.file_type()?;
        let name = value.file_name().to_str().ok_or(anyhow!("Failed to convert name"))?.to_string();
        if file_type.is_dir() {
            Ok(DirEntry0::Folder(name))
        } else if file_type.is_file() {
            Ok(DirEntry0::File(name))
        } else {
            // todo: handle other types, probably links
            Err(anyhow!("{}: Unsupported file type of {:?}", name, file_type))
        }
    }
}


impl PartialOrd<Self> for DirEntry0 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DirEntry0 {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            DirEntry0::Up => {
                match other {
                    DirEntry0::Up => Ordering::Equal,
                    _ => Ordering::Less,
                }
            }
            DirEntry0::Folder(a) => match other {
                DirEntry0::Up => Ordering::Greater,
                DirEntry0::Folder(b) => a.cmp(b),
                DirEntry0::File(_) => Ordering::Less,
            }
            DirEntry0::File(a) => match other {
                DirEntry0::File(b) => a.cmp(b),
                _ => Ordering::Greater,
            }
        }
    }
}

impl Display for DirEntry0 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            DirEntry0::Up => String::from(GO_UP),
            DirEntry0::Folder(name) => name.clone(),
            DirEntry0::File(name) => name.clone(),
        };
        write!(f, "{}", str)
    }
}

impl From<&Metadata> for EntryInfo {
    fn from(value: &Metadata) -> Self {
        Self {
            size: value.len(),
            created_at: value.created().ok(),
            modified_at: value.modified().ok(),
        }
    }
}