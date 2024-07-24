use cursive::event::EventResult;
use cursive::reexports::enumset::EnumSet;
use cursive::theme::{ColorStyle, Effect, Style, Theme};
use cursive::theme::PaletteColor::{Background, HighlightText, Primary};
use cursive::utils::span::SpannedStr;
use cursive::view::Nameable;
use cursive::views::{Canvas, NamedView};
use log::Level;
use metrics::{describe_histogram, Unit};

use crate::actions::action_registry::ActionRegistry;
use crate::highlight::highlighter_registry::cursive_highlighters;
use crate::highlight::style_with_priority::StyleWithPriority;
use crate::model::model::RootModel;
use crate::profiles::OS_PROFILE;
use crate::shared::Shared;
use crate::ui::line_drawer::LineDrawer;
use crate::ui::ui_elements::UIElementName;
use crate::utils::{NumberOfDecimalDigits, stat, stat_l};

const METRIC_DRAW: &str = "draw";
const METRIC_ACTION: &str = "action";

pub fn build_canvas(model: Shared<RootModel>) -> NamedView<Canvas<Shared<RootModel>>> {
    describe_histogram!(METRIC_DRAW, Unit::Microseconds, "Time to draw canvas");
    describe_histogram!(METRIC_ACTION, Unit::Microseconds, "UI action");

    let palette = Theme::default().palette;
    let highlighters = cursive_highlighters(&palette);
    let regular_style = StyleWithPriority::new(Style::from(ColorStyle::new(palette[Primary], palette[HighlightText])), 0, 0);
    let cursor_style = StyleWithPriority::new(Style::from(ColorStyle::new(palette[HighlightText], palette[Primary])), 0xff, 0xff);
    let selection_style = StyleWithPriority::new(Style::from(ColorStyle::new(palette[HighlightText], palette[Background])), 1, 0xff);
    let line_number_style = StyleWithPriority::new(Style {
        color: ColorStyle::new(palette[Primary], palette[HighlightText]),
        effects: EnumSet::only(Effect::Italic)
    }, 1, 0xff);
    Canvas::new(model)
        .with_draw(move |state, printer| stat(METRIC_DRAW, &Unit::Milliseconds, || {
            let mut state = state.get_mut_ref();

            state.set_viewport_height(printer.size.y); // fetches data

            let mut max_line_number = None;
            let mut effective_viewport_width = printer.size.x;
            if state.is_show_line_numbers() {
                if let Some(data) = state.data() {
                    max_line_number = data.lines.iter()
                        .map(|line| *line.line_no.as_ref().unwrap_or(&0))
                        .max();
                    if let Some(max_line_number) = max_line_number {
                        let max_line_number_len = max_line_number.number_of_decimal_digits();
                        effective_viewport_width -= max_line_number_len;
                    }
                }
            }
            state.set_viewport_width(effective_viewport_width);

            if let Some(data) = state.data() {
                let line_drawer = LineDrawer::new()
                    .with_state(&state)
                    .with_highlighters(&highlighters)
                    .with_width(printer.size.x)
                    .with_regular_style(regular_style)
                    .with_cursor_style(cursor_style)
                    .with_selection_style(selection_style)
                    .with_line_number_style(line_number_style)
                    .with_show_line_numbers(state.is_show_line_numbers())
                    .with_max_line_number(max_line_number.unwrap_or(0));
                data.lines.iter()
                    .take(printer.size.y)
                    .map(|line| line_drawer.draw(line))
                    .enumerate()
                    .for_each(|(i, ss)|
                        printer.print_styled((0, i), SpannedStr::from(&ss))
                    );
            } else {
                printer.clear();
            }
        }))
        .with_on_event(move |state, event| {
            let action = {
                let model = &mut state.get_mut_ref();
                let actions = &*model.get_action_registry();
                actions.lookup_by_key(&event).cloned()
            };
            match action {
                Some(action) => {
                    let state = &mut state.get_mut_ref();
                    stat_l(Level::Info, METRIC_ACTION, &Unit::Microseconds, move || {
                        profiling::scope!("ui action", action.description());
                        action.perform_action(state, &event)
                    })
                },
                None => EventResult::Ignored
            }
        })
        .with_name(UIElementName::MainContent)
}