use crate::highlight::highlight::{Highlight, Highlighter};
use crate::highlight::pattern_based_highlighter::PatternBasedHighlighter;

pub struct DateHighlighter<T> {
    pattern_based_highlighter: PatternBasedHighlighter<T>
}

impl <T> DateHighlighter<T> {
    pub fn new(t: T) -> Self {
        let patterns = vec![
            r"(Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec) \d{1,2} \d{2}:\d{2}:\d{2}"
        ];
        DateHighlighter {
            pattern_based_highlighter: PatternBasedHighlighter::new(patterns, t)
        }
    }
}

impl <T> Highlighter<T> for DateHighlighter<T> where T: Clone {
    fn process(&self, str: &str) -> Vec<Highlight<T>> {
        self.pattern_based_highlighter.process(str)
    }
}