mod test_read_delimited {
    extern crate spectral;

    use crate::data_source::read_delimited;
    use crate::data_source::{Data, Line};
    use crate::test_extensions::*;
    use spectral::prelude::*;
    use std::io::{BufReader, Cursor, Seek};

    // read lines forward

    #[test]
    fn read_1_line_forward_from_beginning() {
        let (data, mut reader) =
            test("AAA\nBBB\nCCC", 0, 1, SegmentType::Line);

        assert_that!(data)
            .is_ok()
            .map(|d| &d.lines)
            .has_only_element()
            .is_equal_to(Line::new("AAA", 0, 3));

        assert_that!(reader.stream_position()).is_ok_containing(4);
    }

    #[test]
    fn read_2_lines_forward_from_beginning() {
        let (data, mut reader) =
            test("AAA\nBBB\nCCC", 0, 2, SegmentType::Line);
        let mut lines = assert_that!(data).is_ok().map(|d| &d.lines);
        lines.has_length(2);
        lines.item_at(0).is_equal_to(Line::new("AAA", 0, 3));
        lines.item_at(1).is_equal_to(Line::new("BBB", 4, 7));

        assert_that!(reader.stream_position()).is_ok_containing(8);
    }

    #[test]
    fn read_all_lines_forward_from_beginning() {
        let (data, mut reader) =
            test("AAA\nBBB\nCCC", 0, 3, SegmentType::Line);

        let mut lines = assert_that!(data).is_ok().map(|d| &d.lines);
        lines.has_length(3);
        lines.item_at(0).is_equal_to(Line::new("AAA", 0, 3));
        lines.item_at(1).is_equal_to(Line::new("BBB", 4, 7));
        lines.item_at(2).is_equal_to(Line::new("CCC", 8, 11));

        // In case of EOF reader's position does not exceed length
        assert_that!(reader.stream_position()).is_ok_containing(11);
    }

    #[test]
    fn read_more_lines_than_available_forward_from_beginning() {
        let (data, mut reader) =
            test("AAA\nBBB\nCCC", 0, 10, SegmentType::Line);

        let mut lines = assert_that!(data).is_ok().map(|d| &d.lines);
        lines.has_length(3);
        lines.item_at(0).is_equal_to(Line::new("AAA", 0, 3));
        lines.item_at(1).is_equal_to(Line::new("BBB", 4, 7));
        lines.item_at(2).is_equal_to(Line::new("CCC", 8, 11));

        // In case of EOF reader's position does not exceed length
        assert_that!(reader.stream_position()).is_ok_containing(11);
    }

    #[test]
    fn read_1_line_forward_from_middle_of_line() {
        let (data, mut reader) =
            test("AAA\nBBB\nCCC", 2, 1, SegmentType::Line);

        assert_that!(data)
            .is_ok()
            .map(|d| &d.lines)
            .has_only_element()
            .is_equal_to(Line::new("AAA", 0, 3));

        assert_that!(reader.stream_position()).is_ok_containing(4);
    }

    #[test]
    fn read_2_lines_forward_from_middle_of_line() {
        let (data, mut reader) =
            test("AAA\nBBB\nCCC", 5, 2, SegmentType::Line);

        let mut lines = assert_that!(data).is_ok().map(|d| &d.lines);
        lines.has_length(2);
        lines.item_at(0).is_equal_to(Line::new("BBB", 4, 7));
        lines.item_at(1).is_equal_to(Line::new("CCC", 8, 11));

        // In case of EOF reader's position does not exceed length
        assert_that!(reader.stream_position()).is_ok_containing(11);
    }

    #[test]
    fn read_empty_lines_forward_from_middle_of_line() {
        let (data, mut reader) =
            test("AAA\nBBB\n\n\nCCC", 5, 3, SegmentType::Line);

        let mut lines = assert_that!(data).is_ok().map(|d| &d.lines);
        lines.has_length(3);
        lines.item_at(0).is_equal_to(Line::new("BBB", 4, 7));
        lines.item_at(1).is_equal_to(Line::new("", 8, 8));
        lines.item_at(2).is_equal_to(Line::new("", 9, 9));

        assert_that!(reader.stream_position()).is_ok_containing(10);
    }

