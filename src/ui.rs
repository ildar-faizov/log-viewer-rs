use std::borrow::BorrowMut;
use std::cmp::{max, min};
use std::convert::TryInto;
use std::rc::Rc;
use cursive::View;
use cursive::views::{LinearLayout, TextView, Canvas, NamedView};
use cursive::traits::{Nameable, Resizable};
use cursive::event::EventResult;
use cursive::view::Selector;
use cursive::theme::{Style, ColorStyle, Theme};
use cursive::theme::PaletteColor::{Background, HighlightText, Primary};
use cursive::utils::span::{SpannedStr, IndexedSpan, SpannedString, IndexedCow};
use fluent_integer::Integer;
use crate::actions::action_registry::action_registry;
use crate::highlight::highlighter_registry::cursive_highlighters;
use crate::highlight::style_with_priority::StyleWithPriority;
use crate::{RootModel, Shared, utils};
use crate::highlight::highlight::Highlighter;
use crate::model::rendered::LineRender;
use crate::utils::measure;

pub enum UIElementName {
    MainContent,
    Status
}

impl ToString for UIElementName {
    fn to_string(&self) -> String {
        match self {
            UIElementName::MainContent => "main_content".to_string(),
            UIElementName::Status => "status".to_string(),
        }
    }
}

impl From<UIElementName> for String {
    fn from(x: UIElementName) -> Self {
        x.to_string()
    }
}

pub fn build_ui(model: Shared<RootModel>) -> Box<dyn View> {
    // let mut menu = Menubar::new();
    // menu.add_subtree("File",
    //                  MenuTree::new()
    //                      // .leaf("Open", |s| s.add_layer(Dialog::info("Open file or specify it on command line")))
    //                      .leaf("Open", |cursive| {
    //                          println!("Open clicked");
    //                          let model_ref: &mut RootModelRef = cursive.user_data().unwrap();
    //                          let mut model = model_ref.get_mut();
    //                          model.set_file_name(uuid::Uuid::new_v4().to_string());
    //                      })
    //                      .subtree("Recent", MenuTree::new().with(|tree| {
    //                          for i in 1..100 {
    //                              tree.add_leaf(format!("Item {}", i), |_| ())
    //                          }
    //                      }))
    //                      .delimiter()
    //                      .leaf("Quit", |s| s.quit()));
    // menu.add_subtree("Help",
    //                  MenuTree::new()
    //                      .subtree("Help",
    //                               MenuTree::new()
    //                                   .leaf("General", |s| {
    //                                       s.add_layer(Dialog::info("Help message!"))
    //                                   })
    //                                   .leaf("Online", |s| {
    //                                       s.add_layer(Dialog::info("Online help?"))
    //                                   }))
    //                      .leaf("About",
    //                            |s| s.add_layer(Dialog::info("Cursive v0.0.0"))));

    let mut layout = LinearLayout::vertical();
    // layout.add_child(menu);
    layout.add_child(build_canvas(model).full_height());
    layout.add_child(TextView::new("status")
        .with_name(UIElementName::Status)
        .full_width());

    layout.focus_view(&Selector::Name(UIElementName::MainContent.to_string().as_str()))
        .expect("TODO: panic message");

    Box::new(layout)
}

fn build_canvas(model: Shared<RootModel>) -> NamedView<Canvas<Shared<RootModel>>> {
    let actions = action_registry();

    let palette = Theme::default().palette;
    let highlighters = cursive_highlighters(&palette);
    let regular_style = StyleWithPriority::new(Style::from(ColorStyle::new(palette[Primary], palette[HighlightText])), 0, 0);
    let cursor_style = StyleWithPriority::new(Style::from(ColorStyle::new(palette[HighlightText], palette[Primary])), 1, 1);
    let selection_style = StyleWithPriority::new(Style::from(ColorStyle::new(palette[HighlightText], palette[Background])), 1, 0xff);
    Canvas::new(model.clone())
        .with_draw(move |state, printer| measure("draw",  || {
            let mut state = state.get_mut_ref();
            state.set_viewport_size(Integer::from(printer.size.x), Integer::from(printer.size.y));

            if let Some(data) = state.data() {
                let line_drawer = LineDrawer::new()
                    .with_state(&state)
                    .with_highlighters(&highlighters)
                    .with_width(printer.size.x)
                    .with_regular_style(regular_style)
                    .with_cursor_style(cursor_style)
                    .with_selection_style(selection_style);
                data.lines.iter()
                    .take(printer.size.y)
                    .enumerate()
                    .map(|(i, line)| line_drawer.draw(i, line))
                    .for_each(|(i, ss)|
                        printer.print_styled((0, i), SpannedStr::from(&ss))
                    );
            } else {
                printer.clear();
            }
        }))
        .with_on_event(move |state, event| {
            match actions.get(&event) {
                Some(action) => {
                    log::info!("Event {:?} occurred, action {} will be invoked", event, action.description());
                    let result = action.perform_action(state.get_mut_ref().borrow_mut(), &event);
                    log::info!("Event {:?} handled, action {} finished", event, action.description());
                    result
                },
                None => EventResult::Ignored
            }
        })
        .with_name(UIElementName::MainContent)
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

#[derive(Default)]
struct LineDrawer<'a> {
    state: Option<&'a RootModel>,
    highlighters: Option<&'a Vec<Rc<dyn Highlighter<StyleWithPriority> + 'static>>>,
    width: Option<usize>,
    regular_style: Option<StyleWithPriority>,
    cursor_style: Option<StyleWithPriority>,
    selection_style: Option<StyleWithPriority>,
}

