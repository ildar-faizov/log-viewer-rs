use cursive::{Cursive, View};

pub struct ViewWithCallback {
    pub view: Box<dyn View>,
    pub callback: Box<dyn FnOnce(&mut Cursive)>,
}

impl ViewWithCallback {
    pub fn new(view: Box<dyn View>, callback: Box<dyn FnOnce(&mut Cursive)>) -> Self {
        Self {
            view,
            callback,
        }
    }
}