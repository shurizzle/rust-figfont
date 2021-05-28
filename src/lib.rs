use std::collections::HashMap;
use std::io::{BufReader, Read};

use character::Character;
use header::Header;

use crate::utils::read_line;

pub mod character;
pub mod error;
pub mod header;
pub mod result;
mod utils;

use crate::result::Result;

const DEUTSCH_CODE_POINTS: [i32; 7] = [196, 214, 220, 228, 246, 252, 223];

const STANDARD_FONT: &'static [u8] = include_bytes!("../fonts/standard.flf");

#[derive(Debug)]
pub struct FIGfont {
    header: Header,
    characters: HashMap<i32, Character>,
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

    for _ in 0..header.comment_lines() {
        read_line(&mut bread)?; // TODO: save comments in the header
    }

    let mut characters = HashMap::new();

    for codepoint in 32..127 {
        characters.insert(codepoint, Character::parse(&mut bread, &header)?);
    }

    for codepoint in DEUTSCH_CODE_POINTS.iter() {
        let codepoint = *codepoint;

        characters.insert(codepoint, Character::parse(&mut bread, &header)?);
    }

    for _ in 0..header.codetag_count() {
        let (codepoint, character) = Character::parse_with_codetag(&mut bread, &header)?;
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

    //#[test]
    //fn bubble() {
    //let font = include_bytes!("../fonts/bubble.flf");
    //FIGfont::read_from(&font[..]).unwrap();
    //}

    //#[test]
    //fn digital() {
    //let font = include_bytes!("../fonts/digital.flf");
    //FIGfont::read_from(&font[..]).unwrap();
    //}

    //#[test]
    //fn term() {
    //let font = include_bytes!("../fonts/term.flf");
    //FIGfont::read_from(&font[..]).unwrap();
    //}
}
