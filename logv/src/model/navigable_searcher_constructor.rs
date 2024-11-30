use std::fmt::{Display, Formatter};
use std::io::BufReader;
use derive_builder::Builder;
use thiserror::Error;
use fluent_integer::Integer;
use crate::data_source::Direction;
use crate::data_source::reader_factory::ReaderFactory;
use crate::search::navigable_searcher::NavigableSearcher;
use crate::search::navigable_searcher_impl::NavigableSearcherImpl;
use crate::search::searcher::create_searcher;

#[derive(Builder, Debug)]
#[builder(pattern = "owned")]
pub struct NavigableSearcherConstructor
{
    reader_factory: Box<dyn ReaderFactory>,
    pattern: String,
    is_regexp: bool,
    initial_offset: Option<Integer>,
    is_backward: bool,
}

impl NavigableSearcherConstructor {
    pub fn construct_searcher(self) -> Result<Box<dyn NavigableSearcher>, NavigableSearcherConstructorError> {
        if !self.pattern.is_empty() {
            let reader = self.reader_factory.new_reader()?;
            let searcher = create_searcher(BufReader::new(reader), self.pattern.clone(), self.is_regexp);
            let mut navigable_searcher = NavigableSearcherImpl::new(searcher);
            if let Some(initial_offset) = &self.initial_offset {
                let direction = Direction::from(!self.is_backward);
                navigable_searcher.set_initial_offset(*initial_offset, direction);
            }
            log::info!("Search: {:?}", self.pattern);
            Ok(Box::new(navigable_searcher))
        } else {
            Err(NavigableSearcherConstructorError::PatternIsEmpty)
        }
    }
}

#[derive(Error, Debug)]
pub enum NavigableSearcherConstructorError {
    FileNotSet,
    PatternIsEmpty,
    IO(#[from] std::io::Error),
}

impl Display for NavigableSearcherConstructorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            NavigableSearcherConstructorError::PatternIsEmpty => "Pattern is empty",
            NavigableSearcherConstructorError::FileNotSet => "File (data source) not specified",
            NavigableSearcherConstructorError::IO(err) => &format!("{:?}", err),
        };
        write!(f, "{}", str)
    }
}

impl Display for NavigableSearcherConstructor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let as_regexp = if self.is_regexp {
            " (regexp)"
        } else {
            ""
        };
        write!(f, "'{}' {} in {:?}", self.pattern, as_regexp, self.reader_factory)?;
        Ok(())
    }
}