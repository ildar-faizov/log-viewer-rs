use std::cmp::{min, Ordering};
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::ops::{Deref, Sub};
use crate::data_source::line_registry::{LineRegistry, LineRegistryImpl};
use crate::data_source::{Data, Direction, Line, LineSource, LineSourceBackend, LineSourceImpl};
use fluent_integer::Integer;
use std::sync::Arc;
use anyhow::anyhow;
use itertools::Itertools;
use mucow::MuCow;
use regex::Regex;
use crate::data_source::filtered::offset_mapper::{OffsetEvaluationResult, OffsetMapper, OriginalOffset, ProxyOffset};
use crate::data_source::line_source_holder::ConcreteLineSourceHolder;
use crate::data_source::tokenizer::skip_token;
use crate::interval::PointLocationWithRespectToInterval;
use crate::model::rendered::LineNumberMissingReason;
use crate::utils;

pub struct FilteredLineSource
{
    original: ConcreteLineSourceHolder,
    filter: Box<dyn Fn(&Line) -> bool>,
    offset_mapper: OffsetMapper,
    track_line_number: bool,
    pivots: Vec<(ProxyOffset, ProxyOffset)>,
    line_registry: Arc<LineRegistryImpl>,
}

impl LineSource for FilteredLineSource {
    fn get_length(&self) -> Integer {
        self.original.get_length()
    }

    fn read_lines(&mut self, mut offset: Integer, number_of_lines: Integer) -> Data {
        let (n, sign) = utils::sign(number_of_lines);
        let direction = match sign.cmp(&0) {
            Ordering::Less => Direction::Backward,
            Ordering::Greater => Direction::Forward,
            Ordering::Equal => return Data::default(),
        };
        let mut lines = Vec::with_capacity(n.as_usize());
        while lines.len() < n {
            match direction {
                Direction::Forward => {
                    if let Some(line) = self.read_next_line(offset) {
                        offset = line.end + 1;
                        lines.push(line);
                    } else {
                        break;
                    }
                },
                Direction::Backward => {
                    if let Some(line) = self.read_prev_line(offset) {
                        offset = line.start - 1;
                        lines.push(line);
                    } else {
                        break;
                    }
                },
            }
        }

        if direction == Direction::Backward {
            lines.reverse();
        }

        let start = lines.first().map(|line| line.start);
        let end = lines.last().map(|line| line.end);
        Data {
            lines,
            start,
            end,
        }
    }

    fn read_next_line(&mut self, offset: Integer) -> Option<Line> {
        self.poll(ProxyOffset::from(offset))
    }

    fn read_prev_line(&mut self, offset: Integer) -> Option<Line> {
        self.poll(ProxyOffset::from(offset))
    }

    fn track_line_number(&mut self, track: bool) {
        if self.track_line_number != track {
            self.original.track_line_number(track);
            self.track_line_number = track;
        }
    }

    fn read_raw(&mut self, start: Integer, end: Integer) -> Result<String, ()> {
        if start > end {
            return Err(());
        }
        if start == end {
            return Ok(String::with_capacity(0));
        }
        let capacity = (end - start).as_usize();
        let mut buffer = LimitedBuf::new(capacity);
        self.read_raw_internal(start, &mut buffer);
        buffer.to_string().map_err(|_| ())
    }

    fn skip_token(&mut self, offset: Integer, direction: Direction) -> anyhow::Result<Integer> {
        let cursor = LocalCursor::new(self);
        let mut f = BufReader::new(cursor);
        skip_token(offset, direction, &mut f)
    }

    fn get_line_registry(&self) -> Arc<LineRegistryImpl> {
        Arc::clone(&self.line_registry)
    }
}

impl FilteredLineSource {
    pub fn new(
        original: ConcreteLineSourceHolder,
        mapper: Box<dyn Fn(&Line) -> bool>,
    ) -> Self {
        FilteredLineSource {
            original,
            filter: mapper,
            offset_mapper: OffsetMapper::default(),
            track_line_number: true,
            pivots: Vec::new(),
            line_registry: Arc::new(LineRegistryImpl::new()),
        }
    }

    pub fn with_substring(
        original: ConcreteLineSourceHolder,
        pattern: &str
    ) -> Self {
        let pattern = pattern.to_string();
        let mapper = Box::new(move |ln: &Line| ln.content.contains(&pattern));
        Self::new(original, mapper)
    }

    pub fn get_original(self) -> ConcreteLineSourceHolder {
        self.original
    }

    fn poll(&mut self, offset: ProxyOffset) -> Option<Line> {
        let mut current_offset = offset.clone();
        loop {
            let next_line = match self.offset_mapper.eval(current_offset) {
                OffsetEvaluationResult::Exact(original_offset) => {
                    let d = original_offset - current_offset;
                    self.original.read_next_line(*original_offset)
                        .and_then(|line| {
                            let s = OriginalOffset::from(line.start) - d;
                            let e = OriginalOffset::from(line.end) - d;
                            Some(
                                line.to_builder()
                                    .with_start(*s)
                                    .with_end(*e)
                                    .build()
                            )
                        })
                }
                OffsetEvaluationResult::LastConfirmed(po, oo) => {
                    self.seek_next_line(po + 1, oo + 1)
                }
                OffsetEvaluationResult::Unpredictable => {
                    self.seek_next_line(ProxyOffset::default(), OriginalOffset::default())
                },
                OffsetEvaluationResult::Unreachable => None,
            };
            match next_line {
                None => return None,
                Some(next_line) => {
                    let interval = next_line.as_interval();
                    match interval.point_location(&*offset) {
                        PointLocationWithRespectToInterval::Undefined => return None,
                        PointLocationWithRespectToInterval::Less => {
                            current_offset = ProxyOffset::from(next_line.start - 1);
                        }
                        PointLocationWithRespectToInterval::Belongs => return Some(next_line),
                        PointLocationWithRespectToInterval::Greater => {
                            current_offset = ProxyOffset::from(next_line.end + 1);
                        }
                    }
                }
            }
        }
    }

