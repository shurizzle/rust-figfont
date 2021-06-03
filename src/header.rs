use std::{
    borrow::Cow,
    io::{BufRead, BufReader, Read},
    str::{from_utf8, FromStr},
};

use crate::{
    error::{Error, ParseError},
    result::Result,
    utils::read_line,
};

use bitflags::bitflags;

const MAGIC_NUMBER: &'static [u8] = b"flf2a";

bitflags! {
    /// The FIGfont's layout informations.
    pub struct Layout: u32 {
        /// Equals smushing.
        const HORIZONTAL_EQUAL = 1;
        /// Underscore smushing.
        const HORIZONTAL_LOWLINE = 2;
        /// Hierarchy smushing.
        const HORIZONTAL_HIERARCHY = 4;
        /// Pair brackets smushing.
        const HORIZONTAL_PAIR = 8;
        /// Big X smushing.
        const HORIZONTAL_BIGX = 16;
        /// Hard blank smushing.
        const HORIZONTAL_HARDBLANK = 32;
        /// Apply kerning.
        const HORIZONTAL_KERNING = 64;
        /// Apply smushing.
        const HORIZONTAL_SMUSH = 128;

        const VERTICAL_EQUAL = 256;
        const VERTICAL_LOWLINE = 512;
        const VERTICAL_HIERARCHY = 1024;
        const VERTICAL_PAIR = 2048;
        const VERTICAL_BIGX = 4096;
        const VERTICAL_KERNING = 8192;
        const VERTICAL_SMUSH = 16392;
    }
}

impl FromStr for Layout {
    type Err = Error;

    fn from_str(raw: &str) -> std::result::Result<Self, <Self as FromStr>::Err> {
        let raw: u32 = raw
            .parse()
            .ok()
            .ok_or::<Error>(ParseError::InvalidHeader.into())?;
        Layout::from_bits(raw).ok_or(ParseError::InvalidHeader.into())
    }
}

/// FIGfont's header.
#[derive(Debug)]
pub struct Header {
    hard_blank_char: Vec<u8>,
    height: usize,
    baseline: usize,
    max_length: usize,
    layout: Layout,
    comment: String,
    print_direction: PrintDirection,
    codetag_count: Option<u32>,
}

impl Header {
    pub(crate) fn parse<R: Read>(bread: &mut BufReader<R>) -> Result<Header> {
        parse_header(bread)
    }

    /// Get the hard blank character.
    pub fn hard_blank_char<'a>(&'a self) -> &'a [u8] {
        &self.hard_blank_char[..]
    }

    /// Get the font's height (lines).
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get the font's basline. Unused.
    pub fn baseline(&self) -> usize {
        self.baseline
    }

    /// Get the font's max length. Unused.
    pub fn max_length(&self) -> usize {
        self.max_length
    }

    /// Get the font's layout.
    pub fn layout(&self) -> Layout {
        self.layout
    }

    /// Get the font's comment.
    pub fn comment<'a>(&'a self) -> Cow<'a, str> {
        Cow::Borrowed(&self.comment)
    }

    /// Get the print direction.
    pub fn print_direction(&self) -> PrintDirection {
        self.print_direction
    }

    /// Get the number of codetagged characters.
    pub fn codetag_count(&self) -> Option<u32> {
        self.codetag_count
    }
}

/// Print direction enum.
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

fn read_string_lines<R: Read>(bread: &mut BufReader<R>, num: usize) -> Result<String> {
    let mut lines = String::new();

    for _ in 0..num {
        bread.read_line(&mut lines)?;
    }

    if lines.ends_with("\r\n") {
        lines.truncate(lines.len() - 2);
    } else if lines.ends_with("\n") {
        lines.truncate(lines.len() - 1);
    } else {
        return Err(ParseError::NotEnoughData.into());
    }

    Ok(lines)
}

macro_rules! parse {
    ($arg:expr) => {
        parse!($arg, _)
    };

    ($arg:expr, $t:ty) => {
        match from_utf8($arg)
            .map_err(|_| ParseError::InvalidHeader)?
            .parse::<$t>()
        {
            Ok(res) => Some(res),
            Err(_) => {
                return Err(ParseError::InvalidHeader.into());
            }
        }
    };
}

fn parse_header<R: Read>(bread: &mut BufReader<R>) -> Result<Header> {
    let header = read_line(bread)?;
    let header: Vec<u8> = if header.starts_with(MAGIC_NUMBER) {
        header.into_iter().skip(MAGIC_NUMBER.len()).collect()
    } else {
        return Err(ParseError::InvalidHeader.into());
    };

    let arguments: Vec<&[u8]> = header
        .split(|c| c == &b' ')
        .enumerate()
        .filter(|(i, x)| *i == 0 || !x.is_empty())
        .map(|(_, x)| x)
        .collect();

    if arguments.len() < 6 || arguments.len() > 9 {
        return Err(ParseError::InvalidHeader.into());
    }

    if arguments[0].is_empty() {
        return Err(ParseError::InvalidHeader.into());
    }

    let mut hard_blank_char: Vec<u8> = Vec::with_capacity(arguments[0].len());
    hard_blank_char.extend_from_slice(arguments[0]);
    let height: usize = parse!(arguments[1]).ok_or(ParseError::InvalidHeader)?;
    let baseline: usize = parse!(arguments[2]).ok_or(ParseError::InvalidHeader)?;
    let max_length: usize = parse!(arguments[3]).ok_or(ParseError::InvalidHeader)?;
    let old_layout: i32 = parse!(arguments[4]).ok_or(ParseError::InvalidHeader)?;

    let print_direction: PrintDirection = if arguments.len() > 6 {
        parse!(arguments[6]).ok_or(ParseError::InvalidHeader)?
    } else {
        PrintDirection::LeftToRight
    };

    let layout: Layout = if arguments.len() > 7 {
        parse!(arguments[7]).ok_or(ParseError::InvalidHeader)?
    } else {
        full_layout_from_old_layout(old_layout)
    };

    let codetag_count: Option<u32> = if arguments.len() > 8 {
        Some(parse!(arguments[8]).ok_or(ParseError::InvalidHeader)?)
    } else {
        None
    };

    let comment: String = {
        let comment_lines: usize = parse!(arguments[5]).ok_or(ParseError::InvalidHeader)?;
        read_string_lines(bread, comment_lines)?
    };

    Ok(Header {
        hard_blank_char,
        height,
        baseline,
        max_length,
        comment,
        print_direction,
        layout,
        codetag_count,
    })
}

fn full_layout_from_old_layout(old_layout: i32) -> Layout {
    let raw = if old_layout == 0 {
        64
    } else if old_layout < 0 {
        0
    } else {
        (old_layout as u32 & 31) | 128
    };

    Layout::from_bits_truncate(raw)
}
