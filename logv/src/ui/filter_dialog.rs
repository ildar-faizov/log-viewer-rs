use cursive::Cursive;
use cursive::traits::{Nameable, Resizable};
use cursive::views::{Dialog, EditView, LinearLayout, TextView};
use crate::model::filter_model::{FilterDialogModel, FilterDialogModelEvent};
use crate::ui::ui_utils::PopLayerSafely;
use crate::ui::view_with_callback::{ViewUpdateCallback, ViewWithCallback};
use crate::ui::with_root_model::WithRootModel;

const FILTER_DIALOG: &str = "FilterDialog";
const PATTERN_FIELD: &str = "PatternField";
const NEIGHBOURHOOD_FIELD: &str = "NeighbourhoodField";

pub fn build_filter_dialog(model: &FilterDialogModel) -> ViewWithCallback {
    let do_filter = |app: &mut Cursive, _: &str| {
        let state = &mut *app.get_root_model();
        state.get_filter_dialog_model().set_open(false);
        if let Err(err) = state.filter() {
            state.set_error(Box::new(err));
        }
    };

    let mut layout = LinearLayout::vertical();
    layout.add_child(TextView::new("Enter text or regular expression:"));
    let pattern_field = EditView::new()
        .content(model.get_pattern())
        .on_edit(|app, value, _| {
            let root_model = &mut *app.get_root_model();
            let filter_model = &mut *root_model.get_filter_dialog_model();
            filter_model.set_pattern(value);
        })
        .on_submit(do_filter)
        .with_name(PATTERN_FIELD);
    layout.add_child(pattern_field);
    layout.add_child(TextView::new("Neighbourhood (# of lines before and after match):"));
    let neighbourhood_field = EditView::new()
        .content(model.get_neighbourhood().to_string())
        .on_edit(|app, value, _| {
            let root_model = &mut *app.get_root_model();
            let filter_model = &mut *root_model.get_filter_dialog_model();
            filter_model.set_neighbourhood(value);
        })
        .on_submit(do_filter)
        .with_name(NEIGHBOURHOOD_FIELD);
    layout.add_child(neighbourhood_field);

    let dialog = Dialog::new()
        .title("Filter")
        .content(layout)
        .button("Filter", move |app| {
            let search_field = app.find_name::<EditView>(PATTERN_FIELD)
                .expect("Element not found");
            do_filter(app, search_field.get_content().as_str());
        })
        .button("Cancel", |app| {
            let state = app.get_root_model();
            state.get_filter_dialog_model().set_open(false);
        })
        .full_width()
        .with_name(FILTER_DIALOG);
    ViewWithCallback::with_dummy_callback(dialog)
}

pub fn handle_filter_dialog_model_event(model: &FilterDialogModel, evt: FilterDialogModelEvent) -> ViewUpdateCallback {
    match evt {
        FilterDialogModelEvent::VisibilityChanged(is_visible) => {
            if is_visible {
                build_filter_dialog(model).into()
            } else {
                Box::new(|app: &mut Cursive| app.pop_layer_safely(FILTER_DIALOG))
            }
        }
    }
}