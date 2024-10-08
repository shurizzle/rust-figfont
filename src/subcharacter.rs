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
    Symbol(String),
    /// An hard blank character. It will print a space character.
    /// It's not a SubCharacter::Symbol(" ".to_string()) for implementations
    /// purposes.
    Blank,
}

struct SplitWith<'a, 'b> {
    haystack: Option<&'a [u8]>,
    when: &'b [u8],
}

impl<'a, 'b> Iterator for SplitWith<'a, 'b> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        match self.haystack {
            Some(haystack) => {
                for i in 0usize..haystack.len() {
                    if haystack[i..].starts_with(self.when) {
                        let res = &haystack[..i];
                        self.haystack = Some(&haystack[(self.when.len() + i)..]);
                        return Some(res);
                    }
                }

                let res = Some(haystack);
                self.haystack = None;
                res
            }
            None => None,
        }
    }
}

fn split<'a, 'b>(haystack: &'a [u8], when: &'b [u8]) -> SplitWith<'a, 'b> {
    SplitWith {
        haystack: Some(haystack),
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
                    res.push(SubCharacter::Symbol(g.to_string()));
                }
            }
        }

        Ok(res)
    }

    /// Get the width (number of terminal cells) of the SubCharacter.
    pub fn width(&self) -> usize {
        match self {
            SubCharacter::Blank => 1,
            SubCharacter::Symbol(ref sym) => UnicodeWidthStr::width(sym.as_str()),
        }
    }

    /// Check if it is an hard blank character.
    pub fn is_blank(&self) -> bool {
        matches!(self, SubCharacter::Blank)
    }
}

impl Borrow<str> for SubCharacter {
    fn borrow(&self) -> &str {
        match self {
            SubCharacter::Symbol(ref res) => res,
            SubCharacter::Blank => " ",
        }
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
        SubCharacter::Symbol(c.to_string())
    }
}

impl From<char> for SubCharacter {
    #[inline]
    fn from(c: char) -> Self {
        From::from(&c)
    }
}
