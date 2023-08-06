use std::fmt::{Display, Formatter};
use cursive::event::Event;

pub struct EventDisplay<'a>(&'a Event);

impl<'a> EventDisplay<'a> {
    pub fn new(evt: &'a Event) -> Self {
        Self(evt)
    }
}

impl Display for EventDisplay<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Event::Char(ch) => write!(f, "{}", *ch),
            Event::CtrlChar(ch) => write!(f, "Ctrl+{}", *ch),
            Event::AltChar(ch) => write!(f, "Alt+{}", *ch),
            _ => todo!(),
        }
    }
}

pub struct EventSliceDisplay<'a>(&'a [Event]);

impl<'a> EventSliceDisplay<'a> {
    pub fn new(slice: &'a [Event]) -> Self {
        Self(slice)
    }
}

impl Display for EventSliceDisplay<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let n = self.0.len();
        for i in 0..(n - 1) {
            write!(f, "{}, ", EventDisplay::new(&self.0[i]))?
        }
        if n > 0 {
            write!(f, "{}", EventDisplay::new(&self.0[n - 1]))?
        }
        Ok(())
    }
}