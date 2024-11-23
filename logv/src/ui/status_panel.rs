use cursive::view::{Nameable, Resizable};
use cursive::views::{LinearLayout, TextView, DummyView};
use crate::ui::bgp_status::build_bgp_status;
use crate::ui::ui_elements::UIElementName;
use crate::ui::view_with_callback::ViewWithCallback;

pub const STATUS_PANEL: &str = "StatusPanel";

pub fn build_status_panel() -> ViewWithCallback {
    let mut layout = LinearLayout::horizontal();

    layout.add_child(TextView::empty().no_wrap().with_name(UIElementName::StatusFile));
    layout.add_child(DummyView{}.fixed_width(3));
    layout.add_child(TextView::empty().no_wrap().with_name(UIElementName::StatusHint).full_width());
    layout.add_child(DummyView{}.fixed_width(1));
    let bgp = build_bgp_status();
    layout.add_child(bgp.view);
    layout.add_child(DummyView{}.fixed_width(3));
    layout.add_child(TextView::empty().no_wrap().with_name(UIElementName::StatusPosition));

    ViewWithCallback::new(layout.with_name(STATUS_PANEL).full_width(), bgp.callback)
}