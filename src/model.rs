use std::alloc::handle_alloc_error;
use crossbeam_channel::Sender;
use std::path::{Path, PathBuf};
use std::env::current_dir;
use ModelEvent::*;
use crate::data_source::{Data, LineSource, LineSourceImpl, Line, FileBackend, Direction};
use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use std::borrow::Borrow;
use num_rational::Ratio;
use std::fmt;
use std::cmp::{max, min};
use std::fs::File;
use std::option::Option::Some;
use fluent_integer::Integer;
use num_traits::identities::Zero;
use crate::selection::Selection;
use crate::utils;
use crate::shared::Shared;
use unicode_segmentation::UnicodeSegmentation;
use crate::utils::utf8::GraphemeIndexLookup;

const OFFSET_THRESHOLD: u64 = 8192;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Dimension {
    pub width: Integer,
    pub height: Integer,
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Dimension(w={}, h={})", self.width, self.height)
    }
}

#[derive(Debug)]
pub enum CursorShift {
    X ( Integer ),
    Y ( Integer ),
    TokenForward,
    TokenBackward
}

impl CursorShift {

    pub fn down_by_n(n: Integer) -> Self {
        Self::Y(n)
    }

    pub fn down() -> Self {
        Self::down_by_n(1.into())
    }

    pub fn up_by_n(n: Integer) -> Self {
        Self::Y(-1 * n)
    }

    pub fn up() -> Self {
        Self::up_by_n(1.into())
    }

    pub fn left_by_n(n: Integer) -> Self {
        Self::X(-1 * n)
    }

    pub fn left() -> Self {
        Self::left_by_n(1.into())
    }

    pub fn right_by_n(n: Integer) -> Self {
        Self::X(n)
    }

    pub fn right() -> Self {
        Self::right_by_n(1.into())
    }

    pub fn token_forward() -> Self {
        Self::TokenForward
    }

    pub fn token_backward() -> Self {
        Self::TokenBackward
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
    starting_point: Ratio<Integer>,
    // [0, 1] - initial point in scroll area
    shift: Integer,
}

impl fmt::Display for ScrollPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ScrollPosition(starting_point={}/{}, shift={})", self.starting_point.numer(), self.starting_point.denom(), self.shift)
    }
}

pub struct RootModel {
    model_sender: Sender<ModelEvent>,
    file_name: Option<String>,
    file_size: Integer,
    data: Option<Data>,
    viewport_size: Dimension,
    scroll_position: ScrollPosition,
    horizontal_scroll: Integer,
    cursor: Integer,
    selection: Option<Box<Selection>>,
    datasource: Option<Shared<Box<dyn LineSource>>>,
    error: Option<Box<dyn ToString>>,
}

#[derive(Clone)]
pub struct RootModelRef(Rc<RefCell<RootModel>>);

pub enum ModelEvent {
    FileName(String),
    DataUpdated,
    CursorMoved(Integer),
    Error(String),
    Quit,
}

