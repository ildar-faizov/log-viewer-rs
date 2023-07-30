use fluent_integer::Integer;
use crate::data_source::Direction;
use crate::interval::Interval;
use crate::search::searcher::{Occurrence, SearchError, SearchResult};

pub trait NavigableSearcher {
    /// Returns all occurrences in a given `range`.
    fn find_all_in_range(&mut self, range: Interval<Integer>) -> Result<Vec<Occurrence>, SearchError>;

    /// Returns first occurrence starting from end of last range supplied to `find_all_in_range`
    /// or first occurrence in the whole file (in case of first invocation).
    ///
    /// Depends on `range` of last `find_all_in_range` invocation.
    fn next_occurrence(&mut self, direction: Direction) -> SearchResult;

    fn set_initial_offset(&mut self, offset: Integer);
}