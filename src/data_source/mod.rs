use std::path::PathBuf;
use std::io::{Read, Seek, BufReader, SeekFrom, Cursor};
use std::fs::File;
use std::cmp::Ordering;
use std::cell::RefMut;
use std::ops::Deref;
use std::sync::Arc;
use metrics::{describe_histogram, Unit};
use num_traits::ToPrimitive;
use fluent_integer::Integer;
use crate::advanced_io::advanced_buf_reader::BidirectionalBufRead;
use crate::advanced_io::seek_to::SeekTo;
use crate::shared::Shared;
use crate::utils;
use crate::utils::utf8::UtfChar;
use crate::data_source::char_navigation::{next_char, peek_next_char, peek_prev_char, prev_char};
use crate::data_source::line_registry::{LineRegistry, LineRegistryImpl};
use crate::interval::Interval;
use crate::utils::stat;

pub const BUFFER_SIZE: usize = 1024 * 1024; // 1MB

const METRIC_READ_DELIMITED: &str = "read_delimited";

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Line {
    pub content: String, // TODO use appropriate type
    pub start: Integer, // offset of the first symbol in line
    pub end: Integer, // offset of the first symbol of the next line
    pub line_no: Option<u64>,
}

impl Line {
    pub fn new<T, I>(content: T, start: I, end: I) -> Self
        where T: ToString, I: Into<Integer>
    {
        Line {
            content: content.to_string(),
            start: start.into(),
            end: end.into(),
            line_no: None,
        }
    }

    pub fn new_with_line_no<T, I>(content: T, start: I, end: I, line_no: u64) -> Self
        where T: ToString, I: Into<Integer>
    {
        Line {
            content: content.to_string(),
            start: start.into(),
            end: end.into(),
            line_no: Some(line_no),
        }
    }

