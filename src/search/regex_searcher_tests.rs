use regex::Regex;
use spectral::prelude::*;
use crate::data_source::{Direction, StrBackend};
use crate::interval::Interval;
use crate::search::regex_searcher_impl::RegexSearcherImpl;
use crate::search::searcher::{Occurrence, Searcher};

#[test]
fn test_basic_regex() {
    let source = "Foo bar\nbar baz\n\nfoo bar";
    let regex = Regex::new(r"ba.")
        .expect("Failed to parse regex");
    let mut searcher = RegexSearcherImpl::new(StrBackend::new(source), regex);

    let occurrence = searcher.next_occurrence(Direction::Forward, Interval::all());
    asserting("").that(&occurrence)
        .is_ok_containing(Occurrence::new(4, 7));

    let occurrence = searcher.next_occurrence(Direction::Forward, Interval::builder().left_bound_inclusive(8.into()).right_unbounded().build());
    asserting("").that(&occurrence)
        .is_ok_containing(Occurrence::new(8, 11));
}
//
// fn test_regex(source: &str, pattern: &str, offset: usize) {
//     let (_, substr) = source.split_at(offset);
//     let regex = Regex::new(pattern).expect("Failed to parse regex");
//     let matches = substr.lines().flat_map(|line| {
//         regex.find_iter(line).map(|m| {
//             Range { start: m.start(), end: m.end()}
//         }).collect()
//     }).collect();
// }