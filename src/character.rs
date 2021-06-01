use std::{
    borrow::Cow,
    io::{BufReader, Read},
    str::from_utf8,
};

use encoding::{all::ISO_8859_1, DecoderTrap, Encoding};

use crate::{
    error::{Error, ParseError},
    header::Header,
    result::Result,
    subcharacter::SubCharacter,
    utils::{read_last_line, read_line},
};

#[derive(Debug)]
pub struct FIGcharacter {
    comment: Option<String>,
    lines: Vec<Vec<SubCharacter>>,
}

impl FIGcharacter {
    pub(crate) fn parse<R: Read>(
        bread: &mut BufReader<R>,
        header: &Header,
    ) -> Result<FIGcharacter> {
        read_character(bread, header)
    }

    pub(crate) fn parse_with_codetag<R: Read>(
        bread: &mut BufReader<R>,
        header: &Header,
    ) -> Result<(i32, FIGcharacter)> {
        read_character_with_codetag(bread, header)
    }

    pub fn lines<'a>(&'a self) -> Cow<'a, Vec<Vec<SubCharacter>>> {
        Cow::Borrowed(&self.lines)
    }

    pub fn height(&self) -> usize {
        self.lines.len()
    }

    pub fn width(&self) -> usize {
        self.lines.iter().map(|x| x.len()).max().unwrap_or_default()
    }
}

fn read_character_with_codetag<R: Read>(
    bread: &mut BufReader<R>,
    header: &Header,
) -> Result<(i32, FIGcharacter)> {
    let (codetag, comment) = read_codetag(bread)?;
    let mut character = read_character(bread, header)?;
    character.comment = comment;

    Ok((codetag, character))
}

fn read_codetag<R: Read>(bread: &mut BufReader<R>) -> Result<(i32, Option<String>)> {
    let line = read_line(bread)?;
    let mut line = line.splitn(2, |c| c == &b' ');
    let mut code = line.next().ok_or(ParseError::InvalidCharacter)?;
    let comment = match line.next() {
        Some(bytes) => match ISO_8859_1.decode(bytes, DecoderTrap::Replace) {
            Ok(comment) => Some(comment),
            Err(_) => None,
        },
        None => None,
    };

    let sign: i32 = if code.starts_with(b"-") {
        code = &code[1..];
        -1
    } else {
        1
    };

    let code = if code.starts_with(b"0x") || code.starts_with(b"0X") {
        let code = from_utf8(&code[2..]).map_err(|_| ParseError::InvalidCharacter)?;
        i32::from_str_radix(code, 16)
    } else if code.starts_with(b"0") {
        let code = from_utf8(&code[1..]).map_err(|_| ParseError::InvalidCharacter)?;
        i32::from_str_radix(code, 8)
    } else {
        from_utf8(code)
            .map_err(|_| ParseError::InvalidCharacter)?
            .parse()
    };

    Ok((
        code.map_err(|_| ParseError::InvalidCharacter)? * sign,
        comment,
    ))
}

fn read_character<R: Read>(bread: &mut BufReader<R>, header: &Header) -> Result<FIGcharacter> {
    let mut lines = read_lines(bread, header.height())?;

    let first = &lines[0];

    if first.len() == 0 {
        return Err(ParseError::InvalidCharacter.into());
    }

    let delimiter = *first.last().unwrap();

    if header.height() > 1 {
        let last_i = lines.len() - 1;
        if !lines[last_i].ends_with(&[delimiter][..]) {
            return Err(ParseError::InvalidCharacter.into());
        }

        let new_len = lines[last_i].len() - 1;
        unsafe { lines[last_i].set_len(new_len) };
    }

    for i in 0..lines.len() {
        if lines[i].len() == 0 {
            return Err(ParseError::InvalidCharacter.into());
        }

        if *lines[i].last().unwrap() != delimiter {
            return Err(ParseError::InvalidCharacter.into());
        }

        let len = lines[i].len();
        lines[i].truncate(len - 1);
    }

    let mut res: Vec<Vec<SubCharacter>> = Vec::with_capacity(lines.len());

    for line in lines {
        res.push(
            SubCharacter::split(&line[..], header.hard_blank_char())
                .ok()
                .ok_or::<Error>(ParseError::InvalidCharacter.into())?,
        );
    }

    Ok(FIGcharacter {
        comment: None,
        lines: res,
    })
}

fn read_lines<R: Read>(bread: &mut BufReader<R>, num: usize) -> Result<Vec<Vec<u8>>> {
    let mut lines = Vec::with_capacity(num);

    for _ in 0..(num - 1) {
        lines.push(read_line(bread)?);
    }

    lines.push(read_last_line(bread)?);

    Ok(lines)
}
