use std::cmp::Ordering;
use std::path::PathBuf;
use crossbeam_channel::Sender;
use crate::background_process::background_process_registry::BackgroundProcessRegistry;
use crate::model::abstract_go_to_model::{AbstractGoToModel, GoToError, GoToResult};
use crate::model::model::{ModelEvent, RootModel};
use crate::shared::Shared;
use chrono::prelude::*;
use log::{Level};
use uuid::Uuid;
use fluent_integer::Integer;
use crate::background_process::task_context::TaskContext;
use crate::data_source::{Direction, FileBackend, Line, LineSource, LineSourceImpl};
use crate::model::guess_date_format::{GuessContext, KnownDateFormat};
use crate::utils::measure_l;

pub const DATE_FORMAT: &str = "%d-%b-%Y %T";

pub struct GoToDateModel {
    go_to_model: AbstractGoToModel,
    value: String,
}

impl GoToDateModel {
    pub fn new(
        model_sender: Sender<ModelEvent>,
        background_process_registry: Shared<BackgroundProcessRegistry>,
    ) -> Self {
        let go_to_model = AbstractGoToModel::new(
            model_sender,
            background_process_registry,
            Box::new(ModelEvent::GoToDateOpen),
        );
        Self {
            go_to_model,
            value: String::new(),
        }
    }

    pub fn set_is_open(&mut self, is_open: bool) {
        self.go_to_model.set_is_open(is_open)
    }

    pub fn is_open(&self) -> bool {
        self.go_to_model.is_open()
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, value: &str) {
        self.value = value.to_string()
    }

    pub fn submit(&mut self, file_name: &str, known_date_format: &'static KnownDateFormat, guess_context: GuessContext) -> Result<(), anyhow::Error> {
        let date = NaiveDateTime::parse_from_str(&self.value, DATE_FORMAT)?;
        let date_str = date.to_string();
        let path = PathBuf::from(file_name);
        self.go_to_model.submit(Self::handle_result, move |ctx| {
            let description = format!("Search date {} in {:?}", &date_str, path);
            measure_l(Level::Info, &description, || {
                let total = std::fs::metadata(path.as_path())
                    .map(|m| m.len())?;
                if total == 0 {
                    return Err(GoToError::NotReachable);
                }
                let mut reader = LineSourceImpl::new(FileBackend::new(path));
                let result = Self::bin_search(date, &mut reader, known_date_format, guess_context, ctx);
                log::info!("Search date {} finished: {:?}", date_str, &result);
                result
            })
        });
        Ok(())
    }

    fn handle_result(root_model: &mut RootModel, pid: Uuid, msg: Result<Result<Integer, GoToError>, ()>) -> Result<(), GoToError> {
        let m = &mut root_model.get_go_to_date_model().go_to_model;
        m.handle_result(pid, msg)
    }

    /// Finds line that matches the given `date` best, assuming lines in log file
    /// are sorted in the ascending order according to the date.
    ///
    /// Best match means that the line discovered is the first line with date equal to
    /// requested `date` or the last line among those whose date is less than `date`.
    fn bin_search(
        date: NaiveDateTime,
        reader: &mut dyn LineSource,
        known_date_format: &'static KnownDateFormat,
        guess_ctx: GuessContext,
        ctx: &mut TaskContext<(), GoToResult>,
    ) -> GoToResult {
        let total = reader.get_length();
        let _progress = 0_u8;
        let (mut line1, mut dt1) = Self::take_line(
            reader,
            0,
            total,
            Direction::Forward,
            known_date_format,
            &guess_ctx,
        ).ok_or(GoToError::NotReachable)?;
        if dt1 >= date {
            return Ok(line1.start);
        }

        let (mut line2, mut dt2) = Self::take_line(
            reader,
            total - 1,
            line1.end,
            Direction::Backward,
            known_date_format,
            &guess_ctx,
        ).ok_or(GoToError::NotReachable)?;
        if dt2 < date {
            return Ok(line2.start)
        }

        while !Self::are_lines_same(&line1, &line2) {
            if ctx.interrupted() {
                return Err(GoToError::Cancelled);
            }
            // TODO report progress
            if line2.start - line1.end <= 1 {
                return Ok(line1.start);
            }
            let m = (line1.end + line2.start) / 2.into();
            let (line, dt) = Self::take_line(
                reader,
                m,
                line2.start,
                Direction::Forward,
                known_date_format,
                &guess_ctx
            ).or_else(|| reader.read_next_line(m).map(|ln| (ln, dt2)))
                .ok_or(GoToError::NotReachable)?;

            match dt.cmp(&date) {
                Ordering::Less => (line1, dt1) = (line, dt),
                Ordering::Equal => {
                    let result = Self::earliest_line_with_given_date(reader, (line, dt), known_date_format, &guess_ctx);
                    return Ok(result);
                },
                Ordering::Greater => (line2, dt2) = (line, dt),
            }
        }
        Ok(line1.start)
    }

