use crate::model::model::RootModel;
use crate::ui::ui_elements::UIElementName;
use crate::ui::with_root_model::WithRootModel;
use anyhow::anyhow;
use cursive::view::Nameable;
use cursive::views::{Dialog, EditView};
use cursive::{Cursive, View};

pub fn build_go_to_dialog(root_model: &mut RootModel) -> Box<dyn View> {
    let go_to_model = &*root_model.get_go_to_line_model();
    let value_input = EditView::new()
        .content(go_to_model.get_value())
        .on_submit(|app, _value| submit(app))
        .with_name(UIElementName::GoToValue.to_string());

    let d = Dialog::new()
        .title("Go to")
        .padding_lrtb(1, 1, 1, 1)
        .content(value_input)
        .button("Go", submit);
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
            .find_name::<EditView>(&UIElementName::GoToValue.to_string())
            .ok_or(anyhow!("Element not found"))?;
        value_field.get_content()
    };
    let root_model = &mut *app.get_root_model();
    let go_to_model = &mut *root_model.get_go_to_line_model();
    go_to_model.set_value(&content);
    go_to_model.submit(&file_name)
}
