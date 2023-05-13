use crate::data_source::{Direction, StrBackend};
use crate::search::searcher::create_searcher;
use spectral::prelude::*;

#[test]
fn test_search_letter_forward_once() {
    test_search("ABCDEFGHIJKABCDEFGHIJK", "B", Direction::Forward, 1);
}

#[test]
fn test_search_letter_forward_twice() {
    test_search("ABCDEFGHIJKABCDEFGHIJK", "B", Direction::Forward, 2);
}

#[test]
fn test_search_word_forward_twice() {
    test_search("foo bar foo", "foo", Direction::Forward, 2);
}

#[test]
fn test_search_word_forward_4_times() {
    test_search("foo bar baz foo foo bar baz fffooo", "foo", Direction::Forward, 4);
}

#[test]
fn test_search_letter_backward_once() {
    test_search("ABCDEFGHIJKABCDEFGHIJK", "B", Direction::Backward, 1);
}

#[test]
fn test_search_letter_backward_twice() {
    test_search("ABCDEFGHIJKABCDEFGHIJK", "B", Direction::Backward, 2);
}

#[test]
fn test_search_word_backward_twice() {
    test_search("foo bar foo", "foo", Direction::Backward, 2);
}

#[test]
fn test_search_word_backward_4_times() {
    test_search("foo bar baz foo foo bar baz fffooo", "foo", Direction::Backward, 4);
}

#[test]
fn test_search_forward_exhaustive() {
    test_search("foo bar baz foo foo bar baz fffooo", "foo", Direction::Forward, 10);
}

#[test]
fn test_search_backward_exhaustive() {
    test_search("foo bar baz foo foo bar baz fffooo", "foo", Direction::Backward, 10);
}
#[test]
fn test_search_backward_with_failure() {
    let backend = StrBackend::new("bar foo");
    let mut searcher = create_searcher(backend, "foo".to_string(), 3);
    let result = searcher.next_occurrence(Direction::Backward);
    assert_that(&result).is_err();
}

fn test_search(src: &'static str, pattern: &'static str, direction: Direction, n: u32) {
    let backend = StrBackend::new(src);
    let mut offset = match direction {
        Direction::Forward => 0_usize,
        Direction::Backward => src.len(),
    };
    let mut searcher = create_searcher(backend, pattern.to_string(), offset);
    let find = |p: usize| {
        match direction {
            Direction::Forward => src[p..].find(pattern).map(|r| r + p),
            Direction::Backward => src[..p].rfind(pattern),
        }
    };
    for i in 0..n {
        let result = searcher.next_occurrence(direction);
        let description = format!("Occurrence {}", i + 1);
        if let Some(pos) = find(offset) {
            asserting(&description).that(&result)
                .is_ok_containing(&pos.into());
            match direction {
                Direction::Forward => offset = pos + 1,
                Direction::Backward => offset = pos.saturating_sub(1),
            };
        } else {
            asserting(&description).that(&result)
                .is_err();
        }
    }
}