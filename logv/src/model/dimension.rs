use std::fmt;
use fluent_integer::Integer;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Dimension {
    pub width: Integer,
    pub height: Integer,
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Dimension(w={}, h={})", self.width, self.height)
    }
}

impl Dimension {
    pub fn new<I: Into<Integer>>(width: I, height: I) -> Self {
        Dimension {
            width: width.into(),
            height: height.into()
        }
    }
}

impl Default for Dimension {
    fn default() -> Self {
        Dimension::new(0, 0)
    }
}
