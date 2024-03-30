use cursive::{Cursive, View};
use cursive::view::IntoBoxedView;

pub type ViewUpdateCallback = Box<dyn FnOnce(&mut Cursive)>;

pub struct ViewWithCallback {
    pub view: Box<dyn View>,
    pub callback: ViewUpdateCallback,
}

impl ViewWithCallback {
    pub fn new(view: impl IntoBoxedView, callback: ViewUpdateCallback) -> Self {
        Self {
            view: view.into_boxed_view(),
            callback,
        }
    }

    pub fn with_dummy_callback(view: impl IntoBoxedView) -> Self {
        let callback = Box::new(|_app: &mut Cursive| {});
        Self {
            view: view.into_boxed_view(),
            callback
        }
    }
}

impl<T: IntoBoxedView> From<T> for ViewWithCallback {
    fn from(value: T) -> Self {
        ViewWithCallback::with_dummy_callback(value)
    }
}

impl Into<ViewUpdateCallback> for ViewWithCallback {
    fn into(self) -> ViewUpdateCallback {
        Box::new(move |app: &mut Cursive| {
            app.add_layer(self.view);
            (self.callback)(app);
        })
    }
}