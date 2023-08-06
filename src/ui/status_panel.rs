use cursive::View;
use cursive::view::{Nameable, Resizable};
use cursive::views::{LinearLayout, TextView, DummyView};
use crate::ui::ui_elements::UIElementName;

pub fn build_status_panel() -> Box<dyn View> {
    let mut layout = LinearLayout::horizontal();

    layout.add_child(TextView::empty().no_wrap().with_name(UIElementName::StatusFile));
    layout.add_child(DummyView{}.fixed_width(3));
    layout.add_child(TextView::empty().no_wrap().with_name(UIElementName::StatusHint).full_width());
    layout.add_child(DummyView{}.fixed_width(3));
    layout.add_child(TextView::empty().no_wrap().with_name(UIElementName::StatusPosition));

    Box::new(layout.full_width())
}