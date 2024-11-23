use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::num::ParseIntError;
use std::ops::{Add, Mul, Neg, Sub, AddAssign, SubAssign, Div, Rem, MulAssign};
use num_traits::{One, Zero};
use paste::paste;

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Default)]
pub struct Integer {
    value: i128 // TODO more memory-efficient storage
}

impl Integer {
    pub fn new(value: i128) -> Self {
        Integer {value}
    }

    pub fn abs(&self) -> Self {
        Integer::new(self.value.abs())
    }
}

impl Display for Integer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Debug for Integer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

// === Arithmetic operations ===

impl Add for Integer {
    type Output = Integer;

    fn add(self, rhs: Self) -> Self::Output {
        Integer::new(self.value + rhs.value)
    }
}

impl AddAssign for Integer {
    fn add_assign(&mut self, rhs: Self) {
        self.value += rhs.value;
    }
}

impl Sub for Integer {
    type Output = Integer;

    fn sub(self, rhs: Self) -> Self::Output {
        Integer::new(self.value - rhs.value)
    }
}

impl SubAssign for Integer {
    fn sub_assign(&mut self, rhs: Self) {
        self.value -= rhs.value;
    }
}

impl Mul for Integer {
    type Output = Integer;

    fn mul(self, rhs: Self) -> Self::Output {
        Integer::new(self.value * rhs.value)
    }
}

impl MulAssign for Integer {
    fn mul_assign(&mut self, rhs: Self) {
        self.value *= rhs.value;
    }
}

impl Neg for Integer {
    type Output = Integer;

    fn neg(self) -> Self::Output {
        Integer::new(-self.value)
    }
}

impl Div for Integer {
    type Output = Integer;

    fn div(self, rhs: Self) -> Self::Output {
        Integer::new(self.value.div(rhs.value))
    }
}

impl Rem for Integer {
    type Output = Integer;

    fn rem(self, rhs: Self) -> Self::Output {
        Integer::new(self.value.rem(rhs.value))
    }
}

macro_rules! integer_arithmetics {
    ( $($t: ty)* ) => {
        $(
            impl Add<$t> for Integer {
                type Output = Integer;

                fn add(self, rhs: $t) -> Self::Output {
                    Integer::new(self.value + rhs as i128)
                }
            }

            impl Add<Integer> for $t {
                type Output = Integer;

                fn add(self, rhs: Integer) -> Self::Output {
                    Integer::new(self as i128 + rhs.value)
                }
            }

            impl AddAssign<$t> for Integer {

                fn add_assign(&mut self, rhs: $t) {
                    *self = Integer::new(self.value + rhs as i128);
                }
            }

            impl Sub<$t> for Integer {
                type Output = Integer;

                fn sub(self, rhs: $t) -> Self::Output {
                    Integer::new(self.value - rhs as i128)
                }
            }

            impl Sub<Integer> for $t {
                type Output = Integer;

                fn sub(self, rhs: Integer) -> Self::Output {
                    Integer::new(self as i128 - rhs.value)
                }
            }

            impl SubAssign<$t> for Integer {

                fn sub_assign(&mut self, rhs: $t) {
                    *self = Integer::new(self.value - rhs as i128);
                }
            }

            impl Mul<$t> for Integer {
                type Output = Integer;

                fn mul(self, rhs: $t) -> Self::Output {
                    Integer::new(self.value * rhs as i128)
                }
            }

            impl Mul<Integer> for $t {
                type Output = Integer;

                fn mul(self, rhs: Integer) -> Self::Output {
                    Integer::new(self as i128 * rhs.value)
                }
            }

            // === Comparison ===

            impl PartialEq<Integer> for $t {
                fn eq(&self, other: &Integer) -> bool {
                    Integer::from(*self).eq(other)
                }
            }

            impl PartialOrd<Integer> for $t {
                fn partial_cmp(&self, other: &Integer) -> Option<Ordering> {
                    Integer::from(*self).partial_cmp(other)
                }
            }

            impl PartialEq<$t> for Integer {
                fn eq(&self, other: &$t) -> bool {
                    self.value == *other as i128
                }
            }

            impl PartialOrd<$t> for Integer {
                fn partial_cmp(&self, other: &$t) -> Option<Ordering> {
                    self.value.partial_cmp(&(*other as i128))
                }
            }

            // === Conversions from/into common types ===

            impl From<$t> for Integer {
                fn from(n: $t) -> Self {
                    Integer::new(n as i128)
                }
            }

            impl TryFrom<Integer> for $t {
                type Error = String;

                fn try_from(i: Integer) -> Result<Self, Self::Error> {
                    if <$t>::MIN as i128 <= i.value && i.value <= <$t>::MAX as i128 {
                        Ok(i.value as $t)
                    } else {
                        Err(format!("Numeric value {} does not fit into {} limits", i.value, std::any::type_name::<$t>()))
                    }
                }
            }

            // series of as_XXX methods to convert to primitives without checks
            paste! {
                impl Integer {
                    pub fn [<as_ $t>](&self) -> $t {
                        <$t>::try_from(*self).unwrap()
                    }
                }
            }
        )*
    }
}

// types which can be safely cast to i128
integer_arithmetics!(usize isize u8 i8 u16 i16 u32 i32 u64 i64 i128);

impl Zero for Integer {
    fn zero() -> Self {
        Integer::new(0)
    }

    fn is_zero(&self) -> bool {
        self.value.is_zero()
    }
}

impl One for Integer {
    fn one() -> Self {
        Integer::new(1)
    }
}

impl num_traits::Num for Integer {
    type FromStrRadixErr = ParseIntError;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        let value = i128::from_str_radix(str, radix)?;
        Ok(Integer::new(value))
    }
}

impl num_integer::Integer for Integer {
    fn div_floor(&self, other: &Self) -> Self {
        Integer::new(num_integer::Integer::div_floor(&self.value, &other.value))
    }

    fn mod_floor(&self, other: &Self) -> Self {
        Integer::new(self.value.mod_floor(&other.value))
    }

    fn gcd(&self, other: &Self) -> Self {
        Integer::new(self.value.gcd(&other.value))
    }

    fn lcm(&self, other: &Self) -> Self {
        Integer::new(self.value.lcm(&other.value))
    }

    fn is_multiple_of(&self, other: &Self) -> bool {
        self.value.is_multiple_of(&other.value)
    }

    fn is_even(&self) -> bool {
        self.value.is_even()
    }

    fn is_odd(&self) -> bool {
        self.value.is_odd()
    }

    fn div_rem(&self, other: &Self) -> (Self, Self) {
        let (d, r) = self.value.div_rem(&other.value);
        (Integer::new(d), Integer::new(r))
    }
}

impl PartialEq<&Integer> for Integer {
    fn eq(&self, other: &&Integer) -> bool {
        self.value == (**other).value
    }
}

impl PartialOrd<&Integer> for Integer {
    fn partial_cmp(&self, other: &&Integer) -> Option<Ordering> {
        self.value.partial_cmp(&(**other).value)
    }
}