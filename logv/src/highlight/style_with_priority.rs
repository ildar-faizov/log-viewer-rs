use std::ops::Add;
use cursive::theme::{ColorStyle, Style};

#[derive(Copy, Clone)]
pub struct StyleWithPriority {
    style: Style,
    foreground_priority: u8,
    background_priority: u8
}

impl StyleWithPriority {
    pub fn new(style: Style, foreground_priority: u8, background_priority: u8) -> Self {
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
        let style = Style::from(ColorStyle::new(fg, bg));
        StyleWithPriority {
            style,
            foreground_priority: fp,
            background_priority: bp
        }
    }
}