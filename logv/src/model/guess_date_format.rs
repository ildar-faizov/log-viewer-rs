use crate::data_source::{FileBackend, LineSource, LineSourceImpl};
use chrono::{DateTime, Datelike, NaiveDateTime, Utc};
use lazy_static::lazy_static;
use phf::{phf_map, phf_ordered_set};
use regex::{Captures, Match, Regex};
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::path::PathBuf;
use std::time::SystemTime;

const N: usize = 100;

pub fn guess_date_format(path: PathBuf) -> Option<&'static KnownDateFormat> {
    let time = std::fs::metadata(&path)
        .and_then(|m| m.created())
        .unwrap_or(SystemTime::now());
    let dt: DateTime<Utc> = time.into();
    let ctx = GuessContext::with_year(dt.year() as u16);
    let mut line_source = LineSourceImpl::<File, FileBackend>::from_file_name(path);
    guess_date_format0(&mut line_source, &ctx)
}

fn guess_date_format0(line_source: &mut dyn LineSource, ctx: &GuessContext) -> Option<&'static KnownDateFormat> {
    let data = line_source.read_lines(0.into(), N.into());
    FORMATS
        .iter()
        .map(|known_date_format| {
            let rating: usize = data
                .lines
                .iter()
                .map(|line| known_date_format.guess(&line.content, ctx).0)
                .sum();
            (known_date_format, rating)
        })
        .filter(|(_, n)| *n > 0)
        .max_by_key(|(_, n)| *n)
        .map(|(kdf, _)| kdf)
}

const FORMAT_TO_PATTERN: phf::Map<&str, &str> = phf_map! {
    "%Y" => "\\d{4}",
    "%C" => "\\d{2}",
    "%y" => "\\d{2}",
    "%m" => "0[1-9]|1[0-2]",
    "%b" => "Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec",
    "%B" => "January|February|March|April|May|June|July|August|September|October|November|December",
    "%d" => "0[1-9]|1\\d|2\\d|3[0-1]",
    // "%a" => todo
    // "%A" => todo
    "%H" => "[0-1]\\d|2[0-3]",
    "%I" => "0[0-9]|1[0-2]",
    "%M" => "\\d{2}",
    "%S" => "\\d{2}",
    "%P" => "am|pm",
    "%p" => "AM|PM",
    "%.f" => "\\.\\d+",
    "%.3f" => "\\.\\d{3}",
    "%.6f" => "\\.\\d{6}",
    "%.9f" => "\\.\\d{9}",
};

const DATE_FORMATS: phf::OrderedSet<&str> = phf_ordered_set! {
    "%d-%m-%Y",
    "%d-%m-%y",
    "%d.%m.%Y",
    "%d.%m.%y",
    "%m/%d/%Y",
    "%m/%d/%y",
    "%d-%b-%Y",
    "%d-%b-%y",
    "%d-%B-%Y",
    "%d-%B-%y",
    "%d %b %Y",
    "%d %b %y",
    "%d %B %Y",
    "%d %B %y",
};

lazy_static! {
    static ref INCOMPLETE_DATE_FORMATS: Vec<(&'static str, FormatPreprocessor, Preprocessor)> = {
        vec![
            ("%b %d", |fmt: &str| format!("%Y {}", fmt), |data: &str, ctx: &GuessContext| format!("{} {}", ctx.get_default_year(), data))
        ]
    };
}

const TIME_FORMATS: phf::OrderedSet<&str> = phf_ordered_set! {
    "%H:%M",
    "%H:%M:%S",
    "%H:%M:%S%.f",
    "%H:%M:%S%.3f",
    "%H:%M:%S%.6f",
    "%H:%M:%S%.9f",
    "%I:%M%P",
    "%I:%M%p",
    "%I:%M %P",
    "%I:%M %p",
};

const JOINERS: phf::OrderedSet<&str> = phf_ordered_set! {
    " ",
    "T",
};

