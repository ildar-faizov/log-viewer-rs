use std::rc::Rc;
use cursive::theme::{Color, ColorStyle, ColorType, Palette, Style};
use crate::highlight::date_highlight::DateHighlighter;
use crate::highlight::highlight::Highlighter;
use crate::highlight::style_with_priority::StyleWithPriority;

pub fn cursive_highlighters(_palette: &Palette) -> Vec<Rc<dyn Highlighter<StyleWithPriority> + 'static>> {
    let style = Style::from(ColorStyle::new(ColorType::Color(Color::Rgb(0, 0, 0xff)), ColorType::InheritParent));
    vec![
        Rc::new(DateHighlighter::new(StyleWithPriority::new(style, 0xff, 0)))
    ]
}