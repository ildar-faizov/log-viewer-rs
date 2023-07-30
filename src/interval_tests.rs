use spectral::prelude::*;
use crate::interval::Interval;
use paste::paste;

macro_rules! test_display {
    ($index: literal, $interval: expr, $expected: expr) => {
        paste! {
            #[test]
            fn [<test_display_ $index>]() {
                assert_that!(format!("{}", $interval).as_str()).is_equal_to(&$expected)
            }
        }
    }
}

test_display!(1, Interval::closed(1, 2), "[1, 2]");
test_display!(2, Interval::open_closed(-1, 2), "(-1, 2]");
test_display!(3, Interval::closed_open(-1, 2), "[-1, 2)");
test_display!(4, Interval::open(-3, 0), "(-3, 0)");
test_display!(5, Interval::builder().left_unbounded().right_bound_inclusive(5).build(), "(-∞, 5]");
test_display!(6, Interval::builder().left_unbounded().right_bound_exclusive(5).build(), "(-∞, 5)");
test_display!(7, Interval::builder().left_bound_inclusive(5).right_unbounded().build(), "[5, +∞)");
test_display!(8, Interval::builder().left_bound_exclusive(5).right_unbounded().build(), "(5, +∞)");
test_display!(9, Interval::point(3), "[3, 3]");
test_display!(10, Interval::<i32>::all(), "(-∞, +∞)");
test_display!(11, Interval::<i32>::empty(), "∅");

macro_rules! test_empty {
    ($index: literal, $interval: expr, "empty") => {
        paste! {
            #[test]
            fn [<test_empty_ $index>]() {
                assert_that!($interval.is_empty()).is_true()
            }
        }
    };
    ($index: literal, $interval: expr, "nonempty") => {
        paste! {
            #[test]
            fn [<test_empty_ $index>]() {
                assert_that!($interval.is_empty()).is_false()
            }
        }
    };
}

test_empty!(1, Interval::open(1, 1), "empty");
test_empty!(2, Interval::closed(1, 1), "nonempty");
test_empty!(3, Interval::<i32>::empty(), "empty");
test_empty!(4, Interval::<i32>::all(), "nonempty");
test_empty!(5, Interval::closed_inf(0), "nonempty");