lazy_static! {
    static ref FORMATS: Vec<KnownDateFormat> = {
        let mut formats = vec![];
        for df in DATE_FORMATS.iter() {
            for tf in TIME_FORMATS.iter() {
                for j in JOINERS.iter() {
                    let key = format!("{}{}{}", df, j, tf);
                    formats.push(KnownDateFormat::new(key));
                }
            }
        }
        for (df, f1, f2) in INCOMPLETE_DATE_FORMATS.iter() {
            for tf in TIME_FORMATS.iter() {
                for j in JOINERS.iter() {
                    let key = format!("{}{}{}", df, j, tf);
                    formats.push(KnownDateFormat::new_with_preprocessor(key, Box::new(f1), *f2));
                }
            }
        }
        formats
    };
    static ref PATTERN_PART_REGEX: Regex = Regex::new("%(\\w|\\.[369]?f)").unwrap();
}

pub struct KnownDateFormat {
    date_format: String,
    pattern: Regex,
    preprocessor: Option<Preprocessor>,
}

impl KnownDateFormat {
    pub fn new(date_format: String) -> Self {
        let pattern = Self::build_pattern(&date_format);
        KnownDateFormat {
            date_format,
            pattern,
            preprocessor: None,
        }
    }

    fn new_with_preprocessor(
        date_format: String,
        format_preprocessor: Box<dyn FnOnce(&str) -> String>,
        preprocessor: Preprocessor
    ) -> Self {
        let pattern = Self::build_pattern(date_format.as_str());
        KnownDateFormat {
            date_format: format_preprocessor(date_format.as_str()),
            pattern,
            preprocessor: Some(preprocessor),
        }
    }

    #[allow(dead_code)]
    pub fn get_pattern(&self) -> &Regex {
        &self.pattern
    }

    pub fn get_date_format(&self) -> &str {
        &self.date_format
    }

    pub fn parse(&self, data: &str, context: &GuessContext) -> Option<NaiveDateTime> {
        self.parse_and_match(data, context)
            .map(|(dt, _)| dt)
    }

    pub fn guess(&self, data: &str, context: &GuessContext) -> GuessRating {
        self.parse_and_match(data, context)
            .map(|(_, m)| GuessRating(m.len()))
            .unwrap_or_default()
    }

    pub fn parse_and_match(&self, data: &str, context: &GuessContext) -> Option<(NaiveDateTime, MatchedInterval)> {
        // TODO: probably try parsing all matches
        self.pattern
            .find(data)
            .and_then(|m| {
                let s = m.as_str();
                let data = match &self.preprocessor {
                    Some(f) => Cow::Owned(f(s, context)),
                    None => Cow::Borrowed(s)
                };
                NaiveDateTime::parse_from_str(&data, &self.date_format)
                    .ok()
                    .map(|dt| (dt, MatchedInterval::from(m)))
            })
    }

    fn build_pattern(date_format: &str) -> Regex {
        let pattern = PATTERN_PART_REGEX.replace_all(date_format, |cpt: &Captures| {
            let m = cpt.get(0).unwrap().as_str();
            let pattern = FORMAT_TO_PATTERN.get(m)
                .unwrap_or_else(|| panic!("Unknown part: {}", m));
            format!("({})", pattern)
        });
        Regex::new(&pattern).unwrap()
    }
}

impl Debug for KnownDateFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "date_pattern={:?}, regex='{:?}', has preprocessor={}", self.date_format, self.pattern, self.preprocessor.is_some())
    }
}

pub struct MatchedInterval {
    start: usize,
    end: usize,
}

impl MatchedInterval {
    fn new(start: usize, end: usize) -> Self {
        MatchedInterval {
            start,
            end,
        }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn len(&self) -> usize {
        self.end - self.start + 1
    }
}

impl From<Match<'_>> for MatchedInterval {
    fn from(value: Match) -> Self {
        MatchedInterval::new(value.start(), value.end())
    }
}

pub struct GuessContext {
    default_year: u16,
}

impl GuessContext {
    pub fn with_year(year: u16) -> Self {
        GuessContext {
            default_year: year,
        }
    }

    pub fn get_default_year(&self) -> u16 {
        self.default_year
    }
}

type FormatPreprocessor = fn(&str) -> String;
type Preprocessor = fn(&str, &GuessContext) -> String;

#[derive(Default, Copy, Clone, Debug)]
pub struct GuessRating(usize);

