use std::collections::LinkedList;
use std::io::{BufReader, Read, Seek, SeekFrom};
use fluent_integer::Integer;
use SearchError::IO;
use crate::advanced_io::advanced_buf_reader::BidirectionalBufRead;
use crate::advanced_io::seek_to::SeekTo;
use crate::data_source::{Direction, LineSourceBackend};
use crate::search::searcher::{Searcher, SearchError, SearchResult};
use crate::search::searcher::SearchError::NotFound;

pub struct SearcherImpl<R>
    where R: Read + Seek
{
    f: BufReader<R>,
    pattern: String,
    buffer: LinkedList<u8>,
    last_occurrence: Option<Integer>,
}

impl<R> SearcherImpl<R>
    where R: Read + Seek
{
    pub fn new<B: LineSourceBackend<R>>(backend: B, pattern: String, offset: Integer) -> SearcherImpl<R> {
        let mut reader = backend.new_reader();
        if offset > 0 {
            reader.seek(SeekFrom::Start(offset.as_u64())).expect("Failed to seek");
        }
        SearcherImpl {
            f: reader,
            pattern,
            buffer: LinkedList::default(),
            last_occurrence: None,
        }
    }
}

impl<R> Searcher for SearcherImpl<R>
    where R: Read + Seek
{
    fn next_occurrence(&mut self, direction: Direction) -> SearchResult {
        match direction {
            Direction::Forward => self.scan(),
            Direction::Backward => self.scan_backward()
        }
    }

    fn get_last_occurrence(&self) -> Option<Integer> {
        self.last_occurrence
    }
}

impl<R> SearcherImpl<R> where R: Read + Seek {

    fn scan(&mut self) -> SearchResult {
        let mut offset: Integer = self.f.stream_position().map_err(|e| IO(e))?.into();
        if let Some(p) = &self.last_occurrence {
            offset = *p + 1_u8;
            self.f.seek_to(offset).map_err(|e| IO(e))?;
        }
        self.buffer.clear();
        loop {
            self.fill_buffer()?;
            if self.compare() {
                self.last_occurrence = Some(offset);
                break Ok(offset);
            } else if self.buffer.pop_front().is_some() {
                offset += 1;
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

    fn scan_backward(&mut self) -> SearchResult {
        let mut offset: Integer = self.f.stream_position().map_err(|e| IO(e))?.into();
        if let Some(p) = &self.last_occurrence {
            offset = *p + self.pattern.as_bytes().len() - 1;
            self.f.seek_to(offset).map_err(|e| IO(e))?;
        }
        self.buffer.clear();
        self.f.seek_to(offset).map_err(|e| IO(e))?;
        loop {
            self.fill_buffer_backward()?;
            if offset < 0 {
                break Err(NotFound)
            } else if self.compare() {
                offset -= self.pattern.as_bytes().len();
                self.last_occurrence = Some(offset);
                break Ok(offset);
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