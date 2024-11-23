use uuid::Uuid;
use crate::background_process::background_process_builder::BackgroundProcessBuilder;
use crate::background_process::task_context::TaskContext;
use crate::background_process::background_process_handler::BackgroundProcessHandler;
use crate::background_process::signal::Signal;
use crate::model::model::RootModel;

pub trait RunInBackground: Sized {
    fn run_in_background<T1, T2, M, T, R, L>(
        &mut self,
        title: T1,
        description: T2,
        task: T,
        listener: L
    ) -> BackgroundProcessHandler
    where
        T1: ToString,
        T2: ToString,
        M: Send + 'static,
        R: Send + 'static,
        T: FnOnce(&mut TaskContext<M, R>) -> R,
        T: Send + 'static,
        L: FnMut(&mut RootModel, Signal<M, R>, &Uuid) + 'static;

    fn background_process_builder<M, T, R, L>(&mut self) -> BackgroundProcessBuilder<M, T, R, L, Self>
        where
            M: Send + 'static,
            R: Send + 'static,
            T: FnOnce(&mut TaskContext<M, R>) -> R,
            T: Send + 'static,
            L: FnMut(&mut RootModel, Signal<M, R>, &Uuid) + 'static {
        BackgroundProcessBuilder::new(self)
    }
}



