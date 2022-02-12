use std::io::BufRead;

use cursive::View;
use cursive::views::{LinearLayout, TextView, Canvas, NamedView};
use cursive::traits::{Nameable, Resizable};
use crate::model::{RootModelRef, CursorShift, Dimension};
use cursive::event::{Event, EventResult, Key};
use cursive::view::Selector;
use cursive::theme::{Style, ColorStyle, Theme};
use cursive::theme::PaletteColor::{HighlightText, Primary};
use cursive::utils::span::{SpannedStr, IndexedSpan, SpannedString, IndexedCow};

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
            state.set_viewport_size(printer.size.x, printer.size.y);

            if let Some(data) = state.data() {
                let palette = Theme::default().palette;
                let regular_style = Style::from(ColorStyle::new(palette[Primary], palette[HighlightText]));
                let cursor_style = Style::from(ColorStyle::new(palette[HighlightText], palette[Primary]));

                let horizontal_scroll = state.get_horizontal_scroll();
                let cursor = state.get_cursor_on_screen();
                data.lines.iter()
                    .take(printer.size.y)
                    .enumerate()
                    .filter(|(_, line)| line.content.len() > horizontal_scroll)
                    .map(|(i, line)| {
                        let slice = &line.content.as_str()[horizontal_scroll..];
                        let mut spans = vec![];
                        if let Some(cursor) = cursor {
                            if cursor.height == i {
                                let w = cursor.width;
                                if w > 0 {
                                    spans.push(indexed_span(0, w, regular_style));
                                }
                                spans.push(indexed_span(w, w + 1, cursor_style));
                                if w + 1 < slice.len() {
                                    spans.push(indexed_span(w + 1, slice.len(), regular_style));
                                }
                                log::trace!("w = {}; spans = {:?}", w, spans);
                            }
                        }
                        if spans.is_empty() {
                            spans.push(IndexedSpan::simple_borrowed(slice, regular_style));
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
                    state.scroll(1);
                    EventResult::Consumed(None)
                },
                Event::Ctrl(Key::Up) => {
                    log::info!("Ctrl+Up pressed");
                    let mut state = state.get_mut();
                    state.scroll(-1);
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
                    state.move_cursor(CursorShift::down());
                    EventResult::Consumed(None)
                },
                Event::Key(Key::Up) => {
                    let mut state = state.get_mut();
                    state.move_cursor(CursorShift::up());
                    EventResult::Consumed(None)
                },
                Event::Key(Key::Left) => {
                    let mut state = state.get_mut();
                    state.move_cursor(CursorShift::left());
                    EventResult::Consumed(None)
                },
                Event::Key(Key::Right) => {
                    let mut state = state.get_mut();
                    state.move_cursor(CursorShift::right());
                    EventResult::Consumed(None)
                },
                Event::Key(Key::PageDown) => {
                    let mut state = state.get_mut();
                    let h = state.get_viewport_size().height as isize;
                    if state.scroll(h) {
                        let p = state.data()
                            .and_then(|data| data.lines.first())
                            .map(|line| line.start);
                        if let Some(p) = p {
                            state.move_cursor_to_offset(p);
                        }
                    } else {
                        let p = state.data()
                            .and_then(|data| data.lines.last())
                            .map(|line| line.start);
                        if let Some(p) = p {
                            state.move_cursor_to_offset(p);
                        }
                    }
                    EventResult::Consumed(None)
                },
                Event::Key(Key::PageUp) => {
                    let mut state = state.get_mut();
                    let h = state.get_viewport_size().height as isize;
                    if state.scroll(-h) {
                        let p = state.data()
                            .and_then(|data| data.lines.first())
                            .map(|line| line.start);
                        if let Some(p) = p {
                            state.move_cursor_to_offset(p);
                        }
                    } else {
                        let p = state.data()
                            .and_then(|data| data.lines.first())
                            .map(|line| line.start);
                        if let Some(p) = p {
                            state.move_cursor_to_offset(p);
                        }
                    }
                    EventResult::Consumed(None)
                },
                Event::Key(Key::Home) => {
                    let mut state = state.get_mut();
                    match state.get_cursor_on_screen() {
                        Some(Dimension {height: h, width: _} ) => {
                            let p = state.data()
                                .and_then(|data| data.lines.get(h))
                                .map(|line| line.start);
                            if let Some(p) = p {
                                state.move_cursor_to_offset(p);
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
                        Some(Dimension {height: h, width: w} ) => {
                            let p = state.data()
                                .and_then(|data| data.lines.get(h))
                                .map(|line| line.end);
                            if let Some(p) = p {
                                state.move_cursor_to_offset(p - 1);
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
                    state.move_cursor_to_offset(0);
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

fn indexed_span<T>(start: usize, end: usize, attr: T) -> IndexedSpan<T> {
    IndexedSpan {
        content: IndexedCow::Borrowed {
            start, end
        },
        attr,
        width: end - start
    }
}