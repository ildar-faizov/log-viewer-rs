use crate::model::model::RootModel;
use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use crate::shared::Shared;

pub trait EscapeHandler {
    fn on_esc(&mut self, root_model: &mut RootModel) -> EscapeHandlerResult;
}

pub enum EscapeHandlerResult {
    Ignore,
    Processed,
    Dismiss,
}

#[derive(Debug)]
pub struct EscapeHandlerManager
{
    escape_handler: Shared<CompoundEscapeHandler>,
    callback: fn(&mut RootModel) -> EscapeHandlerResult,
}

impl EscapeHandlerManager
where
{
    pub fn new(
        escape_handler: Shared<CompoundEscapeHandler>,
        callback: fn(&mut RootModel) -> EscapeHandlerResult,
    ) -> Self {
        Self {
            escape_handler,
            callback,
        }
    }

    pub fn toggle(&mut self, state: bool) {
        if state {
            let escape_handler = &mut *self.escape_handler.get_mut_ref();
            escape_handler.add_fn(self.callback);
        }
    }
}

pub struct BasicEscapeHandler<T>(T)
where
    T: Fn(&mut RootModel) -> EscapeHandlerResult;

impl<T> BasicEscapeHandler<T>
where
    T: Fn(&mut RootModel) -> EscapeHandlerResult
{
    pub fn boxed(t: T) -> Box<Self> {
        Box::new(Self(t))
    }
}

impl<T> EscapeHandler for BasicEscapeHandler<T>
where
    T: Fn(&mut RootModel) -> EscapeHandlerResult
{
    fn on_esc(&mut self, root_model: &mut RootModel) -> EscapeHandlerResult {
        (&self.0)(root_model)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HandlerId(usize);

#[derive(Default)]
pub struct CompoundEscapeHandler {
    handlers: VecDeque<(HandlerId, Box<dyn EscapeHandler>)>,
}

impl EscapeHandler for CompoundEscapeHandler {
    fn on_esc(&mut self, root_model: &mut RootModel) -> EscapeHandlerResult {
        let mut to_dismiss = None;
        for (i, (_, h)) in self.handlers.iter_mut().enumerate().rev() {
            match h.on_esc(root_model) {
                EscapeHandlerResult::Processed => return EscapeHandlerResult::Processed,
                EscapeHandlerResult::Dismiss => {
                    to_dismiss = Some(i);
                    break;
                },
                _ => {},
            }
        }
        if let Some(i) = to_dismiss {
            self.handlers.remove(i);
            return EscapeHandlerResult::Processed;
        }
        EscapeHandlerResult::Ignore
    }
}

impl Debug for CompoundEscapeHandler {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CompoundEscapeHandler({:?})", self.handlers.len())
    }
}

impl CompoundEscapeHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, h: Box<dyn EscapeHandler>) -> HandlerId {
        let id = HandlerId(self.handlers.len());
        self.handlers.push_back((id, h));
        id
    }

    pub fn add_fn<F>(&mut self, h: F) -> HandlerId
    where
        F: Fn(&mut RootModel) -> EscapeHandlerResult + 'static
    {
        self.add(BasicEscapeHandler::boxed(h))
    }

    pub fn remove(&mut self, id: &HandlerId) {
        self.handlers.retain(|(i, _)| i != id);
    }
}