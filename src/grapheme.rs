use std::{
    borrow::Borrow,
    str::{self, Utf8Error},
};

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
    pub fn split(raw: &[u8], blank_character: &[u8]) -> Result<Vec<Grapheme>, Utf8Error> {
        let mut res = Vec::new();
        for (i, string) in split(raw, blank_character).enumerate() {
            if i != 0 {
                res.push(Grapheme::Blank);
            }

            if !string.is_empty() {
                let string = str::from_utf8(string)?;
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
