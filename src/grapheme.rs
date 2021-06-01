use std::{borrow::Borrow, str};

use encoding::{all::ISO_8859_1, Encoding};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Grapheme {
    Symbol(String),
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
                    if (&haystack[i..]).starts_with(self.when) {
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

impl Grapheme {
    pub fn split(raw: &[u8], blank_character: &[u8]) -> Result<Vec<Grapheme>, String> {
        let mut res = Vec::new();
        for (i, string) in split(raw, blank_character).enumerate() {
            if i != 0 {
                res.push(Grapheme::Blank);
            }

            if !string.is_empty() {
                let string = ISO_8859_1
                    .decode(string, encoding::DecoderTrap::Strict)
                    .map_err(|e| e.to_string())?
                    .to_string();
                for g in string.graphemes(false) {
                    res.push(Grapheme::Symbol(g.to_string()));
                }
            }
        }

        Ok(res)
    }

    pub fn width(&self) -> usize {
        match self {
            Grapheme::Blank => 1,
            Grapheme::Symbol(ref sym) => UnicodeWidthStr::width(sym.as_str()),
        }
    }
}

impl Borrow<str> for Grapheme {
    fn borrow<'a>(&'a self) -> &'a str {
        match self {
            Grapheme::Symbol(ref res) => res,
            Grapheme::Blank => " ",
        }
    }
}

impl ToString for Grapheme {
    fn to_string(&self) -> String {
        match self {
            Grapheme::Symbol(ref res) => res.to_string(),
            Grapheme::Blank => " ".to_string(),
        }
    }
}

impl From<&char> for Grapheme {
    #[inline]
    fn from(c: &char) -> Self {
        Grapheme::Symbol(c.to_string())
    }
}

impl From<char> for Grapheme {
    #[inline]
    fn from(c: char) -> Self {
        From::from(&c)
    }
}
