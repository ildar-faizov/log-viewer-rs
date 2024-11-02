use crate::advanced_io::seek_to::SeekTo;
use crate::bounded_vec_deque::BoundedVecDeque;
use crate::data_source::filtered::filtered_line_source::LineFilter;
use std::cmp::{min, Ordering};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::sync::Arc;
use thiserror::Error;

pub struct FilteredReader<R: Read + Seek> {
    cache: Cache<R>,
    p: usize, // number of bytes read from next line in cache
}

impl<R> FilteredReader<R>
where R: Read + Seek
{
    pub fn new(reader: BufReader<R>, filter: LineFilter, neighbourhood: u8) -> Self {
        FilteredReader {
            cache: Cache::new(neighbourhood as usize, Arc::clone(&filter), reader),
            p: 0,
        }
    }

    fn reset(&mut self) -> std::io::Result<()> {
        self.cache.reset()?;
        Ok(())
    }

    fn do_read(&mut self, limit: usize, mut collector: impl FnMut(&[u8])) -> std::io::Result<usize> {
        let mut bytes_read = 0;
        while bytes_read < limit {
            let item = match self.cache.next() {
                Ok(t) => t,
                Err(FilterError::IO(err)) => return Err(err),
                Err(FilterError::EOF) => return Ok(bytes_read),
            };
            let bytes : &[u8] = &item.bytes[self.p..];
            let m = bytes.len();
            let n = min(limit - bytes_read, m);
            collector(&bytes[..n]);
            bytes_read += n;
            if n < m {
                // line not read fully
                self.cache.restore();
                self.p += n;
            } else {
                self.p = 0;
            }
        }

        Ok(bytes_read)
    }
}

impl<R> Read for FilteredReader<R>
where R: Read + Seek
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut bytes_read = 0;
        self.do_read(buf.len(), |bytes| {
            let n = bytes.len();
            buf[bytes_read..bytes_read + n].copy_from_slice(bytes);
            bytes_read += n;
        })
    }
}

impl<R> Seek for FilteredReader<R>
where R: Read + Seek
{
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match pos {
            SeekFrom::Start(from_start) => {
                self.reset()?;
                self.do_read(from_start as usize, |_| {}).map(|b| b as u64)
            }
            SeekFrom::End(_from_end) => {todo!()}
            SeekFrom::Current(delta) => {
                match delta.cmp(&0) {
                    Ordering::Less => {todo!()}
                    Ordering::Equal =>
                        Ok(self.cache.pos() + self.p as u64),
                    Ordering::Greater =>
                        self.do_read(delta as usize, |_| {}).map(|b| b as u64),
                }
            }
        }
    }
}

#[derive(Error, Debug)]
enum FilterError {
    #[error("Internal IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("End of file reached without success")]
    EOF,
}

fn is_newline(b: u8) -> bool {
    b == b'\n' || b == b'\r'
}

#[derive(Debug, Clone)]
struct CacheItem {
    offset: u64,
    bytes: Vec<u8>,
    is_ok: bool,
}

impl CacheItem {
    fn new(offset: u64, bytes: Vec<u8>, is_ok: bool) -> Self {
        CacheItem {
            offset,
            bytes,
            is_ok,
        }
    }
}

struct Cache<R: Read + Seek> {
    neighbourhood: usize,
    future: BoundedVecDeque<CacheItem>,
    history: BoundedVecDeque<CacheItem>,
    echo: usize,
    has_match: bool,
    filter: LineFilter,
    reader: BufReader<R>,
    pos: u64,
}

impl<R: Read + Seek> Cache<R> {
    fn new(neighbourhood: usize, filter: LineFilter, reader: BufReader<R>) -> Self {
        Cache {
            neighbourhood,
            future: BoundedVecDeque::with_capacity(neighbourhood + 1),
            history: BoundedVecDeque::with_capacity(neighbourhood),
            echo: 0,
            has_match: false,
            filter,
            reader,
            pos: 0,
        }
    }

    fn next(&mut self) -> Result<CacheItem, FilterError> {
        let res = loop {
            if self.has_match {
                if let Some(item) = self.future.pop_front() {
                    if item.is_ok {
                        self.has_match = false;
                    }
                    break Ok(item);
                }
            }

            let mut buf = vec![];
            let offset = self.reader.stream_position()?;
            let bytes_read = self.reader.read_until(b'\n', &mut buf)?;
            if bytes_read == 0 {
                break Err(FilterError::EOF);
            }

            let is_ok = {
                let line = String::from_utf8_lossy(&buf);
                !(self.filter)(&line).is_empty()
            };

            let cache_item = CacheItem::new(offset, buf, is_ok);

            if is_ok {
                self.echo = self.neighbourhood;
                self.has_match = true;
            } else if self.echo > 0 {
                self.echo -= 1;
                break Ok(cache_item);
            }
            if let Some(el) = self.future.push_back(cache_item) {
                self.history.push_back(el);
            }
        };
        let item = res?;
        self.pos += item.bytes.len() as u64;
        self.history.push_back(item.clone());
        Ok(item)
    }

    // fn prev(&mut self) -> Result<CacheItem, FilterError> {
    //     let mut reader = &mut self.reader;
    //     let mut reverse_echo = self.cache.iter()
    //         .find_position(|item| item.is_ok)
    //         .map(|(p, _)| self.neighbourhood.saturating_sub(p))
    //         .unwrap_or(0);
    //     loop {
    //
    //     }
    // }

    fn restore(&mut self) {
        let Some(item) = self.history.pop_back() else { return; };
        if item.is_ok {
            self.has_match = true;
            self.echo = self.neighbourhood;
        }
        self.pos -= item.bytes.len() as u64;
        self.future.push_front(item);
    }

    fn reset(&mut self) -> std::io::Result<()> {
        self.future.clear();
        self.history.clear();
        self.echo = 0;
        self.has_match = false;
        self.pos = 0;
        self.reader.seek(SeekFrom::Start(0)).map(|_| ())
    }

    fn pos(&self) -> u64 {
        self.pos
    }
}