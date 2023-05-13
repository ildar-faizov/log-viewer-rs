use cursive::theme::{Color, ColorStyle, ColorType, Style};
use fluent_integer::Integer;
use crate::highlight::highlight::{Highlight, Highlighter};
use crate::highlight::pattern_based_highlighter::PatternBasedHighlighter;
use crate::highlight::style_with_priority::StyleWithPriority;
use crate::model::model::RootModel;

pub struct SearchHighlighter<T> {
    current_occurrence_style: T,
    other_occurrence_style: T,
}

impl <T> SearchHighlighter<T> {
    pub fn new(current_occurrence_style: T, other_occurrence_style: T) -> SearchHighlighter<T> {
        SearchHighlighter {
            current_occurrence_style,
            other_occurrence_style
        }
    }
}

impl<T: Clone> Highlighter<T> for SearchHighlighter<T> {
    fn process(&self, str: &str, offset: Integer, model: &RootModel) -> Vec<Highlight<T>> {
        let search_model = model.get_search_model();
        let current_occurrence = search_model.get_current_occurrence();
        let pattern = search_model.get_pattern();
        if !pattern.is_empty() {
            let mut result = vec![];
            let mut p = 0;
            while let Some(q) = str[p..].find(pattern) {
                let style = if current_occurrence.filter(|c| *c == offset + p + q).is_some() {
                    self.current_occurrence_style.clone()
                } else {
                    self.other_occurrence_style.clone()
                };
                result.push(Highlight::new(p + q, p + q + pattern.len(), style));
                p += q + 1;
            }
            result
            // TODO: handle regexp
            // let h = PatternBasedHighlighter::new(vec![pattern], self.other_occurrence_style.clone());
            // h.process(str, model)
        } else {
            vec![]
        }
    }
}

pub fn create_search_highlighter() -> SearchHighlighter<StyleWithPriority> {
    let current_occurrences_style = Style::from(ColorStyle::new(ColorType::InheritParent, ColorType::Color(Color::Rgb(6, 152, 154))));
    let other_occurrences_style = Style::from(ColorStyle::new(ColorType::InheritParent, ColorType::Color(Color::Rgb(114, 159, 207))));
    SearchHighlighter::new(
        StyleWithPriority::new(current_occurrences_style, 0, 0),
        StyleWithPriority::new(other_occurrences_style, 0, 0),
    )
}