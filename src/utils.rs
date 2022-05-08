use stopwatch::Stopwatch;
use fluent_integer::Integer;

pub fn sign(n: Integer) -> (Integer, i8) {
    if n >= 0 {
        (n, 1)
    } else {
        (-n, -1)
    }
}

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

pub fn measure<R, F>(descr: &str, f: F) -> R where
        F: FnOnce() -> R {
    let sw = Stopwatch::start_new();
    let result = f();
    log::trace!("{} {:?}", descr, sw.elapsed());
    result
}

pub mod utf8 {
    use std::io::{BufReader, ErrorKind, Read, Seek};
    use unicode_segmentation::UnicodeSegmentation;

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

    pub trait GraphemeIndexLookup {
        fn offset_to_grapheme_index(&self, offset: usize) -> Result<usize, ()>;

        /// Converts grapheme `index` into real offset.
        ///
        /// # Example
        ///
        /// ```
        /// assert_eq!(3, "€€".grapheme_index_to_offset(1))
        /// ```
        ///
        /// In case index is greater than number of graphemes, returns number of graphemes as Err
        fn grapheme_index_to_offset(&self, index: usize) -> Result<usize, usize>;
    }

    impl GraphemeIndexLookup for str {
        fn offset_to_grapheme_index(&self, offset: usize) -> Result<usize, ()> {
            self.grapheme_indices(true)
                .enumerate()
                .find(|(i, (q, s))| *q <= offset && offset < *q + s.len())
                .map(|(i, _)| i)
                .ok_or(())
        }

        fn grapheme_index_to_offset(&self, index: usize) -> Result<usize, usize> {
            self.grapheme_indices(true)
                .nth(index)
                .map(|(q, _)| q)
                .ok_or(self.grapheme_indices(true).count())
        }
    }
}
