use crate::app_theme::app_theme::AppTheme;
use crate::highlight::custom_highlighter::create_filtered_highlighter;
use crate::highlight::date_highlight::create_date_highlighter;
use crate::highlight::highlight::Highlighter;
use crate::highlight::search_highlighter::create_search_highlighter;
use crate::highlight::style_with_priority::StyleWithPriority;
use std::rc::Rc;

pub fn cursive_highlighters(app_theme: &AppTheme) -> Vec<Rc<dyn Highlighter<StyleWithPriority> + 'static>> {
    vec![
        Rc::new(create_date_highlighter(app_theme)),
        Rc::new(create_search_highlighter(app_theme)),
        Rc::new(create_filtered_highlighter(app_theme)),
    ]
}