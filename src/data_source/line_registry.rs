use std::cmp::Ordering;
use crate::interval::{Interval, IntervalBound};
use fluent_integer::Integer;
use std::io::{BufReader, Read, Seek};
use std::sync::RwLock;
use std::time::{Duration, Instant};
use metrics::{describe_gauge, describe_histogram, gauge, histogram, Unit};
use thiserror::Error;
use crate::data_source::BUFFER_SIZE;
use crate::utils::ToUnit;

const PROGRESS_REPORT_PERIOD: Duration = Duration::from_millis(100);

const METRIC_BUILD: &str = "LineRegistry: build";
const METRIC_READ: &str = "LineRegistry: read";
const METRIC_COUNT: &str = "LineRegistry: count line-breaks";
const METRIC_SUBMIT: &str = "LineRegistry: submit results";
const METRIC_READ_TOTAL: &str = "LineRegistry: read (total)";
const METRIC_COUNT_TOTAL: &str = "LineRegistry: count line-breaks (total)";
const METRIC_SUBMIT_TOTAL: &str = "LineRegistry: submit results (total)";

/// Keeps ordered set of offsets of line break symbols
pub trait LineRegistry {
    fn push<I>(&self, offset: I)
    where
        I: Into<Integer>;

    fn count<I>(&self, range: &Interval<I>) -> LineRegistryResult<usize>
    where
        I: Into<Integer> + Copy + Ord;

    fn build<R, F, G>(&self, reader: &mut BufReader<R>, is_interrupted: F, bytes_processed: G) -> LineRegistryResult<()>
    where
        R: Read + Seek,
        F: Fn() -> bool,
        G: Fn(usize);
}

#[derive(Error, Debug)]
pub enum LineRegistryError {
    #[error("IO error")]
    IO(#[from] std::io::Error),
    #[error("Process has been cancelled")]
    Cancelled,
    #[error("Requested interval {requested:?} has not been crawled yet. {limit:?}")]
    NotReachedYet {
        requested: IntervalBound<Integer>,
        limit: Integer,
    }
}

impl Clone for LineRegistryError {
    fn clone(&self) -> Self {
        match self {
            // TODO: we lose error data while cloning. Probably, use Arc<Error>
            LineRegistryError::IO(io) => LineRegistryError::IO(std::io::Error::from(io.kind())),
            LineRegistryError::Cancelled => LineRegistryError::Cancelled,
            LineRegistryError::NotReachedYet { requested, limit } => LineRegistryError::NotReachedYet {
                requested: *requested,
                limit: *limit
            },
        }
    }
}

impl PartialEq for LineRegistryError {
    fn eq(&self, other: &Self) -> bool {
        match self {
            LineRegistryError::IO(io1) => {
                if let LineRegistryError::IO(io2) = other {
                    io1.kind() == io2.kind()
                } else {
                    false
                }
            }
            LineRegistryError::Cancelled => {
                matches!(other, LineRegistryError::Cancelled)
            }
            LineRegistryError::NotReachedYet { requested: r1, limit: lim1 } => {
                if let LineRegistryError::NotReachedYet { requested: r2, limit: lim2} = other {
                    r1 == r2 && lim1 == lim2
                } else {
                    false
                }
            }
        }
    }
}

impl Eq for LineRegistryError {}

pub type LineRegistryResult<T> = Result<T, LineRegistryError>;

#[derive(Default, Debug)]
pub struct LineRegistryImpl {
    internals: RwLock<Internals>,
}

#[derive(Default, Debug)]
struct Internals {
    line_breaks: Vec<Integer>,
    crawled: Integer,
}

impl LineRegistryImpl {
    pub fn new() -> Self {
        describe_histogram!(METRIC_BUILD, Unit::Milliseconds, METRIC_BUILD);
        describe_histogram!(METRIC_READ, Unit::Microseconds, METRIC_READ);
        describe_histogram!(METRIC_COUNT, Unit::Microseconds, METRIC_COUNT);
        describe_histogram!(METRIC_SUBMIT, Unit::Microseconds, METRIC_SUBMIT);
        describe_gauge!(METRIC_READ_TOTAL, Unit::Microseconds, METRIC_READ_TOTAL);
        describe_gauge!(METRIC_COUNT_TOTAL, Unit::Microseconds, METRIC_COUNT_TOTAL);
        describe_gauge!(METRIC_SUBMIT_TOTAL, Unit::Microseconds, METRIC_SUBMIT_TOTAL);
        LineRegistryImpl {
            internals: RwLock::new(Internals::default()),
        }
    }

    fn with_data<I: Into<Integer> + Copy>(data: Vec<I>) -> Self {
        let line_breaks: Vec<Integer> = data.iter().map(|i| (*i).into()).collect();
        let crawled = *line_breaks.iter().max().unwrap_or(&0.into());
        LineRegistryImpl {
            internals: RwLock::new(Internals {
                line_breaks,
                crawled,
            })
        }
    }
}

impl LineRegistry for LineRegistryImpl {
    fn push<I>(&self, offset: I)
    where
        I: Into<Integer>,
    {
        let mut internals = self.internals.write().unwrap();
        internals.line_breaks.push(offset.into());
    }

