use crate::highlight::custom_highlighter::create_filtered_highlighter;
use crate::highlight::date_highlight::create_date_highlighter;
use crate::highlight::highlight::Highlighter;
use crate::highlight::search_highlighter::create_search_highlighter;
use crate::highlight::style_with_priority::StyleWithPriority;
use cursive::theme::Palette;
use std::rc::Rc;

pub fn cursive_highlighters(_palette: &Palette) -> Vec<Rc<dyn Highlighter<StyleWithPriority> + 'static>> {
    vec![
        Rc::new(create_date_highlighter()),
        Rc::new(create_search_highlighter()),
        Rc::new(create_filtered_highlighter()),
    ]
}