use crate::highlight::style_with_priority::{StylePriority, StyleWithPriority};
use crate::ui::palette_utils::PaletteAdditions;
use cursive::reexports::enumset::EnumSet;
use cursive::theme;
use cursive::theme::PaletteColor::{Background, Highlight, Primary, Tertiary};
use cursive::theme::{ColorStyle, ColorType, Effect, Palette, Style};
use std::collections::HashMap;
use std::ops::Index;

pub struct AppTheme {
    pub name: AppThemeName,
    styles: HashMap<AppThemeKey, StyleWithPriority>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum AppThemeKey {
    Regular,
    Cursor,
    Selection,
    LineNumber,
    Date,
    SearchCurrentOccurrence,
    SearchAnotherOccurrence,
    Filter,
}

impl Index<AppThemeKey> for AppTheme {
    type Output = StyleWithPriority;

    fn index(&self, index: AppThemeKey) -> &Self::Output {
        &self.styles[&index]
    }
}

#[derive(Debug, Copy, Clone)]
pub enum AppThemeName {
    SolarizedLight,
    SolarizedDark,
}

impl AppTheme {
    pub fn load(name: AppThemeName) -> Result<(Self, theme::Theme), theme::Error> {
        let asset = match name {
            AppThemeName::SolarizedLight => include_str!("../assets/solarized-light.toml"),
            AppThemeName::SolarizedDark => include_str!("../assets/solarized-dark.toml"),
        };
        let theme = theme::load_toml(asset)?;
        Ok((Self::new(name, &theme.palette), theme))
    }

    fn new(name: AppThemeName, palette: &Palette) -> Self {
        match name {
            AppThemeName::SolarizedLight => Self::light(palette),
            AppThemeName::SolarizedDark => Self::dark(palette),
        }
    }

    fn light(palette: &Palette) -> Self {
        Self {
            name: AppThemeName::SolarizedLight,
            styles: Self::common(palette),
        }
    }

    fn dark(palette: &Palette) -> Self {
        Self {
            name: AppThemeName::SolarizedDark,
            styles: Self::common(palette),
        }
    }

    fn common(palette: &Palette) -> HashMap<AppThemeKey, StyleWithPriority> {
        let mut styles = HashMap::with_capacity(10);
        styles.insert(
            AppThemeKey::Regular,
            StyleWithPriority::new(
                Style::from(ColorStyle::new(palette[Primary], palette[Background])),
                StylePriority::Regular,
            ),
        );
        styles.insert(
            AppThemeKey::Cursor,
            StyleWithPriority::new(
                Style::from(ColorStyle::new(palette[Background], palette[Primary])),
                StylePriority::Cursor,
            ),
        );
        styles.insert(
            AppThemeKey::Selection,
            StyleWithPriority::new(
                Style::from(ColorStyle::new(palette[Tertiary], palette[Highlight])),
                StylePriority::Selection,
            ),
        );
        styles.insert(
            AppThemeKey::LineNumber,
            StyleWithPriority::new(
                Style {
                    color: ColorStyle::new(palette[Primary], palette[Background]),
                    effects: EnumSet::only(Effect::Italic),
                },
                StylePriority::LineNumber,
            ),
        );
        styles.insert(
            AppThemeKey::Date,
            StyleWithPriority::new(
                Style::from(ColorStyle::new(palette.blue(), ColorType::InheritParent)),
                StylePriority::Date,
            ),
        );
        styles.insert(
            AppThemeKey::SearchCurrentOccurrence,
            StyleWithPriority::new(
                Style::from(ColorStyle::new(palette[Background], palette.yellow())),
                StylePriority::Search,
            ),
        );
        styles.insert(
            AppThemeKey::SearchAnotherOccurrence,
            StyleWithPriority::new(
                Style::from(ColorStyle::new(palette.yellow(), ColorType::InheritParent)),
                StylePriority::Search,
            ),
        );
        styles.insert(
            AppThemeKey::Filter,
            StyleWithPriority::new(
                Style::from(ColorStyle::new(palette.green(), ColorType::InheritParent)),
                StylePriority::Filter,
            ),
        );
        styles
    }
}
