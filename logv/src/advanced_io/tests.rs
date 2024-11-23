mod read_backwards_until {
    use std::io::{BufReader, Cursor, Seek, SeekFrom};
    use itertools::Itertools;
    use paste::paste;
    use crate::advanced_io::advanced_buf_reader::BidirectionalBufRead;
    use spectral::prelude::*;

    const TEXT: &str = "aaa\nbbb\n\ncd";

    macro_rules! test {
        ($start_from: literal, $expected: literal, $expected_buf: literal) => {
            paste! {
                #[test]
                fn [< test_ $start_from >]() {
                    let cursor = Cursor::new(TEXT);
                    let mut reader = BufReader::new(cursor);
                    reader.seek(SeekFrom::Start($start_from))
                        .expect("Failed to move reader position");
                    let mut buf = Vec::new();
                    let result = reader.read_backwards_until(|b| b == b'\n', |b| buf.push(b));
                    buf.reverse();
                    assert_that!(result).is_ok().is_equal_to($expected);
                    assert_that!(buf).is_equal_to($expected_buf.bytes().collect_vec());
                }
            }
        };
    }

    test!(0, 0, "");
    test!(1, 1, "a");
    test!(2, 2, "aa");
    test!(3, 3, "aaa");
    test!(4, 0, "");
    test!(5, 1, "b");
    test!(6, 2, "bb");
    test!(7, 3, "bbb");
    test!(8, 0, "");
    test!(9, 0, "");
    test!(10, 1, "c");
    test!(11, 2, "cd");
}