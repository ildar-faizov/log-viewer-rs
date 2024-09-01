use crate::data_source::line_registry::LineRegistryError;
use crate::data_source::{CustomHighlights, Data, Line, LineBuilder};
use crate::utils::GraphemeRender;
use fluent_integer::Integer;
use thiserror::Error;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LineRender {
    pub content: String,
    pub start: Integer, // offset of the first symbol in line
    pub end: Integer, // offset of the first symbol of the next line
    pub line_no: LineNumberResult,
    pub custom_highlights: CustomHighlights,
    pub render: Vec<GraphemeRender>
}

impl LineRender {
    pub fn new(line: Line) -> Self {
        let render = GraphemeRender::from_string(&line.content);
        LineRender {
            content: line.content,
            start: line.start,
            end: line.end,
            line_no: line.line_no,
            custom_highlights: line.custom_highlights,
            render,
        }
    }

    pub fn find_grapheme_by_offset(&self, offset: Integer) -> Option<(usize, &GraphemeRender)> {
        let r = self.render.binary_search_by_key(&offset, |g| g.original_offset.into());
        let pos = match r {
            Ok(mut c) => {
                loop {
                    let mut should_continue = false;
                    if c > 0 {
                        if let Some(prev) = self.render.get(c - 1) {
                            if prev.original_offset == offset {
                                should_continue = true;
                            }
                        }
                    }
                    if should_continue {
                        c -= 1;
                    } else {
                        break;
                    }
                }
                c
            },
            Err(0) => 0,
            Err(c) => c - 1,
        };
        self.render.get(pos).map(|g| (pos, g))
    }

    pub fn find_grapheme_index_by_offset(&self, offset: Integer) -> Option<usize> {
        self.find_grapheme_by_offset(offset).map(|(pos, _)| pos)
    }

    pub fn to_builder(&self) -> LineRenderBuilder {
        LineRenderBuilder::default()
            .with_content(self.content.clone())
            .with_start(self.start)
            .with_end(self.end)
            .with_line_no(self.line_no.clone())
            .with_custom_highlights(self.custom_highlights.clone())
    }
}

#[derive(Debug, Default, Clone)]
pub struct LineRenderBuilder {
    line_builder: LineBuilder
}

impl LineRenderBuilder {

    pub fn with_content<T: ToString>(mut self, content: T) -> Self {
        self.line_builder = self.line_builder.with_content(content);
        self
    }

    pub fn with_start<I: Into<Integer>>(mut self, start: I) -> Self {
        self.line_builder = self.line_builder.with_start(start);
        self
    }

    pub fn with_end<I: Into<Integer>>(mut self, end: I) -> Self {
        self.line_builder = self.line_builder.with_end(end);
        self
    }

    pub fn with_line_no(mut self, n: LineNumberResult) -> Self {
        self.line_builder = self.line_builder.with_line_no(n);
        self
    }

    pub fn with_custom_highlights(mut self, custom_highlights: CustomHighlights) -> Self {
        self.line_builder = self.line_builder.with_custom_highlights(custom_highlights);
        self
    }

    pub fn build(self) -> LineRender {
        LineRender::new(self.line_builder.build())
    }
}

pub struct DataRender {
    pub lines: Vec<LineRender>,
    pub start: Option<Integer>,
    pub end: Option<Integer>,
}

impl DataRender {
    pub fn new(raw_data: Data) -> Self {
        DataRender {
            lines: raw_data.lines.into_iter().map(LineRender::new).collect(),
            start: raw_data.start,
            end: raw_data.end,
        }
    }
}

// TODO move it to line
#[derive(Error, Debug, Clone, Eq, PartialEq)]
pub enum LineNumberMissingReason {
    #[error("Line numbering is turned off")]
    LineNumberingTurnedOff,
    #[error("Line registry error")]
    Delegate(#[from] LineRegistryError),
    #[error("Missing Data")]
    MissingData,
}

pub type LineNumberResult = Result<u64, LineNumberMissingReason>;