    fn count<I>(&self, range: &Interval<I>) -> LineRegistryResult<usize>
    where
        I: Into<Integer> + Copy + Ord,
    {
        let range = range.map(|i| (*i).into());
        let internals = self.internals.read().unwrap();
        let cmp = range.right_bound.partial_cmp(&internals.crawled).unwrap_or(Ordering::Equal);
        if cmp == Ordering::Greater {
            return Err(LineRegistryError::NotReachedYet {
                requested: range.right_bound,
                limit: internals.crawled,
            })
        }

        let v = &internals.line_breaks;

        let s = match &range.left_bound {
            IntervalBound::PositiveInfinity => None,
            IntervalBound::NegativeInfinity => Some(0),
            IntervalBound::Fixed { value, is_included } => {
                match v.binary_search(value) {
                    Ok(p) => {
                        if *is_included {
                            Some(p)
                        } else {
                            Some(p + 1)
                        }
                    }
                    Err(p) => Some(p),
                }
            }
        };
        let e: Option<Integer> = match &range.right_bound {
            IntervalBound::PositiveInfinity => Some(Integer::from(v.len()) - 1),
            IntervalBound::NegativeInfinity => None,
            IntervalBound::Fixed { value, is_included } => {
                match v.binary_search(&(*value).into()) {
                    Ok(p) => {
                        if *is_included {
                            Some(Integer::from(p))
                        } else {
                            Some(Integer::from(p) - 1)
                        }
                    }
                    Err(p) => Some(Integer::from(p) - 1),
                }
            }
        };
        Ok(s.zip(e).map(|(s, e)| e - s + Integer::from(1_i32)).map(|u| u.as_usize()).unwrap_or(0))
    }

    fn build<R, F, G>(&self, reader: &mut BufReader<R>, is_interrupted: F, bytes_processed: G) -> LineRegistryResult<()>
    where
        R: Read + Seek,
        F: Fn() -> bool,
        G: Fn(usize),
    {
        let sw_total = Instant::now();
        let mut offset: Integer = reader.stream_position()?.into();
        let mut bytes_read = 0;
        let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let mut last_report = Instant::now();
        let mut sw_read = Instant::now();
        let mut data: Vec<Integer> = Vec::new();
        while let Ok(b) = reader.read(&mut buffer) {
            {
                let elapsed = sw_read.elapsed().to_unit(&Unit::Microseconds);
                histogram!(METRIC_READ).record(elapsed);
                gauge!(METRIC_READ_TOTAL).increment(elapsed);
                log::info!("{} << {}", METRIC_READ, elapsed);
            }
            if b == 0 {
                break;
            }

            if is_interrupted() {
                return Err(LineRegistryError::Cancelled);
            }

            let sw = Instant::now();
            let mut p = 0_usize;
            data.clear();
            #[allow(clippy::explicit_counter_loop)] // using enumerate slows it twice
            for ch in &buffer {
                if *ch == b'\n' {
                    data.push(offset + p);
                }
                p += 1;
            }
            {
                let elapsed = sw.elapsed().to_unit(&Unit::Microseconds);
                histogram!(METRIC_COUNT).record(elapsed);
                gauge!(METRIC_COUNT_TOTAL).increment(elapsed);
                log::info!("{} << {}", METRIC_COUNT, elapsed);
            }

            let sw = Instant::now();
            if !data.is_empty() {
                let mut internals = self.internals.write().unwrap();
                data.iter().for_each(|i| internals.line_breaks.push(*i));
                internals.crawled = offset + b;
            }
            {
                let elapsed = sw.elapsed().to_unit(&Unit::Microseconds);
                histogram!(METRIC_SUBMIT).record(elapsed);
                gauge!(METRIC_SUBMIT_TOTAL).increment(elapsed);
                log::info!("{} << {}", METRIC_SUBMIT, elapsed);
            }

            bytes_read += b;
            offset += b;

            if last_report.elapsed() > PROGRESS_REPORT_PERIOD {
                bytes_processed(bytes_read);
                last_report = Instant::now();
            }

            sw_read = Instant::now();
        }
        bytes_processed(bytes_read);
        histogram!(METRIC_BUILD).record(sw_total.elapsed().to_unit(&Unit::Milliseconds));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::data_source::line_registry::{LineRegistry, LineRegistryImpl};
    use crate::interval::Interval;
    use fluent_integer::Integer;
    use spectral::prelude::*;

    const N: usize = 15;

    #[test]
    fn test_count_1() {
        test_count(Interval::closed(1, 3), 3);
    }

    #[test]
    fn test_count_2() {
        test_count(Interval::closed(0, 10), 5);
    }

    #[test]
    fn test_count_3() {
        test_count(Interval::inf_closed(10), 5);
    }

    #[test]
    fn test_count_4() {
        test_count(Interval::empty(), 0);
    }

    #[test]
    fn test_count_5() {
        test_count(Interval::open(1, 5), 2);
    }

    #[test]
    fn test_count_6() {
        test_count(Interval::open(1, 6), 3);
    }

    fn test_count(interval: Interval<i32>, expected: usize) {
        let registry = create_registry();
        let actual = registry.count(&interval);
        let descr = format!(
            "{:?} is expected to have {} elements of {}",
            &registry, &expected, &interval
        );
        asserting!(descr).that(&actual).is_ok_containing(&expected)
    }

    /// Creates a sample registry with N first Fibonacci numbers
    fn create_registry() -> LineRegistryImpl {
        let mut data = vec![];
        let mut a: Integer = 1.into();
        let mut b: Integer = 1.into();
        for i in 0..N {
            data.push(b);
            (a, b) = (b, a + b);
        }
        LineRegistryImpl::with_data(data)
    }
}
