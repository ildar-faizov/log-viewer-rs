use crossbeam_channel::Sender;
use std::path::{Path, PathBuf};
use std::env::current_dir;
use std::fs::read_to_string;
use ModelEvent::*;
use crate::data_source::{DataSource, FileSource, Data, LineSource, LineSourceImpl};
use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use std::borrow::Borrow;
use num_rational::{Rational64, Ratio};
use std::fmt;
use std::ops::Mul;
use std::cmp::{max, min};
use std::io::Read;
use std::option::Option::Some;
use crate::utils;
use std::panic::panic_any;
use crate::shared::Shared;

const BUFFER_SIZE: u64 = 8192;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
struct Dimension {
    width: usize,
    height: usize,
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Dimension(w={}, h={})", self.width, self.height)
    }
}

/* Describes scroll position.
 * starting_point denotes initial scroll position. It is 0 at the beginning. A user
 * may scroll to the end (then it is 1) or choose some point in between. Belongs to [0, 1].
 * shift denotes number of lines to count from starting_point.
 *
 * E.g. when user scrolls 3 lines down from the beginning of the file, starting_point=0 and shift=3.
 * E.g. when user scrolls to the bottom, starting_point=1 and shift=0.
 */
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
struct ScrollPosition {
    starting_point: Ratio<u64>,
    // [0, 1] - initial point in scroll area
    shift: i32,
}

impl fmt::Display for ScrollPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ScrollPosition(starting_point={}, shift={})", self.starting_point, self.shift)
    }
}

pub struct RootModel {
    model_sender: Sender<ModelEvent>,
    file_name: Option<String>,
    file_size: u64,
    file_content: Option<String>,
    data: Option<Data>,
    viewport_size: Dimension,
    scroll_position: ScrollPosition,
    horizontal_scroll: usize,
    viewport_content: Option<String>,
    datasource: Option<Shared<Box<dyn LineSource>>>,
    error: Option<Box<dyn ToString>>,
}

#[derive(Clone)]
pub struct RootModelRef(Rc<RefCell<RootModel>>);

pub enum ModelEvent {
    FileName(String),
    FileContent,
    DataUpdated,
    Error(String),
    Quit,
}

impl RootModel {
    pub fn new(model_sender: Sender<ModelEvent>) -> Self {
        RootModel {
            model_sender,
            file_name: None,
            file_size: 0,
            file_content: None,
            data: None,
            viewport_size: Dimension::default(),
            scroll_position: ScrollPosition::default(),
            horizontal_scroll: 0,
            viewport_content: None,
            datasource: None,
            error: None,
        }
    }

    fn emit_event(&self, event: ModelEvent) {
        self.model_sender.send(event).unwrap();
    }

    pub fn file_name(&self) -> Option<&str> {
        self.file_name.as_ref().map(|s| &s[..])
    }

    pub fn set_file_name(&mut self, value: String) {
        if self.file_name.as_ref().map(|file_name| *file_name != value).unwrap_or(true) {
            log::info!("File name set to {}", value);
            self.file_name = Some(value);
            self.emit_event(FileName(self.file_name.as_ref().unwrap().to_owned()));
            self.load_file();
        }
    }

    pub fn file_content(&self) -> Option<&str> {
        self.file_content.as_ref().map(|s| &s[..])
    }

    fn set_file_content(&mut self, content: String) {
        self.file_content = Some(content);
        self.emit_event(FileContent);
    }

    pub fn data(&self) -> Option<&Data> {
        self.data.as_ref()
    }

    fn set_data(&mut self, data: Data) {
        self.data = Some(data);
        self.emit_event(DataUpdated);
    }

    pub fn error(&self) -> Option<String> {
        self.error.as_ref().map(|t| t.to_string())
    }

    pub fn set_viewport_size(&mut self, width: usize, height: usize) {
        let d = Dimension::new(width, height);
        if self.viewport_size != d {
            log::info!("Viewport size set to {}", d);
            self.viewport_size = d;
            // TODO: emit update
            self.update_viewport_content();
        }
    }

    pub fn set_scroll_position(&mut self, scroll_position: ScrollPosition) {
        if self.scroll_position != scroll_position {
            let previous_scroll_position = self.scroll_position;
            self.scroll_position = scroll_position;
            log::info!("Scroll position set to {}", scroll_position);
            if !self.update_viewport_content() {
                self.scroll_position = previous_scroll_position;
            }
            // TODO: emit update
        }
    }

