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
test_display!(5, Interval::inf_closed(5), "(-∞, 5]");
test_display!(6, Interval::inf_open(5), "(-∞, 5)");
test_display!(7, Interval::closed_inf(5), "[5, +∞)");
test_display!(8, Interval::open_inf(5), "(5, +∞)");
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

macro_rules! test_eq {
    ($index: literal, $int1: expr, $int2: expr, "y") => {
        paste! {
            #[test]
            fn [<test_eq_ $index>]() {
                // assert_that!($int1.eq(&$int2)).is_true()
                assert_that!($int1).is_equal_to(&$int2);
            }
        }
    };
    ($index: literal, $int1: expr, $int2: expr, "n") => {
        paste! {
            #[test]
            fn [<test_ne_ $index>]() {
                assert_that!($int1).is_not_equal_to(&$int2);
            }
        }
    };
}

test_eq!("empty_empty", Interval::<i32>::empty(), Interval::<i32>::empty(), "y");
test_eq!("empty_all", Interval::<i32>::empty(), Interval::<i32>::all(), "n");
test_eq!("all_all", Interval::<i32>::all(), Interval::<i32>::all(), "y");
test_eq!("empty_empty_non_normailized", Interval::<i32>::empty(), Interval::open(0, 0), "y");
test_eq!("empty_non_normailized_empty_non_normailized",
    Interval::closed(5, -1),
    Interval::open(0, 0),
    "y");
test_eq!("finite_open", Interval::open(0, 1), Interval::open(0, 1), "y");
test_eq!("finite_closed", Interval::closed(0, 1), Interval::closed(0, 1), "y");
test_eq!("finite_open_finite_semi_open", Interval::open(0, 1), Interval::open_closed(0, 1), "n");
test_eq!("finite_open_finite_closed", Interval::open(0, 1), Interval::closed(0, 1), "n");
test_eq!("semi_finite_semi_finite", Interval::closed_inf(0), Interval::closed_inf(0), "y");

macro_rules! test_intersection {
    ($index: literal, $a: expr, $b: expr, $expected: expr) => {
        paste! {
            #[test]
            fn [<test_intersect_ $index>]() {
                let description = format!("{} * {} = {}", $a, $b, $expected);
                asserting!(description).that(&$a.intersect(&$b)).is_equal_to(&$expected);
            }

            #[test]
            fn [<test_intersect_ $index _reverse>]() {
                let description = format!("{} * {} = {}", $b, $a, $expected);
                asserting!(description).that(&$b.intersect(&$a)).is_equal_to(&$expected);
            }
        }
    };
}

test_intersection!("all_all", Interval::<i32>::all(), Interval::<i32>::all(), Interval::<i32>::all());
test_intersection!("all_empty", Interval::<i32>::all(), Interval::<i32>::empty(), Interval::<i32>::empty());
test_intersection!("all_point", Interval::<i32>::all(), Interval::point(0), Interval::point(0));
test_intersection!("all_finite", Interval::<i32>::all(), Interval::closed(0, 1), Interval::closed(0, 1));
test_intersection!("oo_oo", Interval::open(0, 5), Interval::open(1, 10), Interval::open(1, 5));
test_intersection!("oc_oo", Interval::open(0, 5), Interval::open_closed(1, 5), Interval::open(1, 5));
test_intersection!("inf_open_open_inf", Interval::inf_open(5), Interval::open_inf(0), Interval::open(0, 5));