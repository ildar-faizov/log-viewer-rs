use crossbeam_channel::{Receiver, Sender, unbounded};
use std::path::{Path, PathBuf};
use std::io::{Error, Read, Seek, ErrorKind, BufReader, BufRead, SeekFrom};
use std::io::SeekFrom::Start;
use std::ptr::write_bytes;
use std::str::from_utf8;
use std::fs::File;
use std::cmp::{max, min};
use crate::utils;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use std::borrow::{BorrowMut, Borrow};
use crate::shared::Shared;

pub const BUFFER_SIZE: u64 = 8192;

#[derive(Debug, Default)]
pub struct Line {
    pub content: String, // TODO use appropriate type
    pub start: u64, // offset of the first symbol in line
    pub end: u64 // offset of the first symbol of the next line
}

impl Line {
    pub fn default_at(start: u64) -> Self {
        let mut line = Line::default();
        line.start = start;
        line
    }
}

#[derive(Debug)]
pub struct Data {
    pub lines: Vec<Line>,
    pub offset: u64,
}

pub trait DataSource {

    fn length(&self) -> Result<u64, Error>;

    /*
     * Returns integer number of lines (Data.lines) and correct offset of
     * the returned data (Data.offset), which is not greater than the offset passed
     * to the function
     */
    fn data(&self, offset: u64, length: u64) -> Result<Data, Error>; // TODO will become Future/Promise

    fn receiver(&self) -> &Receiver<DataSourceUpdateEvent>;
}

pub enum DataSourceUpdateEvent {
    LengthUpdated(u64),
    DataUpdated
}

pub struct FileSource {
    sender: Sender<DataSourceUpdateEvent>,
    receiver: Receiver<DataSourceUpdateEvent>,
    file_name: PathBuf,
}

impl FileSource {
    pub fn new(file_name: PathBuf) -> Self {
        let (sender, receiver) = unbounded();
        FileSource {
            sender,
            receiver,
            file_name
        }
    }

    fn seek_to_line_start(f: &mut BufReader<File>, offset: u64) -> u64 {
        log::trace!("FileSource#seek_to_line_start offset={}", offset);
        let mut offset = offset;
        let mut buffer = [0 as u8; BUFFER_SIZE as usize];
        loop {
            log::trace!("FileSource#seek_to_line_start loop offset={}", offset);
            let delta = if offset >= BUFFER_SIZE {
                BUFFER_SIZE
            } else {
                offset
            };
            if delta == 0 {
                break;
            }
            offset = f.seek(SeekFrom::Current(-1 * (delta as i64))).unwrap();
            let bytes_read = f.take(delta).read(&mut buffer).unwrap();
            for i in (0..bytes_read).rev() {
                if buffer[i] == '\n' as u8 {
                    let result = offset + (i + 1) as u64;
                    let pos = f.seek(SeekFrom::Current(i as i64 - bytes_read as i64 + 1)).unwrap();
                    log::trace!("FileSource#seek_to_line_start return {} file_pos={}", result, pos);
                    return result
                }
            }
        }
        log::trace!("FileSource#seek_to_line_start return {}", offset);
        offset
    }
}

impl DataSource for FileSource {

    fn length(&self) -> Result<u64, Error> {
        Ok(std::fs::metadata(self.file_name.as_path())?.len())
    }

    fn data(&self, offset: u64, length: u64) -> Result<Data, Error> {
        log::trace!("FileSource#data offset={}, length={}", offset, length);
        let mut f = BufReader::new(File::open(&self.file_name)?);
        f.seek(Start(offset))?;
        let start_offset = FileSource::seek_to_line_start(&mut f, offset);
        let length = offset - start_offset + length;
        let end_offset = start_offset + length;
        log::trace!("FileSource#data offset={}, length={}", start_offset, length);
        let mut offset = start_offset;
        let mut buffer = [0 as u8; BUFFER_SIZE as usize];
        let mut lines = vec![];
        let mut line = Line::default_at(offset);
        loop {
            let bytes_read = if offset < end_offset {
                (&mut f).take(end_offset - offset).read(&mut buffer).unwrap_or(0)
            } else {
                0
            };
            log::trace!("FileSource#data offset={}, bytes_read={}", offset, bytes_read);
            if bytes_read > 0 {
                for i in 0..bytes_read {
                    let ch = buffer[i];
                    if ch != '\r' as u8 && ch != '\n' as u8 {
                        line.content.push(ch as char);
                    } else if ch == '\n' as u8 {
                        line.end = offset;
                        lines.push(line);
                        line = Line::default_at(offset + 1);
                        if offset >= end_offset {
                            break;
                        }
                    }
                    offset += 1;
                }
            } else {
                if !line.content.is_empty() {
                    line.end = offset;
                    lines.push(line);
                }
                return Ok(Data{
                    lines,
                    offset: start_offset
                })
            }
        }
    }

    fn receiver(&self) -> &Receiver<DataSourceUpdateEvent> {
        &self.receiver
    }
}

