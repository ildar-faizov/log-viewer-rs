use cursive::event::{Event, EventResult};
use logv_macro::define_action;
use crate::data_source::line_source_holder::LineSourceHolder;
use crate::model::model::RootModel;

const FILTERED_LINE_MISMATCH_WARNING: &'static str = "Line number is relative to filtered data";

#[define_action]
fn go_to_line(model: &mut RootModel, _event: &Event) -> EventResult {
    let warning = model
        .get_datasource_ref()
        .as_ref()
        .filter(|ds| {
            matches!(***ds, LineSourceHolder::Filtered(_))
        })
        .map(|_| FILTERED_LINE_MISMATCH_WARNING);
    let go_to_model = &mut *model.get_go_to_line_model();
    go_to_model.set_warning(warning);
    go_to_model.set_is_open(true);
    EventResult::Consumed(None)
}
