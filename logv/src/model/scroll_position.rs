use fluent_integer::Integer;
use std::fmt;
use std::ops::Add;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum  ScrollPosition {
    FromBeginning { shift: Integer },
    FromEnd { shift: Integer },
}

impl fmt::Display for ScrollPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}

impl ScrollPosition {
    pub fn from_beginning(shift: impl Into<Integer>) -> Self {
        ScrollPosition::FromBeginning { shift: shift.into() }
    }

    pub fn from_end(shift: impl Into<Integer>) -> Self {
        ScrollPosition::FromEnd { shift: shift.into() }
    }
}

impl Add<Integer> for &ScrollPosition {
    type Output = ScrollPosition;

    fn add(self, rhs: Integer) -> Self::Output {
        match self {
            ScrollPosition::FromBeginning { shift } =>
                ScrollPosition::FromBeginning { shift: *shift + rhs },
            ScrollPosition::FromEnd { shift } =>
                ScrollPosition::FromEnd { shift: *shift - rhs},
        }
    }
}

impl Into<Integer> for ScrollPosition {
    fn into(self) -> Integer {
        match self {
            ScrollPosition::FromBeginning { shift } => shift,
            ScrollPosition::FromEnd { shift } => - shift,
        }
    }
}

impl From<Integer> for ScrollPosition {
    fn from(value: Integer) -> Self {
        if value >= 0 {
            ScrollPosition::FromBeginning { shift: value }
        } else {
            ScrollPosition::FromEnd { shift: -value }
        }
    }
}

impl Default for ScrollPosition {
    fn default() -> Self {
        ScrollPosition::FromBeginning { shift: 0.into() }
    }
}
