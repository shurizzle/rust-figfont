use std::collections::HashMap;
use std::io::{BufReader, Read};

use character::FIGcharacter;
use header::Header;

pub mod character;
pub mod error;
pub mod header;
pub mod result;
pub mod subcharacter;
mod utils;

use crate::result::Result;

const DEUTSCH_CODE_POINTS: [i32; 7] = [196, 214, 220, 228, 246, 252, 223];

const STANDARD_FONT: &'static [u8] = include_bytes!("../fonts/standard.flf");

#[derive(Debug)]
pub struct FIGfont {
    header: Header,
    characters: HashMap<i32, FIGcharacter>,
}

impl FIGfont {
    pub fn read_from<R: Read>(reader: R) -> Result<FIGfont> {
        parse(reader)
    }

    pub fn parse<S: AsRef<str>>(text: S) -> Result<FIGfont> {
        Self::read_from(text.as_ref().as_bytes())
    }

    pub fn standard() -> Result<FIGfont> {
        Self::read_from(STANDARD_FONT)
    }
}

fn parse<R: Read>(reader: R) -> Result<FIGfont> {
    let mut bread /* mlmlmlml */ = BufReader::new(reader);

    let header = Header::parse(&mut bread)?;

    let mut characters = HashMap::new();

    for codepoint in 32..127 {
        characters.insert(codepoint, FIGcharacter::parse(&mut bread, &header)?);
    }

    for codepoint in DEUTSCH_CODE_POINTS.iter() {
        let codepoint = *codepoint;

        characters.insert(codepoint, FIGcharacter::parse(&mut bread, &header)?);
    }

    for _ in 0..header.codetag_count() {
        let (codepoint, character) = FIGcharacter::parse_with_codetag(&mut bread, &header)?;
        characters.insert(codepoint, character);
    }

    Ok(FIGfont { header, characters })
}

#[cfg(test)]
mod tests {
    use crate::FIGfont;

    #[test]
    fn default() {
        FIGfont::standard().unwrap();
    }
}
