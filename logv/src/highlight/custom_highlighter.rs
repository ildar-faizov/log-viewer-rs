use crate::app_theme::app_theme::{AppTheme, AppThemeKey};
use crate::data_source::filtered::FILTERED_LINE_SOURCE_CUSTOM_DATA_KEY;
use crate::highlight::highlight::{Highlight, Highlighter};
use crate::highlight::style_with_priority::StyleWithPriority;
use crate::model::model::RootModel;
use crate::model::rendered::LineRender;
use itertools::Itertools;

pub struct CustomHighlighter<T> {
    key: &'static str,
    payload: T,
}

impl<T> CustomHighlighter<T> {
    pub fn new(key: &'static str, payload: T) -> Self {
        Self {
            key,
            payload,
        }
    }
}

impl<T: Clone> Highlighter<T> for CustomHighlighter<T> {
    fn process(&self, line_render: &LineRender, _model: &RootModel) -> Vec<Highlight<T>> {
        line_render.custom_highlights
            .get(self.key)
            .map(|items| {
                items
                    .iter()
                    .map(|item| Highlight::new(item.start(), item.end(), self.payload.clone()))
                    .collect_vec()
            })
            .unwrap_or_default()
    }
}

pub fn create_filtered_highlighter(app_theme: &AppTheme) -> CustomHighlighter<StyleWithPriority> {
    CustomHighlighter::new(
        FILTERED_LINE_SOURCE_CUSTOM_DATA_KEY,
        app_theme[AppThemeKey::Filter]
    )
}