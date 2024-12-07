use std::ops::Add;
use cursive::theme::{ColorStyle, Style};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct StyleWithPriority {
    style: Style,
    foreground_priority: u8,
    background_priority: u8
}

impl StyleWithPriority {
    pub fn new(style: Style, priority: StylePriority) -> Self {
        let (foreground_priority, background_priority) = priority.into();
        Self::raw(style, foreground_priority, background_priority)
    }

    pub fn raw(style: Style, foreground_priority: u8, background_priority: u8) -> Self {
        StyleWithPriority {
            style,
            foreground_priority,
            background_priority
        }
    }

    pub fn get_style(&self) -> Style {
        self.style
    }
}

impl Add for StyleWithPriority {
    type Output = StyleWithPriority;

    fn add(self, rhs: Self) -> Self::Output {
        let (fg, fp) = if self.foreground_priority > rhs.foreground_priority {
            (self.style.color.front, self.foreground_priority)
        } else {
            (rhs.style.color.front, rhs.foreground_priority)
        };
        let (bg, bp) = if self.background_priority > rhs.background_priority {
            (self.style.color.back, self.background_priority)
        } else {
            (rhs.style.color.back, rhs.background_priority)
        };
        let mut effects = self.style.effects.clone();
        effects.insert_all(rhs.style.effects);
        let style = Style {
            color: ColorStyle::new(fg, bg),
            effects
        };
        StyleWithPriority {
            style,
            foreground_priority: fp,
            background_priority: bp
        }
    }
}

pub enum StylePriority {
    Regular,
    Cursor,
    Selection,
    LineNumber,
    Date,
    Search,
    Filter,
}

impl Into<(u8, u8)> for StylePriority {
    fn into(self) -> (u8, u8) {
        match self {
            StylePriority::Regular => (0x00, 0x00),
            StylePriority::Cursor => (0xff, 0xff),
            StylePriority::Selection => (0x01, 0xfe),
            StylePriority::LineNumber => (0x01, 0xff),
            StylePriority::Date => (0x77, 0x77),
            StylePriority::Search => (0x90, 0x90),
            StylePriority::Filter => (0x80, 0x80),
        }
    }
}