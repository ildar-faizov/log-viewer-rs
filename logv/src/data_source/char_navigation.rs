use crate::utils;
use crate::utils::utf8::UtfChar;
use std::io::{BufReader, Read, Seek};

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