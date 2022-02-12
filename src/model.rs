use crossbeam_channel::Sender;
use std::path::{Path, PathBuf};
use std::env::current_dir;
use ModelEvent::*;
use crate::data_source::{Data, LineSource, LineSourceImpl, Line, DataSource};
use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use std::borrow::Borrow;
use num_rational::Ratio;
use std::fmt;
use std::ops::Mul;
use std::cmp::{max, min};
use std::option::Option::Some;
use crate::utils;
use crate::shared::Shared;

const OFFSET_THRESHOLD: u64 = 8192;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Dimension {
    pub width: usize,
    pub height: usize,
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Dimension(w={}, h={})", self.width, self.height)
    }
}

impl Into<(i32, i32)> for Dimension {
    fn into(self) -> (i32, i32) {
        (self.width as i32, self.height as i32)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct CursorShift {
    x: isize,
    y: isize,
}

impl CursorShift {
    pub fn new(x: isize, y: isize) -> Self {
        CursorShift{x, y}
    }

    pub fn down_by_n(n: isize) -> Self {
        Self::new(0, n)
    }

    pub fn down() -> Self {
        Self::down_by_n(1)
    }

    pub fn up_by_n(n: isize) -> Self {
        Self::new(0, -1 * n)
    }

    pub fn up() -> Self {
        Self::up_by_n(1)
    }

    pub fn left_by_n(n: isize) -> Self {
        Self::new(-1 * n, 0)
    }

    pub fn left() -> Self {
        Self::left_by_n(1)
    }

    pub fn right_by_n(n: isize) -> Self {
        Self::new(n, 0)
    }

    pub fn right() -> Self {
        Self::right_by_n(1)
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
    data: Option<Data>,
    viewport_size: Dimension,
    scroll_position: ScrollPosition,
    horizontal_scroll: usize,
    cursor: u64,
    datasource: Option<Shared<Box<dyn LineSource>>>,
    error: Option<Box<dyn ToString>>,
}

#[derive(Clone)]
pub struct RootModelRef(Rc<RefCell<RootModel>>);

pub enum ModelEvent {
    FileName(String),
    DataUpdated,
    CursorMoved(u64),
    Error(String),
    Quit,
}

impl RootModel {
    pub fn new(model_sender: Sender<ModelEvent>) -> Self {
        RootModel {
            model_sender,
            file_name: None,
            file_size: 0,
            data: None,
            viewport_size: Dimension::default(),
            scroll_position: ScrollPosition::default(),
            horizontal_scroll: 0,
            cursor: 0,
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

    pub fn get_viewport_size(&self) -> Dimension {
        self.viewport_size
    }

    fn set_scroll_position(&mut self, scroll_position: ScrollPosition) -> bool {
        if self.scroll_position != scroll_position {
            let previous_scroll_position = self.scroll_position;
            self.scroll_position = scroll_position;
            log::info!("Scroll position set to {}", scroll_position);
            if !self.update_viewport_content() {
                self.scroll_position = previous_scroll_position;
                return false;
            }
            // TODO: emit update
        }
        return true
    }

    pub fn scroll(&mut self, num_of_lines: isize) -> bool {
        if num_of_lines == 0 {
            return true;
        }
        log::trace!("scroll num_of_lines = {}", num_of_lines);
        if let Some(data) = &self.data {
            if let Some(first_line) = data.lines.first() {
                let (n, sign) = utils::sign(num_of_lines);
                let delta =
                    if sign == 1 {
                        if let Some(ds) = &self.datasource {
                            let mut ds = ds.get_mut_ref();
                            ds.set_offset(first_line.start);
                            let h = self.viewport_size.height;
                            let lines = ds.read_lines((n + h) as isize);
                            let k = lines.len();
                            if k > h {
                                (lines.get(k - h).unwrap().start - first_line.start) as i32
                            } else {
                                0
                            }
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
                return self.set_scroll_position(new_scroll_position)
            } else {
                log::warn!("Scroll {} lines failed, no first line", num_of_lines);
            }
        } else {
            log::warn!("No data, cannot move down for {}", num_of_lines);
        }
        return false
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

    fn set_cursor(&mut self, pos: u64) {
        if self.cursor != pos {
            self.cursor = pos;
            log::trace!("Cursor position set to {:?}", pos);
            self.emit_event(CursorMoved(pos));
        }
    }

    // TODO depends on encoding, assume ASCII now
    pub fn move_cursor(&mut self, delta: CursorShift) {
        log::trace!("move_cursor delta = {:?}", delta);
        let current_pos = self.get_cursor_in_cache();
        log::trace!("move_cursor pos = {} -> on_screen = {:?}", self.cursor, current_pos);

        let new_cursor_offset = if delta.x == 0 {
            self.move_cursor_vertically(delta.y, current_pos)
        } else if delta.y == 0 {
            self.move_cursor_horizontally(delta.x, current_pos)
        } else {
            todo!("Moving cursor simultaneously vertically and horizontally is not supported")
        };

        self.move_cursor_to_offset(new_cursor_offset);
    }

    pub fn move_cursor_to_offset(&mut self, pos: u64) -> bool {
        self.set_cursor(pos);
        self.bring_cursor_into_view()
    }

    fn move_cursor_vertically(&mut self, dy: isize, current_pos: Option<Dimension>) -> u64 {
        log::trace!("move_cursor_vertically current_pos = {:?}, deltaY = {}", current_pos, dy);
        if let Some(pos) = current_pos {
            let calc_offset_in_line = |line: &Line| {
                let line_offset = min(max(line.content.len(), 1) - 1, pos.width);
                line.start + line_offset as u64
            };

            let y = pos.height as isize + dy;
            let data = self.data.as_ref().unwrap();
            let n = data.lines.len();
            if y >= 0 && (y as usize) < n {
                let line = data.lines.get(y as usize).unwrap();
                calc_offset_in_line(line)
            } else {
                let mut datasource = self.datasource.as_ref().unwrap().get_mut_ref();
                if y >= 0 && (y as usize) >= n {
                    let y = y as usize - n;
                    datasource.set_offset(if n > 0 {
                        data.lines.last().unwrap().end + 1
                    } else {
                        0
                    });
                    let new_lines = datasource.read_lines((y + 1) as isize);
                    if y < new_lines.len() {
                        let line = new_lines.get(y).unwrap();
                        calc_offset_in_line(line)
                    } else {
                        new_lines.last()
                            .or_else(|| data.lines.last())
                            .map(calc_offset_in_line)
                            .unwrap_or(0)
                    }
                } else { // y < 0
                    datasource.set_offset(data.lines.first().unwrap().start);
                    let new_lines = datasource.read_lines(y);
                    if !new_lines.is_empty() {
                        calc_offset_in_line(new_lines.first().unwrap())
                    } else {
                        calc_offset_in_line(data.lines.first().unwrap())
                    }
                }
            }
        } else {
            todo!("Not implemented yet, should not be called often")
        }
    }

    fn move_cursor_horizontally(&mut self, dx: isize, current_pos: Option<Dimension>) -> u64 {
        if let Some(pos) = current_pos {
            let mut x = pos.width as isize + dx;
            let mut line_no = pos.height;
            let data = self.data.as_ref().unwrap();
            let mut datasource = self.datasource.as_ref().unwrap().get_mut_ref();
            let mut line = data.lines.get(pos.height).unwrap().clone();
            if x >= 0 {
                loop {
                    if (x as usize) < line.content.len() {
                        break line.start + x as u64
                    }
                    x -= line.content.len() as isize;
                    line_no += 1;
                    if line_no < data.lines.len() {
                        line = data.lines.get(line_no).unwrap().clone();
                    } else {
                        datasource.set_offset(line.end + 1);
                        if let Some(next_line) = datasource.read_next_line() {
                            line = next_line;
                        } else {
                            break line.end
                        }
                    }
                }
            } else {
                let mut x = -x as usize;
                let mut line_no = line_no as isize;
                loop {
                    if x == 0 {
                        break line.end
                    }
                    line_no -= 1;
                    if line_no >= 0 {
                        line = data.lines.get(line_no as usize).unwrap().clone();
                    } else {
                        datasource.set_offset(line.start);
                        if let Some(prev_line) = datasource.read_prev_line() {
                            line = prev_line;
                        } else {
                            break line.start
                        }
                    }
                    if x < line.content.len() {
                        break line.end - x as u64
                    } else {
                        x -= line.content.len() as usize;
                    }
                }
            }
        } else {
            todo!("Not implemented yet, should not be called often")
        }
    }

    pub fn get_cursor(&self) -> u64 {
        self.cursor
    }

    /// Calculates cursor position in terms of screen coordinates. Returns `None` if cursor is
    /// outside the cache. The method does not guarantee that the result is inside viewport.
    pub fn get_cursor_on_screen(&self) -> Option<Dimension> {
        self.get_cursor_in_cache()
            .filter(|p| p.width >= self.horizontal_scroll)
            .map(|p| Dimension::new(p.width - self.horizontal_scroll, p.height))
    }

    pub fn quit(&self) {
        // TODO: close datasource
        self.emit_event(Quit);
    }

    pub fn move_cursor_to_end(&mut self) -> bool {
        let offset = self
            .get_datasource_ref()
            .and_then(|mut ds| {
                let len = ds.get_length();
                if len > 0 {
                    ds.set_offset(len - 1);
                    ds.read_next_line()
                } else {
                    None
                }
            })
            .map(|line| {
                if line.end > 0 {
                    line.end - 1
                } else {
                    line.start
                }
            });
        match offset {
            Some(offset) => self.move_cursor_to_offset(offset),
            None => false
        }
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

    /// Returns cursor position in terms of current line cache
    fn get_cursor_in_cache(&self) -> Option<Dimension> {
        if let Some(data) = &self.data {
            let line_count = data.lines.len();
            let search = data.lines
                .binary_search_by(|probe| probe.start.cmp(&self.cursor));
            match search {
                Ok(n) => Some(Dimension::new(0, n)),
                Err(0) => None,
                Err(n) => {
                    let line = data.lines.get(n - 1).unwrap();
                    let x = self.cursor - line.start;
                    if n < line_count || (n == line_count && self.cursor <= line.end) {
                        Some(Dimension::new(x as usize, n - 1))
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        }
    }

    /// Makes viewport fit the cursor, adjusting vertical and horizontal scroll if necessary
    fn bring_cursor_into_view(&mut self) -> bool {
        self.bring_into_view(self.cursor)
    }

    /// Makes `offset` visible, adjusting vertical and horizontal scroll if necessary
    fn bring_into_view(&mut self, offset: u64) -> bool {
        log::trace!("bring_into_view(offset={})", offset);
        // if let Some(mut datasource) = self.get_datasource_ref() {

        let mut datasource = self.get_datasource_ref().unwrap();
            let calc_horizontal_scroll = |line: &Line, off: u64| {
                let h = self.horizontal_scroll;
                let w = self.viewport_size.width;        
                let local_offset = (off - line.start) as usize;
                if local_offset < h {
                    local_offset
                } else if local_offset >= h + w {
                    local_offset - w + 1
                } else {
                    h
                }
            };

            if let Some(data) = self.data.as_ref() {
                let start_offset = data.lines.first().map(|line| line.start).unwrap_or(0);
                let end_offset = data.lines.last().map(|line| line.end).unwrap_or(0);
                log::trace!(target: "bring_into_view", "Data present: {} -> {}", start_offset, end_offset);
                if start_offset <= offset && offset <= end_offset {
                    let search_result = data.lines.binary_search_by_key(&offset, |line| line.start);
                    let line_no = match search_result {
                        Ok(n) => n,
                        Err(0) => 0, // never happens
                        Err(n) => n - 1 // n >= 1
                    };
                    let horizontal_scroll = calc_horizontal_scroll(data.lines.get(line_no).as_ref().unwrap(), offset);
                    drop(datasource);
                    log::trace!("bring_into_view simple case. HScroll = {}", horizontal_scroll);
                    self.set_horizontal_scroll(horizontal_scroll)
                } else if offset < start_offset {
                    // TODO check 2 cases: when difference is fairly small and when it is huge
                    // currently assume it is small
                    if start_offset - offset < OFFSET_THRESHOLD {
                        datasource.set_offset(start_offset);
                        let mut i = 0;
                        let success = loop {
                            if let Some(line) = datasource.read_prev_line() {
                                i -= 1;
                                if line.start <= offset {
                                    break true
                                }
                            } else {
                                break false
                            }
                        };
                        if success {
                            drop(datasource);
                            self.scroll(i)
                        } else {
                            false
                        }
                    } else {
                        drop(datasource);
                        self.scroll_forcibly(offset) && self.bring_into_view(offset)
                    }
                } else { // offset > end_offset
                    // TODO check 2 cases: when difference is fairly small and when it is huge
                    // currently assume it is small
                    log::trace!("bring_into_view 3rd case. (offset - end) = {}", offset - end_offset);
                    if offset - end_offset < OFFSET_THRESHOLD {
                        datasource.set_offset(end_offset + 1);
                        let mut i = 0;
                        let success = loop {
                            if let Some(line) = datasource.read_next_line() {
                                i += 1;
                                if line.end >= offset {
                                    break true
                                }
                            } else {
                                break false
                            }
                        };
                        if success {
                            drop(datasource);
                            log::trace!("bring_into_view 3rd case. scroll {} lines", i);
                            self.scroll(i)
                        } else {
                            false
                        }
                    } else {
                        drop(datasource);
                        self.scroll_forcibly(offset) && self.bring_into_view(offset)
                    }
                }
            } else {
                log::trace!("bring_into_view. Raw case.");
                let line_offset = datasource.set_offset(offset);
                let horizontal_scroll = datasource.read_next_line()
                    .map(|line| calc_horizontal_scroll(&line, offset))
                    .unwrap_or(0);
                drop(datasource);
                let scroll_position = ScrollPosition::new(Ratio::new(line_offset, self.file_size), 0);
                // TODO will be implemented using futures chain
                self.scroll_position = scroll_position;
                self.horizontal_scroll = horizontal_scroll;
                self.update_viewport_content()
            }
        // } else {
        //     false
        // }
    }

    fn get_datasource_ref(&self) -> Option<RefMut<Box<dyn LineSource>>> {
        self.datasource.as_ref().map(|ds| ds.get_mut_ref())
    }

    fn scroll_forcibly(&mut self, offset: u64) -> bool {
        let mut datasource = self.get_datasource_ref().unwrap();
        let mut new_offset = datasource.set_offset(offset);
        let h = self.viewport_size.height;
        let lines_below = datasource.read_lines(h as isize);
        if lines_below.len() < h {
            let k = h - lines_below.len();
            datasource.set_offset(new_offset);
            datasource.read_lines(-(k as isize));
            new_offset = datasource.get_offset();
        }
        let scroll_starting_point = Ratio::new(new_offset, datasource.get_length());
        drop(datasource);
        self.set_scroll_position(ScrollPosition::new(scroll_starting_point, 0))
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