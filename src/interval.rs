use std::cmp::Ordering;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum IntervalBound<T> where T : PartialEq<T> {
    PositiveInfinity,
    NegativeInfinity,
    Fixed {
        value: T,
        is_included: bool,
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Interval<T>
    where T: PartialOrd<T>, T: PartialEq<T>, T: Copy {
    pub left_bound: IntervalBound<T>,
    pub right_bound: IntervalBound<T>,
}

pub struct IntervalBuilder<T> where T: PartialOrd, T: PartialEq, T: Copy {
    object: Interval<T>,
}

impl<T> IntervalBuilder<T>
    where T: PartialOrd, T: PartialEq, T: Copy {

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

impl<T> Default for Interval<T> where T: PartialOrd<T>, T: PartialEq<T>, T: Copy {
    fn default() -> Self {
        Interval {
            left_bound: IntervalBound::NegativeInfinity,
            right_bound: IntervalBound::NegativeInfinity,
        }
    }
}

impl<T> Interval<T> where T: PartialOrd<T>, T: PartialEq<T>, T: Copy {
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
}

impl<T> PartialEq<T> for IntervalBound<T> where T: PartialEq<T> {
    fn eq(&self, other: &T) -> bool {
        match self {
            IntervalBound::NegativeInfinity => false,
            IntervalBound::PositiveInfinity => false,
            IntervalBound::Fixed { value, is_included } => *is_included && value.eq(other)
        }
    }
}

impl<T> PartialOrd<T> for IntervalBound<T> where T: PartialOrd<T> {
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

// TODO
// impl<T> fmt::Debug for Interval<T> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         let (left_bracket, left_bound) = match &self.left_bound {
//             IntervalBound::NegativeInfinity => write!(f, "{}{}", "(", "-∞"),
//             IntervalBound::PositiveInfinity => { write!(f, "∅") },
//             IntervalBound::Fixed { value, true } => ("[", value.fmt(f)?)
//         }
//         write!(f, "{}{}, {}{}")
//     }
// }
//
// #[macro_export]
// macro_rules! interval {
//     ($left:expr, .., $right: expr) => {
//         Interval::builder().left_bound_inclusive($left).right_bound_inclusive($right).build()
//     };
// }