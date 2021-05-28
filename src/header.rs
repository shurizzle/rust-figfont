use std::{
    io::{BufReader, Read},
    str::FromStr,
};

use crate::{error::ParseError, result::Result, utils::read_line};

const MAGIC_NUMBER: &'static str = "flf2";

#[derive(Debug)]
pub struct Header {
    hard_blank_char: char,
    height: usize,
    baseline: u64,
    max_length: u64,
    old_layout: i64,
    full_layout: u64,
    comment_lines: Option<u64>,
    print_direction: Option<PrintDirection>,
    codetag_count: Option<u64>,
}

impl Header {
    pub(crate) fn parse<R: Read>(bread: &mut BufReader<R>) -> Result<Header> {
        parse_header(bread)
    }

    pub fn hard_blank_char(&self) -> char {
        self.hard_blank_char
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn baseline(&self) -> u64 {
        self.baseline
    }

    pub fn max_length(&self) -> u64 {
        self.max_length
    }

    pub fn old_layout(&self) -> i64 {
        self.old_layout
    }

    pub fn full_layout(&self) -> u64 {
        self.full_layout
    }

    pub fn comment_lines(&self) -> u64 {
        self.comment_lines.unwrap_or(0)
    }

    pub fn print_direction(&self) -> Option<PrintDirection> {
        self.print_direction
    }

    pub fn codetag_count(&self) -> u64 {
        self.codetag_count.unwrap_or(0)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum PrintDirection {
    LeftToRight,
    RightToLeft,
}

impl FromStr for PrintDirection {
    type Err = ParseError;

    fn from_str(text: &str) -> std::result::Result<Self, <Self as FromStr>::Err> {
        match text.parse::<u8>() {
            Ok(n) => match n {
                0 => Ok(Self::LeftToRight),
                1 => Ok(Self::RightToLeft),
                _ => Err(ParseError::InvalidHeader),
            },
            Err(_) => Err(ParseError::InvalidHeader),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct HeaderBuilder {
    hard_blank_char: Option<char>,
    height: Option<usize>,
    baseline: Option<u64>,
    max_length: Option<u64>,
    old_layout: Option<i64>,
    full_layout: Option<u64>,
    comment_lines: Option<u64>,
    print_direction: Option<PrintDirection>,
    codetag_count: Option<u64>,
}

macro_rules! u {
    ($what:expr) => {
        $what.ok_or(ParseError::InvalidHeader)?
    };
}

impl HeaderBuilder {
    pub fn build(self) -> Result<Header> {
        Ok(Header {
            hard_blank_char: u!(self.hard_blank_char),
            height: u!(self.height),
            baseline: u!(self.baseline),
            max_length: u!(self.max_length),
            old_layout: u!(self.old_layout),
            full_layout: u!(self.full_layout),
            comment_lines: self.comment_lines,
            print_direction: self.print_direction,
            codetag_count: self.codetag_count,
        })
    }
}

macro_rules! parse {
    ($arg:ident) => {
        match $arg.parse() {
            Ok(res) => Some(res),
            Err(_) => {
                return Err(ParseError::InvalidHeader.into());
            }
        }
    };

    ($arg:ident, $t:ty) => {
        match $arg.parse::<$t>() {
            Ok(res) => Some(res),
            Err(_) => {
                return Err(ParseError::InvalidHeader.into());
            }
        }
    };
}

fn parse_header<R: Read>(bread: &mut BufReader<R>) -> Result<Header> {
    let header = read_line(bread)?;
    let arguments = header.split_whitespace();
    let mut builder = HeaderBuilder::default();

    for (i, arg) in arguments.enumerate() {
        match i {
            0 => {
                if arg.starts_with(MAGIC_NUMBER) {
                    builder.hard_blank_char = Some(
                        arg.chars()
                            .last()
                            .ok_or_else(|| ParseError::InvalidHeader)?,
                    );
                } else {
                    return Err(ParseError::InvalidHeader.into());
                }
            }
            1 => {
                builder.height = parse!(arg);
            }
            2 => {
                builder.baseline = parse!(arg);
            }
            3 => {
                builder.max_length = parse!(arg);
            }
            4 => {
                builder.old_layout = parse!(arg);
                builder.full_layout = Some(full_layout_from_old_layout(
                    *builder.old_layout.as_ref().unwrap(),
                ));
            }
            5 => {
                builder.comment_lines = parse!(arg);
            }
            6 => {
                builder.print_direction = parse!(arg);
            }
            7 => {
                builder.full_layout = parse!(arg);
            }
            8 => {
                builder.codetag_count = parse!(arg);
            }
            _ => {
                return Err(ParseError::InvalidHeader.into());
            }
        }
    }

    builder.build()
}

fn full_layout_from_old_layout(old_layout: i64) -> u64 {
    match old_layout {
        -1 => 0,
        0 => 1 << 6,
        l => l as u64,
    }
}