    #[test]
    fn read_empty_lines_forward_from_delimiter() {
        let (data, mut reader) =
            test("AAA\n\n\nCCC", 4, 3, SegmentType::Line);

        let mut lines = assert_that!(data).is_ok().map(|d| &d.lines);
        lines.has_length(3);
        lines.item_at(0).is_equal_to(Line::new("", 4, 4));
        lines.item_at(1).is_equal_to(Line::new("", 5, 5));
        lines.item_at(2).is_equal_to(Line::new("CCC", 6, 9));

        // In case of EOF reader's position does not exceed length
        assert_that!(reader.stream_position()).is_ok_containing(9);
    }

    #[test]
    fn read_empty_lines_forward_from_delimiter_until_eof() {
        let (data, mut reader) =
            test("AAA\n\n\nCCC", 4, 4, SegmentType::Line);

        let mut lines = assert_that!(data).is_ok().map(|d| &d.lines);
        lines.has_length(3);
        lines.item_at(0).is_equal_to(Line::new("", 4, 4));
        lines.item_at(1).is_equal_to(Line::new("", 5, 5));
        lines.item_at(2).is_equal_to(Line::new("CCC", 6, 9));

        // In case of EOF reader's position does not exceed length
        assert_that!(reader.stream_position()).is_ok_containing(9);
    }

    #[test]
    fn read_empty_lines_forward_from_delimiter_until_eof_when_last_line_is_empty() {
        let (data, mut reader) =
            test("AAA\n\n\n", 4, 4, SegmentType::Line);

        let mut lines = assert_that!(data).is_ok().map(|d| &d.lines);
        lines.has_length(3);
        lines.item_at(0).is_equal_to(Line::new("", 4, 4));
        lines.item_at(1).is_equal_to(Line::new("", 5, 5));
        lines.item_at(2).is_equal_to(Line::new("", 6, 6));

        // In case of EOF reader's position does not exceed length
        assert_that!(reader.stream_position()).is_ok_containing(6);
    }

    #[test]
    fn read_nothing_forward_from_eof() {
        let (data, mut reader) =
            test("AAA\nBBB\nCCC\n", 12, 3, SegmentType::Line);

        assert_that!(data).is_ok().map(|d| &d.lines).is_empty();

        // In case of EOF reader's position does not exceed length
        assert_that!(reader.stream_position()).is_ok_containing(12);
    }

    #[test]
    fn read_forward_from_empty_source() {
        let (data, mut reader) =
            test("", 0, 1, SegmentType::Line);
        assert_that!(data).is_ok().map(|d| &d.lines).is_empty();

        // In case of EOF reader's position does not exceed length
        assert_that!(reader.stream_position()).is_ok_containing(0);
    }

    // read lines backward

    #[test]
    fn read_1_line_backward() {
        let (data, mut reader) =
            test("AAA\nBBB\nCCC", 3, -1, SegmentType::Line);
        let mut lines = assert_that!(data).is_ok().map(|d| &d.lines);
        lines.has_length(1);
        lines.item_at(0).is_equal_to(Line::new("AAA", 0, 3));

        assert_that!(reader.stream_position()).is_ok_containing(0);
    }

    #[test]
    fn read_2_lines_backward() {
        let (data, mut reader) =
            test("AAA\nBBB\nCCC", 7, -2, SegmentType::Line);
        let mut lines = assert_that!(data).is_ok().map(|d| &d.lines);
        lines.has_length(2);
        lines.item_at(0).is_equal_to(Line::new("AAA", 0, 3));
        lines.item_at(1).is_equal_to(Line::new("BBB", 4, 7));

        assert_that!(reader.stream_position()).is_ok_containing(0);
    }

    #[test]
    fn read_all_lines_backward_from_end() {
        let (data, mut reader) =
            test("AAA\nBBB\nCCC", 11, -3, SegmentType::Line);

        let mut lines = assert_that!(data).is_ok().map(|d| &d.lines);
        lines.has_length(3);
        lines.item_at(0).is_equal_to(Line::new("AAA", 0, 3));
        lines.item_at(1).is_equal_to(Line::new("BBB", 4, 7));
        lines.item_at(2).is_equal_to(Line::new("CCC", 8, 11));

        // In case of EOF reader's position does not exceed length
        assert_that!(reader.stream_position()).is_ok_containing(0);
    }

