use super::*;
use crate::data_source::filtered::filtered_line_source::FilteredLineSource;
use crate::data_source::{Line, LineSourceImpl};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use spectral::prelude::*;

lazy_static! {
    static ref ORIGINAL: String = (0..1000).map(|i| format!("Line {}", i)).join("\n");
    static ref LINE_NUMBER_PATTERN: Regex = Regex::new(r"^Line (?P<N>\d+)$").unwrap();
}

fn filter_each_fifth(line: &str) -> Vec<CustomHighlight> {
    LINE_NUMBER_PATTERN
        .captures(line)
        .and_then(|caps| caps.name("N"))
        .iter()
        .filter_map(|m| {
            let line_no = m.as_str().parse::<u64>().ok()?;
            if line_no > 0 && line_no % 5 == 0 {
                Some(CustomHighlight::new(m.start(), m.end()))
            } else {
                None
            }
        })
        .collect_vec()
}

fn create_filtered() -> FilteredLineSource {
    let mut line_source = LineSourceImpl::from_str(&ORIGINAL);
    line_source.track_line_number(true);
    for (p, ch) in ORIGINAL.chars().enumerate() {
        if ch == '\n' {
            line_source.get_line_registry().push(p);
        }
    }
    let original = ConcreteLineSourceHolder::from(line_source);
    FilteredLineSource::new(original, Arc::new(filter_each_fifth), 0)
}

fn expected(n: usize) -> (Vec<Line>, usize) {
    (0..n).fold((vec![], 0_usize), |(mut arr, len), i| {
        let k = 5 * (i + 1) as u64;
        let str = format!("Line {}", k);
        let n = str.len();
        let line = Line::builder()
            .with_content(str)
            .with_start(len)
            .with_end(len + n)
            .with_line_no(Ok(k))
            .with_custom_highlight(FILTERED_LINE_SOURCE_CUSTOM_DATA_KEY, CustomHighlight::new(5_usize, n))
            .build();
        arr.push(line);
        (arr, len + n + 1)
    })
}

#[test]
fn test_read_next_line() {
    let mut proxy = create_filtered();

    let expected = Line::builder()
        .with_content("Line 5")
        .with_start(0)
        .with_end(6)
        .with_line_no(Ok(5))
        .with_custom_highlight(FILTERED_LINE_SOURCE_CUSTOM_DATA_KEY, CustomHighlight::new(5_usize, 6_usize))
        .build();

    assert_that!(proxy.read_next_line(0.into()))
        .is_some()
        .is_equal_to(expected);
}

#[test]
fn test_read_10_lines_forward() {
    let mut proxy = create_filtered();

    let expected = expected(10).0;

    let data = proxy.read_lines(0.into(), 10.into());
    assert_that!(&data.lines).equals_iterator(&expected.iter());
}

#[test]
fn test_read_10_lines_backward() {
    let mut proxy = create_filtered();

    let (expected, last_offset) = expected(2);

    let data = proxy.read_lines(last_offset.saturating_sub(1).into(), (-2).into());
    assert_that!(&data.lines).equals_iterator(&expected.iter());
}

#[test]
fn test_read_prev_line() {
    let mut proxy = create_filtered();

    let expected1 = Line::builder()
        .with_content("Line 5")
        .with_start(0)
        .with_end(6)
        .with_line_no(Ok(5))
        .with_custom_highlight(FILTERED_LINE_SOURCE_CUSTOM_DATA_KEY, CustomHighlight::new(5_usize, 6_usize))
        .build();
    assert_that!(proxy.read_prev_line(0.into()))
        .is_some()
        .is_equal_to(expected1);

    let expected2 = Line::builder()
        .with_content("Line 10")
        .with_start(7)
        .with_end(14)
        .with_line_no(Ok(10))
        .with_custom_highlight(FILTERED_LINE_SOURCE_CUSTOM_DATA_KEY, CustomHighlight::new(5_usize, 7_usize))
        .build();
    assert_that!(proxy.read_prev_line(7.into()))
        .is_some()
        .is_equal_to(expected2);

    assert_that!(proxy.read_prev_line(ORIGINAL.len().into())).is_none();
}

#[test]
fn test_none_match() {
    let original = ConcreteLineSourceHolder::from(LineSourceImpl::from_str(&ORIGINAL));
    let mut proxy = FilteredLineSource::new(original, Arc::new(|_: &str| Vec::default()), 0);

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
    let mut proxy = create_filtered();
    let registry = proxy.get_line_registry();

    let n = 10_usize;
    proxy.read_lines(0.into(), n.into());

    let actual: Vec<Integer> = registry.into_iter().clone().collect_vec();
    let expected = expected(n).0.iter().map(|line| line.end).collect_vec();
    assert_that!(&actual).equals_iterator(&expected.iter());
}

mod read_raw {
    use super::*;
    use crate::data_source::LineSource;
    use paste::paste;

    macro_rules! test {
        ($n: literal, $from: literal, $to: literal, $expected: literal) => {
            paste!{
                #[test]
                fn [<test $n>]() {
                    let mut proxy = create_filtered();

                    let actual = proxy.read_raw($from.into(), $to.into());
                    assert_that!(actual).is_ok_containing(String::from($expected));
                }
            }
        };
    }

    test!(1, 7, 14, "Line 10");
    test!(2, 6, 15, "\nLine 10\n");
    test!(3, 3, 15, "e 5\nLine 10\n");
    test!(4, 3, 22, "e 5\nLine 10\nLine 15");
    test!(5, 0, 22, "Line 5\nLine 10\nLine 15");
    test!(6, 0, 38, "Line 5\nLine 10\nLine 15\nLine 20\nLine 25");
    test!("empty", 0, 0, "");

}

mod skip_token {
    use super::*;
    use crate::data_source::{Direction, LineSource};
    use paste::paste;

    macro_rules! test {
        ($name: literal, $offset: literal, $direction: expr, $expected: literal) => {
            paste!{
                #[test]
                fn [<test_ $name>]() {
                    let mut proxy = create_filtered();

                    let actual = proxy.skip_token($offset.into(), $direction);
                    let expected = $expected;
                    assert_that!(actual).is_ok_containing(&expected.into());
                }
            }
        };
    }

    test!(1, 2, Direction::Forward, 3);
    test!(2, 2, Direction::Backward, 0);
    test!(3, 4, Direction::Forward, 5);
    test!(4, 5, Direction::Forward, 7);
    test!(5, 7, Direction::Backward, 5);
    test!(6, 10, Direction::Backward, 7);
    test!(7, 11, Direction::Backward, 10);
    test!(8, 12, Direction::Backward, 10);
    // todo more tests
}
