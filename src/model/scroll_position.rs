use num_rational::Ratio;
use fluent_integer::Integer;
use std::fmt;
use num_traits::Zero;

/* Describes scroll position.
 * starting_point denotes initial scroll position. It is 0 at the beginning. A user
 * may scroll to the end (then it is 1) or choose some point in between. Belongs to [0, 1].
 * shift denotes number of lines to count from starting_point.
 *
 * E.g. when user scrolls 3 lines down from the beginning of the file, starting_point=0 and shift=3.
 * E.g. when user scrolls to the bottom, starting_point=1 and shift=0.
 */
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct ScrollPosition {
    pub starting_point: Ratio<Integer>,
    // [0, 1] - initial point in scroll area
    pub shift: Integer,
}

impl fmt::Display for ScrollPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ScrollPosition(starting_point={}/{}, shift={})", self.starting_point.numer(), self.starting_point.denom(), self.shift)
    }
}

impl ScrollPosition {
    pub fn new(starting_point: Ratio<Integer>, shift: Integer) -> Self {
        ScrollPosition {
            starting_point,
            shift,
        }
    }
}

impl Default for ScrollPosition {
    fn default() -> Self {
        ScrollPosition::new(Ratio::zero(), 0.into())
    }
}