    #[test]
    fn read_2_lines_backward_from_middle_of_line() {
        let (data, mut reader) =
            test("AAA\nBBB\nCCC", 5, -2, SegmentType::Line);
        let mut lines = assert_that!(data).is_ok().map(|d| &d.lines);
        lines.has_length(2);
        lines.item_at(0).is_equal_to(Line::new("AAA", 0, 3));
        lines.item_at(1).is_equal_to(Line::new("BBB", 4, 7));

        assert_that!(reader.stream_position()).is_ok_containing(0);
    }

    #[test]
    fn read_empty_lines_backward_from_middle_of_line() {
        let (data, mut reader) =
            test("AAA\n\n\n\nCCC", 9, -3, SegmentType::Line);

        let mut lines = assert_that!(data).is_ok().map(|d| &d.lines);
        lines.has_length(3);
        lines.item_at(0).is_equal_to(Line::new("", 5, 5));
        lines.item_at(1).is_equal_to(Line::new("", 6, 6));
        lines.item_at(2).is_equal_to(Line::new("CCC", 7, 10));

        assert_that!(reader.stream_position()).is_ok_containing(4);
    }

    #[test]
    fn read_empty_lines_backward_from_delimiter() {
        let (data, mut reader) =
            test("AAA\n\n\nCCC", 5, -3, SegmentType::Line);

        let mut lines = assert_that!(data).is_ok().map(|d| &d.lines);
        lines.has_length(3);
        lines.item_at(0).is_equal_to(Line::new("AAA", 0, 3));
        lines.item_at(1).is_equal_to(Line::new("", 4, 4));
        lines.item_at(2).is_equal_to(Line::new("", 5, 5));

        // In case of EOF reader's position does not exceed length
        assert_that!(reader.stream_position()).is_ok_containing(0);
    }

    #[test]
    fn read_empty_lines_backward_from_delimiter_until_bof() {
        let (data, mut reader) =
            test("AAA\n\n\nCCC", 4, -4, SegmentType::Line);

        let mut lines = assert_that!(data).is_ok().map(|d| &d.lines);
        lines.has_length(2);
        lines.item_at(0).is_equal_to(Line::new("AAA", 0, 3));
        lines.item_at(1).is_equal_to(Line::new("", 4, 4));

        // In case of EOF reader's position does not exceed length
        assert_that!(reader.stream_position()).is_ok_containing(0);
    }

    #[test]
    fn read_empty_lines_backward_from_delimiter_until_bof_when_first_line_is_empty() {
        let (data, mut reader) =
            test("\n\n\n", 2, -4, SegmentType::Line);

        let mut lines = assert_that!(data).is_ok().map(|d| &d.lines);
        lines.has_length(3);
        lines.item_at(0).is_equal_to(Line::new("", 0, 0));
        lines.item_at(1).is_equal_to(Line::new("", 1, 1));
        lines.item_at(2).is_equal_to(Line::new("", 2, 2));

        // In case of EOF reader's position does not exceed length
        assert_that!(reader.stream_position()).is_ok_containing(0);
    }

    #[test]
    fn read_1_line_backward_from_bof() {
        let (data, mut reader) =
            test("AAA\nBBB\nCCC\n", 0, -1, SegmentType::Line);

        assert_that!(data).is_ok().map(|d| &d.lines)
            .has_only_element().is_equal_to(Line::new("AAA", 0, 3));

        // In case of EOF reader's position does not exceed length
        assert_that!(reader.stream_position()).is_ok_containing(0);
    }

    #[test]
    fn read_backward_from_empty_source() {
        let (data, mut reader) =
            test("", 0, -1, SegmentType::Line);
        assert_that!(data).is_ok().map(|d| &d.lines).is_empty();

        // In case of EOF reader's position does not exceed length
        assert_that!(reader.stream_position()).is_ok_containing(0);
    }