impl RootModel {
    pub fn new(model_sender: Sender<ModelEvent>) -> Self {
        RootModel {
            model_sender,
            file_name: None,
            file_size: 0.into(),
            data: None,
            viewport_size: Dimension::default(),
            scroll_position: ScrollPosition::default(),
            horizontal_scroll: 0.into(),
            cursor: 0.into(),
            selection: None,
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

    pub fn set_viewport_size(&mut self, width: Integer, height: Integer) {
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

    pub fn scroll(&mut self, num_of_lines: Integer) -> bool {
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
                            let h = self.viewport_size.height;
                            let lines = ds.read_lines(first_line.start,n + h).lines;
                            let k = lines.len();
                            if k > h {
                                lines.get((k - h).as_usize()).unwrap().start - first_line.start
                            } else {
                                Integer::zero()
                            }
                        } else {
                            log::warn!("Scroll {} lines failed, no matching line", num_of_lines);
                            Integer::zero()
                        }
                    } else if first_line.start > 0 {
                        if let Some(ds) = &self.datasource {
                            let mut ds = ds.get_mut_ref();
                            log::trace!("scroll({}). set_offset = {}", num_of_lines, first_line.start);
                            // read n lines backward from one symbol before beginning of first_line
                            let offset= ds.read_lines(first_line.start - 1, num_of_lines).start.unwrap();
                            offset - first_line.start
                        } else {
                            log::warn!("Scroll {} lined failed: no datasource", num_of_lines);
                            Integer::zero()
                        }
                    } else {
                        log::warn!("Scroll {} lines not possible, cursor is at the beginning of the source", num_of_lines);
                        Integer::zero()
                    };
                let starting_pont = self.scroll_position.starting_point;
                let shift = self.scroll_position.shift + delta;
                let new_scroll_position = ScrollPosition::new(starting_pont, shift);
                return self.set_scroll_position(new_scroll_position)
            } else {
                log::warn!("Scroll {} lines failed, no first line", num_of_lines);
            }
        } else {
            log::warn!("No data, cannot move down for {}", num_of_lines);
        }
        return false
    }

    pub fn set_horizontal_scroll(&mut self, horizontal_scroll: Integer) -> bool {
        log::trace!("set_horizontal_scroll {}", horizontal_scroll);
        if self.horizontal_scroll < horizontal_scroll {
            if let Some(data) = &self.data {
                let max_length = data.lines.iter()
                    .take(self.viewport_size.height.as_usize())
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

    pub fn get_horizontal_scroll(&self) -> Integer {
        self.horizontal_scroll
    }

    fn set_cursor(&mut self, pos: Integer) {
        if self.cursor != pos {
            self.cursor = pos;
            log::trace!("Cursor position set to {:?}", pos);
            self.emit_event(CursorMoved(pos));
        }
    }

    pub fn move_cursor(&mut self, delta: CursorShift, adjust_selection: bool) {
        log::trace!("move_cursor: delta = {:?}", delta);
        let current_pos = self.get_cursor_in_cache(); // TODO
        log::trace!("move_cursor: pos = {} -> on_screen = {:?}", self.cursor, current_pos);

        if let Some(current_pos) = current_pos {
            let new_cursor_offset = match delta {
                CursorShift::X(x) => self.move_cursor_horizontally(x, current_pos),
                CursorShift::Y(y) => self.move_cursor_vertically(y, current_pos),
                CursorShift::TokenForward => self.get_datasource_ref()
                    .map(|mut ds| ds.skip_token(self.cursor, Direction::Forward))
                    .unwrap()
                    .unwrap_or(self.cursor),
                CursorShift::TokenBackward => self.get_datasource_ref()
                    .map(|mut ds| ds.skip_token(self.cursor, Direction::Backward))
                    .unwrap()
                    .unwrap_or(self.cursor),
            };

            self.move_cursor_to_offset(new_cursor_offset, adjust_selection);
        } else {
            log::error!("move_cursor: Failed to evaluate cursor position in cache");
        }
    }

    pub fn move_cursor_to_offset(&mut self, pos: Integer, adjust_selection: bool) -> bool {
        if let Some(selection) = self.get_selection() {
            if adjust_selection {
                if self.cursor == selection.end {
                    self.set_selection(Selection::create(selection.start, pos));
                } else if self.cursor == selection.start {
                    self.set_selection(Selection::create(pos, selection.end));
                } else {
                    log::warn!("Inconsistent situation cursor is neither in the beginning nor in the end of selection. Cursor {}, selection: {:?}", self.cursor, selection);
                }
            } else {
                self.reset_selection();
            }
        } else if adjust_selection {
            self.set_selection(Selection::create(self.cursor, pos));
        }
        self.set_cursor(pos);
        self.bring_cursor_into_view()
    }

    fn move_cursor_vertically(&mut self, dy: Integer, pos: Dimension) -> Integer {
        log::trace!("move_cursor_vertically current_pos = {:?}, deltaY = {}", pos, dy);
        let calc_offset_in_line = |line: &Line| {
            let graphemes: Vec<(usize, &str)> = line.content.grapheme_indices(true).collect();
            let g = graphemes.get(pos.width.as_usize())
                .or(graphemes.last())
                .map(|(q, _)| *q)
                .unwrap_or(0);
            line.start + g
        };

        let y = pos.height + dy;
        let data = self.data.as_ref().unwrap();
        let n = data.lines.len();
        if y >= 0 && y < n {
            let line = data.lines.get(y.as_usize()).unwrap();
            calc_offset_in_line(line)
        } else {
            let mut datasource = self.datasource.as_ref().unwrap().get_mut_ref();
            if y >= n {
                let y = y - n;
                let offset = data.end.map_or(0.into(), |x| x + 1);
                let new_lines = datasource.read_lines(offset, y + 1).lines;
                if y < new_lines.len() {
                    let line = new_lines.get(y.as_usize()).unwrap();
                    calc_offset_in_line(line)
                } else {
                    new_lines.last()
                        .or_else(|| data.lines.last())
                        .map(calc_offset_in_line)
                        .unwrap_or(0.into())
                }
            } else { // y < 0
                let new_lines = datasource.read_lines(data.start.unwrap() - 1, y).lines;
                if !new_lines.is_empty() {
                    calc_offset_in_line(new_lines.first().unwrap())
                } else {
                    calc_offset_in_line(data.lines.first().unwrap())
                }
            }
        }
    }

    /// Calculates new cursor offset and assigns it to model. `dx` denotes number of *graphemes*
    /// to move over. `pos` denotes cursor position in cache.
    fn move_cursor_horizontally(&mut self, mut dx: Integer, pos: Dimension) -> Integer {
        let direction = if dx >= 0 {
            cursor_helper::Direction::Forward
        } else {
            cursor_helper::Direction::Backward
        };

        let mut line_iterator = cursor_helper::LineIterator::new(
            self.data.as_ref().unwrap(),
            self.datasource.as_ref().unwrap().get_mut_ref(),
            direction,
            pos.height
        );

        let mut position_in_line = pos.width; // negative numbers are counted from end of line
        let mut best_possible_offset = self.cursor;
        loop {
            if let Some(line) = line_iterator.next() {
                let graphemes = line.content.as_str().grapheme_indices(true)
                    .map(|(q, _)| q)
                    .collect::<Vec<usize>>();
                if position_in_line < 0 {
                    position_in_line += graphemes.len() + 1;
                }
                let expected_index = position_in_line + dx;
                if expected_index >= 0 {
                    if let Some(grapheme) = graphemes.get(expected_index.as_usize()) {
                        break line.start + *grapheme;
                    } else {
                        let d = graphemes.len() - position_in_line; // # of symbols remaining to end of line
                        position_in_line = 0.into();
                        dx -= d;
                        best_possible_offset = line.end;
                    }
                } else {
                    dx += position_in_line;
                    position_in_line = Integer::from(-1);
                    best_possible_offset = line.start;
                }
            } else {
                break best_possible_offset;
            }
        }
    }

    pub fn get_cursor(&self) -> Integer {
        self.cursor
    }

    /// Calculates cursor position in terms of screen coordinates. Returns `None` if cursor is
    /// outside the cache. The method does not guarantee that the result is inside viewport.
    ///
    /// Result's `height` is number of line.
    ///
    /// Result's `width` is given in terms of *graphemes*.
    pub fn get_cursor_on_screen(&self) -> Option<Dimension> {
        let horizontal_scroll = self.horizontal_scroll.as_usize();
        let result = self.get_cursor_in_cache().zip(self.data.as_ref())
            .and_then(|(p, data)|
                data.lines.get(p.height.as_usize())
                    .map(|line: &Line| line.content.as_str())
                    .and_then(|s| s.grapheme_indices(true).skip(horizontal_scroll).next())
                    .map(|(first_visible_grapheme, _)| first_visible_grapheme)
                    .filter(|first_visible_grapheme| p.width >= *first_visible_grapheme)
                    .map(|first_visible_grapheme| Dimension {
                        width: p.width - first_visible_grapheme,
                        height: p.height
                    })
            );
        log::trace!("get_cursor_on_screen(horizontal_scroll = {}) -> {:?}", horizontal_scroll, result);
        result
    }

    pub fn quit(&self) {
        // TODO: close datasource
        self.emit_event(Quit);
    }

    pub fn move_cursor_to_end(&mut self) -> bool {
        let offset = self.get_datasource_ref()
            .map(|ds| ds.get_length());
        match offset {
            Some(offset) => self.move_cursor_to_offset(offset, false),
            None => false
        }
    }

    pub fn get_selection(&self) -> Option<Selection> {
        self.selection.as_ref().map(|b| *b.clone())
    }

    fn set_selection(&mut self, selection: Option<Box<Selection>>) {
        self.selection = selection;
        // TODO: emit event
    }

    pub fn select_all(&mut self) {
        let length = self.get_datasource_ref().map(|ds| ds.get_length());
        if let Some(length) = length {
            self.set_selection(Some(Box::new(Selection {
                start: Integer::zero(),
                end: length
            })));
            self.move_cursor_to_offset(Integer::zero(), true); // TODO: use set_cursor and do not scroll, when cursor out of viewport is supported
        }
    }

    pub fn get_selected_content(&self) -> Option<String> {
        self.get_selection().and_then(|selection| {
            self.get_datasource_ref().and_then(|datasource| {
                let result = datasource.read_raw(selection.start, selection.end);
                match result {
                    Ok(s) => Some(s),
                    Err(_) => None
                }
            })
        })
    }

    fn reset_selection(&mut self) {
        self.set_selection(None);
    }

    fn set_error(&mut self, err: Box<dyn ToString>) {
        self.error.replace(err);
        self.emit_event(Error(self.error.as_ref().unwrap().to_string()));
    }

    fn load_file(&mut self) {
        if let Some(path) = self.resolve_file_name() {
            let line_source = LineSourceImpl::<File, FileBackend>::from_file_name(path);
            self.datasource = Some(Shared::new(Box::new(line_source)));
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
        if let Some(datasource) = &self.datasource {
            let mut datasource = datasource.get_mut_ref();
            let source_length = datasource.get_length();
            let offset = (self.scroll_position.starting_point * source_length).to_integer()
                + self.scroll_position.shift;
            log::info!("update_viewport_content offset = {}", offset);
            let data = datasource.read_lines(offset, self.viewport_size.height);
            log::trace!("update_viewport_content data: {:?}", &data.lines[..min(3, data.lines.len())]);

            drop(datasource);
            // check if EOF is reached and viewport is not full
            if data.lines.len() < self.viewport_size.height && offset > 0 {
                false
            } else {
                self.set_data(data);
                true
            }
        } else {
            panic!("Data source is not set");
        }
    }

    /// Returns cursor position in terms of current line cache
    ///
    /// Result's `height` is a line number.
    ///
    /// Result's `width` is a *grapheme* index
    fn get_cursor_in_cache(&self) -> Option<Dimension> {
        let result = if let Some(data) = &self.data {
            let line_count = data.lines.len();
            let search = data.lines
                .binary_search_by(|probe| probe.start.cmp(&self.cursor));
            match search {
                Ok(n) => Some(Dimension::new(0, n)),
                Err(0) => None,
                Err(n) => {
                    let line = data.lines.get(n - 1).unwrap();
                    if n < line_count || (n == line_count && self.cursor <= line.end) {
                        let raw_offset = self.cursor - line.start;
                        let grapheme_index = line.content.as_str().offset_to_grapheme_index(raw_offset.as_usize());
                        let p = match grapheme_index {
                            Ok(g) => g,
                            Err(0) => 0,
                            Err(g) => g - 1,
                        };
                        Some(Dimension::new(p, n - 1))
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        };
        log::trace!("get_cursor_in_cache for offset {} returned {:?}", self.cursor, result);
        result
    }

    /// Makes viewport fit the cursor, adjusting vertical and horizontal scroll if necessary
    fn bring_cursor_into_view(&mut self) -> bool {
        self.bring_into_view(self.cursor)
    }

    /// Makes `offset` visible, adjusting vertical and horizontal scroll if necessary
    fn bring_into_view(&mut self, offset: Integer) -> bool {
        log::trace!("bring_into_view(offset={})", offset);
        // if let Some(mut datasource) = self.get_datasource_ref() {

        let mut datasource = self.get_datasource_ref().unwrap();
        let calc_horizontal_scroll = |line: &Line, off: Integer| {
            let h = self.horizontal_scroll;
            let w = self.viewport_size.width;
            let local_offset = off - line.start;
            if local_offset < h {
                local_offset
            } else if local_offset >= h + w {
                local_offset - w + 1
            } else {
                h
            }
        };

        if let Some(data) = self.data.as_ref() {
            let start_offset = data.lines.first().map(|line| line.start).unwrap_or(0.into());
            let end_offset = data.lines.last().map(|line| line.end).unwrap_or(0.into());
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
                    let mut i = 0_u8.into();
                    let mut current_offset = start_offset;
                    let success = loop {
                        if let Some(line) = datasource.read_prev_line(current_offset - 1) {
                            current_offset = line.start;
                            i -= 1;
                            if current_offset <= offset {
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
            } else {
                // offset > end_offset
                // TODO check 2 cases: when difference is fairly small and when it is huge
                // currently assume it is small
                log::trace!("bring_into_view 3rd case. (offset - end) = {}", offset - end_offset);
                if offset - end_offset < OFFSET_THRESHOLD {
                    let mut i = 0_u8.into();
                    let success = loop {
                        if let Some(line) = datasource.read_next_line(end_offset + 1) {
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
            let (line_offset, horizontal_scroll) = datasource.read_next_line(offset)
                .map(|line| (line.start, calc_horizontal_scroll(&line, offset)))
                .unwrap_or((Integer::zero(), Integer::zero()));
            drop(datasource);
            let scroll_position = ScrollPosition::new(Ratio::new(line_offset, self.file_size), Integer::zero());
            // TODO will be implemented using futures chain
            self.scroll_position = scroll_position;
            self.horizontal_scroll = horizontal_scroll;
            self.update_viewport_content()
        }
    }

    fn get_datasource_ref(&self) -> Option<RefMut<Box<dyn LineSource>>> {
        self.datasource.as_ref().map(|ds| ds.get_mut_ref())
    }

    fn scroll_forcibly(&mut self, offset: Integer) -> bool {
        let mut datasource = self.get_datasource_ref().unwrap();
        let h = self.viewport_size.height;
        let lines_below = datasource.read_lines(offset, h);
        let mut new_offset = lines_below.start.unwrap_or(offset);
        if lines_below.lines.len() < h {
            let k = h - lines_below.lines.len();
            let prev_lines = datasource.read_lines(new_offset, -k);
            if let Some(offset) = prev_lines.start {
                new_offset = offset;
            }
        }
        let scroll_starting_point = Ratio::new(new_offset, datasource.get_length());
        drop(datasource);
        self.set_scroll_position(ScrollPosition::new(scroll_starting_point, 0.into()))
    }
}

impl Dimension {
    fn new<I: Into<Integer>>(width: I, height: I) -> Self {
        Dimension {
            width: width.into(),
            height: height.into()
        }
    }
}

impl Default for Dimension {
    fn default() -> Self {
        Dimension::new(0, 0)
    }
}

impl ScrollPosition {
    fn new(starting_point: Ratio<Integer>, shift: Integer) -> Self {
        ScrollPosition {
            starting_point,
            shift,
        }
    }
}

impl Default for ScrollPosition {
    fn default() -> Self {
        ScrollPosition::new(Ratio::zero(), 0.into())
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

mod cursor_helper {
    use std::borrow::Cow;
    use std::cell::RefMut;
    use fluent_integer::Integer;
    use crate::data_source::{Data, Line, LineSource};

    pub enum Direction {
        Forward,
        Backward
    }

    pub struct LineIterator<'a> {
        cache: &'a Data,
        datasource: RefMut<'a, Box<dyn LineSource>>,
        direction: Direction,
        current_line: Option<Cow<'a, Line>>,
        line_number: Integer,
        started: bool,
        exhausted: bool,
    }

    impl <'a> LineIterator<'a> {
        pub fn new(
            cache: &'a Data,
            datasource: RefMut<'a, Box<dyn LineSource>>,
            direction: Direction,
            line_number: Integer
        ) -> Self {
            LineIterator {
                cache,
                datasource,
                direction,
                current_line: None,
                line_number,
                started: false,
                exhausted: false
            }
        }

        fn read_non_cached_line_backward(&mut self) -> Option<Cow<'a, Line>> {
            let datasource = &mut *self.datasource;
            self.current_line.as_ref()
                .map(|current_line| current_line.start - 1)
                .and_then(|s| datasource.read_prev_line(s))
                .map(Cow::Owned)
        }

        fn read_non_cached_line_forward(&mut self) -> Option<Cow<'a, Line>> {
            let datasource = &mut *self.datasource;
            self.current_line.as_ref()
                .map(|current_line| current_line.end + 1)
                .and_then(|s| datasource.read_next_line(s))
                .map(Cow::Owned)
        }
    }

    impl <'a> Iterator for LineIterator<'a> {
        type Item = Cow<'a, Line>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.exhausted {
                return None;
            }

            if !self.started {
                self.started = true;
                let next_line = self.cache.lines.get(self.line_number.as_usize());
                return match next_line {
                    Some(line_ref) => {
                        self.current_line = Some(Cow::Borrowed(line_ref));
                        Some(Cow::Borrowed(line_ref))
                    },
                    None => {
                        self.exhausted = true;
                        None
                    }
                }
            }

            let next_line = match self.direction {
                Direction::Forward => {
                    let next_line_number: Integer = self.line_number + 1;
                    if self.line_number >= 0 {
                        if let Some(line) = self.cache.lines.get(next_line_number.as_usize()) {
                            Some(Cow::Borrowed(line))
                        } else {
                            self.read_non_cached_line_forward()
                        }
                    } else {
                        panic!("Impossible situation");
                    }
                },
                Direction::Backward => {
                    let next_line_number: Integer = self.line_number - 1;
                    if next_line_number >= 0 {
                        if let Some(line) = self.cache.lines.get(next_line_number.as_usize()) {
                            Some(Cow::Borrowed(line))
                        } else {
                            self.read_non_cached_line_backward()
                        }
                    } else {
                        self.read_non_cached_line_backward()
                    }
                }
            };
            if next_line.is_none() {
                self.exhausted = true;
            }
            self.current_line = next_line;
            self.line_number += match self.direction {
                Direction::Forward => 1,
                Direction::Backward => -1,
            };
            self.current_line.clone()
        }
    }
}