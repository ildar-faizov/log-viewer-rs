use cursive::{View, With};
use cursive::views::{LinearLayout, ScrollView, TextView, Menubar, Dialog, Canvas, NamedView};
use cursive::traits::{Nameable, Resizable};
use cursive::menu::MenuTree;
use crate::model::{RootModel, RootModelRef};
use cursive::event::{Event, EventResult};
use std::rc::Rc;
use std::cell::RefCell;
use std::borrow::BorrowMut;

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
    let mut menu = Menubar::new();
    menu.add_subtree("File",
                     MenuTree::new()
                         // .leaf("Open", |s| s.add_layer(Dialog::info("Open file or specify it on command line")))
                         .leaf("Open", |cursive| {
                             println!("Open clicked");
                             let model_ref: &mut RootModelRef = cursive.user_data().unwrap();
                             let mut model = model_ref.get_mut();
                             model.set_file_name(uuid::Uuid::new_v4().to_string());
                         })
                         .subtree("Recent", MenuTree::new().with(|tree| {
                             for i in 1..100 {
                                 tree.add_leaf(format!("Item {}", i), |_| ())
                             }
                         }))
                         .delimiter()
                         .leaf("Quit", |s| s.quit()));
    menu.add_subtree("Help",
                     MenuTree::new()
                         .subtree("Help",
                                  MenuTree::new()
                                      .leaf("General", |s| {
                                          s.add_layer(Dialog::info("Help message!"))
                                      })
                                      .leaf("Online", |s| {
                                          s.add_layer(Dialog::info("Online help?"))
                                      }))
                         .leaf("About",
                               |s| s.add_layer(Dialog::info("Cursive v0.0.0"))));

    let mut layout = LinearLayout::vertical();
    layout.add_child(menu);
    layout.add_child(build_canvas(model).full_height());
    layout.add_child(TextView::new("status")
        .with_name(UIElementName::Status)
        .full_width());

    Box::new(layout)
}

fn build_canvas(model: RootModelRef) -> NamedView<Canvas<RootModelRef>> {
    Canvas::new(model.clone())
        .with_draw(|state, printer| {
            let mut state = state.get_mut();
            state.set_viewport_size(printer.size.x, printer.size.y);

            if let Some(data) = state.data() {
                data.lines.iter().take(printer.size.y).enumerate().for_each(|(i, line)| {
                    printer.print((0, i), line);
                });
            } else {
                printer.clear();
            }
        })
        .with_on_event(|state, event| {
            if let Event::Char(_c) = event {
                // state.push(c);
                EventResult::Consumed(None)
            } else {
                EventResult::Ignored
            }
        })
        .with_name(UIElementName::MainContent)
}