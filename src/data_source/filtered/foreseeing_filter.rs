use crate::data_source::filtered::caching_filter::CachingFilter;
use crate::data_source::{CustomHighlight, Direction, Line};
use fluent_integer::Integer;
use crate::data_source::filtered::filtered_line_source::LineFilter;
use crate::data_source::line_source_holder::ConcreteLineSourceHolder;

pub struct ForeseeingFilter {
    filter: CachingFilter,
    neighbourhood: u8,
}

impl ForeseeingFilter {

    pub fn new(
        filter: LineFilter,
        neighbourhood: u8,
    ) -> Self {
        Self {
            filter: CachingFilter::new(filter, (neighbourhood * 2) as usize),
            neighbourhood
        }
    }

    pub fn apply(&mut self, origin: &mut ConcreteLineSourceHolder, offset: Integer) -> ForeseeingFilterResult {
        match self.filter.apply(origin, offset, Direction::Forward) {
            Some((line, highlights)) => {
                if !highlights.is_empty() {
                    ForeseeingFilterResult::PreciseMatch(line, highlights)
                } else {
                    let mut offset = line.start - 1;
                    let mut i = self.neighbourhood;
                    loop {
                        if i == 0 || offset < 0 {
                            break;
                        }
                        i -= 1;
                        let Some((ln, hl)) = self.filter.apply(origin, offset, Direction::Backward) else {
                            break;
                        };
                        if !hl.is_empty() {
                            return ForeseeingFilterResult::NeighbourMatch(line);
                        }
                        offset = ln.start - 1;
                    };
                    let mut offset = line.end + 1;
                    let mut i = self.neighbourhood;
                    loop {
                        if i == 0 {
                            break;
                        }
                        i -= 1;
                        let Some((ln, hl)) = self.filter.apply(origin, offset, Direction::Forward) else {
                            return ForeseeingFilterResult::EOF;
                        };
                        if !hl.is_empty() {
                            return ForeseeingFilterResult::NeighbourMatch(line);
                        }
                        offset = ln.end + 1;
                    };
                    ForeseeingFilterResult::NoMatch(line.end + 1)
                }
            },
            None => ForeseeingFilterResult::EOF,
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum ForeseeingFilterResult {
    PreciseMatch(Line, Vec<CustomHighlight>),
    NeighbourMatch(Line),
    NoMatch(Integer),
    EOF,
}

#[cfg(test)]
mod tests {
    use std::cmp::{max, min};
    use std::sync::Arc;
    use fluent_integer::Integer;
    use itertools::Itertools;
    use lazy_static::lazy_static;
    use regex::Regex;
    use crate::data_source::filtered::filtered_line_source::LineFilter;
    use crate::data_source::filtered::foreseeing_filter::{ForeseeingFilter, ForeseeingFilterResult};
    use crate::data_source::line_source_holder::ConcreteLineSourceHolder;
    use crate::data_source::{CustomHighlight, Line, LineSourceImpl};
    use spectral::prelude::*;
    use crate::model::rendered::LineNumberMissingReason;
    use paste::paste;
    use crate::utils::*;

    const LINE_COUNT: u64 = 1000;
    lazy_static! {
        static ref ORIGINAL: String = (0..LINE_COUNT).map(|i| format!("Line {}", i)).join("\n");
        static ref LINE_NUMBER_PATTERN: Regex = Regex::new(r"^Line (?P<N>\d+)$").unwrap();
    }

    fn each_nth(n: u64) -> LineFilter {
        let filter = move |s: &str| -> Vec<CustomHighlight> {
            LINE_NUMBER_PATTERN
                .captures(s)
                .and_then(|caps| caps.name("N"))
                .iter()
                .filter_map(|m| {
                    let line_no = m.as_str().parse::<u64>().ok()?;
                    if line_no % n == 0 {
                        Some(CustomHighlight::new(m.start(), m.end()))
                    } else {
                        None
                    }
                })
                .collect_vec()
        };
        Arc::new(filter)
    }

    fn get_line(n: u64) -> Line {
        let content = format!("Line {}", n);
        let len = content.len() as u64;
        let q = max(n, 1).ilog10();
        let start = n * ("Line \n".len() as u64) + n * (q as u64 + 1) - 10 * (10_i32.pow(q) as u64 - 1) / 9;
        Line::builder()
            .with_content(content)
            .with_start(start)
            .with_end(start + len)
            .with_line_no(Err(LineNumberMissingReason::LineNumberingTurnedOff))
            .build()
    }

    macro_rules! run_test {
        ($each: literal, $neighbourhood: literal) => {
            paste! {
                #[test]
                fn [<test_ $each _ $neighbourhood>]() {
                    test($each, $neighbourhood);
                }
            }
        };
    }

    run_test!(5, 0);
    run_test!(5, 1);
    run_test!(5, 2);
    run_test!(5, 3);

    fn test(each: u64, neighbourhood: u8) {
        let mut ds = ConcreteLineSourceHolder::from(LineSourceImpl::from_str(&ORIGINAL));

        let mut current_line = 0;
        for offset in 0..ORIGINAL.len() {
            let mut subject = ForeseeingFilter::new(each_nth(each), neighbourhood);
            let neighbourhood = neighbourhood as u64;

            let rem = current_line % each;
            let expected = if rem == 0 {
                let expected_line = get_line(current_line);
                let h = "Line ".len();
                let expected_highlight = vec![CustomHighlight::new(h, h + (max(current_line, 1).ilog10() as usize) + 1)];
                ForeseeingFilterResult::PreciseMatch(expected_line, expected_highlight)
            } else if rem <= neighbourhood || rem + neighbourhood >= each {
                let line_no = ((current_line - rem) / each + bool_to_u64(rem > neighbourhood)) * each;
                if line_no < LINE_COUNT {
                    ForeseeingFilterResult::NeighbourMatch(get_line(current_line))
                } else {
                    ForeseeingFilterResult::EOF
                }
            } else if neighbourhood == 0 {
                ORIGINAL[offset..].chars().enumerate()
                    .take_while(|(_, ch)| *ch != '\n')
                    .last()
                    .map(|(p, _)| ForeseeingFilterResult::NoMatch(Integer::from(offset + p + 2)))
                    .unwrap_or(ForeseeingFilterResult::NoMatch(Integer::from(offset + 1)))
            } else {
                ORIGINAL[offset..].chars().enumerate()
                    .filter(|(_, ch)| *ch == '\n')
                    .take(1)
                    .next()
                    .map(|(p, _)| ForeseeingFilterResult::NoMatch(Integer::from(offset + p + 1)))
                    .unwrap_or(ForeseeingFilterResult::EOF)
            };
            let descr = format!("Offset {}; Original: `{}`", offset, &ORIGINAL[offset..min(offset + 20, ORIGINAL.len())]);
            asserting!(descr).that(&subject.apply(&mut ds, offset.into()))
                .is_equal_to(&expected);

            if ORIGINAL.chars().nth(offset).unwrap() == '\n' {
                current_line += 1;
            }
        }
    }
}