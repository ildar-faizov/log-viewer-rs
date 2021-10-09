use crossbeam_channel::{Receiver, Sender, unbounded};
use std::path::{Path, PathBuf};
use std::io::{Error, Read, Seek, ErrorKind, BufReader, BufRead, SeekFrom};
use std::io::SeekFrom::Start;
use std::ptr::write_bytes;
use std::str::from_utf8;
use std::fs::File;
use std::cmp::max;

pub const BUFFER_SIZE: u64 = 8192;

#[derive(Debug)]
pub struct Data {
    pub lines: Vec<String>,
    pub offset: u64,
    pub next_offset: u64
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
                    return offset + i as u64
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
        log::trace!("FileSource#data offset={}, length={}", start_offset, length);
        let mut offset = start_offset;
        let mut buffer = [0 as u8; BUFFER_SIZE as usize];
        let mut lines = vec![];
        let mut line = String::new();
        loop {
            let bytes_read = if offset < length {
                f.read(&mut buffer).unwrap_or(0)
            } else {
                0
            };
            log::trace!("FileSource#data offset={}, bytes_read={}", offset, bytes_read);
            if bytes_read > 0 {
                for i in 0..bytes_read {
                    let ch = buffer[i];
                    offset += 1;
                    if ch != '\r' as u8 && ch != '\n' as u8 {
                        line.push(ch as char)
                    } else if ch == '\n' as u8 {
                        lines.push(line);
                        line = String::new();
                        if offset >= length {
                            break;
                        }
                    }
                }
            } else {
                return Ok(Data{
                    lines,
                    offset: start_offset,
                    next_offset: offset
                })
            }
        }
    }

    fn receiver(&self) -> &Receiver<DataSourceUpdateEvent> {
        &self.receiver
    }
}
