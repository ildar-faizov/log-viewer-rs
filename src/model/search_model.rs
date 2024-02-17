use std::path::PathBuf;
use anyhow::anyhow;
use crossbeam_channel::Sender;
use fluent_integer::Integer;
use crate::background_process::run_in_background::RunInBackground;
use crate::data_source::Direction;
use crate::model::model::ModelEvent;
use crate::model::navigable_searcher_constructor::NavigableSearcherConstructorBuilder;
use crate::model::search::Search;
use crate::shared::Shared;
use crate::utils::event_emitter::EventEmitter;

pub struct SearchModel<R: RunInBackground> {
    model_sender: Sender<ModelEvent>,
    runner: Shared<R>,
    file_name: Option<PathBuf>,
    visible: bool,
    pattern: String,
    is_from_cursor: bool,
    cursor_pos: Option<Integer>,
    is_backward: bool,
    is_regexp: bool,
}

impl<R: RunInBackground + 'static> SearchModel<R> {
    pub fn new(model_sender: Sender<ModelEvent>, runner: Shared<R>) -> Self {
        SearchModel {
            model_sender,
            runner,
            file_name: None,
            visible: false,
            pattern: String::new(),
            is_from_cursor: false,
            cursor_pos: None,
            is_backward: false,
            is_regexp: false,
        }
    }

    pub fn set_file_name(&mut self, file_name: Option<&str>) {
        self.file_name = file_name.map(PathBuf::from);
    }

    pub fn set_visible(&mut self, visible: bool) {
        if self.visible != visible {
            self.visible = visible;
            let evt = ModelEvent::SearchOpen(visible);
            self.model_sender.emit_event(evt);
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

    pub fn start_search(&mut self) -> anyhow::Result<Search> {
        let background_process_registry = self.runner.clone();
        let background_process_registry = &mut *background_process_registry.get_mut_ref();
        let constructor = NavigableSearcherConstructorBuilder::default()
            .file_name(self.file_name.clone())
            .pattern(self.pattern.clone())
            .is_regexp(self.is_regexp)
            .initial_offset(self.cursor_pos.filter(|_| self.is_from_cursor))
            .is_backward(self.is_backward)
            .build()
            .map_err(|e| anyhow!(e.to_string()))?;
        let direction = Direction::from(!self.is_backward);
        let mut search = Search::new(
            self.model_sender.clone(),
            constructor,
            background_process_registry
        );
        search.search(direction)?;
        Ok(search)
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
            let evt = ModelEvent::SearchFromCursor;
            self.model_sender.emit_event(evt);
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
}