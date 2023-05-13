use cursive::theme::{Color, ColorStyle, ColorType, Style};
use fluent_integer::Integer;
use crate::highlight::highlight::{Highlight, Highlighter};
use crate::highlight::pattern_based_highlighter::PatternBasedHighlighter;
use crate::highlight::style_with_priority::StyleWithPriority;
use crate::model::model::RootModel;

pub struct DateHighlighter<T> {
    pattern_based_highlighter: PatternBasedHighlighter<T>
}

impl <T> DateHighlighter<T> {
    pub fn new(t: T) -> Self {
        let patterns = vec![
            r"(Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)\s+\d{1,2}\s+\d{2}:\d{2}:\d{2}"
        ];
        DateHighlighter {
            pattern_based_highlighter: PatternBasedHighlighter::new(patterns, t)
        }
    }
}

impl <T> Highlighter<T> for DateHighlighter<T> where T: Clone {
    fn process(&self, str: &str, offset: Integer, model: &RootModel) -> Vec<Highlight<T>> {
        self.pattern_based_highlighter.process(str, offset, model)
    }
}

pub fn create_date_highlighter() -> DateHighlighter<StyleWithPriority> {
    let style = Style::from(ColorStyle::new(ColorType::Color(Color::Rgb(0, 0, 0xff)), ColorType::InheritParent));
    DateHighlighter::new(StyleWithPriority::new(style, 0xff, 0))
}