    fn are_lines_same(line1: &Line, line2: &Line) -> bool {
        line1.start == line2.start && line1.end == line2.end
    }

    /// Returns earliest line with recognized date in the given direction
    fn take_line<I, J>(
        reader: &mut dyn LineSource,
        offset: I,
        boundary: J,
        direction: Direction,
        known_date_format: &'static KnownDateFormat,
        guess_context: &GuessContext,
    ) -> Option<LineWithDate>
        where I: Into<Integer>,
              J: Into<Integer>
    {
        let mut offset = offset.into();
        let boundary = boundary.into();
        log::trace!("take_lines_while(offset={:?}, dir={:?})", offset, direction);
        let mut best_match: Option<LineWithDate> = None;
        loop {
            match direction {
                Direction::Forward => {
                    if offset >= boundary {
                        return None;
                    }
                }
                Direction::Backward => {
                    if offset <= boundary {
                        return best_match;
                    }
                }
            }
            let line = match direction {
                Direction::Forward => reader.read_next_line(offset),
                Direction::Backward => reader.read_prev_line(offset),
            };
            if let Some(line) = line {
                let dt = known_date_format.parse(&line.content, guess_context);
                if let Some(dt) = dt {
                    match direction {
                        Direction::Forward => return Some((line, dt)),
                        Direction::Backward => {
                            if let Some((best_line, best_dt)) = best_match {
                                if best_dt == dt {
                                    offset = line.start - 1;
                                    best_match = Some((line, dt));
                                } else {
                                    return Some((best_line, best_dt));
                                }
                            } else {
                                offset = line.start - 1;
                                best_match = Some((line, dt));
                            }
                        },
                    }
                } else {
                    if best_match.is_some() {
                        return best_match;
                    }
                    // TODO: limit number of lines where date is not recognized
                    offset = match direction {
                        Direction::Forward => line.end + 1,
                        Direction::Backward => line.start - 1,
                    }
                }
            } else {
                return None;
            }
        }
    }

    fn earliest_line_with_given_date(
        reader: &mut dyn LineSource,
        line: LineWithDate,
        known_date_format: &'static KnownDateFormat,
        guess_context: &GuessContext,
    ) -> Integer {
        let date = line.1;
        let mut result = line;
        let mut found = false;
        while !found {
            found = true;
            let prev = Self::take_line(
                reader,
                result.0.start - 1,
                0,
                Direction::Backward,
                known_date_format,
                guess_context);
            if let Some(candidate) = prev {
                if candidate.1 == date {
                    result = candidate;
                    found = false;
                }
            }
        }
        result.0.start
    }
}

type LineWithDate = (Line, NaiveDateTime);

// Tests are included according to http://xion.io/post/code/rust-unit-test-placement.html
#[cfg(test)]
#[path = "./go_to_date_model_tests.rs"]
mod go_to_date_model_tests;