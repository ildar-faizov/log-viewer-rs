use cursive::Cursive;
use cursive::views::LayerPosition;

pub trait PopLayerSafely {
    fn pop_layer_safely(&mut self, name: &str);
}

impl PopLayerSafely for Cursive {
    fn pop_layer_safely(&mut self, name: &str) {
        let screen = self.screen_mut();
        let layer_position = screen.find_layer_from_name(name);
        let len = screen.len();
        let is_on_top = match layer_position {
            Some(LayerPosition::FromFront(0)) => true,
            Some(LayerPosition::FromBack(u)) => u + 1 == len,
            _ => false,
        };
        if is_on_top {
            self.pop_layer();
        }
    }
}