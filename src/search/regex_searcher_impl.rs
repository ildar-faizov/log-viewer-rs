use std::io::{Read, Seek};
use regex::Regex;
use fluent_integer::Integer;
use crate::data_source::{Direction, LineSource, LineSourceBackend, LineSourceImpl};
use crate::interval::Interval;
use crate::search::search_utils::{calculate_offset_and_boundary, OffsetAndBoundary};
use crate::search::searcher::{Occurrence, Searcher, SearchError, SearchResult};

pub struct RegexSearcherImpl<R, B>
    where R: Read + Seek, B: LineSourceBackend<R>
{
    line_source: LineSourceImpl<R, B>,
    regex: Regex,
}

impl<R, B> RegexSearcherImpl<R, B>
    where R: Read + Seek, B: LineSourceBackend<R>
{
    pub fn new(backend: B, regex: Regex) -> RegexSearcherImpl<R, B> {
        RegexSearcherImpl {
            line_source: LineSourceImpl::new(backend),
            regex,
        }
    }
}

impl<R, B> Searcher for RegexSearcherImpl<R, B>
    where R: Read + Seek, B: LineSourceBackend<R>
{
    fn search(&mut self, direction: Direction, range: Interval<Integer>) -> SearchResult {
        let OffsetAndBoundary {
            offset,
            offset_boundary
        } = calculate_offset_and_boundary(&mut self.line_source, direction, range)?;

        match direction {
            Direction::Forward => self.search_forward(offset, offset_boundary),
            Direction::Backward => self.search_backward(offset, offset_boundary),
        }
    }
}

impl<R, B> RegexSearcherImpl<R, B> where B: LineSourceBackend<R>, R: Read + Seek {
    fn search_forward(&mut self, mut offset: Integer, offset_boundary: Option<Integer>) -> SearchResult {
        loop {
            if offset_boundary.filter(|b| *b < offset).is_some() {
                break Err(SearchError::NotFound)
            }
            if let Some(line) = self.line_source.read_next_line(offset) {
                let at: usize = if offset > line.start && offset < line.end {
                    (offset - line.start).as_usize()
                } else {
                    0
                };
                if let Some(m) = self.regex.find_at(line.content.as_str(), at) {
                    // TODO: check that m.start() returns offset from string start
                    let occurrence = Occurrence::from_match(m) + line.start;
                    break Ok(occurrence)
                } else {
                    offset = line.end + 1
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
            if let Some(line) = self.line_source.read_prev_line(offset) {
                let at: usize = if offset > line.start && offset < line.end {
                    (offset - line.start).as_usize() + 1
                } else {
                    line.content.len()
                };
                if let Some(m) = self.regex.find_iter(&line.content[..at]).last() {
                    let occurrence = Occurrence::from_match(m) + line.start;
                    break Ok(occurrence)
                } else {
                    offset = line.start - 1
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