use crate::advanced_io::advanced_buf_reader::BidirectionalBufRead;
use crate::advanced_io::seek_to::SeekTo;
pub use crate::data_source::custom_highlight::{CustomHighlight, CustomHighlights};
use crate::data_source::line_registry::LineRegistryImpl;
use crate::shared::Shared;
use crate::utils::stat;
use fluent_integer::Integer;
use metrics::{describe_histogram, Unit};
use std::cell::RefMut;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::Arc;
pub use crate::data_source::line::{Line, LineBuilder};
use crate::data_source::read_delimited::read_delimited;

pub const BUFFER_SIZE: usize = 1024 * 1024; // 1MB

const METRIC_READ_DELIMITED: &str = "read_delimited";

#[derive(Debug, Default)]
pub struct Data {
    pub lines: Vec<Line>,
    pub start: Option<Integer>,
    pub end: Option<Integer>,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Direction {
    Forward, Backward
}

impl From<bool> for Direction {
    fn from(value: bool) -> Self {
        if value {
            Direction::Forward
        } else {
            Direction::Backward
        }
    }
}

/// Represents source of lines.
pub trait LineSource {

    /// Reads requested number of lines in any direction. Result contains collection of lines with
    /// their offsets and also overall start and end offsets.
    ///
    /// Reading begins with the line that *contains* `offset`. I.e. if `offset` is not a
    /// beginning of a line, the result will start at *some point that is less than `offset`*.
    ///
    /// ## Example.
    /// Let a file be
    /// > This is a big brown fox.
    /// >
    /// > It can jump.
    ///
    /// Then invoking `read_lines(5, 1)` will return `["This is a big brown fox."]`.
    ///
    /// # Reading in Reverse Direction
    /// If `number_of_lines` is negative, then lines are being read in the reverse direction.
    /// The first line which is included in result is the one that contains `offset`. Lines are
    /// returned in their natural order.
    ///
    /// ## Example
    /// Let a file be (assuming line delimiter is `\n`)
    /// > AAA
    /// >
    /// > BBB
    /// >
    /// > CCC
    /// >
    /// > DDD
    ///
    /// Then `read_lines(4, -1)` returns `["BBB"]`.\
    /// `read_lines(0, -1)` returns `["AAA"]`.\
    /// `read_lines(7, -1)` returns `["BBB"]`.\
    /// `read_lines(5, -2)` returns `["AAA", "BBB"]`.\
    fn read_lines(&mut self, offset: Integer, number_of_lines: Integer) -> Data;

    /// Reads next line
    fn read_next_line(&mut self, offset: Integer) -> Option<Line> {
        self.read_lines(offset, Integer::from(1)).lines.pop()
    }

    /// Reads previous line
    fn read_prev_line(&mut self, offset: Integer) -> Option<Line> {
        self.read_lines(offset, Integer::from(-1)).lines.pop()
    }

    fn track_line_number(&mut self, track: bool);

    fn read_raw(&mut self, start: Integer, end: Integer) -> Result<String, ()>;

    /// Skips token starting from offset +/- 1 (depending on `direction`). A token is a
    /// group of either non-delimiters or delimiters.
    fn skip_token(&mut self, offset: Integer, direction: Direction) -> anyhow::Result<Integer>;

    fn get_line_registry(&self) -> Arc<LineRegistryImpl>;
}

pub trait LineSourceBackend<R: Read> {
    fn get_length(&self) -> u64;

    fn new_reader(&self) -> BufReader<R>;
}

#[derive(Clone)]
pub struct FileBackend {
    file_name: PathBuf
}

impl FileBackend {
    pub fn new(file_name: PathBuf) -> Self {
        FileBackend { file_name }
    }
}

impl LineSourceBackend<File> for FileBackend {
    fn get_length(&self) -> u64 {
        std::fs::metadata(self.file_name.as_path()).unwrap().len()
    }

    fn new_reader(&self) -> BufReader<File> {
        BufReader::new(File::open(&self.file_name).unwrap())
    }
}

#[derive(Clone)]
pub struct StrBackend<'a> {
    s: Cursor<&'a str>
}

impl<'a> StrBackend<'a> {
    pub fn new(s: &'a str) -> Self {
        StrBackend { s: Cursor::new(s) }
    }
}

impl<'a> LineSourceBackend<Cursor<&'a [u8]>> for StrBackend<'a> {
    fn get_length(&self) -> u64 {
        self.s.get_ref().len() as u64
    }

    fn new_reader(&self) -> BufReader<Cursor<&'a [u8]>> {
        let cursor = Cursor::new(self.s.get_ref().as_bytes());
        BufReader::new(cursor)
    }
}

