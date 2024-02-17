use crate::background_process::background_process_handler::BackgroundProcessHandler;
use crate::background_process::run_in_background::RunInBackground;
use crate::background_process::task_context::TaskContext;
use crate::model::model::RootModel;
use std::marker::PhantomData;
use uuid::Uuid;
use crate::background_process::signal::Signal;

pub struct BackgroundProcessBuilder<'a, M, T, R, L, H>
where
    M: Send + 'static,
    R: Send + 'static,
    T: FnOnce(&mut TaskContext<M, R>) -> R,
    T: Send + 'static,
    L: FnMut(&mut RootModel, Signal<M, R>, &Uuid) + 'static,
    H: RunInBackground,
{
    runner: &'a mut H,
    title: Option<String>,
    description: Option<String>,
    task: Option<T>,
    listener: Option<L>,
    phantom_message: PhantomData<M>,
    phantom_result: PhantomData<R>,
}

impl<'a, M, T, R, L, H> BackgroundProcessBuilder<'a, M, T, R, L, H>
where
    M: Send + 'static,
    R: Send + 'static,
    T: FnOnce(&mut TaskContext<M, R>) -> R,
    T: Send + 'static,
    L: FnMut(&mut RootModel, Signal<M, R>, &Uuid) + 'static,
    H: RunInBackground,
{
    pub fn new(runner: &'a mut H) -> Self {
        BackgroundProcessBuilder {
            runner,
            title: None,
            description: None,
            task: None,
            listener: None,
            phantom_message: PhantomData::default(),
            phantom_result: PhantomData::default(),
        }
    }

    pub fn with_title<S: ToString>(mut self, title: S) -> Self {
        self.title.replace(title.to_string());
        self
    }

    pub fn with_description<S: ToString>(mut self, description: S) -> Self {
        self.description.replace(description.to_string());
        self
    }

    pub fn with_task(mut self, task: T) -> Self {
        self.task.replace(task);
        self
    }

    pub fn with_listener(mut self, listener: L) -> Self {
        self.listener.replace(listener);
        self
    }

    pub fn run(self) -> BackgroundProcessHandler {
        let title = self.title.expect("Title is missing");
        let description = self.description.expect("Description is missing");
        let task = self.task.expect("");
        let listener = self.listener.expect("");
        self.runner.run_in_background(title, description, task, listener)
    }
}
