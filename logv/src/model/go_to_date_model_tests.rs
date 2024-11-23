use crate::model::guess_date_format::KnownDateFormat;
use lazy_static::lazy_static;

const TEXT: &str = r#"2022 Feb 12 11:00:00 line1
2022 Feb 12 11:01:00 line2
Line without date
2022 Feb 12 12:00:00 line4
2022 Feb 12 12:01:00 line5
2022 Feb 13 10:01:00 line6"#;

const TEXT2: &str = r#"2022 Feb 12 10:00:00 line 1
2022 Feb 12 10:01:00 line 2.1
2022 Feb 12 10:01:00 line 2.2
2022 Feb 12 10:01:00 line 2.3
2022 Feb 12 10:01:00 line 2.4
2022 Feb 12 10:02:00 line 3"#;

lazy_static! {
    static ref DATE_FORMAT: KnownDateFormat = KnownDateFormat::new(String::from("%Y %b %d %H:%M:%S"));
}

mod test_take_line {
    use super::{DATE_FORMAT, TEXT};
    use crate::data_source::{Direction, LineSourceImpl, StrBackend};
    use crate::model::go_to_date_model::take_line;
    use crate::model::guess_date_format::GuessContext;
    use chrono::NaiveDateTime;
    use spectral::prelude::*;

    #[test]
    fn from_start_forward() {
        let mut src = LineSourceImpl::new(StrBackend::new(TEXT)).into();
        let guess_ctx = GuessContext::with_year(2023);
        let actual = take_line(
            &mut src,
            0,
            TEXT.len(),
            Direction::Forward,
            &DATE_FORMAT,
            &guess_ctx,
        );
        asserting!("Should find first line")
            .that(&actual)
            .is_some()
            .matches(|(line, dt)| {
                asserting!("Line content")
                    .that(line)
                    .map(|l| &l.content)
                    .is_equal_to(&"2022 Feb 12 11:00:00 line1".to_string());
                asserting!("Date")
                    .that(dt)
                    .is_equal_to(&NaiveDateTime::parse_from_str("12-Feb-2022 11:00:00", "%d-%b-%Y %H:%M:%S").unwrap());
                true
            });
    }

    #[test]
    fn with_offset_forward() {
        let mut src = LineSourceImpl::new(StrBackend::new(TEXT)).into();
        let guess_ctx = GuessContext::with_year(2023);
        let actual = take_line(
            &mut src,
            27, // length of 1st line + 1
            TEXT.len(),
            Direction::Forward,
            &DATE_FORMAT,
            &guess_ctx,
        );
        asserting!("Should find second line")
            .that(&actual)
            .is_some()
            .matches(|(line, dt)| {
                asserting!("Line content")
                    .that(line)
                    .map(|l| &l.content)
                    .is_equal_to(&"2022 Feb 12 11:01:00 line2".to_string());
                asserting!("Date")
                    .that(dt)
                    .is_equal_to(&NaiveDateTime::parse_from_str("12-Feb-2022 11:01:00", "%d-%b-%Y %H:%M:%S").unwrap());
                true
            });
    }

    #[test]
    fn skip_forward() {
        let mut src = LineSourceImpl::new(StrBackend::new(TEXT)).into();
        let guess_ctx = GuessContext::with_year(2023);
        let actual = take_line(
            &mut src,
            56, // first 2 lines
            TEXT.len(),
            Direction::Forward,
            &DATE_FORMAT,
            &guess_ctx,
        );
        asserting!("Should find fourth line")
            .that(&actual)
            .is_some()
            .matches(|(line, dt)| {
                asserting!("Line content")
                    .that(line)
                    .map(|l| &l.content)
                    .is_equal_to(&"2022 Feb 12 12:00:00 line4".to_string());
                asserting!("Date")
                    .that(dt)
                    .is_equal_to(&NaiveDateTime::parse_from_str("12-Feb-2022 12:00:00", "%d-%b-%Y %H:%M:%S").unwrap());
                true
            });
    }

