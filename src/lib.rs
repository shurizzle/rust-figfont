use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use error::ParseError;

pub mod character;
pub mod error;
pub mod header;
pub mod result;
pub mod subcharacter;
mod utils;

use crate::result::Result;

const DEUTSCH_CODE_POINTS: [i32; 7] = [196, 214, 220, 228, 246, 252, 223];

const STANDARD_FONT: &[u8] = include_bytes!("../fonts/plain/standard.flf");

pub use crate::{
    character::FIGcharacter,
    header::{Header, Layout, PrintDirection},
    subcharacter::SubCharacter,
};

/// FIGfont reader and parser.
#[derive(Debug, Clone)]
pub struct FIGfont {
    header: Header,
    characters: HashMap<i32, FIGcharacter>,
}

impl FIGfont {
    /// Read and parse a FIGfont from a path. It can be zipped if you have zip
    /// feature enabled.
    pub fn load_from<P: AsRef<Path>>(path: P) -> Result<FIGfont> {
        load_from(path)
    }

    /// Read and parse a FIGfont from a impl Read.
    pub fn read_from<R: Read>(reader: R) -> Result<FIGfont> {
        parse(reader)
    }

    /// Get the standard FIGfont. (hardcoded)
    pub fn standard() -> Result<FIGfont> {
        Self::read_from(STANDARD_FONT)
    }

    /// Get the current FIGfont's header.
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Get the FIGcharacter for the `code` character.
    pub fn get(&self, code: i32) -> &FIGcharacter {
        self.characters
            .get(&code)
            .unwrap_or_else(|| self.characters.get(&126i32).unwrap())
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

    let mut cnt = 0;
    while !bread.fill_buf()?.is_empty() {
        let (codepoint, character) = FIGcharacter::parse_with_codetag(&mut bread, &header)?;
        characters.insert(codepoint, character);
        cnt += 1;
    }

    if let Some(expected_cnt) = header.codetag_count() {
        if expected_cnt != cnt {
            return Err(ParseError::InvalidFont.into());
        }
    }

    Ok(FIGfont { header, characters })
}

#[cfg(feature = "zip")]
fn load_from_zip<P: AsRef<Path>>(path: P) -> Result<FIGfont> {
    use crate::error::Error;
    use zip::ZipArchive;

    let mut zip = ZipArchive::new(File::open(path.as_ref())?)?;

    let file_name = path
        .as_ref()
        .file_name()
        .ok_or::<Error>(ParseError::InvalidFont.into())?
        .to_str()
        .ok_or::<Error>(ParseError::InvalidFont.into())?;

    let f = zip.by_name(file_name)?;

    parse(f)
}

#[cfg(feature = "zip")]
fn is_plain<P: AsRef<Path>>(path: P) -> Result<bool> {
    let mut f = File::open(path)?;
    let mut number: [u8; 5] = [0; 5];
    f.read_exact(&mut number)?;
    Ok(&number == b"flf2a")
}

fn load_from<P: AsRef<Path>>(path: P) -> Result<FIGfont> {
    let path = path.as_ref();
    match path.extension() {
        Some(ext) => {
            if ext != "flf" {
                return Err(ParseError::InvalidExtension.into());
            }
        }
        None => {
            return Err(ParseError::InvalidExtension.into());
        }
    }

    #[cfg(feature = "zip")]
    {
        if is_plain(path)? {
            parse(File::open(path)?)
        } else {
            load_from_zip(path)
        }
    }

    #[cfg(not(feature = "zip"))]
    {
        parse(File::open(path)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::FIGfont;

    #[test]
    fn default() {
        assert!(FIGfont::standard().is_ok());
    }
}