    fn builder() -> LineBuilder {
        LineBuilder::default()
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct LineBuilder {
    content: Option<String>,
    start: Option<Integer>,
    end: Option<Integer>,
    line_no: Option<u64>,
}

impl LineBuilder {

    pub fn with_content<T: ToString>(mut self, content: T) -> Self {
        self.content.replace(content.to_string());
        self
    }

    pub fn with_start<I: Into<Integer>>(mut self, start: I) -> Self {
        self.start.replace(start.into());
        self
    }

    pub fn with_end<I: Into<Integer>>(mut self, end: I) -> Self {
        self.end.replace(end.into());
        self
    }

    pub fn with_line_no(mut self, n: Option<u64>) -> Self {
        if let Some(n) = n {
            self.line_no.replace(n);
        }
        self
    }

    pub fn build(self) -> Line {
        let content = self.content.unwrap();
        let start = self.start.unwrap();
        let end = self.end.unwrap();
        let line_no = self.line_no;
        Line {
            content,
            start,
            end,
            line_no,
        }
    }
}

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

/// Reads a collection of at most `abs(n)` segments (lines, words, etc.) that are delimited by chars that
/// satisfy `is_delimiter` in direction denoted by `sign(n)`.
///
/// Char `ch` is considered to be a delimiter if and only of `is_delimiter(&ch) == true`.
///
/// If `n == 0`, the method returns empty Data with no segments.
#[profiling::function]
pub fn read_delimited<R, F>(
    f: &mut BufReader<R>,
    offset: Integer,
    n: Integer,
    allow_empty_segments: bool,
    line_registry: Option<Arc<LineRegistryImpl>>,
    is_delimiter: F) -> std::io::Result<Data>
    where R: Read + Seek, F: Fn(&char) -> bool
{
    if offset < 0 {
        return Ok(Data::default());
    }

    let direction = match n.cmp(&0.into()) {
        Ordering::Equal => return Ok(Data {
            lines: vec![],
            start: None,
            end: None,
        }),
        Ordering::Greater => Direction::Forward,
        Ordering::Less => Direction::Backward
    };

    let actual_offset: Integer = f.stream_position()?.into();
    let shift = (offset - actual_offset).as_i64();
    f.seek_relative(shift)?;
    let mut current_no = line_registry
        .zip(Some(&direction))
        .and_then(move |(r, direction)| {
            let interval = match direction {
                Direction::Forward => Interval::closed(0.into(), offset),
                Direction::Backward => Interval::closed_open(0.into(), offset),
            };
            r.count(&interval).ok()
        })
        .map(|n| n as u64);

    let mut data = vec![];
    let mut stack = vec![];
    let flush = |s: &mut Vec<char>| -> (String, u64) {
        let mut content: String = s.iter().collect();
        let bytes_trimmed = utils::trim_newline(&mut content);
        s.clear();
        (content, bytes_trimmed as u64)
    };

    match direction {
        Direction::Forward => {
            // move to the beginning of current segment
            while let Some(ch) = peek_prev_char(f)? {
                if is_delimiter(&ch.get_char()) {
                    break;
                } else {
                    prev_char(f)?;
                }
            }

            // read <= n segments
            let mut start = None;
            loop {
                if let Some(ch) = next_char(f)? {
                    if !is_delimiter(&ch.get_char()) {
                        stack.push(ch.get_char());
                        start = start.or(Some(ch.get_offset()));
                    } else {
                        let line_no = current_no;
                        current_no = current_no.map(|n| n + 1);
                        if !stack.is_empty() || allow_empty_segments {
                            let (content, bytes_trimmed) = flush(&mut stack);
                            let line = Line::builder()
                                .with_content(content)
                                .with_start(start.unwrap_or(ch.get_offset()))
                                .with_end(ch.get_offset() - bytes_trimmed)
                                .with_line_no(line_no)
                                .build();
                            data.push(line);
                            if data.len() == n.abs() {
                                break;
                            }
                        }
                        start = Some(ch.get_end());
                    }
                } else {
                    // EOF
                    if !stack.is_empty() || (allow_empty_segments && start.is_some()) {
                        let (content, bytes_trimmed) = flush(&mut stack);
                        let line = Line::builder()
                            .with_content(content)
                            .with_start(start.unwrap())
                            .with_end(f.stream_position()? - bytes_trimmed)
                            .with_line_no(current_no)
                            .build();
                        data.push(line);
                    }
                    break;
                }
            }
        },
        Direction::Backward => {
            // move to the end of current segment
            while let Some(ch) = peek_next_char(f)? {
                if is_delimiter(&ch.get_char()) {
                    break;
                } else {
                    next_char(f)?;
                }
            }

            // read <= n segments
            let mut end = None;
            loop {
                if let Some(ch) = prev_char(f)? {
                    if !is_delimiter(&ch.get_char()) {
                        stack.push(ch.get_char());
                        end = end.or(Some(ch.get_end()));
                    } else {
                        let line_no = current_no;
                        current_no = current_no.map(|n| n.saturating_sub(1));
                        if !stack.is_empty() || allow_empty_segments {
                            stack.reverse();
                            let (content, bytes_trimmed) = flush(&mut stack);
                            let line = Line::builder()
                                .with_content(content)
                                .with_start(ch.get_offset() + 1)
                                .with_end(end.unwrap_or(ch.get_end()) - bytes_trimmed)
                                .with_line_no(line_no)
                                .build();
                            data.push(line);
                            if data.len() == n.abs() {
                                break;
                            }
                        }
                        end = Some(ch.get_offset());
                    }
                } else {
                    // BOF
                    if !stack.is_empty() || (allow_empty_segments && end.is_some()) {
                        stack.reverse();
                        let (content, bytes_trimmed) = flush(&mut stack);
                        let line = Line::builder()
                            .with_content(content)
                            .with_start(0)
                            .with_end(end.unwrap() - bytes_trimmed)
                            .with_line_no(current_no)
                            .build();
                        data.push(line);
                    }
                    break;
                }
            }
            data.reverse();
        },
    }

    log::trace!("current_no = {:?}, offset = {:?}", current_no, f.stream_position());

    let s = data.first().map(|segment| segment.start);
    let e = data.last().map(|segment| segment.end);
    Ok(Data {
        lines: data,
        start: s,
        end: e,
    })
}

/// Represents source of lines.
pub trait LineSource {

    /// Returns length
    fn get_length(&self) -> Integer;

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

    fn read_raw(&self, start: Integer, end: Integer) -> Result<String, ()>;

    /// Skips token starting from offset +/- 1 (depending on `direction`). A token is a
    /// group of either non-delimiters or delimiters.
    fn skip_token(&mut self, offset: Integer, direction: Direction) -> Result<Integer, ()>;

    fn get_line_registry(&self) -> Arc<LineRegistryImpl>;
}

pub trait LineSourceBackend<R: Read> {
    fn get_length(&self) -> u64;

    fn new_reader(&self) -> BufReader<R>;
}

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

impl<R, B> LineSourceImpl<R, B> where R: Read + Seek, B: LineSourceBackend<R> {
    pub fn from_file_name(file_name: PathBuf) -> LineSourceImpl<File, FileBackend> {
        LineSourceImpl::new(FileBackend::new(file_name))
    }

    pub fn from_str(s: &str) -> LineSourceImpl<Cursor<&[u8]>, StrBackend> {
        LineSourceImpl::new(StrBackend::new(s))
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

    fn get_length(&self) -> Integer {
        self.backend.get_length().into()
    }

    fn read_lines(&mut self, offset: Integer, number_of_lines: Integer) -> Data {
        let line_registry = if self.track_line_no {
            Some(Arc::clone(&self.line_registry))
        } else {
            None
        };
        let result = self.with_reader(|mut f| {
            log::trace!("read_lines number_of_lines = {}, offset = {}", number_of_lines, offset);
            stat(METRIC_READ_DELIMITED, &Unit::Microseconds, || {
                read_delimited(&mut f, offset, number_of_lines, true, line_registry, |c| *c == '\n')
            })
        }).unwrap_or(Data::default());

        log::trace!("Result: {:?}", result);

        result
    }

    fn track_line_number(&mut self, track: bool) {
        self.track_line_no = track;
    }

    fn read_raw(&self, start: Integer, end: Integer) -> Result<String, ()> {
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

    fn skip_token(&mut self, offset: Integer, direction: Direction) -> Result<Integer, ()> {
        let mut f = self.reader();

        let actual_offset: Integer = f.stream_position().map_err(|_| ())?.into();
        f.seek_relative((offset - actual_offset).as_i64()).map_err(|_| ())?;

        let take_char0 = match direction {
            Direction::Forward => next_char,
            Direction:: Backward => prev_char,
        };
        let take_char = |reader: &mut BufReader<R>| -> Result<Option<UtfChar>, ()> {
            take_char0(reader).map_err(|_| ())
        };

        if direction == Direction::Backward {
            next_char(&mut f).map_err(|_| ())?;
        }

        let is_delimiter = |ch: &char| !ch.is_alphanumeric() && *ch != '_'; // TODO: better UTF-8 delimiter detection

        enum State {
            InToken,
            InWhitespace,
            DetermineIfTokenBoundary
        }

        if let Some(pattern) = take_char(&mut f)? {
            let mut state = if !is_delimiter(&pattern.get_char()) {
                State::DetermineIfTokenBoundary
            } else {
                State::InWhitespace
            };
            let mut prev_char_offset = pattern.get_offset();
            while let Some(ch) = take_char(&mut f)? {
                match state {
                    State::DetermineIfTokenBoundary => {
                        if !is_delimiter(&ch.get_char()) {
                            state = State::InToken;
                        } else {
                            state = State::InWhitespace;
                        }
                    },
                    State::InWhitespace => if !is_delimiter(&ch.get_char()) {
                        prev_char_offset = ch.get_offset();
                        break;
                    },
                    State::InToken => if is_delimiter(&ch.get_char()) {
                        break;
                    }
                };
                prev_char_offset = ch.get_offset();
            }
            Ok(prev_char_offset.into())
        } else {
            Ok(offset)
        }
    }

    fn get_line_registry(&self) -> Arc<LineRegistryImpl> {
        Arc::clone(&self.line_registry)
    }
}

mod char_navigation {
    use std::io::{BufReader, Read, Seek};
    use crate::utils;
    use crate::utils::utf8::UtfChar;

    pub fn next_char<R: Read + Seek>(reader: &mut BufReader<R>) -> std::io::Result<Option<UtfChar>> {
        utils::utf8::read_utf_char(reader)
    }

    pub fn peek_prev_char<R: Read + Seek>(reader: &mut BufReader<R>) -> std::io::Result<Option<UtfChar>> {
        if reader.stream_position()? == 0 {
            return Ok(None);
        }
        reader.seek_relative(-1)?;
        next_char(reader)
    }

    pub fn prev_char<R: Read + Seek>(reader: &mut BufReader<R>) -> std::io::Result<Option<UtfChar>> {
        let result = peek_prev_char(reader)?;
        if let Some(ch) = result.as_ref() {
            let len = ch.get_char().len_utf8() as i64;
            reader.seek_relative(-len)?;
        }
        Ok(result)
    }

    pub fn peek_next_char<R: Read + Seek>(reader: &mut BufReader<R>) -> std::io::Result<Option<UtfChar>> {
        let result = next_char(reader)?;
        if let Some(ch) = result.as_ref() {
            let len = ch.get_char().len_utf8() as i64;
            reader.seek_relative(-len)?;
        }
        Ok(result)
    }
}

// Tests are included according to http://xion.io/post/code/rust-unit-test-placement.html
#[cfg(test)]
#[path = "./data_source_tests.rs"]
mod data_source_tests;

pub mod line_registry;