use std::borrow::Cow;
use std::fmt::Debug;
use std::time::Duration;
use cursive::utils::span::IndexedCow;
use log::Level;
use metrics::{histogram, Unit};
use stopwatch::Stopwatch;
use unicode_segmentation::UnicodeSegmentation;
use fluent_integer::Integer;
use paste::paste;

pub fn sign(n: Integer) -> (Integer, i8) {
    if n >= 0 {
        (n, 1)
    } else {
        (-n, -1)
    }
}

macro_rules! bool_to_num {
    ($($t: ty)*) => {
        $(
            paste! {
                #[allow(dead_code)]
                pub fn [<bool_to_ $t>](value: bool) -> $t {
                    if value {
                        1 as $t
                    } else {
                        0 as $t
                    }
                }
            }
        )*
    };
}
bool_to_num!(u8 u16 u32 u64 usize i8 i16 i32 i64 i128);

pub fn trim_newline(s: &mut String) -> usize {
    let mut bytes_removed = 0;
    if s.ends_with('\n') {
        s.pop();
        bytes_removed += 1;
    }
    if s.ends_with('\r') {
        s.pop();
        bytes_removed += 1;
    }
    bytes_removed
}

/// Intervals of type [a, b)
pub fn disjoint_intervals<T>(intervals: &Vec<(Integer, Integer, T)>) -> Vec<(Integer, Integer, Vec<T>)>
where T: Copy {
    let mut bounds = vec![];
    for interval in intervals {
        bounds.push(interval.0);
        bounds.push(interval.1);
    }
    bounds.sort();
    bounds.dedup();
    let mut result = vec![];
    let mut s = None;
    for bound in bounds {
        if let Some(s) = s {
            let mut ids = vec![];
            for interval in intervals {
                if interval.0 < bound && interval.1 > s {
                    ids.push(interval.2);
                }
            }
            if !ids.is_empty() {
                result.push((s, bound, ids));
            }
        }
        s = Some(bound);
    }
    result
}

#[allow(dead_code)]
pub fn measure<R, F>(descr: &str, f: F) -> R where
        F: FnOnce() -> R {
    measure_l(Level::Trace, descr, f)
}

pub fn measure_l<R, F>(level: Level, descr: &str, f: F) -> R
    where
        F: FnOnce() -> R
{
    let (result, duration) = measuring_func(f)();
    log::log!(level, "{} {:?}", descr, duration);
    result
}