pub struct LineSourceImpl<R, B> where R: Read, B: LineSourceBackend<R> {
    backend: B,
    file_reader: Option<Shared<BufReader<R>>>,
    track_line_no: bool,
    line_registry: Arc<LineRegistryImpl>,
}

impl<'a> LineSourceImpl<Cursor<&'a [u8]>, StrBackend<'a>> {
    pub fn from_str(s: &'a str) -> LineSourceImpl<Cursor<&'a [u8]>, StrBackend<'a>> {
        LineSourceImpl::new(StrBackend::new(s))
    }
}

impl<R, B> LineSourceImpl<R, B> where R: Read + Seek, B: LineSourceBackend<R> {
    pub fn from_file_name(file_name: PathBuf) -> LineSourceImpl<File, FileBackend> {
        LineSourceImpl::new(FileBackend::new(file_name))
    }

    pub fn new(backend: B) -> LineSourceImpl<R, B> {
        describe_histogram!(METRIC_READ_DELIMITED, Unit::Microseconds, "Reading from file");
        LineSourceImpl {
            backend,
            file_reader: None,
            track_line_no: false,
            line_registry: Arc::new(LineRegistryImpl::new()),
        }
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn get_length(&self) -> Integer {
        self.backend.get_length().into()
    }

    fn reader(&mut self) -> RefMut<'_, BufReader<R>> {
        if self.file_reader.is_none() {
            let f = self.backend.new_reader();
            self.file_reader = Some(Shared::new(f));
        }
        self.file_reader.as_ref().unwrap().get_mut_ref()
    }

    fn with_reader<T, U>(&mut self, f: T) -> U
        where T: FnOnce(RefMut<BufReader<R>>) -> U
    {
        let file_reader = self.reader();
        f(file_reader)
    }
}

impl<R, B> Clone for LineSourceImpl<R, B>
    where
        R: Read + Seek,
        B: LineSourceBackend<R>,
        B: Clone,
{
    fn clone(&self) -> Self {
        let mut result = Self::new(self.backend.clone());
        result.track_line_number(self.track_line_no);
        result
    }
}

impl<R, B> Seek for LineSourceImpl<R, B> where B: LineSourceBackend<R>, R: Read + Seek {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.with_reader(|mut reader| reader.seek(pos))
    }
}

impl<R, B> SeekTo for LineSourceImpl<R, B> where B: LineSourceBackend<R>, R: Read + Seek {

    fn seek_to<I: Into<Integer>>(&mut self, pos: I) -> std::io::Result<()> {
        self.with_reader(|mut reader| reader.seek_to(pos))
    }
}

impl<R, B> LineSource for LineSourceImpl<R, B> where R: Read + Seek, B: LineSourceBackend<R> {

    fn read_lines(&mut self, offset: Integer, number_of_lines: Integer) -> Data {
        let line_registry = if self.track_line_no {
            Some(Arc::clone(&self.line_registry))
        } else {
            None
        };
        let offset = if offset < 0 {
            self.get_length() - offset
        } else {
            offset
        };
        let result = self.with_reader(|mut f| {
            log::trace!("read_lines number_of_lines = {}, offset = {}", number_of_lines, offset);
            stat(METRIC_READ_DELIMITED, &Unit::Microseconds, || {
                read_delimited(&mut f, offset, number_of_lines, true, line_registry, |c| *c == '\n')
            })
        }).unwrap_or_default();

        log::trace!("Result: {:?}", result);

        result
    }

    fn track_line_number(&mut self, track: bool) {
        self.track_line_no = track;
    }

    fn read_raw(&mut self, start: Integer, end: Integer) -> Result<String, ()> {
        let mut f = self.backend.new_reader();
        f.seek(SeekFrom::Start(start.as_u64())).map_err(|_| ())?;
        let len = (end - start).as_usize();
        let mut result: Vec<u8> = Vec::with_capacity(len);
        let consumer = |chunk: &[u8]| {
            for x in chunk {
                result.push(*x);
            }
        };
        f.read_fluently(len as i128, consumer).map_err(|_| ())?;
        Ok(String::from_utf8(result).unwrap())
    }

    fn skip_token(&mut self, offset: Integer, direction: Direction) -> anyhow::Result<Integer> {
        let mut f = self.reader();
        tokenizer::skip_token(offset, direction, &mut *f)
    }

    fn get_line_registry(&self) -> Arc<LineRegistryImpl> {
        Arc::clone(&self.line_registry)
    }
}

// Tests are included according to http://xion.io/post/code/rust-unit-test-placement.html
#[cfg(test)]
#[path = "./data_source_tests.rs"]
mod data_source_tests;

pub mod line_registry;
pub mod filtered;
pub mod line_source_holder;
mod tokenizer;
mod custom_highlight;
mod line;
mod read_delimited;
mod char_navigation;