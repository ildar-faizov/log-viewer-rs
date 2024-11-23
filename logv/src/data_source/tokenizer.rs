use std::io::{BufReader, Read, Seek};
use anyhow::anyhow;
use fluent_integer::Integer;
use crate::data_source::char_navigation::{next_char, prev_char};
use crate::data_source::Direction;
use crate::utils::utf8::UtfChar;

pub fn skip_token<R: Read + Seek>(offset: Integer, direction: Direction, f: &mut BufReader<R>) -> anyhow::Result<Integer> {
    let actual_offset: Integer = f.stream_position()?.into();
    f.seek_relative((offset - actual_offset).as_i64())?;

    let take_char0 = match direction {
        Direction::Forward => next_char,
        Direction:: Backward => prev_char,
    };
    let take_char = |reader: &mut BufReader<R>| -> anyhow::Result<Option<UtfChar>> {
        take_char0(reader).map_err(|e| anyhow!(e))
    };

    if direction == Direction::Backward {
        next_char(f)?;
    }

    if let Some(pattern) = take_char(f)? {
        let mut state = if !is_delimiter(&pattern.get_char()) {
            State::DetermineIfTokenBoundary
        } else {
            State::InWhitespace
        };
        let mut prev_char_offset = pattern.get_offset();
        while let Some(ch) = take_char(f)? {
            match state {
                State::DetermineIfTokenBoundary => {
                    if !is_delimiter(&ch.get_char()) {
                        state = State::InToken;
                    } else {
                        state = State::InWhitespace;
                    }
                },
                State::InWhitespace => if !is_delimiter(&ch.get_char()) {
                    prev_char_offset = ch.get_offset();
                    break;
                },
                State::InToken => if is_delimiter(&ch.get_char()) {
                    break;
                }
            };
            prev_char_offset = ch.get_offset();
        }
        Ok(prev_char_offset.into())
    } else {
        Ok(offset)
    }
}

enum State {
    InToken,
    InWhitespace,
    DetermineIfTokenBoundary
}

fn is_delimiter(ch: &char) -> bool {
    !ch.is_alphanumeric() && *ch != '_' // TODO: better UTF-8 delimiter detection
}