    #[test]
    fn read_1_symbol_line_forward_utf() {
        let (data, _) = test("\u{20AC}\n", 0, 1, SegmentType::Line); // s='€'
        assert_that!(data).is_ok();
        assert_that!(data.unwrap().lines).has_only_element().is_equal_to(Line::new("€", 0, 3));
    }

    #[test]
    fn read_line_forward_utf() {
        let (data, _) =
            test("AAA\r\n\u{201C}Quoted text\u{201D}", 0, 2, SegmentType::Line);

        assert_that!(data).is_ok();
        let lines = data.unwrap().lines;
        assert_that!(lines).has_length(2);
        assert_that!(lines).item_at(0).is_equal_to(Line::new("AAA", 0, 3));
        assert_that!(lines).item_at(1).is_equal_to(Line::new("\u{201C}Quoted text\u{201D}", 5, 22));
    }

    // read words forward

    const WORDS: &str = "Word1 word2  word3    Word4\tword_5";

    #[test]
    fn read_1_word_forward_from_beginning() {
        let (data, mut reader) =
            test(WORDS, 0, 1, SegmentType::Word);
        assert_that!(data).is_ok()
            .map(|d| &d.lines)
            .has_only_element()
            .is_equal_to(Line::new("Word1", 0, 5));

        assert_that!(reader.stream_position()).is_ok_containing(6);
    }

    #[test]
    fn read_1_word_forward_from_middle() {
        let (data, mut reader) =
            test(WORDS, 2, 1, SegmentType::Word);
        assert_that!(data).is_ok()
            .map(|d| &d.lines)
            .has_only_element()
            .is_equal_to(Line::new("Word1", 0, 5));

        assert_that!(reader.stream_position()).is_ok_containing(6);
    }

    #[test]
    fn read_4_words_forward_from_middle() {
        let (data, mut reader) =
            test(WORDS, 2, 4, SegmentType::Word);
        let mut assert_that_lines = assert_that!(data).is_ok().map(|d| &d.lines);
        assert_that_lines.has_length(4);
        assert_that_lines.item_at(0).is_equal_to(Line::new("Word1", 0, 5));
        assert_that_lines.item_at(1).is_equal_to(Line::new("word2", 6, 11));
        assert_that_lines.item_at(2).is_equal_to(Line::new("word3", 13, 18));
        assert_that_lines.item_at(3).is_equal_to(Line::new("Word4", 22, 27));

        assert_that!(reader.stream_position()).is_ok_containing(28);
    }

    #[test]
    fn read_words_forward_from_space() {
        let (data, mut reader) =
            test(WORDS, 12, 2, SegmentType::Word);
        let mut assert_that_lines = assert_that!(data).is_ok().map(|d| &d.lines);
        assert_that_lines.has_length(2);
        assert_that_lines.item_at(0).is_equal_to(Line::new("word3", 13, 18));
        assert_that_lines.item_at(1).is_equal_to(Line::new("Word4", 22, 27));

        assert_that!(reader.stream_position()).is_ok_containing(28);
    }

    #[test]
    fn read_words_forward_until_eof() {
        let (data, mut reader) =
            test(WORDS, 12, 2, SegmentType::Word);
        let mut assert_that_lines = assert_that!(data).is_ok().map(|d| &d.lines);
        assert_that_lines.has_length(2);
        assert_that_lines.item_at(0).is_equal_to(Line::new("word3", 13, 18));
        assert_that_lines.item_at(1).is_equal_to(Line::new("Word4", 22, 27));

        assert_that!(reader.stream_position()).is_ok_containing(28);
    }

    #[test]
    fn read_1_word_backward_from_end() {
        let (data, mut reader) =
            test(WORDS, 5, -1, SegmentType::Word);
        assert_that!(data).is_ok()
            .map(|d| &d.lines)
            .has_only_element()
            .is_equal_to(Line::new("Word1", 0, 5));

        assert_that!(reader.stream_position()).is_ok_containing(0);
    }

    #[test]
    fn read_1_word_backward_from_middle() {
        let (data, mut reader) =
            test(WORDS, 2, -1, SegmentType::Word);
        assert_that!(data).is_ok()
            .map(|d| &d.lines)
            .has_only_element()
            .is_equal_to(Line::new("Word1", 0, 5));

        assert_that!(reader.stream_position()).is_ok_containing(0);
    }

