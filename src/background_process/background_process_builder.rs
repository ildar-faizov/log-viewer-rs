use crate::background_process::background_process_handler::BackgroundProcessHandler;
use crate::background_process::run_in_background::RunInBackground;
use crate::background_process::task_context::TaskContext;
use crate::model::model::RootModel;
use std::marker::PhantomData;

pub struct BackgroundProcessBuilder<'a, M, T, R, L, H>
where
    M: Send + 'static,
    R: Send + 'static,
    T: FnOnce(&TaskContext<M, R>) -> R,
    T: Send + 'static,
    L: FnMut(&mut RootModel, Result<R, M>) + 'static,
    H: RunInBackground,
{
    runner: &'a mut H,
    task: Option<T>,
    listener: Option<L>,
    phantom_message: PhantomData<M>,
    phantom_result: PhantomData<R>,
}

impl<'a, M, T, R, L, H> BackgroundProcessBuilder<'a, M, T, R, L, H>
where
    M: Send + 'static,
    R: Send + 'static,
    T: FnOnce(&TaskContext<M, R>) -> R,
    T: Send + 'static,
    L: FnMut(&mut RootModel, Result<R, M>) + 'static,
    H: RunInBackground,
{
    pub fn new(runner: &'a mut H) -> Self {
        BackgroundProcessBuilder {
            runner,
            task: None,
            listener: None,
            phantom_message: PhantomData::default(),
            phantom_result: PhantomData::default(),
        }
    }

    pub fn with_task(mut self, task: T) -> Self {
        self.task.replace(task);
        self
    }

    pub fn listener(mut self, listener: L) -> Self {
        self.listener.replace(listener);
        self
    }

    pub fn run(self) -> BackgroundProcessHandler {
        let task = self.task.expect("");
        let listener = self.listener.expect("");
        self.runner.run_in_background(task, listener)
    }
}