impl<'a> LineDrawer<'a> {
    fn new<'b>() -> LineDrawer<'b> {
        LineDrawer::default()
    }

    fn with_state(mut self, state: &'a RootModel) -> Self {
        self.state.replace(state);
        self
    }

    fn with_highlighters(mut self, highlighters: &'a Vec<Rc<dyn Highlighter<StyleWithPriority> + 'static>>) -> Self {
        self.highlighters.replace(highlighters);
        self
    }

    fn with_width(mut self, width: usize) -> Self {
        self.width.replace(width);
        self
    }

    fn with_regular_style(mut self, regular_style: StyleWithPriority) -> Self {
        self.regular_style.replace(regular_style);
        self
    }

    fn with_cursor_style(mut self, cursor_style: StyleWithPriority) -> Self {
        self.cursor_style.replace(cursor_style);
        self
    }

    fn with_selection_style(mut self, selection_style: StyleWithPriority) -> Self {
        self.selection_style.replace(selection_style);
        self
    }

    fn draw(&self, i: usize, line: &LineRender) -> (usize, SpannedString<Style>) {
        let state = self.state.unwrap();
        let highlighters = self.highlighters.unwrap();
        let width = self.width.unwrap();
        let regular_style = self.regular_style.unwrap();
        let cursor_style = self.cursor_style.unwrap();
        let selection_style = self.selection_style.unwrap();

        let horizontal_scroll = state.get_horizontal_scroll().as_usize();
        let cursor = state.get_cursor();

        let get_visible_graphemes = || line.render.iter()
            .skip(horizontal_scroll)
            .take(width);

        let display_str = get_visible_graphemes()
            .map(|g| g.render.resolve(line.content.as_str()))
            .fold(String::with_capacity(width), |mut acc, item| {
                acc += item;
                acc
            });
        let selection = state.get_selection();

        if let Some(first_grapheme) = get_visible_graphemes().next() {
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
                .flat_map(|highlighter| highlighter.process(line.content.as_str()))
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
            log::trace!("{}: {}, spans = {:?}", i, display_str, spans);
            (i, SpannedString::with_spans(display_str, spans))
        } else {
            (i, SpannedString::new())
        }
    }
}

struct SpanProducer {
    intervals: Vec<(Integer, Integer, StyleWithPriority)>,
    shift: usize,
    limit: usize,
}

impl SpanProducer {
    fn new(shift: usize, limit: usize) -> Self {
        SpanProducer {
            intervals: vec![],
            shift,
            limit,
        }
    }

    fn add_interval<A, B>(&mut self, s: A, e: B, style: StyleWithPriority)
        where A: Into<Integer>, B: Into<Integer> {
        self.add_interval_without_shift(s.into() - self.shift, e.into() - self.shift, style)
    }

    fn add_interval_without_shift<A, B>(&mut self, s: A, e: B, style: StyleWithPriority)
        where A: Into<Integer>, B: Into<Integer> {
        let s = max(s.into(), 0_u8.into());
        let e = min(e.into(), self.limit.into());
        if s < e {
            self.intervals.push((s, e, style));
        }
    }

    fn disjoint_intervals(&self) -> Vec<(Integer, Integer, Vec<StyleWithPriority>)> {
        utils::disjoint_intervals(&self.intervals)
    }
}