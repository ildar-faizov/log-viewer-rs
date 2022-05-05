use std::path::PathBuf;
use std::io::{Read, Seek, BufReader, SeekFrom, Cursor};
use std::fs::File;
use std::cmp::Ordering;
use std::cell::RefMut;
use fluent_integer::Integer;
use crate::shared::Shared;
use crate::utils;

pub const BUFFER_SIZE: usize = 8192;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Line {
    pub content: String, // TODO use appropriate type
    pub start: Integer, // offset of the first symbol in line
    pub end: Integer // offset of the first symbol of the next line
}

impl Line {
    pub fn new<T, I>(content: T, start: I, end: I) -> Self
        where T: ToString, I: Into<Integer>
    {
        Line {
            content: content.to_string(),
            start: start.into(),
            end: end.into()
        }
    }
}

#[derive(Debug, Default)]
pub struct Data {
    pub lines: Vec<Line>,
    pub start: Option<Integer>,
    pub end: Option<Integer>
}

#[derive(PartialEq, Eq)]
enum Direction {
    Forward, Backward
}

/// Reads a collection of at most `abs(n)` segments (lines, words, etc.) that are delimited by chars that
/// satisfy `is_delimiter` in direction denoted by `sign(n)`.
///
/// Char `ch` is considered to be a delimiter if and only of `is_delimiter(&ch) == true`.
///
/// If `n == 0`, the method returns empty Data with no segments.
pub fn read_delimited<R, F>(
    f: &mut BufReader<R>,
    offset: Integer,
    n: Integer,
    allow_empty_segments: bool,
    is_delimiter: F) -> std::io::Result<Data>
    where R: Read + Seek, F: Fn(&char) -> bool
{
    if offset < 0 {
        return Ok(Data::default());
    }

    let actual_offset: Integer = f.stream_position()?.into();
    f.seek_relative((offset - actual_offset).as_i64())?;
    let direction = match n.cmp(&0.into()) {
        Ordering::Equal => return Ok(Data {
            lines: vec![],
            start: None,
            end: None
        }),
        Ordering::Greater => Direction::Forward,
        Ordering::Less => Direction::Backward
    };

    let next_char = |reader: &mut BufReader<R>| -> std::io::Result<Option<char>> {
        let mut buf = [0_u8; 1];
        match reader.read(&mut buf)? {
            0 => Ok(None),
            _ => Ok(Some(char::from(buf[0]))),
        }
    };

    let peek_prev_char = |reader: &mut BufReader<R>| -> std::io::Result<Option<char>> {
        if reader.stream_position()? == 0 {
            return Ok(None);
        }
        reader.seek_relative(-1)?;
        next_char(reader)
    };

    let prev_char = |reader: &mut BufReader<R>| -> std::io::Result<Option<char>> {
        let result = peek_prev_char(reader)?;
        if result.is_some() {
            reader.seek_relative(-1)?;
        }
        Ok(result)
    };

    let peek_next_char = |reader: &mut BufReader<R>| -> std::io::Result<Option<char>> {
        let result = next_char(reader)?;
        if result.is_some() {
            reader.seek_relative(-1)?;
        }
        Ok(result)
    };

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
                if is_delimiter(&ch) {
                    break;
                } else {
                    prev_char(f)?;
                }
            }

            // read <= n segments
            let mut start = None;
            loop {
                if let Some(ch) = next_char(f)? {
                    let char_offset = f.stream_position()? - 1;
                    if !is_delimiter(&ch) {
                        stack.push(ch);
                        start = start.or(Some(char_offset));
                    } else {
                        if !stack.is_empty() || allow_empty_segments {
                            let (content, bytes_trimmed) = flush(&mut stack);
                            data.push(Line::new(content, start.unwrap_or(char_offset), char_offset - bytes_trimmed));
                            if data.len() == n.abs() {
                                break;
                            }
                        }
                        start = Some(char_offset + 1);
                    }
                } else {
                    // EOF
                    if !stack.is_empty() || (allow_empty_segments && start.is_some()) {
                        let (content, bytes_trimmed) = flush(&mut stack);
                        data.push(Line::new(content, start.unwrap(), f.stream_position()? - bytes_trimmed));
                    }
                    break;
                }
            }
        },
        Direction::Backward => {
            // move to the end of current segment
            while let Some(ch) = peek_next_char(f)? {
                if is_delimiter(&ch) {
                    break;
                } else {
                    next_char(f)?;
                }
            }

            // read <= n segments
            let mut end = None;
            loop {
                if let Some(ch) = prev_char(f)? {
                    let char_offset = f.stream_position()?;
                    if !is_delimiter(&ch) {
                        stack.push(ch);
                        end = end.or(Some(char_offset + 1));
                    } else {
                        if !stack.is_empty() || allow_empty_segments {
                            stack.reverse();
                            let (content, bytes_trimmed) = flush(&mut stack);
                            data.push(Line::new(content, char_offset + 1, end.unwrap_or(char_offset + 1) - bytes_trimmed));
                            if data.len() == n.abs() {
                                break;
                            }
                        }
                        end = Some(char_offset);
                    }
                } else {
                    // BOF
                    if !stack.is_empty() || (allow_empty_segments && end.is_some()) {
                        stack.reverse();
                        let (content, bytes_trimmed) = flush(&mut stack);
                        data.push(Line::new(content, 0, end.unwrap() - bytes_trimmed));
                    }
                    break;
                }
            }
            data.reverse();
        },
    }

    let s = data.first().map(|segment| segment.start);
    let e = data.last().map(|segment| segment.end);
    Ok(Data {
        lines: data,
        start: s,
        end: e
    })
}