    #[test]
    fn read_4_words_backward_from_middle() {
        let (data, mut reader) =
            test(WORDS, 24, -4, SegmentType::Word);
        let mut assert_that_lines = assert_that!(data).is_ok().map(|d| &d.lines);
        assert_that_lines.has_length(4);
        assert_that_lines.item_at(0).is_equal_to(Line::new("Word1", 0, 5));
        assert_that_lines.item_at(1).is_equal_to(Line::new("word2", 6, 11));
        assert_that_lines.item_at(2).is_equal_to(Line::new("word3", 13, 18));
        assert_that_lines.item_at(3).is_equal_to(Line::new("Word4", 22, 27));

        assert_that!(reader.stream_position()).is_ok_containing(0);
    }

    #[test]
    fn read_words_backward_from_space() {
        let (data, mut reader) =
            test(WORDS, 12, -2, SegmentType::Word);
        let mut assert_that_lines = assert_that!(data).is_ok().map(|d| &d.lines);
        assert_that_lines.has_length(2);
        assert_that_lines.item_at(0).is_equal_to(Line::new("Word1", 0, 5));
        assert_that_lines.item_at(1).is_equal_to(Line::new("word2", 6, 11));

        assert_that!(reader.stream_position()).is_ok_containing(0);
    }

    #[test]
    fn read_words_backward_until_bof() {
        let (data, mut reader) =
            test(WORDS, 12, -3, SegmentType::Word);
        let mut assert_that_lines = assert_that!(data).is_ok().map(|d| &d.lines);
        assert_that_lines.has_length(2);
        assert_that_lines.item_at(0).is_equal_to(Line::new("Word1", 0, 5));
        assert_that_lines.item_at(1).is_equal_to(Line::new("word2", 6, 11));

        assert_that!(reader.stream_position()).is_ok_containing(0);
    }

    // utility

    enum SegmentType {
        Line, Word
    }

    fn test(s: &str, offset: u64, n: i32, segment_type: SegmentType) -> (std::io::Result<Data>, BufReader<Cursor<&str>>) {
        let mut reader = BufReader::new(Cursor::new(s));
        let (allow_empty_segments, delimiter): (bool, fn(&char) -> bool) = match segment_type {
            SegmentType::Line => (true, |c: &char| *c == '\n'),
            SegmentType::Word => (false, |c: &char| !c.is_alphanumeric() && *c != '_'),
        };
        let data = read_delimited(
            &mut reader,
            offset.into(),
            n.into(),
            allow_empty_segments,
            None,
            delimiter);
        (data, reader)
    }
}

mod test_read_lines {
    extern crate spectral;

    use crate::data_source::line_registry::LineRegistry;
    use crate::data_source::LineSourceImpl;
    use crate::data_source::{Line, LineSource};
    use crate::test_extensions::*;
    use spectral::prelude::*;
    use std::io::{BufReader, Cursor};

    const LINES_UNIX: &'static str = "AAA\nBBB\nCCC\nDDD";
    const LINES_WINDOWS: &'static str = "AAA\r\nBBB\r\nCCC\r\nDDD";

    #[test]
    fn read_1_line() {
        let lines = test(LINES_UNIX, 0, 1);

        assert_that!(lines).has_only_element().is_equal_to(Line::new_with_line_no("AAA", 0, 3, 0));
    }

    #[test]
    fn read_2_lines() {
        let lines = test(LINES_UNIX, 0, 2);

        assert_that!(lines).has_length(2);
        assert_that!(lines).item_at(0).is_equal_to(Line::new_with_line_no("AAA", 0, 3, 0));
        assert_that!(lines).item_at(1).is_equal_to(Line::new_with_line_no("BBB", 4, 7, 1));
    }

