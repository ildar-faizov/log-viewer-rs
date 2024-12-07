use cursive::theme::{Color, Palette};

#[allow(dead_code)]
pub trait PaletteAdditions {
    fn yellow(&self) -> Color;

    fn orange(&self) -> Color;

    fn red(&self) -> Color;

    fn magenta(&self) -> Color;

    fn violet(&self) -> Color;

    fn blue(&self) -> Color;

    fn cyan(&self) -> Color;

    fn green(&self) -> Color;
}

impl PaletteAdditions for Palette {
    fn yellow(&self) -> Color {
        custom_color(self, "yellow")
    }

    fn orange(&self) -> Color {
        custom_color(self, "orange")
    }

    fn red(&self) -> Color {
        custom_color(self, "red")
    }

    fn magenta(&self) -> Color {
        custom_color(self, "magenta")
    }

    fn violet(&self) -> Color {
        custom_color(self, "violet")
    }

    fn blue(&self) -> Color {
        custom_color(self, "blue")
    }

    fn cyan(&self) -> Color {
        custom_color(self, "cyan")
    }

    fn green(&self) -> Color {
        custom_color(self, "green")
    }
}

fn custom_color(palette: &Palette, key: &str) -> Color {
    *palette.custom(key).unwrap()
}
