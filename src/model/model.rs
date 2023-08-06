use crossbeam_channel::Sender;
use std::path::{Path, PathBuf};
use std::env::current_dir;
use ModelEvent::*;
use crate::data_source::{Data, Direction, FileBackend, LineSource, LineSourceImpl};
use std::cell::RefMut;
use num_rational::Ratio;
use std::cmp::min;
use std::fs::File;
use std::option::Option::Some;
use fluent_integer::Integer;
use num_traits::identities::Zero;
use crate::selection::Selection;
use crate::utils;
use crate::shared::Shared;
use crate::model::cursor_helper;
use crate::model::cursor_shift::CursorShift;
use crate::model::dimension::Dimension;
use crate::model::rendered::{DataRender, LineRender};
use crate::model::scroll_position::ScrollPosition;
use crate::model::search_model::SearchModel;
use crate::search::searcher::SearchResult;
use crate::utils::GraphemeRender;
use crate::model::search::Search;

const OFFSET_THRESHOLD: u64 = 8192;

pub struct RootModel {
    model_sender: Sender<ModelEvent>,
    file_name: Option<String>,
    file_size: Integer,
    data: Option<DataRender>,
    viewport_height: Integer,
    viewport_width: Integer,
    scroll_position: ScrollPosition,
    horizontal_scroll: Integer,
    cursor: Integer,
    selection: Option<Box<Selection>>,
    datasource: Option<Shared<Box<dyn LineSource>>>,
    error: Option<Box<dyn ToString>>,
    show_line_numbers: bool,
    search_model: Shared<SearchModel>,
    current_search: Shared<Option<Search>>,
}

#[derive(Debug)]
pub enum ModelEvent {
    FileName(String, u64),
    DataUpdated,
    CursorMoved(CursorPosition),
    SearchOpen(bool),
    Search(SearchResult),
    SearchFromCursor,
    Error(String),
    Quit,
}

#[derive(Debug)]
pub struct CursorPosition {
    pub line_no: u64,
    pub position_in_line: u64,
    pub offset: u64,
}