pub fn stat<R, F>(descr: &'static str, unit: &Unit, f: F) -> R
    where F: FnOnce() -> R {
    stat_l(Level::Trace, descr, unit, f)
}

pub fn stat_l<R, F>(level: Level, descr: &'static str, unit: &Unit, f: F) -> R
    where F: FnOnce() -> R {
    let (result, duration) = measuring_func(f)();
    log::log!(level, "{} {:?}", descr, duration);
    histogram!(descr).record(duration.to_unit(unit));
    result
}

fn measuring_func<'a, R, F>(f: F) -> Box<dyn FnOnce() -> (R, Duration) + 'a>
where F: FnOnce() -> R + 'a {
    Box::new(|| {
        let sw = Stopwatch::start_new();
        let result = f();
        (result, sw.elapsed())
    })
}

pub trait ToUnit {
    fn to_unit(&self, unit: &Unit) -> f64;
}

impl ToUnit for Duration {
    fn to_unit(&self, unit: &Unit) -> f64 {
        match unit {
            Unit::Seconds => self.as_secs_f64(),
            Unit::Milliseconds => self.as_millis() as f64,
            Unit::Microseconds => self.as_micros() as f64,
            Unit::Nanoseconds => self.as_nanos() as f64,
            _ => panic!("")
        }
    }
}

pub trait NumberOfDecimalDigits {
    fn number_of_decimal_digits(&self) -> usize;
}

macro_rules! number_of_decimal_digits_impl_unsigned {
    ( $($t: ty)* ) => {
        $(
            impl NumberOfDecimalDigits for $t {
                #[inline]
                fn number_of_decimal_digits(&self) -> usize {
                    (self.checked_ilog10().unwrap_or_default() + 1) as usize
                }
            }
        )*
    };
}

number_of_decimal_digits_impl_unsigned!(u64 u32 u16 u8);

pub mod utf8 {
    use std::io::{BufReader, ErrorKind, Read, Seek};

    pub enum UnicodeByteType {
        Single,
        Continuation,
        FirstOf2,
        FirstOf3,
        FirstOf4
    }

    pub fn utf_byte_type(b: u8) -> Result<UnicodeByteType, ()> {
        if b >> 7 == 0 {
            Ok(UnicodeByteType::Single)
        } else if b >> 6 == 0b10 {
            Ok(UnicodeByteType::Continuation)
        } else if b >> 5 == 0b110 {
            Ok(UnicodeByteType::FirstOf2)
        } else if b >> 4 == 0b1110 {
            Ok(UnicodeByteType::FirstOf3)
        } else if b >> 3 == 0b11110 {
            Ok(UnicodeByteType::FirstOf4)
        } else {
            Err(())
        }
    }

    pub struct UtfChar {
        ch: char,
        offset: u64,
    }

    impl UtfChar {
        fn from_u8(b: u8, offset: u64) -> Self {
            UtfChar {
                ch: char::from(b),
                offset,
            }
        }

        fn from_little_endian(bytes: [u8; 4], offset: u64) -> std::io::Result<Self> {
            let s = std::str::from_utf8(&bytes)
                .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?;
            match s.chars().next() {
                Some(ch) => Ok(UtfChar {
                    ch, offset
                }),
                None => panic!("Invalid UTF-8 sequence at offset {}: {:?}", offset, bytes) // TODO proper Err
            }
        }

        pub fn get_char(&self) -> char {
            self.ch
        }

        pub fn get_offset(&self) -> u64 {
            self.offset
        }

        pub fn get_end(&self) -> u64 {
            self.offset + (self.ch.len_utf8() as u64)
        }
    }

    // TODO: big endian support
    pub fn read_utf_char<R>(reader: &mut BufReader<R>) -> std::io::Result<Option<UtfChar>>
        where R: Read + Seek
    {
        let offset = reader.stream_position()?;
        read_utf_char_internal(reader, offset)
    }

    // TODO: big endian support
    fn read_utf_char_internal<R>(reader: &mut BufReader<R>, offset: u64) -> std::io::Result<Option<UtfChar>>
    where R: Read + Seek
    {
        let mut buf = [0_u8; 1];
        let bytes_read = reader.read(&mut buf)?;
        if bytes_read == 1 {
            let b = buf[0];
            match utf_byte_type(b) {
                Ok(UnicodeByteType::Single) => Ok(Some(UtfChar::from_u8(b, offset))),
                Ok(UnicodeByteType::FirstOf2) => match reader.read(&mut buf)? {
                    0 => panic!("Invalid UTF-8 sequence at {}", offset), // TODO: Err()
                    1 => Ok(Some(UtfChar::from_little_endian([b, buf[0], 0, 0], offset)?)),
                    _ => panic!("Impossible") // TODO err
                },
                Ok(UnicodeByteType::FirstOf3) => {
                    let mut buf2 = [0_u8; 2];
                    match reader.read(&mut buf2)? {
                        2 => Ok(Some(UtfChar::from_little_endian([b, buf2[0], buf2[1], 0], offset)?)),
                        _ => panic!("Invalid UTF-8 sequence at {}", offset), // TODO: Err()
                    }
                },
                Ok(UnicodeByteType::FirstOf4) => {
                    let mut buf2 = [0_u8; 3];
                    match reader.read(&mut buf2)? {
                        3 => Ok(Some(UtfChar::from_little_endian([b, buf2[0], buf2[1], buf2[2]], offset)?)),
                        _ => panic!("Invalid UTF-8 sequence at {}", offset), // TODO: Err()
                    }
                },
                Ok(UnicodeByteType::Continuation) => {
                    if offset == 0 {
                        panic!("") // todo proper Err
                    }
                    reader.seek_relative(-2)?;
                    read_utf_char_internal(reader, offset - 1)
                },
                Err(_) => panic!("Failed to recognize byte type at offset {}, value: {}", offset, b) // TODO proper Err
            }
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GraphemeRender {
    pub original_offset: usize,
    pub render_offset: usize,
    pub render: IndexedCow,
    pub is_first_in_original: bool,
}

impl GraphemeRender {

    pub fn from_string(string: &String) -> Vec<GraphemeRender> {
        let mut result = vec![];
        let mut render_offset = 0_usize;
        for (original_offset, original_grapheme) in string.grapheme_indices(true) {
            let render = GraphemeRender::render(original_grapheme);
            match render {
                Cow::Borrowed(_) => {
                    let render_len = render.len();
                    result.push(GraphemeRender {
                        original_offset,
                        render_offset,
                        render: IndexedCow::Borrowed {
                            start: original_offset,
                            end: original_offset + render_len
                        },
                        is_first_in_original: true,
                    });
                    render_offset += render_len;
                },
                Cow::Owned(_) => {
                    for (i, g) in render.to_string().graphemes(true).enumerate() {
                        result.push(GraphemeRender {
                            original_offset,
                            render_offset,
                            render: IndexedCow::Owned(g.to_string()),
                            is_first_in_original: i == 0,
                        });
                        render_offset += g.len();
                    }
                }
            }
        }
        result
    }

    fn render(grapheme: &str) -> Cow<str> {
        if grapheme.eq("\t") {
            Cow::Owned(String::from("    "))
        } else if grapheme.eq("\u{FEFF}") {
            Cow::Owned(String::with_capacity(0))
        } else {
            Cow::Borrowed(grapheme)
        }
    }
}

pub mod event_emitter {
    use std::fmt::Debug;
    use crossbeam_channel::Sender;

    pub trait EventEmitter<T: Debug> {
        /// Convenient method for crossbeam_channel::Sender::send
        fn emit_event(&self, evt: T);
    }

    impl<T: Debug> EventEmitter<T> for Sender<T> {
        fn emit_event(&self, evt: T) {
            let msg = format!("Failed to send event: {:?}", evt);
            self.send(evt)
                .unwrap_or_else(|_| { panic!("{}", msg) });
        }
    }
}

#[cfg(not(test))]
pub fn print_debug<D, F>(_: F)
    where
    D: Debug,
    F: FnOnce() -> D
{
    // no op
}

#[cfg(test)]
pub fn print_debug<D, F>(f: F)
where
    D: Debug,
    F: FnOnce() -> D
{
    println!("{:?}", f());
}

#[macro_export]
macro_rules! tout {
    ($fmt:expr, $($args:tt)*) => {
        crate::utils::print_debug(|| format!($fmt, $($args)*));
    }
}
