use std::borrow::Cow;
use std::cell::RefMut;
use fluent_integer::Integer;
use crate::data_source::{Direction, LineSource};
use crate::model::rendered::{DataRender, LineRender};

pub struct LineIterator<'a> {
    cache: &'a DataRender,
    datasource: RefMut<'a, Box<dyn LineSource>>,
    direction: Direction,
    current_line: Option<Cow<'a, LineRender>>,
    line_number: Integer,
    started: bool,
    exhausted: bool,
}

impl<'a> LineIterator<'a> {
    pub fn new(
        cache: &'a DataRender,
        datasource: RefMut<'a, Box<dyn LineSource>>,
        direction: Direction,
        line_number: Integer,
    ) -> Self {
        LineIterator {
            cache,
            datasource,
            direction,
            current_line: None,
            line_number,
            started: false,
            exhausted: false,
        }
    }

    fn read_non_cached_line_backward(&mut self) -> Option<Cow<'a, LineRender>> {
        let datasource = &mut *self.datasource;
        self.current_line.as_ref()
            .map(|current_line| current_line.start - 1)
            .and_then(|s| datasource.read_prev_line(s))
            .map(LineRender::new)
            .map(Cow::Owned)
    }

    fn read_non_cached_line_forward(&mut self) -> Option<Cow<'a, LineRender>> {
        let datasource = &mut *self.datasource;
        self.current_line.as_ref()
            .map(|current_line| current_line.end + 1)
            .and_then(|s| datasource.read_next_line(s))
            .map(LineRender::new)
            .map(Cow::Owned)
    }

    fn replace_empty_line_with_space(line_render: Cow<LineRender>) -> Cow<LineRender> {
        if line_render.content.is_empty() {
            let replacement = line_render.to_builder()
                .with_content(String::from(' '))
                .build();
            Cow::Owned(replacement)
        } else {
            line_render
        }
    }
}

impl<'a> Iterator for LineIterator<'a> {
    type Item = Cow<'a, LineRender>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        if !self.started {
            self.started = true;
            let next_line = self.cache.lines.get(self.line_number.as_usize());
            return match next_line {
                Some(line_ref) => {
                    self.current_line = Some(Cow::Borrowed(line_ref));
                    Some(LineIterator::replace_empty_line_with_space(Cow::Borrowed(line_ref)))
                }
                None => {
                    self.exhausted = true;
                    None
                }
            };
        }

        let next_line = match self.direction {
            Direction::Forward => {
                let next_line_number: Integer = self.line_number + 1;
                if self.line_number >= 0 {
                    if let Some(line) = self.cache.lines.get(next_line_number.as_usize()) {
                        Some(Cow::Borrowed(line))
                    } else {
                        self.read_non_cached_line_forward()
                    }
                } else {
                    panic!("Impossible situation");
                }
            }
            Direction::Backward => {
                let next_line_number: Integer = self.line_number - 1;
                if next_line_number >= 0 {
                    if let Some(line) = self.cache.lines.get(next_line_number.as_usize()) {
                        Some(Cow::Borrowed(line))
                    } else {
                        self.read_non_cached_line_backward()
                    }
                } else {
                    self.read_non_cached_line_backward()
                }
            }
        };
        if next_line.is_none() {
            self.exhausted = true;
        }
        self.current_line = next_line;
        self.line_number += match self.direction {
            Direction::Forward => 1,
            Direction::Backward => -1,
        };
        self.current_line.clone().map(LineIterator::replace_empty_line_with_space)
    }
}
