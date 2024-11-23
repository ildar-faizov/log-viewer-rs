use std::convert::TryFrom;
use fluent_integer::Integer;
use ::paste::paste;

macro_rules! test_plus {
    ( $t1: ty, $t2: ty, $v1: literal, $v2: literal, $expected: literal) => {
        paste! {
            #[test]
            fn [<$t1 _plus_ $t2>]() {
                let i = Integer::from($v1 as $t1);
                let j: $t2 = $v2;
                assert_eq!(Integer::new($expected), i + j);
            }

            #[test]
            fn [<$t1 _plus_assign_ $t2>]() {
                let mut i = Integer::from($v1 as $t1);
                let j: $t2 = $v2;
                i += j;
                assert_eq!(Integer::new($expected), i);
            }
        }
    }
}

// usize + T
test_plus!(usize, usize, 2, 8, 10);
test_plus!(usize, isize, 2, -3, -1);
test_plus!(usize, u8, 2, 255, 257);
test_plus!(usize, i8, 2, -127, -125);
test_plus!(usize, u16, 120, 65535, 65655);
test_plus!(usize, i16, 2001, -32000, -29999);
test_plus!(usize, u32, 120, 100_000, 100_120);
test_plus!(usize, i32, 2001, -100_000, -97999);
test_plus!(usize, u64, 120, 9223372036854775808, 9223372036854775928); // 120 + 2^63
test_plus!(usize, i64, 2001, -9223372036854775807, -9223372036854773806); // 2001 + -(2^63 - 1)

// isize + T
test_plus!(isize, usize, -2, 65000, 64998);
test_plus!(isize, isize, -2, -10, -12);
// TODO other types

// TODO minus, multiplication, negation

macro_rules! test_try_from {
    ( $( $t: ty ),* ) => {
        $(
            paste! {
                #[test]
                fn [<test_try_from_integer_for_ $t _max>]() {
                    let value: $t = <$t>::MAX;
                    let i = Integer::from(value);
                    assert_eq!(Ok(value), <$t>::try_from(i));
                }

                #[test]
                fn [<test_try_from_integer_for_ $t _min>]() {
                    let value: $t = <$t>::MIN;
                    let i = Integer::from(value);
                    assert_eq!(Ok(value), <$t>::try_from(i));
                }

                #[test]
                fn [<test_try_from_integer_for_ $t _max_overflow>]() {
                    let i = Integer::from(<$t>::MAX as i128 + 1);
                    assert!(<$t>::try_from(i).is_err());
                }

                #[test]
                fn [<test_try_from_integer_for_ $t _min_overflow>]() {
                    let i = Integer::from(<$t>::MIN as i128 - 1);
                    assert!(<$t>::try_from(i).is_err());
                }
            }
        )*
    }
}

test_try_from!(usize, isize, i8, u8, i16, u16, i32, u32, i64, u64);

macro_rules! test_as_type {
    ( $( $t: ty ),* ) => {
        $(
            paste! {
                #[test]
                fn [<test_as_ $t _max>]() {
                    let value: $t = <$t>::MAX;
                    let i = Integer::from(value);
                    assert_eq!(value, i.[<as_ $t>]());
                }
            }
        )*
    }
}

test_as_type!(usize, isize, i8, u8, i16, u16, i32, u32, i64, u64);

macro_rules! test_cmp {
    ( $( $t: ty),* ) => {
        $(
            paste! {
                #[test]
                fn [<test_integer_gt_than_ $t>]() {
                    let value: $t = 0;
                    let i = Integer::from(1);
                    assert!(i > value);
                }

                // TODO ge, lt, le, ==
            }
        )*
    }
}

test_cmp!(usize, isize, i8, u8, i16, u16, i32, u32, i64, u64);

// TODO abs