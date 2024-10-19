use crate::advanced_io::advanced_buf_reader::BidirectionalBufRead;
use crate::advanced_io::seek_to::SeekTo;
use crate::data_source::filtered::filtered_line_source::LineFilter;
use crate::data_source::filtered::offset_mapper::{OriginalOffset, ProxyOffset};
use std::cmp::{max, min, Ordering};
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::sync::Arc;
use thiserror::Error;

pub struct FilteredReader<R>
where R: Read + Seek
{
    reader: BufReader<R>,
    current_position: ProxyOffset,
    original_offset: Option<OriginalOffset>,
    cache: Cache,
}

impl<R> FilteredReader<R>
where R: Read + Seek
{
    pub fn new(reader: BufReader<R>, filter: LineFilter, neighbourhood: u8) -> Self {
        FilteredReader {
            reader,
            current_position: ProxyOffset::from(0),
            original_offset: None,
            cache: Cache::new(neighbourhood as usize, Arc::clone(&filter)),
        }
    }

    fn ensure_original_offset(&mut self) -> Result<OriginalOffset, FilterError> {
        if self.original_offset.is_none() {
            let next_line = self.next_line()?;
            self.original_offset = Some(OriginalOffset::from(next_line.offset));
            self.cache.push_front(next_line);
        }
        self.original_offset.ok_or(FilterError::EOF)
    }

    fn next_line(&mut self) -> Result<CacheItem, FilterError> {
        self.cache.next(&mut self.reader)
    }

    fn reset(&mut self) -> std::io::Result<()> {
        self.reader.seek(SeekFrom::Start(0))?;
        self.cache.reset();
        self.current_position = ProxyOffset::from(0);
        self.original_offset = None;
        Ok(())
    }

    fn do_read(&mut self, limit: usize, mut collector: impl FnMut(&[u8])) -> std::io::Result<usize> {
        let mut original_offset: u64 = match self.ensure_original_offset() {
            Ok(t) => t,
            Err(FilterError::IO(err)) => return Err(err),
            Err(FilterError::EOF) => return Ok(0),
        }.as_u64();
        self.reader.seek_to(original_offset)?;
        self.reader.read_backwards_until(is_newline, drop)?;

        let mut bytes_read = 0;
        while bytes_read < limit {
            let CacheItem {
                offset, bytes, is_ok
            } = match self.next_line() {
                Ok(t) => t,
                Err(FilterError::IO(err)) => return Err(err),
                Err(FilterError::EOF) => return Ok(bytes_read),
            };
            original_offset = max(original_offset, offset);
            let m = bytes.len();
            if original_offset <= offset + m as u64 {
                let d = (original_offset - offset) as usize;
                let n = min(limit - bytes_read, m - d);
                if n > 0 {
                    collector(&bytes[d..d + n]);
                    bytes_read += n;
                    original_offset += n as u64;
                    self.current_position += bytes_read;
                    if d + n < m {
                        // current line was not read fully, so return it to cache
                        self.cache.push_front(CacheItem::new(offset, bytes, is_ok));
                    }
                } else {
                    break
                }
            }
            self.original_offset = Some(OriginalOffset::from(original_offset));
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
            SeekFrom::End(from_end) => {todo!()}
            SeekFrom::Current(delta) => {
                match delta.cmp(&0) {
                    Ordering::Less => {todo!()}
                    Ordering::Equal =>
                        Ok(self.current_position.as_u64()),
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

struct Cache {
    neighbourhood: usize,
    cache: VecDeque<CacheItem>,
    echo: usize,
    has_match: bool,
    filter: LineFilter,
}

impl Cache {
    fn new(neighbourhood: usize, filter: LineFilter) -> Self {
        Cache {
            neighbourhood,
            cache: VecDeque::with_capacity(2 * neighbourhood + 1),
            echo: 0,
            has_match: false,
            filter,
        }
    }

    fn next<R>(&mut self, reader: &mut BufReader<R>) -> Result<CacheItem, FilterError>
    where
        R: Read + Seek
    {
        loop {
            if self.has_match {
                if let Some(item) = self.cache.pop_front() {
                    if item.is_ok {
                        self.has_match = false;
                    }
                    reader.seek(SeekFrom::Start(item.offset + item.bytes.len() as u64))?;
                    break Ok(item);
                }
            }

            let mut buf = vec![];
            let offset = reader.stream_position()?;
            let bytes_read = reader.read_until(b'\n', &mut buf)?;
            if bytes_read == 0 {
                break Err(FilterError::EOF);
            }

            let is_ok = {
                let line = String::from_utf8_lossy(&buf);
                !(self.filter)(&line).is_empty()
            };

            let cache_item = CacheItem::new(offset, buf, is_ok);
            while self.cache.len() > self.neighbourhood {
                self.cache.pop_front();
            }

            if is_ok {
                self.echo = self.neighbourhood;
                self.has_match = true;
            } else if self.echo > 0 {
                self.echo -= 1;
                break Ok(cache_item);
            }
            self.cache.push_back(cache_item);
        }
    }

    fn push_front(&mut self, item: CacheItem) {
        if item.is_ok {
            self.has_match = true;
            self.echo = self.neighbourhood;
        }
        self.cache.push_front(item);
    }

    fn reset(&mut self) {
        self.cache.clear();
        self.echo = 0;
        self.has_match = false;
    }
}