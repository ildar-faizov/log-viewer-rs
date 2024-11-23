use crate::advanced_io::advanced_buf_reader::BidirectionalBufRead;
use crate::advanced_io::seek_to::SeekTo;
use crate::bounded_vec_deque::BoundedVecDeque;
use crate::data_source::filtered::filtered_line_source::LineFilter;
use fluent_integer::Integer;
use std::cmp::{min, Ordering};
use std::fmt::{Debug, Formatter};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::sync::Arc;
use thiserror::Error;
use crate::tout;

pub struct FilteredReader<R: Read + Seek> {
    cache: Cache<R>,
    p: usize, // number of bytes read from next line in cache
}

impl<R> FilteredReader<R>
where
    R: Read + Seek,
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

    fn do_read(
        &mut self,
        limit: usize,
        mut collector: impl FnMut(&[u8]),
    ) -> std::io::Result<usize> {
        let mut bytes_read = 0;
        while bytes_read < limit {
            let item = match self.cache.next() {
                Ok(t) => t,
                Err(FilterError::IO(err)) => return Err(err),
                Err(FilterError::EOF) => return Ok(bytes_read),
                Err(FilterError::BOF) => return Ok(bytes_read),
            };
            let bytes: &[u8] = &item.bytes[self.p..];
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

    fn skip_backwards(
        &mut self,
        limit: usize,
    ) -> std::io::Result<usize> {
        let mut bytes_read = 0;
        if self.p > 0 {
            bytes_read += min(self.p, limit);
            self.p -= bytes_read;
        }
        while bytes_read < limit {
            let item = match self.cache.prev() {
                Ok(t) => t,
                Err(FilterError::IO(err)) => return Err(err),
                Err(FilterError::EOF) => return Ok(bytes_read),
                Err(FilterError::BOF) => return Ok(bytes_read),
            };
            let m = item.bytes.len();
            let n = min(limit - bytes_read, m);
            bytes_read += n;
            self.p = m - n;
        }

        Ok(bytes_read)
    }
}

impl<R> Read for FilteredReader<R>
where
    R: Read + Seek,
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
where
    R: Read + Seek,
{
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match pos {
            SeekFrom::Start(from_start) => {
                self.reset()?;
                self.do_read(from_start as usize, |_| {})
            }
            SeekFrom::End(from_end) => {
                // Warning: this is ineffective, but the only way to return correct position from the start
                let size = self.do_read(usize::MAX, |_| {})?;
                if from_end >= 0 {
                    return Ok(size as u64);
                }
                Ok(size - self.skip_backwards(-from_end as usize)?)
            }
            SeekFrom::Current(delta) => {
                match delta.cmp(&0) {
                    Ordering::Less => {
                        self.skip_backwards(-delta as usize)?;
                    }
                    Ordering::Equal => (),
                    Ordering::Greater => {
                        self.do_read(delta as usize, |_| {})?;
                    }
                };
                Ok(self.cache.pos() as usize + self.p)
            },
        }.map(|b| b as u64)
    }
}

#[derive(Error, Debug)]
enum FilterError {
    #[error("Internal IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("End of file reached without success")]
    EOF,
    #[error("Beginning of file reached without success")]
    BOF,
}

#[derive(Clone)]
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

impl Debug for CacheItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}:{:?} {:?}",
                                 self.offset,
                                 String::from_utf8_lossy(&self.bytes),
                                 self.is_ok))
    }
}

struct Cache<R: Read + Seek> {
    neighbourhood: usize,
    future: SubCache,
    history: SubCache,
    filter: LineFilter,
    reader: BufReader<R>,
    pos: u64,
}

impl<R: Read + Seek> Cache<R> {
    fn new(neighbourhood: usize, filter: LineFilter, reader: BufReader<R>) -> Self {
        Cache {
            neighbourhood,
            future: SubCache::with_capacity(neighbourhood + 1),
            history: SubCache::with_capacity(neighbourhood + 1),
            filter,
            reader,
            pos: 0,
        }
    }

    fn next(&mut self) -> Result<CacheItem, FilterError> {
        tout!("next begin: {:?}", &self);

        if let Some(last) = self.future.back().or(self.history.front()) {
            self.reader.seek_to(Integer::from(last.offset) + last.bytes.len())?;
        }
        let item = Self::take(
            self.neighbourhood,
            self.filter.clone(),
            &mut self.reader,
            &mut self.future,
            &mut self.history,
            |reader| {
                let mut buf = vec![];
                let offset = reader.stream_position()?;
                let bytes_read = reader.read_until(b'\n', &mut buf)?;
                if bytes_read == 0 {
                    return Err(FilterError::EOF);
                }

                Ok((offset, buf))
            },
        )?;
        self.pos += item.bytes.len() as u64;

        tout!("next end: {:?}", self);

        Ok(item)
    }

