use std::io::{BufRead, BufReader, ErrorKind, Read, Seek};
use std::ops::Deref;
use fluent_integer::Integer;
use crate::advanced_io::advanced_buf_reader::BidirectionalBufRead;
use crate::advanced_io::seek_to::SeekTo;

#[derive(Debug)]
pub struct RawLine {
    bytes: Vec<u8>,
    start: u64,
}

impl RawLine {
    pub fn new(bytes: Vec<u8>, start: u64) -> RawLine {
        RawLine { bytes, start }
    }

    pub fn read_from<R: Read + Seek, I: Into<Integer>>(
        reader: &mut BufReader<R>,
        offset: I,
    ) -> std::io::Result<RawLine> {
        reader.seek_to(offset)?;
        let mut buf = Vec::new();
        reader.read_backwards_until(|b| b == b'\n', drop_byte)?;
        let start = reader.stream_position()?;
        let bytes_read = reader.read_until(b'\n', &mut buf)?;
        if bytes_read == 0 {
            return Err(std::io::Error::from(ErrorKind::UnexpectedEof));
        }
        Ok(Self::new(buf, start))
    }

    pub fn read_backwards_from<R: Read + Seek, I: Into<Integer>>(
        reader: &mut BufReader<R>,
        offset: I,
    ) -> std::io::Result<RawLine> {
        reader.seek_to(offset)?;
        let mut buf = Vec::new();
        let bytes_read = reader.read_backwards_until(is_new_line, drop_byte)?;
        if bytes_read == 0 {
            return if reader.stream_position()? > 0 {
                reader.seek_relative(-1)?;
                reader.read_backwards_until(is_new_line, |b| buf.push(b))?;
                buf.reverse();
                Ok(Self::new(buf, reader.stream_position()?))
            } else {
                Err(std::io::Error::from(ErrorKind::UnexpectedEof))
            }
        }
        let start = reader.stream_position()?;
        reader.read_until(b'\n', &mut buf)?;
        Ok(Self::new(buf, start))
    }

    pub fn start(&self) -> u64 {
        self.start
    }

    pub fn end(&self) -> u64 {
        self.start + self.bytes.len() as u64
    }
}

impl Deref for RawLine {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

fn is_new_line(b: u8) -> bool {
    b == b'\n'
}

fn drop_byte(_: u8) {

}