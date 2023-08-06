use cursive::View;
use cursive::views::{Dialog, TextView};

pub fn build_error_dialog(err: &str) -> Box<dyn View> {
    let dialog = Dialog::new()
        .title("Error")
        .padding_lrtb(5, 5, 1, 1)
        .content(TextView::new(err))
        .button("Ok", move |app| {
            app.pop_layer();
        });
    Box::new(dialog)
}