    fn prev(&mut self) -> Result<CacheItem, FilterError> {
        tout!("prev begin: {:?}", &self);

        if let Some(first) = self.history.back().or(self.future.front()) {
            self.reader.seek_to(first.offset)?;
        }
        let item = Self::take(
            self.neighbourhood,
            self.filter.clone(),
            &mut self.reader,
            &mut self.history,
            &mut self.future,
            |reader| {
                let mut buf = vec![];
                let bytes_read = reader.read_fluently(-1, |b| buf.push(b[0]))?;
                if bytes_read == 0 {
                    return Err(FilterError::BOF);
                }
                reader.read_backwards_until(|b| b == b'\n', |b| buf.push(b))?;
                buf.reverse();
                let offset = reader.stream_position()?;

                Ok((offset, buf))
            },
        )?;
        self.pos -= item.bytes.len() as u64;
        tout!("prev end: {:?}", &self);
        Ok(item)
    }

    fn take(
        neighbourhood: usize,
        filter: LineFilter,
        reader: &mut BufReader<R>,
        future: &mut SubCache,
        history: &mut SubCache,
        line_reader: impl Fn(&mut BufReader<R>) -> Result<(u64, Vec<u8>), FilterError>,
    ) -> Result<CacheItem, FilterError> {
        tout!("take begin: h={:?} f={:?}", history, future);
        let item = loop {
            if future.nearest_match().is_some() {
                if let Some(item) = future.pop_front() {
                    break item;
                }
            }

            if let Some(echo) = history.nearest_match() {
                if *echo < neighbourhood {
                    let item = future.pop_front()
                        .map_or_else(|| Self::read_cache_item(reader, &filter, &line_reader), Ok)?;
                    break item;
                }
            }

            let cache_item = Self::read_cache_item(reader, &filter, &line_reader)?;
            if let Some(el) = future.push_back(cache_item) {
                history.push_front(el);
            }
        };
        history.push_front(item.clone());
        tout!("take end: h={:?} f={:?}", history, future);
        Ok(item)
    }

    fn read_cache_item(
        reader: &mut BufReader<R>,
        filter: &LineFilter,
        line_reader: &impl Fn(&mut BufReader<R>) -> Result<(u64, Vec<u8>), FilterError>,
    ) -> Result<CacheItem, FilterError> {
        let (offset, buf) = line_reader(reader)?;
        let is_ok = {
            let line = String::from_utf8_lossy(&buf);
            !filter(&line).is_empty()
        };
        Ok(CacheItem::new(offset, buf, is_ok))
    }

    fn restore(&mut self) {
        let Some(item) = self.history.pop_front() else {
            return;
        };
        self.pos -= item.bytes.len() as u64;
        self.future.push_front(item);
    }

    fn reset(&mut self) -> std::io::Result<()> {
        self.future.clear();
        self.history.clear();
        self.pos = 0;
        self.reader.seek(SeekFrom::Start(0)).map(|_| ())
    }

    fn pos(&self) -> u64 {
        self.pos
    }
}

#[derive(Debug)]
struct SubCache {
    items: BoundedVecDeque<CacheItem>,
    matching_indices: Vec<usize>,
}

impl SubCache {
    pub fn with_capacity(capacity: usize) -> Self {
        SubCache {
            items: BoundedVecDeque::with_capacity(capacity),
            matching_indices: vec![],
        }
    }

    fn nearest_match(&self) -> Option<&usize> {
        self.matching_indices.first()
    }

    fn push_back(&mut self, value: CacheItem) -> Option<CacheItem> {
        let is_ok = value.is_ok;
        let extra = self.items.push_back(value);
        if let Some(e) = extra.as_ref() {
            if e.is_ok {
                self.matching_indices.remove(0);
                self.alter_indices(|i| *i -= 1)
            }
        }
        if is_ok {
            self.matching_indices.push(self.items.len() - 1);
        }

        extra
    }

    fn push_front(&mut self, value: CacheItem) -> Option<CacheItem> {
        let is_ok = value.is_ok;
        let extra = self.items.push_front(value);
        if extra.as_ref().map(|item| item.is_ok).unwrap_or(false) {
            self.matching_indices.pop();
        }
        self.alter_indices(|i| *i += 1);
        if is_ok {
            self.matching_indices.insert(0, 0);
        }
        extra
    }

    fn pop_front(&mut self) -> Option<CacheItem> {
        let res = self.items.pop_front();
        if res.as_ref().map(|item| item.is_ok).unwrap_or(false) {
            self.matching_indices.remove(0);
        }
        if res.is_some() {
            self.alter_indices(|i| *i -= 1);
        }
        res
    }

    fn front(&self) -> Option<&CacheItem> {
        self.items.front()
    }

    fn back(&self) -> Option<&CacheItem> {
        self.items.back()
    }

    fn clear(&mut self) {
        self.items.clear();
        self.matching_indices.clear();
    }

    fn alter_indices(&mut self, op: impl Fn(&mut usize)) {
        for el in self.matching_indices.iter_mut() {
            op(el)
        }
    }
}

impl<R: Read + Seek> Debug for Cache<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Cache history={:?} future={:?}, pos={:?}", self.history, self.future, self.pos))
    }
}
