use regex::Regex;
use fluent_integer::Integer;
use crate::highlight::highlight::{Highlight, Highlighter};
use crate::model::model::RootModel;
use crate::utils::measure;

pub struct PatternBasedHighlighter<T> {
    patterns: Vec<Regex>,
    t: T
}

impl <T> PatternBasedHighlighter<T> {
    pub fn new(patterns: Vec<&str>, t: T) -> Self {
        PatternBasedHighlighter {
            patterns: patterns.iter().map(|p| Regex::new(p).unwrap()).collect(),
            t
        }
    }
}

impl <T> Highlighter<T> for PatternBasedHighlighter<T> where T: Clone {
    fn process(&self, str: &str, _offset: Integer, _model: &RootModel) -> Vec<Highlight<T>> {
        measure("PatternBasedHighlight process", || {
            self.patterns.iter().flat_map(|regex|
                regex.captures_iter(str)
                    .flat_map(|caps| {
                        caps.iter()
                            .flatten()
                            .map(|c| Highlight::new(c.start(), c.end(), self.t.clone()))
                            .collect::<Vec<Highlight<T>>>()
                    })
                    .collect::<Vec<Highlight<T>>>()
            ).collect::<Vec<Highlight<T>>>()
        })
    }
}