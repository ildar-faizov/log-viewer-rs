use cursive::{View, With};
use cursive::views::{LinearLayout, ScrollView, TextView, Menubar, Dialog};
use cursive::traits::{Nameable, Resizable};
use cursive::menu::MenuTree;
use crate::model::RootModel;

pub fn build_ui() -> Box<dyn View>
{
    let mut menu = Menubar::new();
    menu.add_subtree("File",
                     MenuTree::new()
                         // .leaf("Open", |s| s.add_layer(Dialog::info("Open file or specify it on command line")))
                         .leaf("Open", |cursive| {
                             let model: &mut RootModel = cursive.user_data().unwrap();
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
    layout.add_child(ScrollView::new(TextView::new(get_content()).with_name("main_content")));
    layout.add_child(TextView::new("status")
        .with_name("status")
        .full_width());
    Box::new(layout)
}

fn get_content() -> String {
    (0..1000)
        .map(|i| format!("Line {}", i))
        .fold(String::new(), |mut acc: String, line: String| {
            acc.push_str("\n");
            acc.push_str(line.as_str());
            acc
        })
}