use std::{
    borrow::Borrow,
    fmt::{Display, Formatter},
    str,
};

use encoding::{all::ISO_8859_1, Encoding};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// A SubCharacter is a single real character
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubCharacter {
    /// The actual subcharacter
    Symbol(Box<str>),
    /// An hard blank character. It will print a space character.
    /// It's not a SubCharacter::Symbol(" ".to_string()) for implementations
    /// purposes.
    Blank,
}

struct SplitWith<'a, 'b> {
    haystack: Option<&'a [u8]>,
    when: &'b [u8],
}

#[inline(always)]
fn split_at<'a>(haystack: &'a [u8], when: &[u8]) -> (&'a [u8], Option<&'a [u8]>) {
    if when.is_empty() {
        if haystack.is_empty() {
            return (haystack, None);
        }

        let (a, b) = haystack.split_at(1);
        return (a, if b.is_empty() { None } else { Some(b) });
    }

    let mut off = 0usize;
    while off < haystack.len() {
        match memchr::memchr(when[0], &haystack[off..]).map(|n| n + off) {
            Some(i) => {
                if let Some(rest) = haystack[i..].strip_prefix(when) {
                    return (
                        &haystack[..i],
                        if rest.is_empty() { None } else { Some(rest) },
                    );
                }
                off = i + 1;
            }
            None => return (haystack, None),
        }
    }

    (haystack, None)
}

impl<'a, 'b> Iterator for SplitWith<'a, 'b> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let res;
        (res, self.haystack) = split_at(self.haystack?, self.when);
        Some(res)
    }
}

fn split<'a, 'b>(haystack: &'a [u8], when: &'b [u8]) -> SplitWith<'a, 'b> {
    SplitWith {
        haystack: if haystack.is_empty() {
            None
        } else {
            Some(haystack)
        },
        when,
    }
}

impl SubCharacter {
    /// Split a Latin1-encoded string in a Vec<SubCharacter>
    pub fn split(raw: &[u8], blank_character: &[u8]) -> Result<Vec<SubCharacter>, String> {
        let mut res = Vec::new();
        for (i, string) in split(raw, blank_character).enumerate() {
            if i != 0 {
                res.push(SubCharacter::Blank);
            }

            if !string.is_empty() {
                let string = ISO_8859_1
                    .decode(string, encoding::DecoderTrap::Strict)
                    .map_err(|e| e.to_string())?
                    .to_string();
                for g in string.graphemes(false) {
                    res.push(SubCharacter::Symbol(g.to_string().into_boxed_str()));
                }
            }
        }

        Ok(res)
    }

    /// Get the width (number of terminal cells) of the SubCharacter.
    pub fn width(&self) -> usize {
        match self {
            SubCharacter::Blank => 1,
            SubCharacter::Symbol(ref sym) => UnicodeWidthStr::width(sym.as_ref()),
        }
    }

    /// Check if it is an hard blank character.
    pub fn is_blank(&self) -> bool {
        matches!(self, SubCharacter::Blank)
    }
}

impl AsRef<str> for SubCharacter {
    fn as_ref(&self) -> &str {
        match self {
            SubCharacter::Symbol(ref res) => res,
            SubCharacter::Blank => " ",
        }
    }
}

impl Borrow<str> for SubCharacter {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

impl Display for SubCharacter {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            SubCharacter::Blank => write!(fmt, " "),
            SubCharacter::Symbol(c) => write!(fmt, "{}", c),
        }
    }
}

impl From<&char> for SubCharacter {
    #[inline]
    fn from(c: &char) -> Self {
        SubCharacter::Symbol(c.to_string().into_boxed_str())
    }
}

impl From<char> for SubCharacter {
    #[inline]
    fn from(c: char) -> Self {
        From::from(&c)
    }
}
