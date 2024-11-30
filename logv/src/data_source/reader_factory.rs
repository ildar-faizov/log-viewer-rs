use crate::data_source::filtered::filtered_reader::FilteredReader;
use crate::data_source::line_source_holder::{ConcreteLineSourceHolder, LineSourceHolder};
use crate::data_source::LineSourceBackend;
use std::fmt::Debug;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};

pub trait ReaderFactory: Send + Sync + Debug {
    fn new_reader(&self) -> std::io::Result<UniversalReadSeek>;
}

pub trait HasReaderFactory {
    fn reader_factory(&self) -> Box<dyn ReaderFactory>;
}

pub enum BasicReadSeek {
    FileBased(File),
    CursorBased(Cursor<Vec<u8>>),
}

impl Read for BasicReadSeek {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            BasicReadSeek::FileBased(inner) => inner.read(buf),
            BasicReadSeek::CursorBased(inner) => inner.read(buf),
        }
    }
}

impl Seek for BasicReadSeek {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match self {
            BasicReadSeek::FileBased(inner) => inner.seek(pos),
            BasicReadSeek::CursorBased(inner) => inner.seek(pos),
        }
    }
}

pub enum UniversalReadSeek {
    Basic(BasicReadSeek),
    Filtered(FilteredReader<BasicReadSeek>),
}

impl Read for UniversalReadSeek {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            UniversalReadSeek::Basic(inner) => inner.read(buf),
            UniversalReadSeek::Filtered(inner) => inner.read(buf),
        }
    }
}

impl Seek for UniversalReadSeek {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match self {
            UniversalReadSeek::Basic(inner) => inner.seek(pos),
            UniversalReadSeek::Filtered(inner) => inner.seek(pos),
        }
    }
}

impl HasReaderFactory for LineSourceHolder {
    fn reader_factory(&self) -> Box<dyn ReaderFactory> {
        match self {
            LineSourceHolder::Concrete(c) => c.reader_factory(),
            LineSourceHolder::Filtered(f) => f.reader_factory(),
        }
    }
}

impl HasReaderFactory for ConcreteLineSourceHolder {
    fn reader_factory(&self) -> Box<dyn ReaderFactory> {
        match self {
            ConcreteLineSourceHolder::FileBased(inner) => {
                inner.backend.reader_factory()
            }
            ConcreteLineSourceHolder::ConstantBased(inner) => {
                inner.backend.reader_factory()
            }
        }
    }
}

pub mod string {
    use super::*;

    #[derive(Clone, Debug)]
    pub struct StringBasedReaderFactory(String);

    impl ReaderFactory for StringBasedReaderFactory {
        fn new_reader(&self) -> std::io::Result<UniversalReadSeek> {
            let v = Vec::from(self.0.clone());
            Ok(UniversalReadSeek::Basic(BasicReadSeek::CursorBased(Cursor::new(v))))
        }
    }

    impl StringBasedReaderFactory {
        pub fn new<S: ToString>(s: S) -> Self {
            Self(s.to_string())
        }
    }
}

pub mod file {
    use crate::data_source::reader_factory::{BasicReadSeek, ReaderFactory, UniversalReadSeek};
    use std::fs::File;
    use std::path::PathBuf;

    #[derive(Clone, Debug)]
    pub struct FileBasedReaderFactory(PathBuf);

    impl ReaderFactory for FileBasedReaderFactory {
        fn new_reader(&self) -> std::io::Result<UniversalReadSeek> {
            Ok(UniversalReadSeek::Basic(BasicReadSeek::FileBased(File::open(&self.0)?)))
        }
    }

    impl FileBasedReaderFactory {
        pub fn new(file_name: PathBuf) -> Self {
            Self(file_name)
        }
    }
}

pub mod filtered {
    use std::fmt::{Debug, Formatter};
    use crate::data_source::filtered::filtered_reader::FilteredReader;
    use crate::data_source::reader_factory::{ReaderFactory, UniversalReadSeek};
    use crate::data_source::CustomHighlight;
    use std::io::BufReader;
    use std::sync::Arc;

    pub struct FilteredReaderFactory
    {
        factory: Box<dyn ReaderFactory>,
        filter: Arc<dyn Fn(&str) -> Vec<CustomHighlight> + Sync + Send + 'static>,
        neighbourhood: u8,
    }

    impl ReaderFactory for FilteredReaderFactory
    {
        fn new_reader(&self) -> std::io::Result<UniversalReadSeek> {
            let inner = self.factory.new_reader()?;
            match inner {
                UniversalReadSeek::Basic(basic) => {
                    let f = FilteredReader::new(BufReader::new(basic), self.filter.clone(), self.neighbourhood);
                    Ok(UniversalReadSeek::Filtered(f))
                }
                UniversalReadSeek::Filtered(_) => todo!(),
            }
        }
    }

    impl Debug for FilteredReaderFactory {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}, neighbourhood = {:?}", &self.factory, self.neighbourhood)
        }
    }

    impl FilteredReaderFactory
    {
        pub fn new(
            factory: Box<dyn ReaderFactory>,
            filter: Arc<dyn Fn(&str) -> Vec<CustomHighlight> + Sync + Send + 'static>,
            neighbourhood: u8,
        ) -> Self {
            Self {
                factory,
                filter,
                neighbourhood,
            }
        }
    }
}
