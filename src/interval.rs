use std::cmp::{max, min, Ordering};
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

#[derive(Copy, Clone, Debug)]
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

    pub fn inf_closed(s: T) -> Self {
        Self::builder().left_unbounded().right_bound_inclusive(s).build()
    }

    pub fn open_inf(s: T) -> Self {
        Self::builder().left_bound_exclusive(s).right_unbounded().build()
    }

    pub fn inf_open(s: T) -> Self {
        Self::builder().left_unbounded().right_bound_exclusive(s).build()
    }

    pub fn point(point: T) -> Self {
        Self::closed(point, point)
    }

    pub fn intersect(&self, other: &Interval<T>) -> Interval<T> {
        let left_bound = max(LeftBound(self.left_bound), LeftBound(other.left_bound)).0;
        let right_bound = min(RightBound(self.right_bound), RightBound(other.right_bound)).0;
        Interval {
            left_bound,
            right_bound
        }
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

    pub fn point_location(&self, point: &T) -> PointLocationWithRespectToInterval {
        if self.is_empty() {
            return PointLocationWithRespectToInterval::Undefined;
        }
        let bound = IntervalBound::Fixed { value: *point, is_included: true };
        let left = self.left_bound.cmp(&bound, BoundSide::Left);
        let right = self.right_bound.cmp(&bound, BoundSide::Right);
        match (left, right) {
            (Ordering::Less, Ordering::Less) => PointLocationWithRespectToInterval::Greater,
            (Ordering::Greater, Ordering::Greater) => PointLocationWithRespectToInterval::Less,
            _ => PointLocationWithRespectToInterval::Belongs,
        }
    }

    pub fn map<U, F>(&self, f: F) -> Interval<U>
        where
            U: Ord + Eq + Copy,
            F: Fn(&T) -> U,
            F: Copy,
    {
        let f = &f;
        Interval {
            left_bound: self.left_bound.map(f),
            right_bound: self.right_bound.map(f),
        }
    }
}

impl<T> PartialEq<Self> for Interval<T> where T: Copy + Eq + Ord {
    fn eq(&self, other: &Self) -> bool {
        let empty1 = self.is_empty();
        let empty2 = other.is_empty();
        if !empty1 && !empty2 {
            self.left_bound.eq(&other.left_bound)
                && self.right_bound.eq(&other.right_bound)
        } else {
            empty1 == empty2
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum PointLocationWithRespectToInterval {
    Undefined, // empty interval
    Less,
    Belongs,
    Greater,
}

enum BoundSide {
    Left,
    Right
}

impl<T> IntervalBound<T> where T: Ord {
    pub fn map<U, F>(&self, f: F) -> IntervalBound<U>
    where
        U: Ord,
        F: FnOnce(&T) -> U
    {
        match &self {
            IntervalBound::Fixed { value, is_included } => IntervalBound::Fixed {
                value: f(value),
                is_included: *is_included,
            },
            IntervalBound::NegativeInfinity => IntervalBound::NegativeInfinity,
            IntervalBound::PositiveInfinity => IntervalBound::PositiveInfinity,
        }
    }

    fn cmp(&self, other: &IntervalBound<T>, side: BoundSide) -> Ordering {
        match &self {
            IntervalBound::NegativeInfinity => {
                match other {
                    IntervalBound::NegativeInfinity => Ordering::Equal,
                    _ => Ordering::Less,
                }
            },
            IntervalBound::PositiveInfinity => {
                match other {
                    IntervalBound::PositiveInfinity => Ordering::Equal,
                    _ => Ordering::Greater,
                }
            },
            IntervalBound::Fixed { value: v1, is_included: i1 } => {
                match other {
                    IntervalBound::NegativeInfinity => Ordering::Greater,
                    IntervalBound::PositiveInfinity => Ordering::Less,
                    IntervalBound::Fixed { value: v2, is_included: i2 } => {
                        match v1.cmp(v2) {
                            Ordering::Equal => {
                                if *i1 == *i2 {
                                    Ordering::Equal
                                } else {
                                    match side {
                                        BoundSide::Left => if *i1 { Ordering::Less } else { Ordering::Greater },
                                        BoundSide::Right => if *i1 { Ordering::Greater } else { Ordering::Less },
                                    }
                                }
                            },
                            result => result
                        }
                    }
                }
            }
        }
    }
}

#[derive(Eq, PartialEq)]
struct LeftBound<T>(IntervalBound<T>) where T: Ord + Eq + Copy;

#[derive(Eq, PartialEq)]
struct RightBound<T>(IntervalBound<T>) where T: Ord + Eq + Copy;

impl<T> PartialOrd<Self> for LeftBound<T> where T: Ord + Eq + Copy {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0, BoundSide::Left))
    }
}

impl<T> Ord for LeftBound<T> where T: Ord + Eq + Copy {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0, BoundSide::Left)
    }
}

impl<T> PartialOrd<Self> for RightBound<T> where T: Ord + Eq + Copy {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0, BoundSide::Right))
    }
}

impl<T> Ord for RightBound<T> where T: Ord + Eq + Copy {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0, BoundSide::Right)
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