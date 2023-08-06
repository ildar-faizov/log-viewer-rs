use cursive::{Cursive, View};
use cursive::view::{Nameable, Resizable};
use cursive::views::{Checkbox, Dialog, EditView, LinearLayout, TextView};
use crate::model::model::RootModel;
use crate::shared::Shared;
use crate::ui::ui_elements::UIElementName;
use crate::ui::with_root_model::WithRootModel;

pub fn build_search_ui(state: Shared<RootModel>) -> Box<dyn View> {
    let do_search = |app: &mut Cursive, search_str: &str| {
        let mut root_model = app.get_root_model();
        let mut search_model = root_model.get_search_model();
        if search_model.is_from_cursor() {
            search_model.set_cursor(root_model.get_cursor());
        }
        search_model.set_pattern(search_str);
        let search = search_model.start_search();
        match search {
            Ok(search) => {
                search_model.set_visible(false);
                drop(search_model);
                root_model.set_current_search(Some(search));
            },
            Err(err) => {
                drop(search_model);
                root_model.set_error(Box::new(err));
            }
        }

    };
    let root_model = state.get_mut_ref();
    let search_model = root_model.get_search_model();

    let mut layout = LinearLayout::vertical();
    layout.add_child(TextView::new("Enter text or regular expression:"));
    let search_field = EditView::new()
        .content(search_model.get_pattern())
        .on_submit(do_search)
        .with_name(UIElementName::SearchField.to_string());
    layout.add_child(search_field);

    let mut search_settings_panel = LinearLayout::horizontal();
    search_settings_panel.add_child(Checkbox::new()
        .with_checked(search_model.is_from_cursor())
        .on_change(|app, is_checked| {
            let model = app.get_root_model();
            model.get_search_model().set_from_cursor(is_checked);
        })
        .with_name(UIElementName::SearchFromCursor.to_string()));
    search_settings_panel.add_child(TextView::new("From cursor"));
    search_settings_panel.add_child(Checkbox::new()
        .with_checked(search_model.is_backward())
        .with_enabled(search_model.is_from_cursor())
        .on_change(|app, is_checked| {
            let model = app.get_root_model();
            model.get_search_model().set_backward(is_checked);
        })
        .with_name(UIElementName::SearchBackward.to_string()));
    search_settings_panel.add_child(TextView::new("Backward"));
    search_settings_panel.add_child(Checkbox::new()
        .with_checked(search_model.is_regexp())
        .on_change(|app, is_checked| {
            let model = app.get_root_model();
            model.get_search_model().set_regexp(is_checked);
        })
        .with_name(UIElementName::SearchRegexp.to_string())
    );
    search_settings_panel.add_child(TextView::new("Regexp"));
    layout.add_child(search_settings_panel);

    let dialog = Dialog::new()
        .title("Search")
        .content(layout)
        .button("Search", move |app| {
            let search_field = app.find_name::<EditView>(&UIElementName::SearchField.to_string())
                .expect("Element not found");
            do_search(app, search_field.get_content().as_str());
        })
        .button("Cancel", |app| {
            let state = app.get_root_model();
            state.get_search_model().set_visible(false);
        })
        .full_width();
    Box::new(dialog)
}