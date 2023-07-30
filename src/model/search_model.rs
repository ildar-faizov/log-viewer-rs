use std::path::PathBuf;
use crossbeam_channel::Sender;
use fluent_integer::Integer;
use crate::data_source::{Direction, FileBackend};
use crate::interval::Interval;
use crate::model::model::ModelEvent;
use crate::search::navigable_searcher::NavigableSearcher;
use crate::search::navigable_searcher_impl::NavigableSearcherImpl;
use crate::search::searcher::{create_searcher, Occurrence, SearchError};
use crate::search::searcher::SearchError::NotFound;

pub struct SearchModel {
    model_sender: Sender<ModelEvent>,
    file_name: Option<PathBuf>,
    searcher: Option<Box<dyn NavigableSearcher>>,
    visible: bool,
    pattern: String,
    is_from_cursor: bool,
    cursor_pos: Option<Integer>,
    is_backward: bool,
    is_regexp: bool,
    // current search:
    occurrences: Option<Vec<Occurrence>>,
    last_occurrence: Option<Occurrence>,
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
            occurrences: None,
            last_occurrence: None,
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
        self.occurrences = None;
        self.last_occurrence = None;
        let direction = if self.is_backward {
            Direction::Backward
        } else {
            Direction::Forward
        };
        self.search(direction);
    }

    pub fn search(&mut self, direction: Direction) {
        log::info!("Search: {:?}", self.pattern);
        let next_occurrence = self.occurrences.as_ref().zip(self.last_occurrence)
            .and_then(|(list, item)| {
                list.iter().position(|x| *x == item).zip(Some(list))
            }).and_then(|(p, list)| {
                match direction {
                    Direction::Forward => list.get(p + 1),
                    Direction::Backward =>
                        if p > 0 {
                            list.get(p - 1)
                        } else {
                            None
                        }
                }
            });
        let result = if let Some(t) = next_occurrence {
            Ok(*t)
        } else if let Some(searcher) = &mut self.searcher {
            searcher.next_occurrence(direction)
        } else {
            Err(NotFound)
        };
        if let Ok(last_occurrence) = &result {
            self.last_occurrence = Some(last_occurrence.clone());
        }
        self.emit_event(ModelEvent::Search(result));
    }

    pub fn get_current_occurrence(&mut self, viewport: Interval<Integer>) -> Result<(Vec<Occurrence>, Option<usize>), SearchError> {
        let result = self.searcher.as_mut().ok_or(NotFound)?.find_all_in_range(viewport)?;
        self.occurrences = Some(result.clone());
        let p = self.last_occurrence.and_then(|last_occurrence|
            result.iter().position(|x| *x == last_occurrence));
        Ok((result, p))
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
                let searcher = create_searcher(backend, self.pattern.clone(), self.is_regexp);
                let mut navigable_searcher = NavigableSearcherImpl::new(searcher);
                if self.is_from_cursor {
                    navigable_searcher.set_initial_offset(*&self.cursor_pos.unwrap())
                }
                self.searcher = Some(Box::new(navigable_searcher));
            }
        }
    }
}