    #[test]
    fn from_start_backward() {
        let mut src = LineSourceImpl::new(StrBackend::new(TEXT)).into();
        let guess_ctx = GuessContext::with_year(2023);
        let actual = take_line(
            &mut src,
            1,
            0,
            Direction::Backward,
            &DATE_FORMAT,
            &guess_ctx,
        );
        asserting!("Should find first line backwards")
            .that(&actual)
            .is_some()
            .matches(|(line, dt)| {
                asserting!("Line content")
                    .that(line)
                    .map(|l| &l.content)
                    .is_equal_to(&"2022 Feb 12 11:00:00 line1".to_string());
                asserting!("Date")
                    .that(dt)
                    .is_equal_to(&NaiveDateTime::parse_from_str("12-Feb-2022 11:00:00", "%d-%b-%Y %H:%M:%S").unwrap());
                true
            });
    }

    #[test]
    fn empty_forward() {
        let mut src = LineSourceImpl::new(StrBackend::new("")).into();
        let guess_ctx = GuessContext::with_year(2023);
        let actual = take_line(
            &mut src,
            0,
            0,
            Direction::Forward,
            &DATE_FORMAT,
            &guess_ctx,
        );
        asserting!("Should not find anything")
            .that(&actual)
            .is_none();
    }

    #[test]
    fn empty_backward() {
        let mut src = LineSourceImpl::new(StrBackend::new("")).into();
        let guess_ctx = GuessContext::with_year(2023);
        let actual = take_line(
            &mut src,
            0,
            0,
            Direction::Backward,
            &DATE_FORMAT,
            &guess_ctx,
        );
        asserting!("Should not find anything")
            .that(&actual)
            .is_none();
    }
}

mod test_bin_search {
    use super::{DATE_FORMAT, TEXT};
    use crate::background_process::task_context::TaskContext;
    use crate::data_source::{LineSourceImpl, StrBackend};
    use crate::model::abstract_go_to_model::GoToResult;
    use crate::model::go_to_date_model::bin_search;
    use crate::model::go_to_date_model::go_to_date_model_tests::TEXT2;
    use crate::model::guess_date_format::GuessContext;
    use chrono::NaiveDateTime;
    use spectral::prelude::*;
    use uuid::Uuid;

    #[test]
    fn exact_match() {
        let actual = do_search(TEXT, "2022-02-12 12:00:00");
        asserting!("Find line with exact matching date")
            .that(&actual)
            .is_ok_containing(&72.into());
    }

    #[test]
    fn non_exact_match_in_middle() {
        let actual = do_search(TEXT, "2022-02-12 11:30:00");
        asserting!("Find line with exact matching date")
            .that(&actual)
            .is_ok_containing(&27.into());
    }

    #[test]
    fn date_before_first_line() {
        let actual = do_search(TEXT, "2022-01-01 00:00:00");
        asserting!("Find line with exact matching date")
            .that(&actual)
            .is_ok_containing(&0.into());
    }

    #[test]
    fn date_after_last_line() {
        let actual = do_search(TEXT, "2025-01-01 00:00:00");
        asserting!("Find line with exact matching date")
            .that(&actual)
            .is_ok_containing(&126.into());
    }

    #[test]
    fn exact_first_match() {
        let actual = do_search(TEXT2, "2022-02-12 10:01:00");
        asserting!("Fine first line with exactly matching date")
            .that(&actual)
            .is_ok_containing(&28.into());
    }

    fn do_search(text: &'static str, date: &'static str) -> GoToResult {
        let line_source = LineSourceImpl::new(StrBackend::new(text));
        let mut src = line_source.into();
        let guess_ctx = GuessContext::with_year(2023);
        let mut ctx = create_task_ctx();
        let target_date = NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M:%S").unwrap();
        bin_search(
            target_date,
            &mut src,
            &DATE_FORMAT,
            guess_ctx,
            &mut ctx)
    }

    fn create_task_ctx() -> TaskContext<(), GoToResult> {
        let (msg_sender, _) = crossbeam_channel::unbounded();
        let (_, interrupt_receiver) = crossbeam_channel::unbounded();
        TaskContext::new(msg_sender, interrupt_receiver, Uuid::new_v4())
    }
}