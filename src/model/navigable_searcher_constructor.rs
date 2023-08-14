use std::path::PathBuf;
use derive_builder::Builder;
use fluent_integer::Integer;
use crate::data_source::{Direction, FileBackend};
use crate::search::navigable_searcher::NavigableSearcher;
use crate::search::navigable_searcher_impl::NavigableSearcherImpl;
use crate::search::searcher::create_searcher;

#[derive(Builder, Debug)]
pub struct NavigableSearcherConstructor {
    file_name: Option<PathBuf>,
    pattern: String,
    is_regexp: bool,
    initial_offset: Option<Integer>,
    is_backward: bool,
}

impl NavigableSearcherConstructor {
    pub fn construct_searcher(self) -> Result<Box<dyn NavigableSearcher>, NavigableSearcherConstructorError> {
        if let Some(file_name) = &self.file_name {
            if !self.pattern.is_empty() {
                let backend = FileBackend::new(file_name.clone());
                let searcher = create_searcher(backend, self.pattern.clone(), self.is_regexp);
                let mut navigable_searcher = NavigableSearcherImpl::new(searcher);
                if let Some(initial_offset) = &self.initial_offset {
                    let direction = Direction::from(!self.is_backward);
                    navigable_searcher.set_initial_offset(*initial_offset, direction);
                }
                log::info!("Search: {:?}", self.pattern);
                return Ok(Box::new(navigable_searcher));
            } else {
                Err(NavigableSearcherConstructorError::PatternIsEmpty)
            }
        } else {
            Err(NavigableSearcherConstructorError::FileNotSet)
        }
    }
}

pub enum NavigableSearcherConstructorError {
    FileNotSet,
    PatternIsEmpty,
}

impl ToString for NavigableSearcherConstructorError {
    fn to_string(&self) -> String {
        let str = match self {
            NavigableSearcherConstructorError::PatternIsEmpty => "Pattern is empty",
            NavigableSearcherConstructorError::FileNotSet => "File (data source) not specified",
        };
        str.to_string()
    }
}