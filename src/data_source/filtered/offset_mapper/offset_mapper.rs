use std::ops::{Add, Deref, Sub};
use fluent_integer::Integer;
use paste::paste;

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

/// Represents a piecewise-linear function that maps offset in proxy LineSource to real offset.
///
/// Contains points of distortion in form `x -> f(x) - x`.
/// The last entry contains largest confirmed offset, so it's value may be equal to the previous one.
#[derive(Default)]
pub struct OffsetMapper {
    pivots: Vec<(ProxyOffset, OffsetDelta)>,
}

impl OffsetMapper {
    pub fn eval(&self, x: ProxyOffset) -> OffsetEvaluationResult {
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

    pub fn add(&mut self, x: ProxyOffset, y: OriginalOffset) -> Result<(), ()> {
        let delta = y - x;
        let mut iter = self.pivots.iter().rev();
        let mut drop_last = false;
        if let Some(&(x_n, y_n)) = iter.next() {
            if x_n > x || (x_n == x && y_n != delta) {
                return Err(());
            }
            if x_n < x && y_n == delta {
                if let Some(&(x_n1, y_n1)) = iter.next() {
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

    pub fn confirm(&mut self, x: ProxyOffset) {
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
}

#[cfg(test)]
#[path="./tests.rs"]
mod tests;
