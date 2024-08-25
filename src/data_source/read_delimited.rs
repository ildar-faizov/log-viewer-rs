use std::cmp::Ordering;
use std::io::{BufReader, Read, Seek};
use std::sync::Arc;
use fluent_integer::Integer;
use crate::data_source::{Data, Direction, Line};
use crate::data_source::char_navigation::{next_char, peek_next_char, peek_prev_char, prev_char};
use crate::data_source::line_registry::{LineRegistry, LineRegistryImpl};
use crate::interval::Interval;
use crate::model::rendered::LineNumberMissingReason;
use crate::utils;

/// Reads a collection of at most `abs(n)` segments (lines, words, etc.) that are delimited by chars that
/// satisfy `is_delimiter` in direction denoted by `sign(n)`.
///
/// Char `ch` is considered to be a delimiter if and only of `is_delimiter(&ch) == true`.
///
/// If `n == 0`, the method returns empty Data with no segments.
#[profiling::function]
pub fn read_delimited<R, F>(
    f: &mut BufReader<R>,
    offset: Integer,
    n: Integer,
    allow_empty_segments: bool,
    line_registry: Option<Arc<LineRegistryImpl>>,
    is_delimiter: F) -> std::io::Result<Data>
where R: Read + Seek, F: Fn(&char) -> bool
{
    if offset < 0 {
        return Ok(Data::default());
    }

    let direction = match n.cmp(&0.into()) {
        Ordering::Equal => return Ok(Data {
            lines: vec![],
            start: None,
            end: None,
        }),
        Ordering::Greater => Direction::Forward,
        Ordering::Less => Direction::Backward
    };

    let actual_offset: Integer = f.stream_position()?.into();
    let shift = (offset - actual_offset).as_i64();
    f.seek_relative(shift)?;
    let mut current_no = line_registry
        .zip(Some(&direction))
        .ok_or(LineNumberMissingReason::LineNumberingTurnedOff)
        .and_then(move |(r, direction)| {
            let interval = match direction {
                Direction::Forward => Interval::closed(0.into(), offset),
                Direction::Backward => Interval::closed_open(0.into(), offset),
            };
            r.count(&interval).map_err(LineNumberMissingReason::Delegate)
        })
        .map(|n| n as u64);

    let mut data = vec![];
    let mut stack = vec![];
    let flush = |s: &mut Vec<char>| -> (String, u64) {
        let mut content: String = s.iter().collect();
        let bytes_trimmed = utils::trim_newline(&mut content);
        s.clear();
        (content, bytes_trimmed as u64)
    };

    match direction {
        Direction::Forward => {
            // move to the beginning of current segment
            while let Some(ch) = peek_prev_char(f)? {
                if is_delimiter(&ch.get_char()) {
                    break;
                } else {
                    prev_char(f)?;
                }
            }

            // read <= n segments
            let mut start = None;
            loop {
                if let Some(ch) = next_char(f)? {
                    if !is_delimiter(&ch.get_char()) {
                        stack.push(ch.get_char());
                        start = start.or(Some(ch.get_offset()));
                    } else {
                        let line_no = current_no.clone();
                        current_no = current_no.map(|n| n + 1);
                        if !stack.is_empty() || allow_empty_segments {
                            let (content, bytes_trimmed) = flush(&mut stack);
                            let line = Line::builder()
                                .with_content(content)
                                .with_start(start.unwrap_or(ch.get_offset()))
                                .with_end(ch.get_offset() - bytes_trimmed)
                                .with_line_no(line_no)
                                .build();
                            data.push(line);
                            if data.len() == n.abs() {
                                break;
                            }
                        }
                        start = Some(ch.get_end());
                    }
                } else {
                    // EOF
                    if !stack.is_empty() || (allow_empty_segments && start.is_some()) {
                        let (content, bytes_trimmed) = flush(&mut stack);
                        let line = Line::builder()
                            .with_content(content)
                            .with_start(start.unwrap())
                            .with_end(f.stream_position()? - bytes_trimmed)
                            .with_line_no(current_no.clone())
                            .build();
                        data.push(line);
                    }
                    break;
                }
            }
        },
        Direction::Backward => {
            // move to the end of current segment
            while let Some(ch) = peek_next_char(f)? {
                if is_delimiter(&ch.get_char()) {
                    break;
                } else {
                    next_char(f)?;
                }
            }

            // read <= n segments
            let mut end = None;
            loop {
                if let Some(ch) = prev_char(f)? {
                    if !is_delimiter(&ch.get_char()) {
                        stack.push(ch.get_char());
                        end = end.or(Some(ch.get_end()));
                    } else {
                        let line_no = current_no.clone();
                        current_no = current_no.map(|n| n.saturating_sub(1));
                        if !stack.is_empty() || allow_empty_segments {
                            stack.reverse();
                            let (content, bytes_trimmed) = flush(&mut stack);
                            let line = Line::builder()
                                .with_content(content)
                                .with_start(ch.get_offset() + 1)
                                .with_end(end.unwrap_or(ch.get_end()) - bytes_trimmed)
                                .with_line_no(line_no)
                                .build();
                            data.push(line);
                            if data.len() == n.abs() {
                                break;
                            }
                        }
                        end = Some(ch.get_offset());
                    }
                } else {
                    // BOF
                    if !stack.is_empty() || (allow_empty_segments && end.is_some()) {
                        stack.reverse();
                        let (content, bytes_trimmed) = flush(&mut stack);
                        let line = Line::builder()
                            .with_content(content)
                            .with_start(0)
                            .with_end(end.unwrap() - bytes_trimmed)
                            .with_line_no(current_no.clone())
                            .build();
                        data.push(line);
                    }
                    break;
                }
            }
            data.reverse();
        },
    }

    log::trace!("current_no = {:?}, offset = {:?}", &current_no, f.stream_position());

    let s = data.first().map(|segment| segment.start);
    let e = data.last().map(|segment| segment.end);
    Ok(Data {
        lines: data,
        start: s,
        end: e,
    })
}