use std::borrow::BorrowMut;
use std::cmp::{max, min};
use std::convert::TryInto;
use cursive::View;
use cursive::views::{LinearLayout, TextView, Canvas, NamedView};
use cursive::traits::{Nameable, Resizable};
use crate::model::RootModelRef;
use cursive::event::EventResult;
use cursive::view::Selector;
use cursive::theme::{Style, ColorStyle, Theme};
use cursive::theme::PaletteColor::{Background, HighlightText, Primary};
use cursive::utils::span::{SpannedStr, IndexedSpan, SpannedString, IndexedCow};
use fluent_integer::Integer;
use crate::actions::action_registry::action_registry;
use crate::highlight::highlighter_registry::cursive_highlighters;
use crate::highlight::style_with_priority::StyleWithPriority;
use crate::utils;
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

pub fn build_ui(model: RootModelRef) -> Box<dyn View> {
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

    layout.focus_view(&Selector::Name(UIElementName::MainContent.to_string().as_str()));

    Box::new(layout)
}

fn build_canvas(model: RootModelRef) -> NamedView<Canvas<RootModelRef>> {
    let actions = action_registry();

    let palette = Theme::default().palette;
    let highlighters = cursive_highlighters(&palette);
    let regular_style = StyleWithPriority::new(Style::from(ColorStyle::new(palette[Primary], palette[HighlightText])), 0, 0);
    let cursor_style = StyleWithPriority::new(Style::from(ColorStyle::new(palette[HighlightText], palette[Primary])), 1, 1);
    let selection_style = StyleWithPriority::new(Style::from(ColorStyle::new(palette[HighlightText], palette[Background])), 1, 0xff);
    Canvas::new(model.clone())
        .with_draw(move |state, printer| measure("draw",  || {
            let mut state = state.get_mut();
            state.set_viewport_size(Integer::from(printer.size.x), Integer::from(printer.size.y));

            if let Some(data) = state.data() {
                let horizontal_scroll = state.get_horizontal_scroll().as_usize();
                let cursor = state.get_cursor_on_screen();
                data.lines.iter()
                    .take(printer.size.y)
                    .enumerate()
                    .filter(|(_, line)| line.content.len() > horizontal_scroll)
                    .map(|(i, line)| {
                        let slice = &line.content.as_str()[horizontal_scroll..];
                        let selection = state.get_selection();
                        let mut intervals = SpanProducer::new(min(printer.size.x, slice.len()));
                        intervals.add_interval(0, slice.len(), regular_style);
                        if let Some(cursor) = cursor {
                            if cursor.height == i {
                                intervals.add_interval(cursor.width, cursor.width + 1, cursor_style);
                            }
                        }
                        if let Some(selection) = selection {
                            let slice_offset = line.start + horizontal_scroll;
                            if selection.start <= line.end && selection.end >= slice_offset {
                                intervals.add_interval(selection.start - slice_offset, selection.end - slice_offset, selection_style);
                            }
                        }

                        highlighters.iter()
                            .flat_map(|h| h.process(line.content.as_str()))
                            .map(|highlight| (Integer::from(highlight.get_start()) - horizontal_scroll, Integer::from(highlight.get_end()) - horizontal_scroll, highlight.get_payload()))
                            .for_each(|(s, e, style)| intervals.add_interval(s, e, style));

                        let disjoint_intervals = intervals.disjoint_intervals();
                        let mut spans = vec![];
                        for interval in disjoint_intervals {
                            let style = interval.2.iter()
                                .fold(regular_style, |s1, s2| s1 + *s2)
                                .get_style();
                            spans.push(indexed_span(interval.0, interval.1, style));
                        }
                        (i, SpannedString::with_spans(slice, spans))
                    })
                    .for_each(|(i, ss)| {
                        printer.print_styled((0, i), SpannedStr::from(&ss));
                    });
            } else {
                printer.clear();
            }
        }))
        .with_on_event(move |state, event| {
            match actions.get(&event) {
                Some(action) => {
                    log::info!("Event {:?} occurred, action {} will be invoked", event, action.description());
                    let result = action.perform_action(state.get_mut().borrow_mut(), &event);
                    log::info!("Event {:?} handled, action {} finished", event, action.description());
                    result
                },
                None => EventResult::Ignored
            }
        })
        .with_name(UIElementName::MainContent)
}

fn indexed_span<T, I1, I2>(start: I1, end: I2, attr: T) -> IndexedSpan<T>
    where I1: TryInto<usize>, I2: TryInto<usize>
{
    let start = start.try_into().unwrap_or(0);
    let end = end.try_into().unwrap_or(0);
    IndexedSpan {
        content: IndexedCow::Borrowed {
            start, end
        },
        attr,
        width: end - start
    }
}

struct SpanProducer {
    intervals: Vec<(Integer, Integer, StyleWithPriority)>,
    width: usize
}

impl SpanProducer {
    fn new(width: usize) -> Self {
        SpanProducer {
            intervals: vec![],
            width
        }
    }

    fn add_interval<A, B>(&mut self, s: A, e: B, style: StyleWithPriority)
        where A: Into<Integer>, B: Into<Integer> {
        let s = max(s.into(), 0_u8.into());
        let e = min(e.into(), self.width.into());
        if s < e {
            self.intervals.push((s, e, style));
        }
    }

    fn disjoint_intervals(&self) -> Vec<(Integer, Integer, Vec<StyleWithPriority>)> {
        utils::disjoint_intervals(&self.intervals)
    }
}