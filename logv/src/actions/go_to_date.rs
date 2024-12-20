use cursive::event::{Event, EventResult};
use logv_macro::define_action;

use crate::model::go_to_date_model::DATE_FORMAT;
use crate::model::model::RootModel;

#[define_action]
fn go_to_date(model: &mut RootModel, _event: &Event) -> EventResult {
    let go_to_date_model = &mut *model.get_go_to_date_model();
    if go_to_date_model.get_value().is_empty() {
        let known_date_format = model.get_date_format();
        if let Some(known_date_format) = known_date_format {
            if let Some(data) = model.data() {
                let guess_ctx = model.get_date_guess_context();
                let sample_date = data.lines.iter()
                    .filter_map(|line| known_date_format.parse(&line.content, &guess_ctx))
                    .next();
                if let Some(sample_date) = sample_date {
                    go_to_date_model.set_value(&sample_date.format(DATE_FORMAT).to_string())
                }
            }
        }
    }
    go_to_date_model.set_is_open(true);
    EventResult::Consumed(None)
}