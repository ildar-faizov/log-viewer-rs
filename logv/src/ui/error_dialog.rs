use cursive::View;
use cursive::views::{Dialog, TextView};
use crate::ui::with_root_model::WithRootModel;

pub fn build_error_dialog(err: &str) -> Box<dyn View> {
    let dialog = Dialog::new()
        .title("Error")
        .padding_lrtb(5, 5, 1, 1)
        .content(TextView::new(err))
        .button("Ok", move |app| {
            let mut state = app.get_root_model();
            state.reset_error();
        });
    Box::new(dialog)
}