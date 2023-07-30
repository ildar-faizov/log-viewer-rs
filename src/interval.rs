use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum IntervalBound<T> where T : Eq {
    PositiveInfinity,
    NegativeInfinity,
    Fixed {
        value: T,
        is_included: bool,
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Interval<T>
    where T: Ord, T: Eq, T: Copy {
    pub left_bound: IntervalBound<T>,
    pub right_bound: IntervalBound<T>,
}

pub struct IntervalBuilder<T> where T: Ord, T: Eq, T: Copy {
    object: Interval<T>,
}

impl<T> IntervalBuilder<T>
    where T: Ord, T: Eq, T: Copy {

    pub fn left_bound_inclusive(self, value: T) -> Self {
        self.left_bound(value, true)
    }

    pub fn left_bound_exclusive(self, value: T) -> Self {
        self.left_bound(value, false)
    }

    pub fn left_bound(mut self, value: T, is_included: bool) -> Self {
        self.object.left_bound = IntervalBound::Fixed {
            value,
            is_included
        };
        self
    }

    pub fn left_unbounded(mut self) -> Self {
        self.object.left_bound = IntervalBound::NegativeInfinity;
        self
    }

    pub fn right_bound_inclusive(self, value: T) -> Self {
        self.right_bound(value, true)
    }

    pub fn right_bound_exclusive(self, value: T) -> Self {
        self.right_bound(value, false)
    }

    pub fn right_bound(mut self, value: T, is_included: bool) -> Self {
        self.object.right_bound = IntervalBound::Fixed {
            value,
            is_included,
        };
        self
    }

    pub fn right_unbounded(mut self) -> Self {
        self.object.right_bound = IntervalBound::PositiveInfinity;
        self
    }

    pub fn build(self) -> Interval<T> {
        self.object
    }
}

impl<T> Default for Interval<T> where T: Ord, T: Eq, T: Copy {
    fn default() -> Self {
        Interval {
            left_bound: IntervalBound::NegativeInfinity,
            right_bound: IntervalBound::NegativeInfinity,
        }
    }
}

impl<T> Interval<T> where T: Ord, T: Eq, T: Copy {
    pub fn builder() -> IntervalBuilder<T> {
        IntervalBuilder {
            object: Interval::default()
        }
    }

    pub fn all() -> Self {
        Interval {
            left_bound: IntervalBound::NegativeInfinity,
            right_bound: IntervalBound::PositiveInfinity,
        }
    }

    pub fn empty() -> Self {
        Interval {
            left_bound: IntervalBound::NegativeInfinity,
            right_bound: IntervalBound::NegativeInfinity,
        }
    }

    pub fn closed(s: T, e: T) -> Self {
        Self::builder().left_bound_inclusive(s).right_bound_inclusive(e).build()
    }

    pub fn open(s: T, e: T) -> Self {
        Self::builder().left_bound_exclusive(s).right_bound_exclusive(e).build()
    }

    pub fn open_closed(s: T, e: T) -> Self {
        Self::builder().left_bound_exclusive(s).right_bound_inclusive(e).build()
    }

    pub fn closed_open(s: T, e: T) -> Self {
        Self::builder().left_bound_inclusive(s).right_bound_exclusive(e).build()
    }

    pub fn closed_inf(s: T) -> Self {
        Self::builder().left_bound_inclusive(s).right_unbounded().build()
    }

    pub fn point(point: T) -> Self {
        Self::closed(point, point)
    }

    pub fn contains_point(&self, value: &T) -> bool {
        self.left_bound <= *value && self.right_bound >= *value
    }

    pub fn contains_interval(&self, _other: &Interval<T>) -> bool {
        todo!()
    }

    pub fn to_builder(self) -> IntervalBuilder<T> {
        IntervalBuilder {
            object: self
        }
    }

    pub fn is_empty(&self) -> bool {
        match self.left_bound {
            IntervalBound::NegativeInfinity => self.right_bound == IntervalBound::NegativeInfinity,
            IntervalBound::PositiveInfinity => true,
            IntervalBound::Fixed { value: lb, is_included: lbb } => {
                match self.right_bound {
                    IntervalBound::PositiveInfinity => false,
                    IntervalBound::NegativeInfinity => true,
                    IntervalBound::Fixed { value: rb, is_included: rbb } => {
                        match lb.cmp(&rb) {
                            Ordering::Less => false,
                            Ordering::Greater => true,
                            Ordering::Equal => !lbb || !rbb,
                        }
                    }
                }
            }
        }
    }
}

impl<T> PartialEq<T> for IntervalBound<T> where T: Eq {
    fn eq(&self, other: &T) -> bool {
        match self {
            IntervalBound::NegativeInfinity => false,
            IntervalBound::PositiveInfinity => false,
            IntervalBound::Fixed { value, is_included } => *is_included && value.eq(other)
        }
    }
}

impl<T> PartialOrd<T> for IntervalBound<T> where T: Ord {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        match &self {
            IntervalBound::NegativeInfinity => Some(Ordering::Less),
            IntervalBound::PositiveInfinity => Some(Ordering::Greater),
            IntervalBound::Fixed { value, is_included } => {
                match value.partial_cmp(other) {
                    Some(Ordering::Equal) => {
                        if *is_included {
                            Some(Ordering::Equal)
                        } else {
                            None
                        }
                    },
                    t => t
                }
            }
        }
    }
}

impl<T> Display for Interval<T>
    where T: Ord, T: Eq, T: Copy, T: Display
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "∅")
        }
        match &self.left_bound {
            IntervalBound::NegativeInfinity => write!(f, "(-∞")?,
            IntervalBound::PositiveInfinity => {
                return write!(f, "∅")
            },
            IntervalBound::Fixed { value, is_included } => {
                let bracket = if *is_included { "[" } else { "(" };
                write!(f, "{}{}", bracket, value)?
            }
        };
        write!(f, ", ")?;
        match &self.right_bound {
            IntervalBound::NegativeInfinity => {
                return write!(f, "∅")
            },
            IntervalBound::PositiveInfinity => write!(f, "+∞)")?,
            IntervalBound::Fixed { value, is_included } => {
                let bracket = if *is_included { "]" } else { ")" };
                write!(f, "{}{}", value, bracket)?
            }
        };
        Ok(())
    }
}

// Tests are included according to http://xion.io/post/code/rust-unit-test-placement.html
#[cfg(test)]
#[path = "./interval_tests.rs"]
mod interval_tests;