    fn seek_next_line(&mut self, proxy_offset: ProxyOffset, original_offset: OriginalOffset) -> Option<Line> {
        match self.do_seek_next_line(proxy_offset, original_offset) {
            None => None,
            Some(line) => {
                self.line_registry.push(line.end);
                Some(line)
            }
        }
    }

    fn do_seek_next_line(&mut self, proxy_offset: ProxyOffset, original_offset: OriginalOffset) -> Option<Line> {
        let mut ox: Integer = *original_offset;
        while let Some(next_line) = self.original.read_next_line(ox) {
            let s = next_line.start;
            let e = next_line.end;
            if (*self.filter)(&next_line) {
                self.offset_mapper.add(proxy_offset, OriginalOffset::from(s)).unwrap();
                self.offset_mapper.confirm(proxy_offset + (e - s));
                return Some(Line {
                    content: next_line.content,
                    start: *proxy_offset,
                    end: e - s + *proxy_offset,
                    line_no: Err(LineNumberMissingReason::LineNumberingTurnedOff), // todo
                });
            }
            ox = e + 1;
        }

        // todo: do I need to map proxy_offset -> +Infinity
        None
    }

    fn read_raw_internal(&mut self, start: Integer, buffer: &mut LimitedBuf) {
        let mut offset = start;

        while !buffer.is_full() {
            let Some(line) = self.read_next_line(offset) else { break; };
            let skip = (offset - line.start).as_usize();
            let bytes = &line.content.as_bytes()[skip..];
            buffer.extend_from_slice(bytes);
            buffer.new_line();
            offset = line.end + 1;
        }
    }
}

struct LocalCursor<'a> {
    src: &'a mut FilteredLineSource,
    pos: Integer,
}

impl<'a> LocalCursor<'a> {
    fn new(src: &'a mut FilteredLineSource) -> Self {
        LocalCursor {
            src,
            pos: 0.into(),
        }
    }
}

impl<'a> Read for LocalCursor<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut buf = LimitedBuf::from_buf(buf);
        self.src.read_raw_internal(self.pos, &mut buf);
        self.pos += buf.position();
        Ok(buf.position())
    }
}

impl<'a> Seek for LocalCursor<'a> {
    fn seek(&mut self, shift: SeekFrom) -> std::io::Result<u64> {
        match shift {
            SeekFrom::Start(p) => {
                self.pos = p.into();
            },
            SeekFrom::End(_) => panic!("Seek from end is not supported for FilteredLineSource"), // todo better error
            SeekFrom::Current(delta) => {
                self.pos += delta;
                if self.pos < 0 {
                    panic!() // todo better error
                }
            }
        }
        Ok(self.pos.as_u64())
    }
}

struct LimitedBuf<'a> {
    buffer: MuCow<'a, [u8]>,
    limit: usize,
    p: usize,
}

impl LimitedBuf<'static> {
    fn new(capacity: usize) -> Self {
        LimitedBuf {
            buffer: MuCow::Owned(Vec::with_capacity(capacity)),
            limit: capacity,
            p: 0
        }
    }
}

impl<'a> LimitedBuf<'a> {

    fn from_buf(v: &'a mut [u8]) -> Self {
        let limit = v.len();
        LimitedBuf {
            buffer: MuCow::Borrowed(v),
            limit,
            p: 0,
        }
    }

    fn extend_from_slice(&mut self, slice: &[u8]) {
        let n = min(slice.len(), self.limit - self.p);
        if n == 0 {
            return;
        }
        let slice = &slice[..n];
        match &mut self.buffer {
            MuCow::Borrowed(buffer) => {
                let mut buffer = &mut buffer[self.p..self.p + n];
                buffer.copy_from_slice(slice);
            }
            MuCow::Owned(ref mut vec) => {
                vec.extend_from_slice(slice);
            }
        }
        self.p += n;
    }

    fn new_line(&mut self) {
        if !self.is_full() {
            match &mut self.buffer {
                MuCow::Borrowed(buffer) => buffer[self.p] = b'\n',
                MuCow::Owned(ref mut vec) => vec.push(b'\n'),
            }
            self.p += 1;
        }
    }

    fn is_full(&self) -> bool {
        self.p >= self.limit
    }

    fn to_string(self) -> anyhow::Result<String> {
        match self.buffer {
            MuCow::Borrowed(_) => Err(anyhow!("Not supported")),
            MuCow::Owned(v) => Ok(String::from_utf8(v)?),
        }
    }

    fn position(&self) -> usize {
        self.p
    }
}

#[cfg(test)]
#[path="./tests.rs"]
mod tests;

