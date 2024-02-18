use cursive::View;
use cursive::view::{Resizable, Selector};
use cursive::views::LinearLayout;
use crate::model::model::RootModel;
use crate::shared::Shared;
use crate::ui::canvas::build_canvas;
use crate::ui::status_panel::build_status_panel;
use crate::ui::ui_elements::UIElementName;

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
    layout.add_child(build_status_panel().view);

    layout.focus_view(&Selector::Name(UIElementName::MainContent.to_string().as_str()))
        .expect("TODO: panic message");

    Box::new(layout)
}
