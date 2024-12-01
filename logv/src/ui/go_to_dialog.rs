use crate::data_source::line_source_holder::LineSourceHolder;
use crate::data_source::reader_factory::HasReaderFactory;
use crate::model::model::RootModel;
use crate::ui::ui_elements::UIElementName;
use crate::ui::with_root_model::WithRootModel;
use anyhow::anyhow;
use cursive::view::Nameable;
use cursive::views::{Dialog, EditView, LinearLayout, TextView};
use cursive::{Cursive, View};
use cursive::theme::{ColorStyle, PaletteColor, Style, Theme};
use cursive::utils::span::SpannedString;

pub fn build_go_to_dialog(root_model: &mut RootModel) -> Box<dyn View> {
    let go_to_model = &*root_model.get_go_to_line_model();

    let mut layout = LinearLayout::vertical();

    layout.add_child(TextView::new("Enter line number:"));

    let value_input = EditView::new()
        .content(go_to_model.get_value())
        .on_submit(|app, _value| submit(app))
        .with_name(UIElementName::GoToValue.to_string());
    layout.add_child(value_input);

    if let Some(warning) = go_to_model.get_warning() {
        let text = SpannedString::styled(
            warning,
            Style::from(ColorStyle::from(Theme::default().palette[PaletteColor::Highlight])));
        let warning_box = TextView::new(text);
        layout.add_child(warning_box);
    }

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
    let (reader_factory, length) = {
        let root_model = &mut *app.get_root_model();
        let ds = &*root_model
            .get_datasource_ref()
            .ok_or(anyhow!("Data is not set"))?;
        let length = match ds {
            LineSourceHolder::Concrete(c) => Some(c.get_length()),
            LineSourceHolder::Filtered(_) => None,
        };
        (ds.reader_factory(), length)
    };
    let content = {
        app
            .find_name::<EditView>(&UIElementName::GoToValue.to_string())
            .ok_or(anyhow!("Element not found"))?.get_content()
    };
    let root_model = &mut *app.get_root_model();
    let go_to_model = &mut *root_model.get_go_to_line_model();
    let line_registry = root_model.get_line_registry();
    go_to_model.set_line_registry(line_registry);
    go_to_model.set_value(&content);
    go_to_model.submit(reader_factory, length)
}

fn cancel(app: &mut Cursive) {
    let root_model = &mut *app.get_root_model();
    let go_to_line_model = &mut *root_model.get_go_to_line_model();
    go_to_line_model.set_is_open(false);
}