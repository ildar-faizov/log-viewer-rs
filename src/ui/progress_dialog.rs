use crate::model::progress_model::{ProgressModel, ProgressModelEvent};
use crate::ui::view_with_callback::{ViewUpdateCallback, ViewWithCallback};
use cursive::traits::Nameable;
use cursive::view::IntoBoxedView;
use cursive::views::{Dialog, LinearLayout, ProgressBar, TextView};
use cursive::{Cursive, View};
use crate::ui::ui_utils::PopLayerSafely;
use crate::ui::with_root_model::WithRootModel;

const DIALOG: &str = "ProgressDialog";
const DESCRIPTION: &str = "ProgressDialog.Description";
const PROGRESS_BAR: &str = "ProgressDialog.ProgressBar";

pub fn build_progress_dialog(model: &ProgressModel) -> ViewWithCallback {
    let dialog = Dialog::new()
        .title(model.get_title())
        .content(build_dialog_content(model))
        .with_name(DIALOG)
        .into_boxed_view();
    let callback = Box::new(|app: &mut Cursive| {});
    ViewWithCallback::new(dialog, callback)
}

pub fn handle_progress_model_event(
    model: &ProgressModel,
    evt: ProgressModelEvent,
) -> ViewUpdateCallback {
    match evt {
        ProgressModelEvent::TitleUpdated => {
            let callback = update_title(model);
            Box::new(|app: &mut Cursive| {
                app.call_on_name(DIALOG, |dialog: &mut Dialog| callback(dialog));
            })
        }
        ProgressModelEvent::DescriptionUpdated => {
            let callback = update_description(model);
            Box::new(|app: &mut Cursive| {
                app.call_on_name(DESCRIPTION, |t: &mut TextView| callback(t));
            })
        }
        ProgressModelEvent::ProgressUpdated => {
            let callback = update_progress(model);
            Box::new(|app: &mut Cursive| {
                app.call_on_name(PROGRESS_BAR, |t: &mut ProgressBar| callback(t));
            })
        }
        ProgressModelEvent::Toggle => {
            if model.is_open() {
                Box::new(|app| {
                    let view_with_callback = {
                        let root_model = &mut *app.get_root_model();
                        let progress_model = &*root_model.get_progress_model();
                        build_progress_dialog(progress_model)
                    };
                    app.add_layer(view_with_callback.view);
                    (view_with_callback.callback)(app);
                })
            } else {
                Box::new(|app| {
                    app.pop_layer_safely(DIALOG);
                })
            }
        }
    }
}

fn build_dialog_content(model: &ProgressModel) -> Box<dyn View> {
    LinearLayout::vertical()
        .child(TextView::new(model.get_description()).with_name(DESCRIPTION))
        .child(ProgressBar::new().with_name(PROGRESS_BAR))
        .into_boxed_view()
}

fn update_title(model: &ProgressModel) -> Box<dyn FnOnce(&mut Dialog)> {
    let title = model.get_title().to_string();
    Box::new(move |dialog| dialog.set_title(title))
}

fn update_description(model: &ProgressModel) -> Box<dyn FnOnce(&mut TextView)> {
    let description = model.get_description().to_string();
    Box::new(move |t| t.set_content(description))
}

fn update_progress(model: &ProgressModel) -> Box<dyn FnOnce(&mut ProgressBar)> {
    let progress = model.get_progress();
    Box::new(move |bar| bar.set_value(progress as usize))
}
