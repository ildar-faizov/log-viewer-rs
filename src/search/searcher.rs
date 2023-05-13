use std::io::{Read, Seek};
use fluent_integer::Integer;
use crate::data_source::{Direction, LineSourceBackend};
use crate::search::searcher_impl::SearcherImpl;

pub trait Searcher {
    /// Searches for `pattern` in the `direction` starting from `offset`.
    ///
    /// `pattern` is treated as text (not regular expression). The result contains offset of the first
    /// character of the match. `pattern` **must not** contain multiple lines, the search is
    /// performed line by line.
    fn next_occurrence(&mut self, direction: Direction) -> SearchResult;
}

#[derive(Debug)]
pub enum SearchError {
    NotFound,
    IO(std::io::Error)
}

pub type SearchResult = Result<Integer, SearchError>;

pub fn create_searcher<R: Read + Seek + 'static, B: LineSourceBackend<R>, I: Into<Integer>>(backend: B, pattern: String, offset: I) -> Box<dyn Searcher> {
    Box::new(SearcherImpl::new(backend, pattern, offset.into()))
}