use std::cmp::Ordering;
use std::ops::{Deref, Sub};
use crate::data_source::line_registry::{LineRegistry, LineRegistryImpl};
use crate::data_source::{Data, Direction, Line, LineSource};
use fluent_integer::Integer;
use std::sync::Arc;
use crate::data_source::filtered::offset_mapper::{OffsetEvaluationResult, OffsetMapper, OriginalOffset, ProxyOffset};
use crate::interval::PointLocationWithRespectToInterval;
use crate::model::rendered::LineNumberMissingReason;
use crate::utils;

pub struct FilteredLineSource<T>
where
    T: LineSource,
{
    original: T,
    filter: Box<dyn Fn(&Line) -> bool>,
    offset_mapper: OffsetMapper,
    track_line_number: bool,
    pivots: Vec<(ProxyOffset, ProxyOffset)>,
    line_registry: Arc<LineRegistryImpl>,
}

impl<T> LineSource for FilteredLineSource<T>
where
    T: LineSource,
{
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

    fn read_raw(&self, start: Integer, end: Integer) -> Result<String, ()> {
        todo!()
    }

    fn skip_token(&mut self, offset: Integer, direction: Direction) -> Result<Integer, ()> {
        todo!()
    }

    fn get_line_registry(&self) -> Arc<LineRegistryImpl> {
        Arc::clone(&self.line_registry)
    }
}

impl<T> FilteredLineSource<T>
where
    T: LineSource
{
    pub fn new(
        original: T,
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
                }
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
}

#[cfg(test)]
#[path="./tests.rs"]
mod tests;

