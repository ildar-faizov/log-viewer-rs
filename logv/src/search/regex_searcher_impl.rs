use crate::advanced_io::raw_line::RawLine;
use crate::data_source::Direction;
use crate::interval::Interval;
use crate::search::search_utils::{calculate_offset_and_boundary, OffsetAndBoundary};
use crate::search::searcher::{Occurrence, SearchError, SearchResult, Searcher};
use fluent_integer::Integer;
use regex::Regex;
use std::io::{BufReader, Read, Seek};

pub struct RegexSearcherImpl<R>
    where R: Read + Seek
{
    reader: BufReader<R>,
    regex: Regex,
}

impl<R> RegexSearcherImpl<R>
    where R: Read + Seek
{
    pub fn new(reader: BufReader<R>, regex: Regex) -> RegexSearcherImpl<R> {
        RegexSearcherImpl {
            reader,
            regex,
        }
    }
}

impl<R> Searcher for RegexSearcherImpl<R>
    where R: Read + Seek
{
    fn search(&mut self, direction: Direction, range: Interval<Integer>) -> SearchResult {
        let OffsetAndBoundary {
            offset,
            offset_boundary
        } = calculate_offset_and_boundary(&mut self.reader, direction, range)?;

        match direction {
            Direction::Forward => self.search_forward(offset, offset_boundary),
            Direction::Backward => self.search_backward(offset, offset_boundary),
        }
    }
}

impl<R> RegexSearcherImpl<R>
where
    R: Read + Seek
{
    fn search_forward(&mut self, mut offset: Integer, offset_boundary: Option<Integer>) -> SearchResult {
        loop {
            if offset_boundary.filter(|b| *b < offset).is_some() {
                break Err(SearchError::NotFound)
            }
            if let Ok(line) = RawLine::read_from(&mut self.reader, offset) {
                let start: Integer = line.start().into();
                let end: Integer = line.end().into();
                let at: usize = if offset > start && offset < end {
                    (offset - start).as_usize()
                } else {
                    0
                };
                let content = String::from_utf8_lossy(&line);
                if let Some(m) = self.regex.find_at(&content, at) {
                    // TODO: check that m.start() returns offset from string start
                    let occurrence = Occurrence::from_match(m) + start;
                    break Ok(occurrence)
                } else {
                    offset = end + 1
                }
            } else {
                break Err(SearchError::NotFound)
            }
        }
    }

    fn search_backward(&mut self, mut offset: Integer, offset_boundary: Option<Integer>) -> SearchResult {
        loop {
            if offset_boundary.unwrap_or(0.into()) > offset {
                break Err(SearchError::NotFound)
            }
            if let Ok(line) = RawLine::read_backwards_from(&mut self.reader, offset) {
                let start: Integer = line.start().into();
                let end: Integer = line.end().into();
                let at: usize = if offset > start && offset < end {
                    (offset - start).as_usize() + 1
                } else {
                    line.len()
                };
                let content = String::from_utf8_lossy(&line);
                if let Some(m) = self.regex.find_iter(&content[..at]).last() {
                    let occurrence = Occurrence::from_match(m) + start;
                    break Ok(occurrence)
                } else {
                    offset = start - 1
                }
            } else {
                break Err(SearchError::NotFound)
            }
        }
    }
}

// Tests are included according to http://xion.io/post/code/rust-unit-test-placement.html
#[cfg(test)]
#[path = "./regex_searcher_tests.rs"]
mod regex_searcher_tests;