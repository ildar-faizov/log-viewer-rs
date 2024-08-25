use std::collections::HashMap;
use fluent_integer::Integer;
use crate::data_source::{CustomHighlight, CustomHighlights};
use crate::interval::Interval;
use crate::model::rendered::{LineNumberMissingReason, LineNumberResult};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Line {
    pub content: String, // TODO use appropriate type
    pub start: Integer, // offset of the first symbol in line
    pub end: Integer, // offset of the first symbol of the next line
    pub line_no: LineNumberResult,
    /// Every producer can store additional data along with the line itself
    pub custom_highlights: CustomHighlights,
}

impl Line {
    pub fn new<T, I>(content: T, start: I, end: I) -> Self
    where T: ToString, I: Into<Integer>
    {
        Line {
            content: content.to_string(),
            start: start.into(),
            end: end.into(),
            line_no: Err(LineNumberMissingReason::LineNumberingTurnedOff),
            custom_highlights: HashMap::new(),
        }
    }

    pub fn new_with_line_no<T, I>(content: T, start: I, end: I, line_no: u64) -> Self
    where T: ToString, I: Into<Integer>
    {
        Line {
            content: content.to_string(),
            start: start.into(),
            end: end.into(),
            line_no: Ok(line_no),
            custom_highlights: HashMap::new(),
        }
    }

    pub fn builder() -> LineBuilder {
        LineBuilder::default()
    }

    pub fn to_builder(self) -> LineBuilder {
        LineBuilder::default()
            .with_start(self.start)
            .with_end(self.end)
            .with_line_no(self.line_no)
            .with_content(self.content)
    }

    pub fn as_interval(&self) -> Interval<Integer> {
        Interval::closed(self.start, self.end)
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct LineBuilder {
    content: Option<String>,
    start: Option<Integer>,
    end: Option<Integer>,
    line_no: Option<LineNumberResult>,
    custom_highlights: Option<CustomHighlights>,
}

impl LineBuilder {

    pub fn with_content<T: ToString>(mut self, content: T) -> Self {
        self.content.replace(content.to_string());
        self
    }

    pub fn with_start<I: Into<Integer>>(mut self, start: I) -> Self {
        self.start.replace(start.into());
        self
    }

    pub fn with_end<I: Into<Integer>>(mut self, end: I) -> Self {
        self.end.replace(end.into());
        self
    }

    pub fn with_line_no(mut self, n: LineNumberResult) -> Self {
        self.line_no.replace(n);
        self
    }

    pub fn with_custom_highlight(mut self, key: &'static str, value: CustomHighlight) -> Self {
        self.custom_highlights
            .get_or_insert_with(|| HashMap::new())
            .entry(key)
            .or_default()
            .push(value);
        self
    }

    pub fn with_custom_highlights(mut self, key: &'static str, mut value: Vec<CustomHighlight>) -> Self {
        self.custom_highlights
            .get_or_insert_with(|| HashMap::new())
            .entry(key)
            .or_default()
            .append(&mut value);
        self
    }

    pub fn build(self) -> Line {
        let content = self.content.unwrap();
        let start = self.start.unwrap();
        let end = self.end.unwrap();
        let line_no = self.line_no.unwrap_or(Err(LineNumberMissingReason::LineNumberingTurnedOff));
        let custom_data = self.custom_highlights.unwrap_or_default();
        Line {
            content,
            start,
            end,
            line_no,
            custom_highlights: custom_data,
        }
    }
}