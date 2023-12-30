use std::cmp::Ordering;
use cursive::{Cursive, View};
use cursive::traits::Resizable;
use cursive::view::IntoBoxedView;
use cursive::views::Dialog;
use cursive_table_view::{TableView, TableViewItem};
use crate::model::metrics_model::{MetricsModel, MetricsModelEvent, SingleMetrics};
use crate::ui::with_root_model::WithRootModel;

const NA: &str = "-";

pub fn handle_metrics_model_event(app: &mut Cursive, evt: MetricsModelEvent) {
    match evt {
        MetricsModelEvent::Open(is_open) => {
            if is_open {
                let dialog = {
                    let root_model = app.get_root_model();
                    let metrics_model = root_model.get_metrics_model();
                    build_metrics_dialog(&*metrics_model)
                };
                app.add_layer(dialog);
            } else {
                app.pop_layer();
            }
        }
    }
}

pub fn build_metrics_dialog(model: &MetricsModel) -> Box<dyn View> {
    Dialog::new()
        .title("Application Metrics")
        .content(build_content(model))
        .button("Close", close)
        .into_boxed_view()
}

fn build_content(model: &MetricsModel) -> Box<dyn View> {
    let mut table = MetricsTable::new()
        .column(Column::Description, "Metrics", |c| c.ordering(Ordering::Equal).width(40))
        .column(Column::MeasureUnit, "Unit", |c| c.ordering(Ordering::Equal).width(7))
        .column(Column::Number, "#", |c| c.ordering(Ordering::Equal).width(5))
        .column(Column::P50, "P50", |c| c.ordering(Ordering::Equal).width(7))
        .column(Column::P90, "P90", |c| c.ordering(Ordering::Equal).width(7))
        .column(Column::P99, "P99", |c| c.ordering(Ordering::Equal).width(7))
        .column(Column::Max, "Max", |c| c.ordering(Ordering::Equal).width(7));
    let data = model.get_data();
    table.set_items(data);
    table.disable();
    table.min_size((100, 20))
        .into_boxed_view()
}

fn close(app: &mut Cursive) {
    let root_model = &mut *app.get_root_model();
    let metrics_model = &mut *root_model.get_metrics_model();
    metrics_model.set_open(false);
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum Column {
    Description,
    MeasureUnit,
    Number,
    P50,
    P90,
    P99,
    Max,
}

impl TableViewItem<Column> for SingleMetrics {
    fn to_column(&self, column: Column) -> String {
        match column {
            Column::Description => self.description.clone(),
            Column::MeasureUnit => self.unit.unwrap_or(NA).to_string(),
            Column::Number => self.count.to_string(),
            Column::P50 => self.p50.map(|v| v.to_string()).unwrap_or(NA.to_string()),
            Column::P90 => self.p90.map(|v| v.to_string()).unwrap_or(NA.to_string()),
            Column::P99 => self.p99.map(|v| v.to_string()).unwrap_or(NA.to_string()),
            Column::Max => self.max.map(|v| v.to_string()).unwrap_or(NA.to_string()),
        }
    }

    fn cmp(&self, other: &Self, column: Column) -> Ordering
        where
            Self: Sized {
        match column {
            Column::Description => self.description.cmp(&other.description),
            Column::MeasureUnit => self.unit.cmp(&other.unit),
            Column::Number => self.count.cmp(&other.count),
            Column::P50 => self.p50.partial_cmp(&other.p50).unwrap_or(Ordering::Equal),
            Column::P90 => self.p90.partial_cmp(&other.p90).unwrap_or(Ordering::Equal),
            Column::P99 => self.p99.partial_cmp(&other.p99).unwrap_or(Ordering::Equal),
            Column::Max => self.max.partial_cmp(&other.max).unwrap_or(Ordering::Equal),
        }
    }
}

type MetricsTable = TableView<SingleMetrics, Column>;