    pub fn scroll(&mut self, num_of_lines: isize) {
        if num_of_lines == 0 {
            return;
        }
        log::trace!("scroll num_of_lines = {}", num_of_lines);
        if let Some(data) = &self.data {
            if let Some(first_line) = data.lines.first() {
                let (n, sign) = utils::sign(num_of_lines);
                let delta =
                    if sign == 1 {
                        if let Some(line) = data.lines.get(n - 1) {
                            (line.end - first_line.start + 1) as i32
                        } else {
                            log::warn!("Scroll {} lines failed, no matching line", num_of_lines);
                            0
                        }
                    } else if first_line.start > 0 {
                        if let Some(ds) = &self.datasource {
                            let mut ds = ds.get_mut_ref();
                            log::trace!("scroll({}). set_offset = {}", num_of_lines, first_line.start);
                            ds.set_offset(first_line.start);
                            ds.read_lines(num_of_lines);
                            let offset = ds.get_offset();
                            offset as i32 - first_line.start as i32
                        } else {
                            log::warn!("Scroll {} lined failed: no datasource", num_of_lines);
                            0
                        }
                    } else {
                        log::warn!("Scroll {} lines not possible, cursor is at the beginning of the source", num_of_lines);
                        0
                    };
                let starting_pont = &self.scroll_position.starting_point;
                let shift = &self.scroll_position.shift + delta;
                let new_scroll_position = ScrollPosition::new(*starting_pont, shift);
                self.set_scroll_position(new_scroll_position);
            } else {
                log::warn!("Scroll {} lines failed, no first line", num_of_lines);
            }
        } else {
            log::warn!("No data, cannot move down for {}", num_of_lines);
        }
    }

    pub fn set_horizontal_scroll(&mut self, horizontal_scroll: usize) -> bool {
        log::trace!("set_horizontal_scroll {}", horizontal_scroll);
        if self.horizontal_scroll < horizontal_scroll {
            if let Some(data) = &self.data {
                let max_length = data.lines.iter()
                    .take(self.viewport_size.height)
                    .map(|line| line.content.len())
                    .max();
                log::trace!("set_horizontal_scroll max_length = {:?}", max_length);
                if let Some(max_length) = max_length {
                    if horizontal_scroll + self.viewport_size.width <= max_length {
                        log::trace!("set_horizontal_scroll success");
                        self.horizontal_scroll = horizontal_scroll;
                        self.emit_event(DataUpdated);
                        return true;
                    }
                }
            }
        } else if self.horizontal_scroll > horizontal_scroll {
            log::trace!("set_horizontal_scroll success");
            self.horizontal_scroll = horizontal_scroll;
            self.emit_event(DataUpdated);
            return true;
        }
        false
    }

    pub fn get_horizontal_scroll(&self) -> usize {
        self.horizontal_scroll
    }

    pub fn quit(&self) {
        // TODO: close datasource
        self.emit_event(Quit);
    }

    fn set_error(&mut self, err: Box<dyn ToString>) {
        self.error.replace(err);
        self.emit_event(Error(self.error.as_ref().unwrap().to_string()));
    }

    fn load_file(&mut self) {
        if let Some(path) = self.resolve_file_name() {
            self.datasource = Some(Shared::new(Box::new(LineSourceImpl::new(path))));
            self.update_viewport_content();
        }
    }

    fn resolve_file_name(&self) -> Option<PathBuf> {
        self.file_name.as_ref().map(|fname| {
            let p = Path::new(fname);
            if !p.is_absolute() {
                let mut buf = current_dir().unwrap();
                buf.push(p);
                buf
            } else {
                let mut buf = PathBuf::new();
                buf.push(p);
                buf
            }
        })
    }

    fn update_viewport_content(&mut self) -> bool {
        if self.viewport_size.height == 0 {
            return true;
        }
        let data = if let Some(datasource) = &self.datasource {
            let mut datasource = datasource.get_mut_ref();
            let source_length = datasource.get_length();
            let offset = self.scroll_position.starting_point.mul(source_length).to_integer()
                .wrapping_add(self.scroll_position.shift as u64);
            log::info!("update_viewport_content offset = {}", offset);
            datasource.set_offset(offset);
            let lines = datasource.read_lines(self.viewport_size.height as isize);
            log::trace!("update_viewport_content data: {:?}", &lines[..min(3, lines.len())]);

            // check if EOF is reached and viewport is not full
            if lines.len() < self.viewport_size.height && offset > 0 {
                None
            } else {
                Some(Data {
                    lines,
                    offset: datasource.get_offset()
                })
            }
        } else {
            panic!(String::from("Data source is not set"));
        };
        match data {
            Some(data) => {
                self.set_data(data);
                true
            },
            _ => false
        }
    }
}

impl Dimension {
    fn new(width: usize, height: usize) -> Self {
        Dimension { width, height }
    }
}

impl Default for Dimension {
    fn default() -> Self {
        Dimension::new(0, 0)
    }
}

impl ScrollPosition {
    fn new(starting_point: Ratio<u64>, shift: i32) -> Self {
        ScrollPosition {
            starting_point,
            shift,
        }
    }
}

impl Default for ScrollPosition {
    fn default() -> Self {
        ScrollPosition::new(Ratio::new(0, 1), 0)
    }
}

impl RootModelRef {
    pub fn new(model: RootModel) -> Self {
        RootModelRef(Rc::new(RefCell::new(model)))
    }

    pub fn get_mut(&self) -> RefMut<'_, RootModel> {
        let s: &RefCell<RootModel> = self.0.borrow();
        s.borrow_mut()
    }
}