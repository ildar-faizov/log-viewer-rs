use std::io::{Read, Seek};
use std::ops::Add;
use regex::{Match, Regex};
use fluent_integer::Integer;
use crate::data_source::{Direction, LineSourceBackend};
use crate::interval::Interval;
use crate::search::regex_searcher_impl::RegexSearcherImpl;
use crate::search::searcher_impl::SearcherImpl;

// closed segment
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Occurrence {
    pub start: Integer,
    pub end: Integer,
}

pub trait Searcher {
    fn search(&mut self, direction: Direction, range: Interval<Integer>) -> SearchResult;
}

#[derive(Debug)]
pub enum SearchError {
    NotFound,
    IO(std::io::Error)
}

pub type SearchResult = Result<Occurrence, SearchError>;

pub fn create_searcher<R: Read + Seek + 'static, B: LineSourceBackend<R> + 'static>(backend: B, pattern: String, is_regex: bool) -> Box<dyn Searcher> {
    if is_regex {
        Box::new(RegexSearcherImpl::new(backend, Regex::new(pattern.as_str()).unwrap()))
    } else {
        Box::new(SearcherImpl::new(backend, pattern))
    }
}

impl Occurrence {
    pub fn new<I, J>(start: I, end: J) -> Self
        where I: Into<Integer>, J: Into<Integer> {
        Occurrence {
            start: start.into(),
            end: end.into(),
        }
    }

    pub fn with_len<I, J>(start: I, len: J) -> Self
        where I: Into<Integer>, J: Into<Integer> {
        let s = start.into();
        Self::new(s, s + len.into())
    }

    pub fn from_match(m: Match) -> Self {
        Self::new(m.start(), m.end())
    }

}

impl<I: Into<Integer>> Add<I> for Occurrence {
    type Output = Occurrence;

    fn add(self, rhs: I) -> Self::Output {
        let rhs = rhs.into();
        Occurrence::new(self.start + rhs, self.end + rhs)
    }
}

impl PartialEq<SearchError> for SearchError {
    fn eq(&self, other: &SearchError) -> bool {
        self.is_not_found() && other.is_not_found()
    }
}

impl SearchError {
    pub fn is_not_found(&self) -> bool {
        matches!(&self, SearchError::NotFound)
    }
}