use std::convert::TryInto;
use cursive::View;
use cursive::views::{LinearLayout, TextView, Canvas, NamedView};
use cursive::traits::{Nameable, Resizable};
use crate::model::{RootModelRef, CursorShift, Dimension};
use cursive::event::{Event, EventResult, Key};
use cursive::view::Selector;
use cursive::theme::{Style, ColorStyle, Theme};
use cursive::theme::PaletteColor::{HighlightText, Primary, Secondary};
use cursive::utils::span::{SpannedStr, IndexedSpan, SpannedString, IndexedCow};
use num_traits::{One, Zero};
use fluent_integer::Integer;
use crate::utils;

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
    Canvas::new(model.clone())
        .with_draw(|state, printer| {
            let mut state = state.get_mut();
            state.set_viewport_size(Integer::from(printer.size.x), Integer::from(printer.size.y));

            if let Some(data) = state.data() {
                let palette = Theme::default().palette;
                let regular_style = Style::from(ColorStyle::new(palette[Primary], palette[HighlightText]));
                let cursor_style = Style::from(ColorStyle::new(palette[HighlightText], palette[Primary]));
                let selection_style = Style::from(ColorStyle::new(palette[HighlightText], palette[Secondary]));

                let horizontal_scroll = state.get_horizontal_scroll().as_usize();
                let cursor = state.get_cursor_on_screen();
                data.lines.iter()
                    .take(printer.size.y)
                    .enumerate()
                    .filter(|(_, line)| line.content.len() > horizontal_scroll)
                    .map(|(i, line)| {
                        let slice = &line.content.as_str()[horizontal_scroll..];
                        let selection = state.get_selection();
                        let mut intervals: Vec<(Integer, Integer, &str)> = vec![(Integer::from(0), Integer::from(slice.len()), "main")];
                        if let Some(cursor) = cursor {
                            if cursor.height == i {
                                intervals.push((cursor.width.into(), cursor.width + 1, "cursor"));
                            }
                        }
                        if let Some(selection) = selection {
                            let slice_offset = line.start + horizontal_scroll;
                            if selection.start <= line.end && selection.end >= slice_offset {
                                intervals.push((selection.start - slice_offset, selection.end - slice_offset, "selection"));
                            }
                        }
                        let disjoint_intervals = utils::disjoint_intervals(&intervals);
                        let mut spans = vec![];
                        for interval in disjoint_intervals {
                            if interval.2.contains(&"main") {
                                let style = if interval.2.contains(&"cursor") {
                                    cursor_style
                                } else if interval.2.contains(&"selection") {
                                    selection_style
                                } else {
                                    regular_style
                                };
                                spans.push(indexed_span(interval.0, interval.1, style));
                            }
                        }
                        (i, SpannedString::with_spans(slice, spans))
                    })
                    .for_each(|(i, ss)| {
                        printer.print_styled((0, i), SpannedStr::from(&ss));
                    });
            } else {
                printer.clear();
            }
        })
        .with_on_event(|state, event| {
            match event {
                Event::Ctrl(Key::Down) => {
                    log::info!("Ctrl+Down pressed");
                    let mut state = state.get_mut();
                    state.scroll(Integer::one());
                    EventResult::Consumed(None)
                },
                Event::Ctrl(Key::Up) => {
                    log::info!("Ctrl+Up pressed");
                    let mut state = state.get_mut();
                    state.scroll(Integer::from(-1));
                    EventResult::Consumed(None)
                },
                Event::Ctrl(Key::Left) => {
                    log::info!("Ctrl+Left pressed");
                    let mut state = state.get_mut();
                    let horizontal_scroll = state.get_horizontal_scroll();
                    if horizontal_scroll > 0 {
                        state.set_horizontal_scroll(horizontal_scroll - 1);
                    }
                    EventResult::Consumed(None)
                },
                Event::Ctrl(Key::Right) => {
                    log::info!("Ctrl+Right pressed");
                    let mut state = state.get_mut();
                    let horizontal_scroll = state.get_horizontal_scroll();
                    if state.set_horizontal_scroll(horizontal_scroll + 1) {
                        EventResult::Consumed(None)
                    } else {
                        EventResult::Ignored
                    }
                },
                Event::Key(Key::Down) => {
                    let mut state = state.get_mut();
                    state.move_cursor(CursorShift::down(), false);
                    EventResult::Consumed(None)
                },
                Event::Key(Key::Up) => {
                    let mut state = state.get_mut();
                    state.move_cursor(CursorShift::up(), false);
                    EventResult::Consumed(None)
                },
                Event::Key(Key::Left) => {
                    let mut state = state.get_mut();
                    state.move_cursor(CursorShift::left(), false);
                    EventResult::Consumed(None)
                },
                Event::Key(Key::Right) => {
                    let mut state = state.get_mut();
                    state.move_cursor(CursorShift::right(), false);
                    EventResult::Consumed(None)
                },
                Event::Shift(Key::Down) => {
                    let mut state = state.get_mut();
                    state.move_cursor(CursorShift::down(), true);
                    EventResult::Consumed(None)
                },
                Event::Shift(Key::Up) => {
                    let mut state = state.get_mut();
                    state.move_cursor(CursorShift::up(), true);
                    EventResult::Consumed(None)
                },
                Event::Shift(Key::Left) => {
                    let mut state = state.get_mut();
                    state.move_cursor(CursorShift::left(), true);
                    EventResult::Consumed(None)
                },
                Event::Shift(Key::Right) => {
                    let mut state = state.get_mut();
                    state.move_cursor(CursorShift::right(), true);
                    EventResult::Consumed(None)
                },
                Event::Key(Key::PageDown) => {
                    let mut state = state.get_mut();
                    let h = state.get_viewport_size().height;
                    if state.scroll(h) {
                        let p = state.data()
                            .and_then(|data| data.lines.first())
                            .map(|line| line.start);
                        if let Some(p) = p {
                            state.move_cursor_to_offset(p, false);
                        }
                    } else {
                        let p = state.data()
                            .and_then(|data| data.lines.last())
                            .map(|line| line.start);
                        if let Some(p) = p {
                            state.move_cursor_to_offset(p, false);
                        }
                    }
                    EventResult::Consumed(None)
                },
                Event::Key(Key::PageUp) => {
                    let mut state = state.get_mut();
                    let h = state.get_viewport_size().height;
                    if state.scroll(-h) {
                        let p = state.data()
                            .and_then(|data| data.lines.first())
                            .map(|line| line.start);
                        if let Some(p) = p {
                            state.move_cursor_to_offset(p, false);
                        }
                    } else {
                        let p = state.data()
                            .and_then(|data| data.lines.first())
                            .map(|line| line.start);
                        if let Some(p) = p {
                            state.move_cursor_to_offset(p, false);
                        }
                    }
                    EventResult::Consumed(None)
                },
                Event::Key(Key::Home) => {
                    let mut state = state.get_mut();
                    match state.get_cursor_on_screen() {
                        Some(Dimension {height: h, width: _} ) => {
                            let p = state.data()
                                .and_then(|data| data.lines.get(h.as_usize()))
                                .map(|line| line.start);
                            if let Some(p) = p {
                                state.move_cursor_to_offset(p, false);
                                EventResult::Consumed(None)
                            } else {
                                EventResult::Ignored
                            }
                        },
                        _ => EventResult::Ignored
                    }
                },
                Event::Key(Key::End) => {
                    let mut state = state.get_mut();
                    match state.get_cursor_on_screen() {
                        Some(Dimension {height: h, width: _} ) => {
                            let p = state.data()
                                .and_then(|data| data.lines.get(h.as_usize()))
                                .map(|line| line.end);
                            if let Some(p) = p {
                                state.move_cursor_to_offset(p - 1, false);
                                EventResult::Consumed(None)
                            } else {
                                EventResult::Ignored
                            }
                        },
                        _ => EventResult::Ignored
                    }
                },
                Event::Ctrl(Key::Home) => {
                    let mut state = state.get_mut();
                    state.move_cursor_to_offset(Integer::zero(), false);
                    EventResult::Consumed(None)
                },
                Event::Ctrl(Key::End) => {
                    let mut state = state.get_mut();
                    if state.move_cursor_to_end() {
                        EventResult::Consumed(None)
                    } else {
                        EventResult::Ignored
                    }
                },
                Event::CtrlChar('a') => {
                    let mut state = state.get_mut();
                    state.select_all();
                    EventResult::Consumed(None)
                },
                Event::Char('q') => {
                    let state = state.get_mut();
                    state.quit();
                    EventResult::Consumed(None)
                },
                _ => EventResult::Ignored
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