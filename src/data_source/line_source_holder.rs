use std::fs::File;
use std::io::Cursor;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use fluent_integer::Integer;

use crate::data_source::filtered::FilteredLineSource;
use crate::data_source::line_registry::LineRegistryImpl;
use crate::data_source::{Data, Direction, FileBackend, Line, LineSource, LineSourceBackend, LineSourceImpl, StrBackend};

#[derive(Clone)]
pub enum ConcreteLineSourceHolder {
    FileBased(LineSourceImpl<File, FileBackend>),
    ConstantBased(LineSourceImpl<Cursor<&'static [u8]>, StrBackend<'static>>),
}

impl Deref for ConcreteLineSourceHolder {
    type Target = dyn LineSource;

    fn deref(&self) -> &Self::Target {
        match self {
            ConcreteLineSourceHolder::FileBased(obj) => obj,
            ConcreteLineSourceHolder::ConstantBased(obj) => obj,
        }
    }
}

impl DerefMut for ConcreteLineSourceHolder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            ConcreteLineSourceHolder::FileBased(obj) => obj,
            ConcreteLineSourceHolder::ConstantBased(obj) => obj,
        }
    }
}

impl From<LineSourceImpl<Cursor<&'static [u8]>, StrBackend<'static>>> for ConcreteLineSourceHolder {
    fn from(value: LineSourceImpl<Cursor<&'static [u8]>, StrBackend<'static>>) -> Self {
        ConcreteLineSourceHolder::ConstantBased(value)
    }
}

impl From<LineSourceImpl<File, FileBackend>> for ConcreteLineSourceHolder {
    fn from(value: LineSourceImpl<File, FileBackend>) -> Self {
        ConcreteLineSourceHolder::FileBased(value)
    }
}

impl ConcreteLineSourceHolder {
    pub fn get_length(&self) -> Integer {
        match &self {
            ConcreteLineSourceHolder::FileBased(h) => h.get_length(),
            ConcreteLineSourceHolder::ConstantBased(h) => h.get_length(),
        }
    }
}

pub enum LineSourceHolder {
    Concrete(ConcreteLineSourceHolder),
    Filtered(FilteredLineSource),
}

impl Deref for LineSourceHolder {
    type Target = dyn LineSource;

    fn deref(&self) -> &Self::Target {
        match self {
            LineSourceHolder::Concrete(concrete) => concrete.deref(),
            LineSourceHolder::Filtered(obj) => obj,
        }
    }
}

impl DerefMut for LineSourceHolder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            LineSourceHolder::Concrete(concrete) => concrete.deref_mut(),
            LineSourceHolder::Filtered(obj) => obj,
        }
    }
}

impl<T> From<T> for LineSourceHolder
where T: Into<ConcreteLineSourceHolder> {
    fn from(value: T) -> Self {
        LineSourceHolder::Concrete(value.into())
    }
}

impl From<FilteredLineSource> for LineSourceHolder {
    fn from(value: FilteredLineSource) -> Self {
        LineSourceHolder::Filtered(value)
    }
}

impl LineSource for LineSourceHolder {

    fn read_lines(&mut self, offset: Integer, number_of_lines: Integer) -> Data {
        self.deref_mut().read_lines(offset, number_of_lines)
    }

    fn read_next_line(&mut self, offset: Integer) -> Option<Line> {
        self.deref_mut().read_next_line(offset)
    }

    fn read_prev_line(&mut self, offset: Integer) -> Option<Line> {
        self.deref_mut().read_prev_line(offset)
    }

    fn track_line_number(&mut self, track: bool) {
        self.deref_mut().track_line_number(track)
    }

    fn read_raw(&mut self, start: Integer, end: Integer) -> Result<String, ()> {
        self.deref_mut().read_raw(start, end)
    }

    fn skip_token(&mut self, offset: Integer, direction: Direction) -> anyhow::Result<Integer> {
        self.deref_mut().skip_token(offset, direction)
    }

    fn get_line_registry(&self) -> Arc<LineRegistryImpl> {
        self.deref().get_line_registry()
    }
}