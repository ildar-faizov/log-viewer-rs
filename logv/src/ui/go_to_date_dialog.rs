use std::borrow::Cow;
use anyhow::anyhow;
use cursive::{Cursive, View};
use cursive::view::Nameable;
use cursive::views::{Dialog, EditView, LinearLayout, TextView};
use crate::model::model::RootModel;
use crate::ui::ui_elements::UIElementName;
use crate::ui::with_root_model::WithRootModel;

pub fn build_go_to_date_dialog(root_model: &mut RootModel) -> Box<dyn View> {
    let go_to_date_model = &mut *root_model.get_go_to_date_model();

    let mut layout = LinearLayout::vertical();
    layout.add_child(TextView::new("Enter date (dd-MMM-yyyy HH:mm:ss):"));
    layout.add_child(EditView::new()
        .content(go_to_date_model.get_value())
        .on_edit(|app, value, cursor| {
            // Simulation of "insert" mode
            // A better approach is to create a convenient date/time input

            let mut new_value = None;
            {
                let root_model = &mut *app.get_root_model();
                let go_to_date_model = &mut *root_model.get_go_to_date_model();
                if cursor > 0 {
                    let expected_prev_value = format!("{}{}", &value[0..cursor - 1], &value[cursor..]);
                    let prev_value = go_to_date_model.get_value();
                    if expected_prev_value == prev_value {
                        let s = if cursor < value.len() {
                            Cow::Owned(format!("{}{}", &value[0..cursor], &value[cursor + 1..]))
                        } else {
                            Cow::Borrowed(&value[0..cursor])
                        };
                        go_to_date_model.set_value(&s);
                        new_value = Some((s, cursor));
                    }
                }
            }
            if let Some((s, cursor)) = new_value {
                app.call_on_name(&UIElementName::GoToDateValue.to_string(), |edit: &mut EditView| {
                    edit.set_content(s);
                    edit.set_cursor(cursor);
                });
            } else {
                let root_model = &mut *app.get_root_model();
                let go_to_date_model = &mut *root_model.get_go_to_date_model();
                go_to_date_model.set_value(value);
            }
        })
        .on_submit(|app, _value| submit(app))
        .with_name(UIElementName::GoToDateValue.to_string())
    );

    let d = Dialog::new()
        .title("Go to")
        .padding_lrtb(1, 1, 1, 1)
        .content(layout)
        .button("Go", submit)
        .button("Cancel", cancel);
    Box::new(d)
}

fn submit(app: &mut Cursive) {
    let res = try_submit(app);
    if let Err(err) = res {
        let root_model = &mut *app.get_root_model();
        root_model.set_error(Box::new(err));
    }
}

fn try_submit(app: &mut Cursive) -> anyhow::Result<()> {
    let file_name = {
        let root_model = &mut *app.get_root_model();
        root_model
            .file_name()
            .ok_or(anyhow!("File is not set"))?
            .to_string()
    };
    let content = {
        let value_field = app
            .find_name::<EditView>(&UIElementName::GoToDateValue.to_string())
            .ok_or(anyhow!("Element not found"))?;
        value_field.get_content()
    };
    let root_model = &mut *app.get_root_model();
    let known_date_format = root_model.get_date_format()
        .ok_or(anyhow!("Date format is not recognized for file"))?;
    let guess_ctx = root_model.get_date_guess_context();
    let go_to_date_model = &mut *root_model.get_go_to_date_model();
    go_to_date_model.set_value(&content);
    go_to_date_model.submit(&file_name, known_date_format, guess_ctx)
}

fn cancel(app: &mut Cursive) {
    let root_model = &mut *app.get_root_model();
    let go_to_date_model = &mut *root_model.get_go_to_date_model();
    go_to_date_model.set_is_open(false);
}