use std::io::{Read, Seek};
use regex::Regex;
use fluent_integer::Integer;
use crate::data_source::{Direction, LineSource, LineSourceBackend, LineSourceImpl};
use crate::interval::Interval;
use crate::search::search_utils::calculate_offset_and_boundary;
use crate::search::searcher::{Occurrence, Searcher, SearchError, SearchResult};

pub struct RegexSearcherImpl<R, B>
    where R: Read + Seek, B: LineSourceBackend<R>
{
    line_source: LineSourceImpl<R, B>,
    regex: Regex,
    offset: Integer,
}

impl<R, B> RegexSearcherImpl<R, B>
    where R: Read + Seek, B: LineSourceBackend<R>
{
    pub fn new(backend: B, regex: Regex) -> RegexSearcherImpl<R, B> {
        RegexSearcherImpl {
            line_source: LineSourceImpl::new(backend),
            regex,
            offset: 0.into(), // todo remove field
        }
    }
}

impl<R, B> Searcher for RegexSearcherImpl<R, B>
    where R: Read + Seek, B: LineSourceBackend<R>
{
    fn next_occurrence(&mut self, direction: Direction, range: Interval<Integer>) -> SearchResult {
        let offset_and_boundary = calculate_offset_and_boundary(&mut self.line_source, direction, range)?;
        self.offset = offset_and_boundary.offset;
        let offset_boundary = offset_and_boundary.offset_boundary;

        match direction {
            Direction::Forward => {
                loop {
                    if offset_boundary.filter(|b| *b < self.offset).is_some() {
                        break Err(SearchError::NotFound)
                    }
                    if let Some(line) = self.line_source.read_next_line(self.offset) {
                        let at: usize = if self.offset > line.start && self.offset < line.end {
                            (self.offset - line.start).as_usize()
                        } else {
                            0
                        };
                        if let Some(m) = self.regex.find_at(line.content.as_str(), at) {
                            // TODO: check that m.start() returns offset from string start
                            let occurrence = Occurrence::from_match(m) + line.start;
                            self.offset = occurrence.start + 1;
                            break Ok(occurrence)
                        } else {
                            self.offset = line.end + 1
                        }
                    } else {
                        break Err(SearchError::NotFound)
                    }
                }
            },
            Direction::Backward => {
                todo!("Not implemented yet")
            }
        }
    }
}

// Tests are included according to http://xion.io/post/code/rust-unit-test-placement.html
#[cfg(test)]
#[path = "./regex_searcher_tests.rs"]
mod regex_searcher_tests;