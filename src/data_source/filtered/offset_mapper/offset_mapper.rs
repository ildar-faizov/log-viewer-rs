use fluent_integer::Integer;
use paste::paste;
use std::ops::{Add, Deref, Sub};

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone, Default)]
pub struct ProxyOffset(Integer);

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone, Default)]
pub struct OriginalOffset(Integer);

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone, Default)]
pub struct OffsetDelta(Integer); // OriginalOffset - ProxyOffset

impl Sub<ProxyOffset> for OriginalOffset {
    type Output = OffsetDelta;

    fn sub(self, rhs: ProxyOffset) -> Self::Output {
        OffsetDelta(self.0 - rhs.0)
    }
}

impl Sub<OffsetDelta> for OriginalOffset {
    type Output = ProxyOffset;

    fn sub(self, rhs: OffsetDelta) -> Self::Output {
        ProxyOffset::from(self.0 - rhs.0)
    }
}

impl Add<OffsetDelta> for ProxyOffset {
    type Output = OriginalOffset;

    fn add(self, rhs: OffsetDelta) -> Self::Output {
        OriginalOffset(self.0 + rhs.0)
    }
}

macro_rules! offset_impl {
        ($t: ty) => {
            impl Deref for $t {
                type Target = Integer;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl<I> From<I> for $t
            where
                I: Into<Integer>
            {
                fn from(i: I) -> Self {
                    paste! { [<$t>](i.into()) }
                }
            }

            impl<I> Add<I> for $t
            where
                I: Into<Integer>
            {
                type Output = $t;

                fn add(self, rhs: I) -> Self::Output {
                    paste! { [<$t>](self.0 + rhs.into()) }
                }
            }
        };
    }

offset_impl!(OriginalOffset);
offset_impl!(ProxyOffset);
offset_impl!(OffsetDelta);

#[derive(Debug)]
pub enum OffsetEvaluationResult {
    Exact(OriginalOffset),
    LastConfirmed(ProxyOffset, OriginalOffset),
    Unpredictable,
}

#[derive(Default)]
pub struct OffsetMapper {
    positive: PositiveOffsetMapper,
    negative: PositiveOffsetMapper,
}

/// Represents a piecewise-linear function that maps offset in proxy LineSource to real offset.
///
/// Contains points of distortion in form `x -> f(x) - x`.
/// The last entry contains largest confirmed offset, so its value may be equal to the previous one.
pub trait IOffsetMapper {
    fn eval(&self, x: ProxyOffset) -> OffsetEvaluationResult;

    fn add(&mut self, x: ProxyOffset, y: OriginalOffset) -> Result<(), ()>;

    fn confirm(&mut self, x: ProxyOffset);

    fn get_highest_known(&self) -> Option<(ProxyOffset, OriginalOffset)>;
}

impl IOffsetMapper for OffsetMapper {
    fn eval(&self, x: ProxyOffset) -> OffsetEvaluationResult {
        if *x >= 0 {
            self.positive.eval(x)
        } else {
            match self.negative.eval(invert(x)) {
                OffsetEvaluationResult::Exact(p) =>
                    OffsetEvaluationResult::Exact(invert(p)),
                OffsetEvaluationResult::LastConfirmed(po, oo) =>
                    OffsetEvaluationResult::LastConfirmed(invert(po), invert(oo)),
                r =>
                    r,
            }
        }
    }

    fn add(&mut self, x: ProxyOffset, y: OriginalOffset) -> Result<(), ()> {
        if *x >= 0 {
            self.positive.add(x, y)
        } else {
            self.negative.add(invert(x), invert(y))
        }
    }

    fn confirm(&mut self, x: ProxyOffset) {
        if *x >= 0 {
            self.positive.confirm(x)
        } else {
            self.negative.confirm(invert(x))
        }
    }

    fn get_highest_known(&self) -> Option<(ProxyOffset, OriginalOffset)> {
        self.positive.get_highest_known()
    }
}

#[derive(Default)]
struct PositiveOffsetMapper {
    pivots: Vec<(ProxyOffset, OffsetDelta)>,
}

impl IOffsetMapper for PositiveOffsetMapper {
    fn eval(&self, x: ProxyOffset) -> OffsetEvaluationResult {
        let result = self.pivots.binary_search_by_key(&x, |&(x_i, _)| x_i);
        let el = match result {
            Ok(q) => self.pivots.get(q),
            Err(0) => return OffsetEvaluationResult::Unpredictable,
            Err(q) => if q < self.pivots.len() { self.pivots.get(q - 1) } else { None }
        };
        match el {
            Some(&(_, y)) => OffsetEvaluationResult::Exact(x + y),
            None => {
                match self.pivots.last() {
                    Some(&(x_n, y_n)) => OffsetEvaluationResult::LastConfirmed(x_n, x_n + y_n),
                    None => OffsetEvaluationResult::Unpredictable,
                }
            }
        }
    }

    fn add(&mut self, x: ProxyOffset, y: OriginalOffset) -> Result<(), ()> {
        let delta = y - x;
        let mut iter = self.pivots.iter().rev();
        let mut drop_last = false;
        if let Some(&(x_n, y_n)) = iter.next() {
            if x_n > x || (x_n == x && y_n != delta) {
                return Err(());
            }
            if x_n < x && y_n == delta {
                if let Some(&(_, y_n1)) = iter.next() {
                    if y_n1 == y_n {
                        drop_last = true;
                    }
                }
            }
        }
        if drop_last {
            self.pivots.pop();
        }
        self.pivots.push((x, delta));
        Ok(())
    }

    fn confirm(&mut self, x: ProxyOffset) {
        match self.pivots.last() {
            None => {
                self.pivots.push((x, OffsetDelta(0.into())));
            }
            Some(&(x_n, y_n)) => {
                if x_n < x {
                    self.pivots.push((x, y_n));
                }
            }
        }
    }

    fn get_highest_known(&self) -> Option<(ProxyOffset, OriginalOffset)> {
        self.pivots.last().map(|(p, d)| (*p, *p + *d))
    }
}

fn invert<T>(x: T) -> T
where
    T: Deref<Target = Integer>,
    T: From<Integer>
{
    T::from(- *x - 1)
}

#[cfg(test)]
#[path="./tests.rs"]
mod tests;
