use cursive::{Cursive, View};
use cursive::traits::{Finder, Nameable, Resizable};
use cursive::view::IntoBoxedView;
use cursive::views::{DummyView, LinearLayout, ProgressBar};
use crate::model::bgp_model::{BGPModel, BGPModelEvent};
use crate::ui::status_panel::STATUS_PANEL;
use crate::ui::view_with_callback::{ViewUpdateCallback, ViewWithCallback};

const BGP_PROGRESS_BAR: &str = "BGPProgressBar";
const BGP_DUMMY_PANEL: &str = "BGPDummyPanel";

pub fn build_bgp_status() -> ViewWithCallback {
    ViewWithCallback::with_dummy_callback(build_dummy_panel())
}

pub fn handle_bgp_event(model: &BGPModel, _evt: BGPModelEvent) -> ViewUpdateCallback {
    let process_count = model.get_number();
    let overall_progress = model.get_overall_progress();
    Box::new(move |app: &mut Cursive| {
        app.call_on_name(STATUS_PANEL, |sp: &mut LinearLayout| {
            if process_count == 0 {
                if let Some(p) = sp.find_child_from_name(BGP_PROGRESS_BAR) {
                    sp.remove_child(p);
                    sp.insert_child(p, build_dummy_panel());
                };
            } else {
                if let Some(p) = sp.find_child_from_name(BGP_DUMMY_PANEL) {
                    sp.remove_child(p);
                    sp.insert_child(p, build_progress_bar());
                }
                sp.call_on_name(BGP_PROGRESS_BAR, |pb: &mut ProgressBar| {
                    pb.set_value(overall_progress as usize);
                });
            }
        });
    })
}

fn build_progress_bar() -> Box<dyn View> {
    ProgressBar::new()
        .with_name(BGP_PROGRESS_BAR)
        .full_width()
        .into_boxed_view()
}

fn build_dummy_panel() -> Box<dyn View> {
    DummyView {}
        .with_name(BGP_DUMMY_PANEL)
        .full_width()
        .into_boxed_view()
}