/// Represents source of lines.
/// Keeps offset always at the beginning of the line (or at EOF).
/// The underlying DataSource is kept open, implementation is stateful
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

    fn read_raw(&self, start: Integer, end: Integer) -> Result<String, ()>;

    /// Reads designated number of words starting from the given offset.
    ///
    /// If the offset is in the middle of the word, this word is included in the result independently
    /// of the direction.
    /// Result contains tuple with words and offset.
    fn read_words(&mut self, offset: Integer, number_of_words: Integer) -> Result<(Vec<String>, Integer), ()>;
}

pub trait LineSourceBackend<R: Read> {
    fn get_length(&self) -> u64;

    fn new_reader(&self) -> BufReader<R>;
}

pub struct FileBackend {
    file_name: PathBuf
}

impl FileBackend {
    fn new(file_name: PathBuf) -> Self {
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
    file_reader: Option<Shared<BufReader<R>>>
}

impl<R, B> LineSourceImpl<R, B> where R: Read + Seek, B: LineSourceBackend<R> {
    pub fn from_file_name(file_name: PathBuf) -> LineSourceImpl<File, FileBackend> {
        LineSourceImpl {
            backend: FileBackend::new(file_name),
            file_reader: None
        }
    }

    pub fn from_str(s: &str) -> LineSourceImpl<Cursor<&[u8]>, StrBackend> {
        LineSourceImpl {
            backend: StrBackend::new(s),
            file_reader: None
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

impl<R, B> LineSource for LineSourceImpl<R, B> where R: Read + Seek, B: LineSourceBackend<R> {

    fn get_length(&self) -> Integer {
        self.backend.get_length().into()
    }

    fn read_lines(&mut self, offset: Integer, number_of_lines: Integer) -> Data {
        self.with_reader(|mut f| {
            log::trace!("read_lines number_of_lines = {}, offset = {}", number_of_lines, offset);
            read_delimited(&mut f, offset, number_of_lines, true, |c| *c == '\n')
        }).unwrap_or(Data::default())
    }

    fn read_raw(&self, start: Integer, end: Integer) -> Result<String, ()> {
        let mut f = self.backend.new_reader();
        f.seek(SeekFrom::Start(start.as_u64())).map_err(|_| ())?;
        let mut f = f.take((end - start).as_u64());
        let mut buf = [0 as u8; BUFFER_SIZE];
        let mut result = String::new();
        loop {
            let bytes_read = f.read(&mut buf);
            match bytes_read {
                Ok(0) => break,
                Ok(bytes_read) => result.push_str(&*String::from_utf8(Vec::from(&buf[0..bytes_read])).unwrap()),
                Err(_) => break
            }
        }
        Ok(result)
    }

    fn read_words(&mut self, offset: Integer, number_of_words: Integer) -> Result<(Vec<String>, Integer), ()> {
        self.with_reader(|mut f| {
            let is_delimiter = char::is_ascii_whitespace;
            let result = read_delimited(&mut f, offset, number_of_words, false, is_delimiter);
            result
                .map(|data| data.lines.into_iter().map(|line| line.content).collect())
                .map(|words| (words, f.stream_position().unwrap().into()))
                .map_err(|_| ())
        })
    }
}

// Tests are included according to http://xion.io/post/code/rust-unit-test-placement.html
#[cfg(test)]
#[path = "./data_source_tests.rs"]
mod data_source_tests;