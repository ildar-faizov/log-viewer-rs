use std::collections::LinkedList;
use std::io::{BufReader, Read, Seek};
use fluent_integer::Integer;
use SearchError::IO;
use crate::advanced_io::advanced_buf_reader::BidirectionalBufRead;
use crate::data_source::{Direction, LineSourceBackend};
use crate::interval::Interval;
use crate::search::search_utils::calculate_offset_and_boundary;
use crate::search::searcher::{Occurrence, Searcher, SearchError, SearchResult};
use crate::search::searcher::SearchError::NotFound;

pub struct SearcherImpl<R>
    where R: Read + Seek
{
    f: BufReader<R>,
    pattern: String,
    buffer: LinkedList<u8>,
}

impl<R> SearcherImpl<R>
    where R: Read + Seek
{
    pub fn new<B: LineSourceBackend<R>>(backend: B, pattern: String) -> SearcherImpl<R> {
        let reader = backend.new_reader();
        SearcherImpl {
            f: reader,
            pattern,
            buffer: LinkedList::default(),
        }
    }
}

impl<R> Searcher for SearcherImpl<R>
    where R: Read + Seek
{
    fn next_occurrence(&mut self, direction: Direction, range: Interval<Integer>) -> SearchResult {
        let offset_boundary = calculate_offset_and_boundary(&mut self.f, direction, range)?.offset_boundary;

        match direction {
            Direction::Forward => self.scan(offset_boundary),
            Direction::Backward => self.scan_backward(offset_boundary)
        }
    }
}

impl<R> SearcherImpl<R> where R: Read + Seek {

    fn scan(&mut self, offset_boundary: Option<Integer>) -> SearchResult {
        let mut offset: Integer = self.f.stream_position().map_err(|e| IO(e))?.into();
        self.buffer.clear();
        loop {
            self.fill_buffer()?;
            if self.compare() {
                break Ok(Occurrence::with_len(offset, self.buffer.len()));
            } else if self.buffer.pop_front().is_some() {
                offset += 1;
                if offset_boundary.filter(|b| offset > b).is_some() {
                    break Err(NotFound);
                }
            } else {
                break Err(NotFound);
            }
        }
    }

    fn fill_buffer(&mut self) -> Result<(), SearchError> {
        let m = self.buffer.len();
        let n = self.pattern.as_bytes().len();
        let delta = n.saturating_sub(m) as i128;
        if delta > 0 {
            self.f.read_fluently(delta, |chunk| {
                for ch in chunk {
                    self.buffer.push_back(*ch);
                }
            }).map_err(|e| IO(e))?;
        }
        if self.buffer.len() < n {
            return Err(NotFound);
        }
        Ok(())
    }

    fn compare(&self) -> bool {
        let pattern = self.pattern.as_bytes();
        self.buffer.len() == pattern.len() &&
            self.buffer.iter()
                .zip(pattern.iter())
                .all(|(a, b)| *a == *b)
    }

    fn scan_backward(&mut self, offset_boundary: Option<Integer>) -> SearchResult {
        let mut offset: Integer = self.f.stream_position().map_err(|e| IO(e))?.into();
        let pattern_len = self.pattern.as_bytes().len();
        self.buffer.clear();
        loop {
            self.fill_buffer_backward()?;
            if offset - pattern_len < offset_boundary.unwrap_or_default() {
                break Err(NotFound)
            } else if self.compare() {
                offset -= pattern_len;
                break Ok(Occurrence::with_len(offset, pattern_len));
            } else if self.buffer.pop_back().is_some() {
                offset -= 1;
            } else {
                break Err(NotFound);
            }
        }
    }

    fn fill_buffer_backward(&mut self) -> Result<(), SearchError> {
        let m = self.buffer.len();
        let n = self.pattern.as_bytes().len();
        if m < n {
            self.f.read_fluently((n - m) as i64 * -1, |chunk| {
                for ch in chunk {
                    self.buffer.push_front(*ch);
                }
            }).map_err(|e| IO(e))?;
            Ok(())
        } else {
            Err(NotFound)
        }
    }

}

// Tests are included according to http://xion.io/post/code/rust-unit-test-placement.html
#[cfg(test)]
#[path = "./searcher_tests.rs"]
mod searcher_tests;