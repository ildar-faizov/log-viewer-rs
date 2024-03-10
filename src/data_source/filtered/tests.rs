use clap::builder::IntoResettable;
use super::*;
use crate::data_source::filtered::filtered_line_source::FilteredLineSource;
use crate::data_source::{Line, LineSourceImpl, StrBackend};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use spectral::prelude::*;

lazy_static! {
    static ref ORIGINAL: String = { (0..1000).map(|i| format!("Line {}", i)).join("\n") };
    static ref LINE_NUMBER_PATTERN: Regex = { Regex::new(r"^Line (?P<N>\d+)$").unwrap() };
}

fn filter_each_fifth(line: &Line) -> bool {
    LINE_NUMBER_PATTERN
        .captures(&line.content)
        .and_then(|caps| caps.name("N"))
        .and_then(|m| m.as_str().parse::<u64>().ok())
        .filter(|i| (*i > 0) && (i % 5 == 0))
        .is_some()
}

fn expected(n: usize) -> (Vec<Line>, usize) {
    (0..n).fold((vec![], 0_usize), |(mut arr, len), i| {
        let str = format!("Line {}", 5 * (i + 1));
        let n = str.len();
        let line = Line::new(str, len, len + n);
        arr.push(line);
        (arr, len + n + 1)
    })
}

#[test]
fn test_read_next_line() {
    let original = LineSourceImpl::from_str(&ORIGINAL);
    let mut proxy = FilteredLineSource::new(original, Box::new(filter_each_fifth));

    assert_that!(proxy.read_next_line(0.into()))
        .is_some()
        .is_equal_to(Line::new("Line 5", 0, 6));
}

#[test]
fn test_read_10_lines_forward() {
    let original = LineSourceImpl::from_str(&ORIGINAL);
    let mut proxy = FilteredLineSource::new(original, Box::new(filter_each_fifth));

    let expected = expected(10).0;

    let data = proxy.read_lines(0.into(), 10.into());
    assert_that!(&data.lines).equals_iterator(&expected.iter());
}

#[test]
fn test_read_10_lines_backward() {
    let original = LineSourceImpl::from_str(&ORIGINAL);
    let mut proxy = FilteredLineSource::new(original, Box::new(filter_each_fifth));

    let (expected, last_offset) = expected(10);

    let data = proxy.read_lines(last_offset.saturating_sub(1).into(), (-10).into());
    assert_that!(&data.lines).equals_iterator(&expected.iter());
}

#[test]
fn test_read_prev_line() {
    let original = LineSourceImpl::from_str(&ORIGINAL);
    let mut proxy = FilteredLineSource::new(original, Box::new(filter_each_fifth));

    assert_that!(proxy.read_prev_line(0.into()))
        .is_some()
        .is_equal_to(Line::new("Line 5", 0, 6));

    assert_that!(proxy.read_prev_line(7.into()))
        .is_some()
        .is_equal_to(Line::new("Line 10", 7, 14));

    assert_that!(proxy.read_prev_line(ORIGINAL.len().into())).is_none();
}

#[test]
fn test_none_match() {
    let original = LineSourceImpl::from_str(&ORIGINAL);
    let mut proxy = FilteredLineSource::new(original, Box::new(|_: &Line| false));

    assert_that!(proxy.read_next_line(0.into())).is_none();
    assert_that!(proxy.read_next_line(100.into())).is_none();
    assert_that!(proxy.read_prev_line(0.into())).is_none();
    assert_that!(proxy.read_prev_line(100.into())).is_none();
    assert_that!(proxy.read_lines(0.into(), 10.into()))
        .map(|d| &d.lines)
        .is_empty();
    assert_that!(proxy.read_lines(100.into(), (-10).into()))
        .map(|d| &d.lines)
        .is_empty();
}

#[test]
fn test_line_registry() {
    let original = LineSourceImpl::from_str(&ORIGINAL);
    let mut proxy = FilteredLineSource::new(original, Box::new(filter_each_fifth));
    let registry = proxy.get_line_registry();

    let n = 10_usize;
    proxy.read_lines(0.into(), n.into());

    let actual: Vec<Integer> = registry.into_iter().clone().collect_vec();
    let expected = expected(n).0.iter().map(|line| line.end).collect_vec();
    assert_that!(&actual).equals_iterator(&expected.iter());
}