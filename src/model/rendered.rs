use fluent_integer::Integer;
use crate::data_source::{Data, Line};
use crate::utils::GraphemeRender;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct LineRender {
    pub content: String,
    pub start: Integer, // offset of the first symbol in line
    pub end: Integer, // offset of the first symbol of the next line
    pub render: Vec<GraphemeRender>
}

impl LineRender {
    pub fn new(line: Line) -> Self {
        let render = GraphemeRender::from_string(&line.content);
        LineRender {
            content: line.content,
            start: line.start,
            end: line.end,
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
}

pub struct DataRender {
    pub lines: Vec<LineRender>,
    pub start: Option<Integer>,
    pub end: Option<Integer>,
}

impl DataRender {
    pub fn new(raw_data: Data) -> Self {
        DataRender {
            lines: raw_data.lines.into_iter().map(|line| LineRender::new(line)).collect(),
            start: raw_data.start,
            end: raw_data.end,
        }
    }
}
