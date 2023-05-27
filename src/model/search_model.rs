use std::path::PathBuf;
use crossbeam_channel::Sender;
use fluent_integer::Integer;
use crate::data_source::{Direction, FileBackend};
use crate::model::model::ModelEvent;
use crate::search::searcher::{create_searcher, Searcher};
use crate::search::searcher::SearchError::NotFound;


pub struct SearchModel {
    model_sender: Sender<ModelEvent>,
    file_name: Option<PathBuf>,
    searcher: Option<Box<dyn Searcher>>,
    visible: bool,
    pattern: String,
    is_from_cursor: bool,
    cursor_pos: Option<Integer>,
    is_backward: bool,
    is_regexp: bool,
}

impl SearchModel {
    pub fn new(model_sender: Sender<ModelEvent>) -> Self {
        SearchModel {
            model_sender,
            file_name: None,
            searcher: None,
            visible: false,
            pattern: String::new(),
            is_from_cursor: false,
            cursor_pos: None,
            is_backward: false,
            is_regexp: false,
        }
    }

    pub fn set_file_name(&mut self, file_name: String) {
        self.file_name = Some(PathBuf::from(file_name));
        self.update_searcher();
    }

    pub fn set_visible(&mut self, visible: bool) {
        if self.visible != visible {
            self.visible = visible;
            self.emit_event(ModelEvent::SearchOpen(visible));
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_pattern<T: Into<String>>(&mut self, pattern: T) {
        self.pattern = pattern.into();
        self.update_searcher();
    }

    pub fn get_pattern(&self) -> &str {
        self.pattern.as_str()
    }

    pub fn start_search(&mut self) {
        let direction = if self.is_backward {
            Direction::Backward
        } else {
            Direction::Forward
        };
        self.search(direction);
    }

    pub fn search(&mut self, direction: Direction) {
        log::info!("Search: {:?}", self.pattern);
        let result = if let Some(searcher) = &mut self.searcher {
            searcher.next_occurrence(direction)
        } else {
            Err(NotFound)
        };
        self.emit_event(ModelEvent::Search(result));
    }

    pub fn get_current_occurrence(&self) -> Option<Integer> {
        self.searcher.as_ref()
            .map(|searcher| searcher.get_last_occurrence())
            .flatten()
    }

    pub fn is_from_cursor(&self) -> bool {
        self.is_from_cursor
    }

    pub fn set_from_cursor(&mut self, is_from_cursor: bool) {
        if self.is_from_cursor != is_from_cursor {
            self.is_from_cursor = is_from_cursor;
            self.emit_event(ModelEvent::SearchFromCursor);
        }
    }

    pub fn set_cursor(&mut self, cursor: Integer) {
        self.cursor_pos = Some(cursor);
    }

    pub fn is_backward(&self) -> bool {
        self.is_backward
    }

    pub fn set_backward(&mut self, is_backward: bool) {
        self.is_backward = is_backward;
    }

    fn emit_event(&self, evt: ModelEvent) {
        let msg = format!("Failed to send event: {:?}", evt);
        self.model_sender.send(evt)
            .expect(msg.as_str());
    }

    fn update_searcher(&mut self) {
        self.searcher = None;
        if let Some(file_name) = &self.file_name {
            if !self.pattern.is_empty() {
                let backend = FileBackend::new(file_name.clone());
                let offset = if self.is_from_cursor {
                    *&self.cursor_pos.unwrap()
                } else {
                    0.into()
                };
                let searcher = create_searcher(backend, self.pattern.clone(), offset);
                self.searcher = Some(searcher)
            }
        }
    }
}