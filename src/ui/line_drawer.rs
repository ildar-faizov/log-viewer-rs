use std::rc::Rc;
use std::fmt::Write;
use cursive::theme::Style;
use cursive::utils::span::{IndexedCow, IndexedSpan, SpannedString};
use crate::data_source::line_registry::LineRegistryError;
use crate::highlight::highlight::Highlighter;
use crate::highlight::style_with_priority::StyleWithPriority;
use crate::model::model::RootModel;
use crate::model::rendered::{LineNumberMissingReason, LineRender};
use crate::ui::span_producer::SpanProducer;

const LINE_NO_DELIMITER: &str = " | ";
const LINE_NO_DELIMITER_WIDTH: usize = LINE_NO_DELIMITER.len();
const LINE_NO_DELIMITER_BYTE_WIDTH: usize = LINE_NO_DELIMITER.as_bytes().len();
// const LOADING_INDICATOR: &str = "⌛"; TODO for some reason printing line with this symbol drops one space
const LOADING_INDICATOR: &str = "⧖";
const ERROR_INDICATOR: &str = "⚠";

#[derive(Default)]
pub struct LineDrawer<'a> {
    state: Option<&'a RootModel>,
    highlighters: Option<&'a Vec<Rc<dyn Highlighter<StyleWithPriority> + 'static>>>,
    width: Option<usize>,
    regular_style: Option<StyleWithPriority>,
    cursor_style: Option<StyleWithPriority>,
    selection_style: Option<StyleWithPriority>,
    line_number_style: Option<StyleWithPriority>,
    show_line_numbers: bool,
    max_line_number: u64,
}

impl<'a> LineDrawer<'a> {
    pub fn new<'b>() -> LineDrawer<'b> {
        LineDrawer::default()
    }

    pub fn with_state(mut self, state: &'a RootModel) -> Self {
        self.state.replace(state);
        self
    }

    pub fn with_highlighters(mut self, highlighters: &'a Vec<Rc<dyn Highlighter<StyleWithPriority> + 'static>>) -> Self {
        self.highlighters.replace(highlighters);
        self
    }

    pub fn with_width(mut self, width: usize) -> Self {
        self.width.replace(width);
        self
    }

    pub fn with_regular_style(mut self, regular_style: StyleWithPriority) -> Self {
        self.regular_style.replace(regular_style);
        self
    }

    pub fn with_cursor_style(mut self, cursor_style: StyleWithPriority) -> Self {
        self.cursor_style.replace(cursor_style);
        self
    }

    pub fn with_selection_style(mut self, selection_style: StyleWithPriority) -> Self {
        self.selection_style.replace(selection_style);
        self
    }

    pub fn with_line_number_style(mut self, line_number_style: StyleWithPriority) -> Self {
        self.line_number_style.replace(line_number_style);
        self
    }

    pub fn with_show_line_numbers(mut self, show_line_numbers: bool) -> Self {
        self.show_line_numbers = show_line_numbers;
        self
    }

    pub fn with_max_line_number(mut self, max_line_number: u64) -> Self {
        self.max_line_number = max_line_number;
        self
    }

