use std::cmp::{max, min};
use std::io::{BufReader, Read, Seek};

pub trait BidirectionalBufRead {
    fn read_fluently<I, F>(&mut self, n: I, consumer: F) -> std::io::Result<usize>
    where
        I: Into<i128>,
        F: FnMut(&[u8]);

    fn read_backwards_until<P, C>(&mut self, predicate: P, collector: C) -> std::io::Result<usize>
    where
        P: Fn(u8) -> bool,
        C: FnMut(u8);
}

impl<R: Seek + Read> BidirectionalBufRead for BufReader<R> {
    fn read_fluently<I, F>(&mut self, n: I, mut consumer: F) -> std::io::Result<usize>
    where
        I: Into<i128>,
        F: FnMut(&[u8]),
    {
        let mut n = n.into();
        let mut buffer = [0_u8; 8192];
        let bytes_to_read = if n < 0 {
            let stream_position = self.stream_position()? as i128;
            n = max(n, -stream_position);
            self.seek_relative(n as i64)?;
            -n
        } else {
            n
        } as usize;
        let mut bytes_read = 0_usize;
        while bytes_read < bytes_to_read {
            let d = min((bytes_to_read - bytes_read) as usize, buffer.len());
            let b = self.read(&mut buffer[0..d])?;
            if b == 0 {
                break;
            }
            bytes_read += b;
            if n < 0 {
                let _ = &buffer[0..b].reverse();
            }
            consumer(&buffer[0..b]);
        }
        if n < 0 {
            self.seek_relative(-(bytes_read as i64))?;
        }
        Ok(bytes_read)
    }

    fn read_backwards_until<P, C>(&mut self, predicate: P, mut collector: C) -> std::io::Result<usize>
    where
        P: Fn(u8) -> bool,
        C: FnMut(u8),
    {
        let mut bytes_read = 0;
        let mut buf = [0_u8; 1];
        loop {
            if self.stream_position()? == 0 {
                break;
            }
            self.seek_relative(-1)?;
            if self.read(&mut buf)? == 1 && !predicate(buf[0]) {
                collector(buf[0]);
                bytes_read += 1;
                self.seek_relative(-1)?;
            } else {
                break;
            }
        }
        Ok(bytes_read)
    }
}