/// Represents source of lines.
/// Keeps offset always at the beginning of the line (or at EOF).
/// The underlying DataSource is kept open, implementation is stateful
pub trait LineSource {

    /// Returns current offset
    fn get_offset(&self) -> u64;

    /// Sets offset to the nearest preceding beginning of line, returning it
    fn set_offset(&mut self, offset: u64) -> u64;

    /// Returns length
    fn get_length(&self) -> u64;

    /// Reads requested number of lines in any direction
    fn read_lines(&mut self, number_of_lines: isize) -> Vec<Line>;
}

pub struct LineSourceImpl {
    file_name: PathBuf,
    offset: u64,
    file_reader: Option<Shared<BufReader<File>>>
}

impl LineSourceImpl {
    pub fn new(file_name: PathBuf) -> Self {
        LineSourceImpl {
            file_name,
            offset: 0,
            file_reader: None
        }
    }

    fn file_reader(&mut self) -> RefMut<'_, BufReader<File>> {
        if self.file_reader.is_none() {
            let f = BufReader::new(File::open(&self.file_name).unwrap());
            self.file_reader = Some(Shared::new(f));
        }
        self.file_reader.as_ref().unwrap().get_mut_ref()
    }

    fn with_file_reader<T, U>(&mut self, f: T) -> U
        where T: FnOnce(RefMut<BufReader<File>>) -> U
    {
        let file_reader = self.file_reader();
        f(file_reader)
    }
}

impl LineSource for LineSourceImpl {
    fn get_offset(&self) -> u64 {
        self.offset
    }

    fn set_offset(&mut self, offset: u64) -> u64 {
        let result = self.with_file_reader(|mut f| {
            let offset = f.seek(SeekFrom::Start(offset)).unwrap();
            FileSource::seek_to_line_start(&mut *f, offset)
        });
        self.offset = result;
        log::trace!("set_offset {} -> {}", offset, result);
        result
    }

    fn get_length(&self) -> u64 {
        std::fs::metadata(self.file_name.as_path()).unwrap().len()
    }

    fn read_lines(&mut self, number_of_lines: isize) -> Vec<Line> {
        log::trace!("read_lines number_of_lines = {}, offset = {}", number_of_lines, self.offset);
        let mut result = Vec::with_capacity(number_of_lines.abs() as usize);
        let mut offset = self.offset;
        self.with_file_reader(|mut f| {
            if number_of_lines > 0 {
                let mut number_of_lines = number_of_lines;
                while number_of_lines > 0 {
                    let mut line = String::new();
                    match f.read_line(&mut line) {
                        Ok(0) => break,
                        Ok(bytes_read) => {
                            let mut bytes_read = bytes_read as u64;
                            let line_offset = offset;
                            offset += bytes_read;
                            bytes_read -= utils::trim_newline(&mut line) as u64;
                            result.push(Line {
                                content: line,
                                start: line_offset,
                                end: line_offset + bytes_read
                            });
                            number_of_lines -= 1;
                        },
                        Err(_) => break
                    }
                }
            } else if number_of_lines < 0 {
                let number_of_lines = (-1 * number_of_lines) as usize;

                let mut stack = Vec::with_capacity(BUFFER_SIZE as usize);
                let mut buf = [0 as u8; BUFFER_SIZE as usize];
                loop {
                    let delta = min(offset, BUFFER_SIZE);
                    log::trace!("read_lines offset = {}, delta = {}", offset, delta);
                    let file_offset = f.seek(SeekFrom::Current(delta as i64 * -1)).unwrap();
                    let bytes_read = {
                        f.by_ref().take(delta).read(&mut buf).unwrap()
                    };
                    log::trace!("read_lines file_offset={} bytes_read={}", file_offset, bytes_read);
                    for i in (0..bytes_read).rev() {
                        if buf[i] == 0x0A && !stack.is_empty() {
                            let mut current_stack = stack;
                            stack = Vec::with_capacity(BUFFER_SIZE as usize);
                            current_stack.reverse();
                            let line_length = current_stack.len() as u64;
                            let line_offset = offset - line_length;
                            let mut raw_string = String::from_utf8(current_stack).unwrap();
                            let b = utils::trim_newline(&mut raw_string);
                            result.push(Line {
                                content: raw_string,
                                start: line_offset,
                                end: line_offset + line_length - b as u64
                            });
                            offset = line_offset;
                            log::trace!("read_lines line {:?}, offset = {}", result.last(), offset);
                            if result.len() == number_of_lines {
                                break;
                            }
                        }
                        stack.push(buf[i]);
                        log::trace!("read_lines i={} stack = {:?}", i, stack);
                    }
                    if offset == 0 || result.len() == number_of_lines {
                        break;
                    }
                }
                f.seek(SeekFrom::Start(offset));

                result.reverse();
            }
        });
        self.offset = offset;
        log::trace!("read_lines offset = {}", self.offset);
        result
    }

}