    #[test]
    fn read_all_lines() {
        let lines = test(LINES_UNIX, 0, 4);

        assert_that!(lines).has_length(4);
        assert_that!(lines).item_at(0).is_equal_to(Line::new_with_line_no("AAA", 0, 3, 0));
        assert_that!(lines).item_at(1).is_equal_to(Line::new_with_line_no("BBB", 4, 7, 1));
        assert_that!(lines).item_at(2).is_equal_to(Line::new_with_line_no("CCC", 8, 11, 2));
        assert_that!(lines).item_at(3).is_equal_to(Line::new_with_line_no("DDD", 12, 15, 3));
    }

    #[test]
    fn read_from_middle_of_line() {
        let lines = test(LINES_UNIX, 2, 2);

        assert_that!(lines).has_length(2);
        assert_that!(lines).item_at(0).is_equal_to(Line::new_with_line_no("AAA", 0, 3, 0));
        assert_that!(lines).item_at(1).is_equal_to(Line::new_with_line_no("BBB", 4, 7, 1));
    }

    #[test]
    fn read_more_than_available() {
        let lines = test(LINES_UNIX, 2, 10);

        assert_that!(lines).has_length(4);
        assert_that!(lines).item_at(0).is_equal_to(Line::new_with_line_no("AAA", 0, 3, 0));
        assert_that!(lines).item_at(1).is_equal_to(Line::new_with_line_no("BBB", 4, 7, 1));
        assert_that!(lines).item_at(2).is_equal_to(Line::new_with_line_no("CCC", 8, 11, 2));
        assert_that!(lines).item_at(3).is_equal_to(Line::new_with_line_no("DDD", 12, 15, 3));
    }

    #[test]
    fn read_1_line_backward() {
        let lines = test(LINES_UNIX, 2, -1);

        assert_that!(lines).has_only_element().is_equal_to(Line::new_with_line_no("AAA", 0, 3, 0));
    }

    #[test]
    fn read_1_line_backward_from_delimiter() {
        let lines = test(LINES_UNIX, 3, -1);

        assert_that!(lines).has_only_element().is_equal_to(Line::new_with_line_no("AAA", 0, 3, 0));
    }

    #[test]
    fn read_1_line_with_windows_delimiter() {
        let lines = test(LINES_WINDOWS, 0, 1);

        assert_that!(lines).has_only_element().is_equal_to(Line::new_with_line_no("AAA", 0, 3, 0));
    }

    fn test(s: &str, offset: u64, n: i32) -> Vec<Line> {
        let mut line_source = LineSourceImpl::from_str(s);
        line_source.track_line_number(true);
        build_line_registry(&mut line_source, s);
        let data = line_source.read_lines(offset.into(), n.into());
        data.lines
    }

    fn build_line_registry<T>(line_source: &mut T, s: &str)
        where
            T : LineSource
    {
        let line_registry = line_source.get_line_registry();
        let mut reader = BufReader::new(Cursor::new(s));
        let result = line_registry.build(&mut reader, || false, |_b| {});
        assert_that!(&result).is_ok();
    }
}

mod test_skip_token {
    use crate::data_source::{Direction, LineSource, LineSourceImpl};
    use fluent_integer::Integer;
    use spectral::prelude::*;

    #[test]
    fn read_1_token_forward() {
        let src = "(a) b\n";
        let offset = test_skip_token(src, 2, Direction::Forward);
        assert_that!(offset).is_ok_containing(&4.into());
    }

    #[test]
    fn read_1_token_forward_across_line() {
        let src = "A\nBB\nCCC";
        let offset = test_skip_token(src, 0, Direction::Forward);
        assert_that!(offset).is_ok_containing(&2.into());
    }

    #[test]
    fn read_1_token_forward_across_line_should_skip_linebreaks() {
        let src = "(a)\nb\n";
        let offset = test_skip_token(src, 2, Direction::Forward);
        assert_that!(offset).is_ok_containing(&4.into());
    }

    #[test]
    fn read_1_token_backward_across_line() {
        let src = "(a)\nb\n";
        let offset = test_skip_token(src, 4, Direction::Backward);
        assert_that!(offset).is_ok_containing(&1.into());
    }

    fn test_skip_token(s: &str, offset: u64, direction: Direction) -> anyhow::Result<Integer> {
        let mut line_source = LineSourceImpl::from_str(s);
        line_source.skip_token(offset.into(), direction)
    }
}