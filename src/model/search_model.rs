use std::path::PathBuf;
use crossbeam_channel::Sender;
use fluent_integer::Integer;
use crate::data_source::{Direction, FileBackend};
use crate::model::model::ModelEvent;
use crate::model::search::Search;
use crate::search::navigable_searcher::NavigableSearcher;
use crate::search::navigable_searcher_impl::NavigableSearcherImpl;
use crate::search::searcher::create_searcher;

pub struct SearchModel {
    model_sender: Sender<ModelEvent>,
    file_name: Option<PathBuf>,
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
    }

    pub fn get_pattern(&self) -> &str {
        self.pattern.as_str()
    }

    pub fn start_search(&mut self) -> Result<Search, SearchModelError> {
        self.evaluate_searcher().map(|s| {
            let direction = Direction::from(!self.is_backward);
            let mut search = Search::new(self.model_sender.clone(), s);
            search.search(direction);
            search
        })
    }

    pub fn is_from_cursor(&self) -> bool {
        self.is_from_cursor
    }

    pub fn set_from_cursor(&mut self, is_from_cursor: bool) {
        if self.is_from_cursor != is_from_cursor {
            self.is_from_cursor = is_from_cursor;
            if !is_from_cursor {
                self.cursor_pos = None;
            }
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

    pub fn is_regexp(&self) -> bool {
        self.is_regexp
    }

    pub fn set_regexp(&mut self, is_regexp: bool) {
        self.is_regexp = is_regexp;
    }

    fn emit_event(&self, evt: ModelEvent) {
        let msg = format!("Failed to send event: {:?}", evt);
        self.model_sender.send(evt)
            .expect(msg.as_str());
    }

    fn evaluate_searcher(&mut self) -> Result<Box<dyn NavigableSearcher>, SearchModelError> {
        if let Some(file_name) = &self.file_name {
            if !self.pattern.is_empty() {
                let backend = FileBackend::new(file_name.clone());
                let searcher = create_searcher(backend, self.pattern.clone(), self.is_regexp);
                let mut navigable_searcher = NavigableSearcherImpl::new(searcher);
                if self.is_from_cursor {
                    let direction = Direction::from(!self.is_backward);
                    navigable_searcher.set_initial_offset(*&self.cursor_pos.unwrap(), direction);
                }
                log::info!("Search: {:?}", self.pattern);
                return Ok(Box::new(navigable_searcher));
            } else {
                Err(SearchModelError::PatternIsEmpty)
            }
        } else {
            Err(SearchModelError::FileNotSet)
        }
    }
}

pub enum SearchModelError {
    FileNotSet,
    PatternIsEmpty,
}

impl ToString for SearchModelError {
    fn to_string(&self) -> String {
        let str = match self {
            SearchModelError::PatternIsEmpty => "Pattern is empty",
            SearchModelError::FileNotSet => "File (data source) not specified",
        };
        str.to_string()
    }
}