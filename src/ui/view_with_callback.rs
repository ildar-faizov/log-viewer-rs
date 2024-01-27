use cursive::{Cursive, View};

pub type ViewUpdateCallback = Box<dyn FnOnce(&mut Cursive)>;

pub struct ViewWithCallback {
    pub view: Box<dyn View>,
    pub callback: ViewUpdateCallback,
}

impl ViewWithCallback {
    pub fn new(view: Box<dyn View>, callback: ViewUpdateCallback) -> Self {
        Self {
            view,
            callback,
        }
    }
}