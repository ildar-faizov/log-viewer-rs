use std::cmp::{min, Ordering};
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Error};
use mucow::MuCow;
use uuid::Uuid;

use crate::background_process::background_process_handler::BackgroundProcessHandler;
use crate::background_process::buffered_message_sender::BufferedMessageSender;
use crate::background_process::run_in_background::RunInBackground;
use crate::background_process::signal::Signal;
use crate::data_source::filtered::offset_mapper::{IOffsetMapper, OffsetDelta, OffsetEvaluationResult, OffsetMapper, OriginalOffset, ProxyOffset};
use crate::data_source::line_registry::{LineRegistry, LineRegistryImpl};
use crate::data_source::line_source_holder::{ConcreteLineSourceHolder, LineSourceHolder};
use crate::data_source::tokenizer::skip_token;
use crate::data_source::{CustomHighlight, Data, Direction, Line, LineSource, LineSourceBackend};
use crate::interval::PointLocationWithRespectToInterval;
use crate::model::model::RootModel;
use crate::utils;
use fluent_integer::Integer;
use crate::background_process::task_context::TaskContext;
use crate::data_source::filtered::foreseeing_filter::{ForeseeingFilter, ForeseeingFilterResult};
use crate::data_source::reader_factory::filtered::FilteredReaderFactory;
use crate::data_source::reader_factory::{HasReaderFactory, ReaderFactory};

pub type LineFilter = Arc<dyn Fn(&str) -> Vec<CustomHighlight> + Sync + Send + 'static>;

pub type Callback = Box<dyn Fn(&mut RootModel) + 'static>;

const PUSH_INTERVAL: Duration = Duration::from_millis(500);
const MESSAGE_LIMIT: usize = 1024;

pub const FILTERED_LINE_SOURCE_CUSTOM_DATA_KEY: &str = "FilteredLineSourceCustomData";

pub struct FilteredLineSource
{
    id: Uuid,
    original: ConcreteLineSourceHolder,
    original_filter: LineFilter,
    filter: ForeseeingFilter,
    neighbourhood: u8,
    offset_mapper: OffsetMapper,
    track_line_number: bool,
    line_registry: Arc<LineRegistryImpl>,
    handler: Option<BackgroundProcessHandler>,
    highest_scanned_original_offset: OriginalOffset,
}

impl LineSource for FilteredLineSource {

