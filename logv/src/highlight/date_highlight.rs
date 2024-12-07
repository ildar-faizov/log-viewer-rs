use crate::highlight::highlight::{Highlight, Highlighter};
use crate::highlight::style_with_priority::StyleWithPriority;
use crate::model::model::RootModel;
use crate::model::rendered::LineRender;
use crate::app_theme::app_theme::{AppTheme, AppThemeKey};

pub struct DateHighlighter<T> {
    payload: T,
}

impl <T> DateHighlighter<T> {
    pub fn new(payload: T) -> Self {
        DateHighlighter {
            payload
        }
    }
}

impl <T> Highlighter<T> for DateHighlighter<T> where T: Clone {
    fn process(&self, line: &LineRender, model: &RootModel) -> Vec<Highlight<T>> {
        if let Some(kdf) = model.get_date_format() {
            let ctx = model.get_date_guess_context();
            if let Some((_, m)) = kdf.parse_and_match(&line.content, &ctx) {
                return vec![Highlight::new(m.start(), m.end(), self.payload.clone())];
            }
        }
        vec![]
    }
}

pub fn create_date_highlighter(app_theme: &AppTheme) -> DateHighlighter<StyleWithPriority> {
    DateHighlighter::new(app_theme[AppThemeKey::Date])
}