    pub fn draw(&self, line: &LineRender) -> SpannedString<Style> {
        let mut result = self.draw_line_number(line)
            .unwrap_or_else(|err| {
                log::warn!("Failed to draw line number: {:?}", err);
                SpannedString::new()
            });

        let state = self.state.unwrap();
        let highlighters = self.highlighters.unwrap();
        let line_span_width = result.source().len();
        let width = self.width.map(|w| w.saturating_sub(line_span_width)).unwrap();
        let regular_style = self.regular_style.unwrap();
        let cursor_style = self.cursor_style.unwrap();
        let selection_style = self.selection_style.unwrap();

        let horizontal_scroll = state.get_horizontal_scroll().as_usize();
        let cursor = state.get_cursor();

        let get_visible_graphemes = || line.render.iter()
            .skip(horizontal_scroll)
            .take(width);

        let selection = state.get_selection();

        let line_str = if let Some(first_grapheme) = get_visible_graphemes().next() {
            let display_str = get_visible_graphemes()
                .map(|g| g.render.resolve(line.content.as_str()))
                .fold(String::with_capacity(width), |mut acc, item| {
                    acc += item;
                    acc
                });
            let first_offset = first_grapheme.render_offset;
            let display_len = get_visible_graphemes().count();

            let mut intervals = SpanProducer::new(horizontal_scroll, display_len);
            intervals.add_interval_without_shift(0_u8, display_len, regular_style);

            if cursor >= line.start && cursor <= line.end {
                if let Some((pos, g)) = line.find_grapheme_by_offset(cursor - line.start) {
                    if g.is_first_in_original {
                        intervals.add_interval(pos, pos + 1, cursor_style);
                    }
                }
            }

            if let Some(selection) = selection {
                if selection.start <= line.end && selection.end >= (line.start + first_grapheme.original_offset) {
                    let selection_start = line.find_grapheme_index_by_offset(selection.start - line.start);
                    let selection_end = line.find_grapheme_by_offset(selection.end - line.start);
                    if let Some((s, (mut e, g))) = selection_start.zip(selection_end) {
                        if g.original_offset < selection.end - line.start {
                            e += 1;
                        }
                        intervals.add_interval(s, e, selection_style);
                    }
                }
            }

            highlighters.iter()
                .flat_map(|highlighter| highlighter.process(line.content.as_str(), line.start, state))
                .for_each(|h| {
                    let s = line.find_grapheme_index_by_offset(h.get_start().into());
                    let e = line.find_grapheme_index_by_offset(h.get_end().into());
                    if let Some((s, e)) = s.zip(e) {
                        intervals.add_interval(s, e, h.get_payload());
                    }
                });

            let disjoint_intervals = intervals.disjoint_intervals();
            let mut spans = vec![];
            for interval in disjoint_intervals {
                let style = interval.2.iter()
                    .fold(regular_style, |s1, s2| s1 + *s2)
                    .get_style();
                let s = get_visible_graphemes().nth(interval.0.as_usize()).map(|g| g.render_offset - first_offset).unwrap();
                let e = get_visible_graphemes().nth(interval.1.as_usize() - 1)
                    .map(|g| g.render_offset + g.render.resolve(line.content.as_str()).len() - first_offset)
                    .unwrap();
                spans.push(indexed_span(s, e, (interval.1 - interval.0).as_usize(), style));
            }
            SpannedString::with_spans(display_str, spans)
        } else {
            let mut spans = vec![];
            let mut s = String::with_capacity(1);
            if cursor >= line.start && cursor <= line.end {
                spans.push(indexed_span(0, 1, 1, cursor_style.get_style()));
                s.push(' ');
            }
            SpannedString::with_spans(s, spans)
        };

        result.append(line_str);
        result
    }

    fn draw_line_number(&self, line: &LineRender) -> Result<SpannedString<Style>, std::fmt::Error> {
        if !self.show_line_numbers {
            return Ok(SpannedString::new());
        }

        let line_number_width = self.max_line_number.checked_ilog10().unwrap_or(0) as usize + 1;
        let mut prefix = String::new();
        match &line.line_no {
            Ok(line_no) => {
                write!(
                    &mut prefix,
                    "{number:>width$}",
                    number = line_no + 1,
                    width = line_number_width
                )?;
            },
            Err(LineNumberMissingReason::Delegate(LineRegistryError::NotReachedYet { .. })) => {
                write!(
                    &mut prefix,
                    "{:width$}",
                    LOADING_INDICATOR,
                    width = line_number_width
                )?;
            },
            Err(_) => {
                write!(
                    &mut prefix,
                    "{:width$}",
                    ERROR_INDICATOR,
                    width = line_number_width
                )?;
            }
        }
        let line_number_offset = prefix.as_bytes().len();
        prefix += LINE_NO_DELIMITER;
        let spans = vec![
            indexed_span(0, line_number_offset, line_number_width, self.line_number_style.unwrap().get_style()),
            indexed_span(line_number_offset, line_number_offset + LINE_NO_DELIMITER_BYTE_WIDTH, LINE_NO_DELIMITER_WIDTH, self.regular_style.unwrap().get_style())
        ];
        Ok(SpannedString::with_spans(prefix, spans))
    }
}

fn indexed_span<T, I1, I2>(start: I1, end: I2, width: usize, attr: T) -> IndexedSpan<T>
    where I1: TryInto<usize>, I2: TryInto<usize>
{
    let start = start.try_into().unwrap_or(0);
    let end = end.try_into().unwrap_or(0);
    IndexedSpan {
        content: IndexedCow::Borrowed {
            start, end
        },
        attr,
        width
    }
}
