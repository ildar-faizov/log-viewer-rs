use crate::background_process::background_process_builder::BackgroundProcessBuilder;
use crate::background_process::task_context::TaskContext;
use crate::background_process::background_process_handler::BackgroundProcessHandler;
use crate::model::model::RootModel;

pub trait RunInBackground: Sized {
    fn run_in_background<M, T, R, L>(&mut self, task: T, listener: L) -> BackgroundProcessHandler
    where
        M: Send + 'static,
        R: Send + 'static,
        T: FnOnce(&mut TaskContext<M, R>) -> R,
        T: Send + 'static,
        L: FnMut(&mut RootModel, Result<R, M>) + 'static;

    fn background_process_builder<M, T, R, L>(&mut self) -> BackgroundProcessBuilder<M, T, R, L, Self>
        where
            M: Send + 'static,
            R: Send + 'static,
            T: FnOnce(&mut TaskContext<M, R>) -> R,
            T: Send + 'static,
            L: FnMut(&mut RootModel, Result<R, M>) + 'static {
        BackgroundProcessBuilder::new(self)
    }
}



