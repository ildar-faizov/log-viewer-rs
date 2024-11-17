use std::io::{BufReader, Read, Seek};
use fluent_integer::Integer;

pub trait SeekTo {
    fn seek_to<I : Into<Integer>>(&mut self, pos: I) -> std::io::Result<()>;
}

impl<R: Read + Seek> SeekTo for BufReader<R> {
    fn seek_to<I : Into<Integer>>(&mut self, pos: I) -> std::io::Result<()> {
        let cur: Integer = self.stream_position()?.into();
        let shift = pos.into() - cur;
        if shift == 0 {
            return Ok(());
        }
        self.seek_relative(shift.as_i64())
    }
}