#[cfg(test)]
mod tests {
    use super::guess_date_format0;
    use crate::data_source::LineSourceImpl;
    use lazy_static::lazy_static;
    use spectral::prelude::*;

    use crate::model::guess_date_format::{GuessContext, FORMATS};
    use chrono::NaiveDateTime;

    lazy_static! {
        static ref PARSE_TEST_CASES: Vec<(&'static str, &'static str)> = vec![
            ("10-Nov-2023 10:00am", "%d-%b-%Y %I:%M%P"),
            ("10 Nov 23 10:00", "%d %b %y %H:%M"),
            ("10.11.2023 22:30:10", "%d.%m.%Y %H:%M:%S"),
            // ("Feb 12 13:30:06", "%b %d %H:%M:%S"),
        ];

        static ref TEST_CASES: Vec<(&'static str, Option<&'static str>)> = {
            let mut test_cases = vec![
                ("abc", None),
                ("123: 10.11.2023 22:30:10 The text goes here", Some("%d.%m.%Y %H:%M:%S")),
            ];
            for (input, pattern) in PARSE_TEST_CASES.iter() {
                test_cases.push((input, Some(pattern)));
            }
            test_cases
        };

        static ref GUESS_TEST_CASES: Vec<GuessTestCase> = {
            vec![
                GuessTestCase::new("%Y %b %d %H:%M:%S", true, "Feb 13 23:55:01", "02/13/23 23:55:01"),
                GuessTestCase::new("%d.%m.%Y %H:%M:%S", false, "10.11.2023 22:30:10", "11/10/23 22:30:10"),
            ]
        };
    }

    #[test]
    fn test_guess_date_format0() {
        let ctx = GuessContext::with_year(2023);
        for (input, expected) in TEST_CASES.iter() {
            let mut line_source = LineSourceImpl::from_str(input);
            let actual = guess_date_format0(&mut line_source, &ctx)
                .map(|kdf| kdf.get_date_format().to_string());
            let description = format!("{} => {:?}", input, expected);
            asserting(&description).that(&actual.as_deref()).is_equal_to(expected);
        }
    }

    #[test]
    fn test_concrete_patterns() {
        for (input, pattern) in PARSE_TEST_CASES.iter() {
            let description = format!("{} should match {:?}", input, pattern);
            asserting(&description).that(&NaiveDateTime::parse_from_str(input, pattern)).is_ok();
        }
    }

    #[test]
    fn test_parse() {
        let ctx = GuessContext::with_year(2023);
        for test_case in GUESS_TEST_CASES.iter() {
            let description = format!("Pattern '{}' (has preprocessor = {}) should be recognized", test_case.date_format, test_case.has_preprocessor);
            let format = FORMATS
                .iter()
                .find(|kdf| {
                    if !kdf.date_format.as_str().eq(test_case.date_format) {
                        return false;
                    }
                    if test_case.has_preprocessor && kdf.preprocessor.is_none() {
                        return false;
                    }
                    if !test_case.has_preprocessor && kdf.preprocessor.is_some() {
                        return false;
                    }
                    return true;
                });
            asserting(&description).that(&format).is_some();

            let format = format.unwrap();
            let description = format!("'{}' should be parsed as '{}' by '{}'", test_case.input, test_case.expected, test_case.date_format);
            let actual = format.parse(test_case.input, &ctx)
                .map(|dt| dt.format("%D %T").to_string());
            asserting(&description).that(&actual)
                .is_some()
                .is_equal_to(&test_case.expected.to_string());
        }
    }

    // #[test]
    // fn print_formats() {
    //     for fmt in FORMATS.iter() {
    //         println!("{:?}", fmt);
    //     }
    //     panic!()
    // }

    struct GuessTestCase {
        pub date_format: &'static str,
        pub has_preprocessor: bool,
        pub input: &'static str,
        pub expected: &'static str,
    }

    impl GuessTestCase {
        pub fn new(
            date_format: &'static str,
            has_preprocessor: bool,
            input: &'static str,
            expected: &'static str,
        ) -> Self {
            Self {
                date_format,
                has_preprocessor,
                input,
                expected,
            }
        }
    }
}
