use regex::Regex;
use spectral::prelude::*;
use fluent_integer::Integer;
use crate::data_source::{Direction, StrBackend};
use crate::interval::Interval;
use crate::search::regex_searcher_impl::RegexSearcherImpl;
use crate::search::searcher::{Occurrence, Searcher, SearchError};

const TEXT: &str = "Foo bar\nbar baz\n\nfoo bar";

#[test]
fn test_basic_regex() {
    let source = "Foo bar\nbar baz\n\nfoo bar";
    let regex = Regex::new(r"ba.")
        .expect("Failed to parse regex");
    let mut searcher = RegexSearcherImpl::new(StrBackend::new(source), regex);

    let occurrence = searcher.search(Direction::Forward, Interval::all());
    asserting("").that(&occurrence)
        .is_ok_containing(Occurrence::new(4, 7));

    let occurrence = searcher.search(Direction::Forward, Interval::closed_inf(8.into()));
    asserting("").that(&occurrence)
        .is_ok_containing(Occurrence::new(8, 11));
}

#[test]
fn test_regex_searcher_whole_interval() {
    test_regex(
        TEXT,
        r"ba.",
        Direction::Forward,
        Interval::all(),
        Some(Occurrence::new(4, 7))
    )
}

#[test]
fn test_regex_searcher_2nd_match() {
    test_regex(
        TEXT,
        r"ba.",
        Direction::Forward,
        Interval::closed_inf(8.into()),
        Some(Occurrence::new(8, 11))
    )
}

#[test]
fn test_regex_searcher_not_found() {
    test_regex(
        TEXT,
        r"^baz",
        Direction::Forward,
        Interval::all(),
        None
    )
}

#[test]
fn test_regex_searcher_backward() {
    test_regex(
        TEXT,
        r"ba.",
        Direction::Backward,
        Interval::all(),
        Some(Occurrence::new(21, 24))
    )
}

#[test]
fn test_regex_searcher_backward_2nd_match() {
    test_regex(
        TEXT,
        r"ba.",
        Direction::Backward,
        Interval::closed(0.into(), 21.into()),
        Some(Occurrence::new(12, 15))
    )
}

#[test]
fn test_regex_searcher_backward_not_found() {
    test_regex(
        TEXT,
        r"^baz",
        Direction::Backward,
        Interval::all(),
        None
    )
}

fn test_regex(source: &str, pattern: &str, direction: Direction, range: Interval<Integer>, expected: Option<Occurrence>) {
    let regex = Regex::new(pattern).expect("Failed to parse regex");
    let mut searcher = RegexSearcherImpl::new(StrBackend::new(source), regex);
    let result = searcher.search(direction, range);
    let description = format!("Search '{}' in '{}', range {}", pattern, source, range);
    match expected {
        Some(occurrence) =>
            asserting(&description).that(&result)
                .is_ok_containing(&occurrence),
        None =>
            asserting(&description).that(&result)
                .is_err_containing(&SearchError::NotFound)
    }
}