    fn read_lines(&mut self, mut offset: Integer, number_of_lines: Integer) -> Data {
        let (n, sign) = utils::sign(number_of_lines);
        let direction = match sign.cmp(&0) {
            Ordering::Less => Direction::Backward,
            Ordering::Greater => Direction::Forward,
            Ordering::Equal => return Data::default(),
        };
        let mut lines = Vec::with_capacity(n.as_usize());
        while lines.len() < n && offset >= 0 {
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
        filter: LineFilter,
        neighbourhood: u8,
    ) -> Self {
        let foreseeing_filter = ForeseeingFilter::new(Arc::clone(&filter), neighbourhood);
        FilteredLineSource {
            id: Uuid::new_v4(),
            original,
            original_filter: filter,
            filter: foreseeing_filter,
            neighbourhood,
            offset_mapper: OffsetMapper::default(),
            track_line_number: true,
            line_registry: Arc::new(LineRegistryImpl::new()),
            handler: None,
            highest_scanned_original_offset: Default::default(),
        }
    }

    pub fn with_substring(
        original: ConcreteLineSourceHolder,
        pattern: &str,
        neighbourhood: u8,
    ) -> Self {
        let pattern = pattern.to_string();
        let mapper = Arc::new(move |s: &str|
            s.match_indices(&pattern)
                .map(|(i, m)| CustomHighlight::new(i, i + m.len()))
                .collect()
        );
        Self::new(original, mapper, neighbourhood)
    }

    pub fn destroy(mut self) -> ConcreteLineSourceHolder {
        if let Some(handler) = &self.handler.as_mut() {
            handler.interrupt();
        }
        self.original
    }

    pub fn reader_factory(&self) -> Box<dyn ReaderFactory> {
        let factory = FilteredReaderFactory::new(
            self.original.reader_factory(),
            self.original_filter.clone(),
            self.neighbourhood
        );
        Box::new(factory)
    }

    pub fn get_length(&self) -> Option<Integer> {
        let original_length = self.original.get_length();
        if original_length <= *self.highest_scanned_original_offset {
            self.offset_mapper.get_highest_known()
                .map(|(proxy_offset, _)| *proxy_offset)
        } else {
            None
        }
    }

    pub fn build_offset_mapper<T: RunInBackground>(&mut self, runner: &mut T, on_finish: Callback) {
        match &self.original {
            ConcreteLineSourceHolder::FileBased(ls) => {
                self.build_offset_mapper_0(runner, ls.backend().clone(), on_finish);
            }
            ConcreteLineSourceHolder::ConstantBased(ls) => {
                self.build_offset_mapper_0(runner, ls.backend().clone(), on_finish);
            }
        }
    }

    fn build_offset_mapper_0<T, R, B>(&mut self, runner: &mut T, backend: B, on_finish: Callback)
    where
        T: RunInBackground,
        R: Read + Seek,
        B: LineSourceBackend<R> + Send + 'static
    {
        if self.handler.is_some() {
            return;
        }
        let self_id = self.id.clone();
        let filter = Arc::clone(&self.original_filter);
        let neighbourhood = self.neighbourhood as usize;
        let handler = runner.background_process_builder::<Vec<Message>, _, anyhow::Result<Integer>, _>()
            .with_title("Scanning...")
            .with_description("Building complete filtered output")
            .with_task(move |ctx| {
                Self::full_scan(backend, filter, neighbourhood, ctx)
            })
            .with_listener(move |model, msg, id| {
                match msg {
                    Signal::Custom(data) => {
                        perform_on_self(self_id, model, |ls| ls.accept_offset_mapper(data, id));
                    },
                    Signal::Complete(result) => {
                        perform_on_self(self_id, model, |ls| ls.accept_offset_mapper_finish(id, result));
                        on_finish(model);
                    }
                    _ => {}
                }
            })
            .run();
        self.handler = Some(handler);
    }

    fn full_scan<R, B>(
        backend: B,
        filter: Arc<dyn Fn(&str) -> Vec<CustomHighlight> + Sync + Send>,
        neighbourhood: usize,
        ctx: &mut TaskContext<Vec<Message>, anyhow::Result<Integer>>
    ) -> Result<Integer, Error>
    where
        R: Read + Seek,
        B: LineSourceBackend<R> + Send + 'static
    {
        let mut message_sender = BufferedMessageSender::new(MESSAGE_LIMIT, PUSH_INTERVAL, ctx);

        let total = backend.get_length();
        let mut proxy_offset = ProxyOffset::from(0);
        let mut original_offset = OriginalOffset::from(0);

        let mut reader = backend.new_reader();
        let mut cache: VecDeque<CacheItem> = VecDeque::with_capacity(neighbourhood + 1);
        let mut line = String::new();
        let mut echo = 0;
        while let Ok(bytes_read) = reader.read_line(&mut line) {
            if bytes_read == 0 {
                break;
            }
            let trimmed = utils::trim_newline(&mut line);
            cache.push_back(CacheItem {
                original_offset,
                line_length: bytes_read - trimmed,
                bytes_read,
            });
            while cache.len() > neighbourhood + 1 {
                cache.pop_front();
            }
            let is_match = !filter(&line).is_empty();
            if is_match || echo > 0 {
                if is_match {
                    echo = neighbourhood;
                } else {
                    echo -= 1;
                }
                while let Some(item) = cache.pop_front() {
                    message_sender.push(Message {
                        proxy_offset,
                        original_offset: item.original_offset,
                        line_length: item.line_length.into(),
                    })?;
                    proxy_offset = proxy_offset + item.bytes_read;
                }
            }
            original_offset = original_offset + bytes_read;
            ctx.update_progress_u64(original_offset.as_u64(), total);
            if ctx.interrupted_debounced(PUSH_INTERVAL) {
                return Err(anyhow!("Cancelled"));
            }
            line.clear();
        }

        Ok(*original_offset)
    }

    fn accept_offset_mapper(&mut self, msgs: Vec<Message>, id: &Uuid) {
        if self.handler.is_none() {
            log::warn!("Cannot update FilteredLineSource, it's been destroyed");
            return;
        }
        let handler = self.handler.as_ref().unwrap();
        if handler.get_id() != id {
            log::warn!("Trying to assign results of {} to {}", id, handler.get_id());
            return;
        }
        for msg in msgs {
            let Message {
                proxy_offset,
                original_offset,
                line_length,
            } = msg;
            self.offset_mapper.add(proxy_offset, original_offset).unwrap_or_default(); // skip error
            self.offset_mapper.confirm(proxy_offset + line_length);
            self.line_registry.push(*(proxy_offset + line_length));
            self.highest_scanned_original_offset = OriginalOffset::from(original_offset);
        }
    }

    fn accept_offset_mapper_finish(&mut self, id: &Uuid, res: anyhow::Result<Integer>) {
        if self.handler.is_none() {
            log::warn!("Cannot update FilteredLineSource, it's been destroyed");
            return;
        }
        let handler = self.handler.as_ref().unwrap();
        if handler.get_id() != id {
            log::warn!("Trying to assign results of {} to {}", id, handler.get_id());
            return;
        }
        self.handler = None;

        if let Ok(original_offset) = res {
            self.highest_scanned_original_offset = OriginalOffset::from(original_offset);
        }
    }

    fn poll(&mut self, offset: ProxyOffset) -> Option<Line> {
        if offset < 0.into() {
            return None
        }
        let mut current_offset = offset.clone();
        loop {
            let next_line = match self.offset_mapper.eval(current_offset) {
                OffsetEvaluationResult::Exact(original_offset) => {
                    let d = original_offset - current_offset;
                    let filter_result = self.filter.apply(&mut self.original, *original_offset);
                    match filter_result {
                        ForeseeingFilterResult::PreciseMatch(ln, matches) =>
                            Some(self.convert_line(ln, matches, d)),
                        ForeseeingFilterResult::NeighbourMatch(ln) =>
                            Some(self.convert_line(ln, Vec::new(), d)),
                        _ => None,
                    }
                }
                OffsetEvaluationResult::LastConfirmed(po, oo) => {
                    self.seek_next_line(po + 1, oo + 1)
                }
                OffsetEvaluationResult::Unpredictable => {
                    self.seek_next_line(ProxyOffset::default(), OriginalOffset::default())
                },
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
                        PointLocationWithRespectToInterval::Belongs => {
                            return Some(next_line)
                        },
                        PointLocationWithRespectToInterval::Greater => {
                            current_offset = ProxyOffset::from(next_line.end + 1);
                        }
                    }
                }
            }
        }
    }

    fn convert_line(&self, line: Line, matches: Vec<CustomHighlight>, d: OffsetDelta) -> Line {
        let s = OriginalOffset::from(line.start) - d;
        let e = OriginalOffset::from(line.end) - d;
        line.to_builder()
            .with_start(*s)
            .with_end(*e)
            .with_multiple_custom_highlights(FILTERED_LINE_SOURCE_CUSTOM_DATA_KEY, matches)
            .build()
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
        loop {
            match self.filter.apply(&mut self.original, ox) {
                ForeseeingFilterResult::PreciseMatch(line, matches) => {
                    break Some(self.accept_line(proxy_offset, line, matches))
                }
                ForeseeingFilterResult::NeighbourMatch(line) => {
                    break Some(self.accept_line(proxy_offset, line, Vec::new()))
                }
                ForeseeingFilterResult::NoMatch(offset) => {
                    self.highest_scanned_original_offset = OriginalOffset::from(offset);
                    ox = offset;
                },
                ForeseeingFilterResult::EOF => break None,
            }
        }
    }

    fn accept_line(&mut self, proxy_offset: ProxyOffset, line: Line, matches: Vec<CustomHighlight>) -> Line {
        let s = line.start;
        let e = line.end;
        self.highest_scanned_original_offset = OriginalOffset::from(e);
        self.offset_mapper.add(proxy_offset, OriginalOffset::from(s)).unwrap();
        let e1 = proxy_offset + (e - s);
        self.offset_mapper.confirm(e1);
        line.to_builder()
            .with_start(*proxy_offset)
            .with_end(*e1)
            .with_multiple_custom_highlights(FILTERED_LINE_SOURCE_CUSTOM_DATA_KEY, matches)
            .build()
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
                let buffer = &mut buffer[self.p..self.p + n];
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

#[derive(Eq, PartialEq, Debug)]
struct Message {
    proxy_offset: ProxyOffset,
    original_offset: OriginalOffset,
    line_length: Integer,
}

struct CacheItem {
    original_offset: OriginalOffset,
    line_length: usize,
    bytes_read: usize,
}

fn perform_on_self<F>(id: Uuid, model: &mut RootModel, f: F)
where
    F: FnOnce(&mut FilteredLineSource) -> ()
{
    let Some(mut ds) = model.get_datasource_ref() else { return; };
    let holder = &mut *ds;
    match holder {
        LineSourceHolder::Filtered(ls) => {
            if ls.id == id {
                f(ls)
            }
        },
        _ => {},
    };
}

#[cfg(test)]
#[path="./tests.rs"]
mod tests;