impl RootModel {
    pub fn new(model_sender: Sender<ModelEvent>) -> Shared<RootModel> {
        let sender = model_sender.clone();
        let root_model = RootModel {
            model_sender,
            file_name: None,
            file_size: 0.into(),
            data: None,
            viewport_height: 0.into(),
            viewport_width: 0.into(),
            scroll_position: ScrollPosition::default(),
            horizontal_scroll: 0.into(),
            cursor: 0.into(),
            selection: None,
            datasource: None,
            error: None,
            show_line_numbers: true,
            search_model: Shared::new(SearchModel::new(sender)),
            current_search: Shared::new(None),
        };

        Shared::new(root_model)
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
            self.file_name = Some(value.clone());
            self.search_model.get_mut_ref().set_file_name(value.clone());
            self.load_file();
        }
    }

    pub fn data(&self) -> Option<&DataRender> {
        self.data.as_ref()
    }

    fn set_data(&mut self, data: Data) {
        self.data = Some(DataRender::new(data));
        self.emit_event(DataUpdated);
        self.emit_cursor_moved();
    }

    pub fn error(&self) -> Option<String> {
        self.error.as_ref().map(|t| t.to_string())
    }

    pub fn set_viewport_height<I: Into<Integer>>(&mut self, height: I) {
        let height = height.into();
        if self.viewport_height != height {
            log::info!("Viewport height set to {}", height);
            self.viewport_height = height;
            // TODO: emit event
            self.update_viewport_content();
        }
    }

    pub fn get_viewport_height(&self) -> Integer {
        self.viewport_height
    }

    pub fn set_viewport_width<I: Into<Integer>>(&mut self, width: I) {
        let width = width.into();
        if self.viewport_width != width {
            log::info!("Viewport width set to {}", width);
            self.viewport_width = width;
            // TODO: emit event
        }
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
                            let h = self.viewport_height;
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
                    .take(self.viewport_height.as_usize())
                    .map(|line| line.content.len())
                    .max();
                log::trace!("set_horizontal_scroll max_length = {:?}", max_length);
                if let Some(max_length) = max_length {
                    if horizontal_scroll + self.viewport_width <= max_length {
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
            self.emit_cursor_moved();
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
        let calc_offset_in_line = |line: &LineRender| {
            let graphemes: Vec<&GraphemeRender> = line.render.iter().collect();
            let g = graphemes.get(pos.width.as_usize())
                .or(graphemes.last())
                .map(|ch| ch.render_offset)
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
                let mut new_lines = datasource.read_lines(offset, y + 1).lines;
                if y < new_lines.len() {
                    let line = LineRender::new(new_lines.remove(y.as_usize()));
                    calc_offset_in_line(&line)
                } else {
                    new_lines.pop()
                        .map(|x| LineRender::new(x))
                        .as_ref()
                        .or_else(|| data.lines.last())
                        .map(calc_offset_in_line)
                        .unwrap_or(0.into())
                }
            } else { // y < 0
                let mut new_lines = datasource.read_lines(data.start.unwrap() - 1, y).lines;
                if !new_lines.is_empty() {
                    calc_offset_in_line(&LineRender::new(new_lines.remove(0)))
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
            Direction::Forward
        } else {
            Direction::Backward
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
                let get_graphemes = || line.render.iter();
                let line_len = get_graphemes().count();
                if position_in_line < 0 {
                    position_in_line += line_len + 1;
                }
                let expected_index = position_in_line + dx;
                if expected_index >= 0 {
                    let get_grapheme = match direction {
                        Direction::Forward => get_graphemes()
                            .skip(expected_index.as_usize())
                            .skip_while(|ch| !ch.is_first_in_original)
                            .next(),
                        Direction::Backward => get_graphemes()
                            .take(expected_index.as_usize() + 1)
                            .rev()
                            .skip_while(|ch| !ch.is_first_in_original)
                            .next()
                    };
                    if let Some(grapheme) = get_grapheme {
                        break line.start + grapheme.original_offset;
                    // } else if expected_index == 0 && line_len == 0 {
                    //     break line.start;
                    } else {
                        let d = line_len - position_in_line; // # of symbols remaining to end of line
                        position_in_line = 0.into();
                        dx -= d;
                        // if line_len == 0 {
                        //     dx -= 1;
                        // }
                        best_possible_offset = line.end;
                    }
                } else {
                    dx += position_in_line;
                    // if line_len == 0 {
                    //     dx += 1;
                    // }
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
                    .and_then(|s| s.render.get(horizontal_scroll))
                    .filter(|first_visible_grapheme| p.width >= first_visible_grapheme.render_offset)
                    .map(|first_visible_grapheme| Dimension {
                        width: p.width - first_visible_grapheme.render_offset,
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

    pub fn is_show_line_numbers(&self) -> bool {
        self.show_line_numbers
    }

    pub fn set_show_line_numbers(&mut self, show_line_numbers: bool) {
        self.show_line_numbers = show_line_numbers;
        if let Some(ds) = &self.datasource {
            ds.get_mut_ref().track_line_number(show_line_numbers);
        }
    }

    pub fn move_cursor_to_end(&mut self) -> bool {
        let offset = self.get_datasource_ref()
            .map(|ds| ds.get_length());
            // .map(|len| len - bool_to_u64(len > 0, 1, 0));
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

    pub fn set_error(&mut self, err: Box<dyn ToString>) {
        let str = err.to_string();
        self.error.replace(err);
        self.emit_event(Error(str));
    }

    fn load_file(&mut self) {
        if let Some(path) = self.resolve_file_name() {
            let mut line_source = LineSourceImpl::<File, FileBackend>::from_file_name(path);
            if self.show_line_numbers {
                line_source.track_line_number(true);
            }
            let file_size = line_source.get_length();
            self.datasource = Some(Shared::new(Box::new(line_source)));
            self.emit_event(FileName(self.file_name.as_ref().unwrap().to_owned(), file_size.as_u64()));
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
        if self.viewport_height == 0 {
            return true;
        }
        if let Some(datasource) = &self.datasource {
            let mut datasource = datasource.get_mut_ref();
            let source_length = datasource.get_length();
            let offset = (self.scroll_position.starting_point * source_length).to_integer()
                + self.scroll_position.shift;
            log::info!("update_viewport_content offset = {}", offset);
            let data = datasource.read_lines(offset, self.viewport_height);
            log::trace!("update_viewport_content data: {:?}", &data.lines[..min(3, data.lines.len())]);

            drop(datasource);
            // check if EOF is reached and viewport is not full
            if data.lines.len() < self.viewport_height && offset > 0 {
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
    /// Result's `width` is a *rendered grapheme* index
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
                        let grapheme_index = line.find_grapheme_index_by_offset(raw_offset);
                        Some(Dimension::new(grapheme_index.unwrap_or(0), n - 1))
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
        let calc_horizontal_scroll = |line: &LineRender, off: Integer| {
            let h = self.horizontal_scroll;
            let w = self.viewport_width;
            let local_offset = off - line.start;
            line.find_grapheme_index_by_offset(local_offset)
                .map(Integer::from)
                .map(|index| {
                    if index < h {
                        index
                    } else if index >= h + w {
                        index - w + 1
                    } else {
                        h
                    }
                })
                .unwrap_or(h)
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
                    let mut current_offset = end_offset;
                    let success = loop {
                        if let Some(line) = datasource.read_next_line(current_offset + 1) {
                            current_offset = line.end;
                            i += 1;
                            if current_offset >= offset {
                                break true
                            }
                        } else {
                            i += 1;
                            break true
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
                .map(|line| (line.start, calc_horizontal_scroll(&LineRender::new(line), offset)))
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
        let h = self.viewport_height;
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

    pub fn get_search_model(&self) -> RefMut<SearchModel> {
        self.search_model.get_mut_ref()
    }

    pub fn get_current_search(&self) -> RefMut<Option<Search>> {
        self.current_search.get_mut_ref()
    }

    pub fn set_current_search(&mut self, search: Option<Search>) {
        let mut r = self.current_search.get_mut_ref();
        *r = search;
    }

    fn emit_cursor_moved(&self) {
        if let Some(cp) = &self.get_cursor_in_cache() {
            let i = cp.height.as_usize();
            let option = self.data.as_ref().and_then(|render| render.lines.get(i).and_then(|line| line.line_no));
            self.emit_event(CursorMoved(CursorPosition {
                line_no: option.unwrap(),
                position_in_line: cp.width.as_u64(),
                offset: self.cursor.as_u64(),
            }))
        }
    }
}