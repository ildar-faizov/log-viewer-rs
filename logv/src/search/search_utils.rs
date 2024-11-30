use crate::advanced_io::seek_to::SeekTo;
use crate::data_source::Direction;
use crate::interval::{Interval, IntervalBound};
use crate::search::searcher::SearchError;
use crate::search::searcher::SearchError::NotFound;
use fluent_integer::Integer;
use std::cmp::max;
use std::io::{Seek, SeekFrom};

pub struct OffsetAndBoundary {
    pub offset: Integer,
    pub offset_boundary: Option<Integer>
}

pub fn calculate_offset_and_boundary<S: Seek + SeekTo>(reader: &mut S, direction: Direction, range: Interval<Integer>) -> Result<OffsetAndBoundary, SearchError> {
    let (bound, bound2, d) = match direction {
        Direction::Forward => (range.left_bound, range.right_bound, Integer::from(1)),
        Direction::Backward => (range.right_bound, range.left_bound, Integer::from(-1)),
    };
    let offset = match bound {
        IntervalBound::PositiveInfinity => {
            match direction {
                Direction::Forward => return Err(NotFound),
                Direction::Backward => reader.seek(SeekFrom::End(0)).map(Integer::from)?,
            }
        },
        IntervalBound::NegativeInfinity => {
            match direction {
                Direction::Forward => 0.into(),
                Direction::Backward => return Err(NotFound),
            }
        },
        IntervalBound::Fixed { value, is_included } =>
            value + if is_included { 0.into() } else { d }
    };

    let offset = max(offset, 0.into()); // ensure offset in reader

    reader.seek_to(offset)?;

    let offset_boundary = match bound2 {
        IntervalBound::PositiveInfinity => {
            match direction {
                Direction::Forward => None,
                Direction::Backward => return Err(NotFound),
            }
        },
        IntervalBound::NegativeInfinity => {
            match direction {
                Direction::Forward => return Err(NotFound),
                Direction::Backward => None,
            }
        },
        IntervalBound::Fixed { value, is_included } =>
            Some(value - if is_included { 0.into() } else { d })
    };

    Ok(OffsetAndBoundary {
        offset,
        offset_boundary
    })
}