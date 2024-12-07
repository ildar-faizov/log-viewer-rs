use crate::app_theme::app_theme::{AppTheme, AppThemeKey};
use crate::highlight::highlight::{Highlight, Highlighter};
use crate::highlight::style_with_priority::StyleWithPriority;
use crate::immediate::Immediate;
use crate::interval::Interval;
use crate::model::model::RootModel;
use crate::model::rendered::LineRender;

pub struct SearchHighlighter<T> {
    current_occurrence_style: T,
    other_occurrence_style: T,
}

impl <T> SearchHighlighter<T> {
    pub fn new(current_occurrence_style: T, other_occurrence_style: T) -> SearchHighlighter<T> {
        SearchHighlighter {
            current_occurrence_style,
            other_occurrence_style
        }
    }
}

impl<T: Clone> Highlighter<T> for SearchHighlighter<T> {
    fn process(&self, line_render: &LineRender, model: &RootModel) -> Vec<Highlight<T>> {
        let str = &line_render.content;
        let offset = line_render.start;
        if let Some(search) = model.get_current_search().as_mut() {
            let viewport = model.data().map(|dr|
                Interval::closed(dr.start.unwrap(), dr.end.unwrap())
            );
            if let Immediate::Immediate(current_occurrence) = search.get_current_occurrence(viewport.unwrap()) {
                return current_occurrence.map(|(occurrences, p)| {
                    occurrences.iter().enumerate().filter_map(|(i, occurrence)| {
                        if occurrence.end < offset || occurrence.start > offset + str.len() {
                            None
                        } else {
                            let s = (occurrence.start - offset).as_usize();
                            let e = (occurrence.end - offset).as_usize();
                            let payload = if Some(i) == p {
                                self.current_occurrence_style.clone()
                            } else {
                                self.other_occurrence_style.clone()
                            };
                            Some(Highlight::new(s, e, payload))
                        }
                    }).collect()
                }).unwrap_or_default()
            }
        }
        vec![]
    }
}

pub fn create_search_highlighter(app_theme: &AppTheme) -> SearchHighlighter<StyleWithPriority> {
    SearchHighlighter::new(
        app_theme[AppThemeKey::SearchCurrentOccurrence],
        app_theme[AppThemeKey::SearchAnotherOccurrence],
    )
}