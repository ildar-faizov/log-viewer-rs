use cursive::{View, With};
use cursive::views::{LinearLayout, ScrollView, TextView, Menubar, Dialog};
use cursive::traits::{Nameable, Resizable};
use cursive::menu::MenuTree;
use crate::model::RootModel;

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

pub fn build_ui() -> Box<dyn View> {
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
    layout.add_child(ScrollView::new(TextView::new("").with_name(UIElementName::MainContent)).full_height());
    layout.add_child(TextView::new("status")
        .with_name(UIElementName::Status)
        .full_width());

    Box::new(layout)
}
