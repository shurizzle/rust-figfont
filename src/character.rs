use std::{
    borrow::Cow,
    io::{BufReader, Read},
    str::from_utf8,
};

use crate::{
    error::ParseError,
    header::Header,
    result::Result,
    utils::{read_last_line, read_line},
};

#[derive(Debug)]
pub struct Character {
    lines: Vec<Vec<u8>>,
}

impl Character {
    pub(crate) fn parse<R: Read>(bread: &mut BufReader<R>, header: &Header) -> Result<Character> {
        read_character(bread, header)
    }

    pub(crate) fn parse_with_codetag<R: Read>(
        bread: &mut BufReader<R>,
        header: &Header,
    ) -> Result<(i32, Character)> {
        read_character_with_codetag(bread, header)
    }

    pub fn lines<'a>(&'a self) -> Cow<'a, Vec<Vec<u8>>> {
        Cow::Borrowed(&self.lines)
    }
}

fn read_character_with_codetag<R: Read>(
    bread: &mut BufReader<R>,
    header: &Header,
) -> Result<(i32, Character)> {
    let codetag = read_codetag(bread)?;
    let character = read_character(bread, header)?;

    Ok((codetag, character))
}

fn read_codetag<R: Read>(bread: &mut BufReader<R>) -> Result<i32> {
    let line = read_line(bread)?;
    let mut code = line
        .splitn(2, |c| c == &b' ')
        .next()
        .ok_or(ParseError::InvalidCharacter)?;

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

    Ok(code.map_err(|_| ParseError::InvalidCharacter)? * sign)
}

fn read_character<R: Read>(bread: &mut BufReader<R>, header: &Header) -> Result<Character> {
    let mut lines = read_lines(bread, header.height())?;

    let first = &lines[0];

    if first.len() == 0 {
        return Err(ParseError::InvalidCharacter.into());
    }

    let delimiter = *first.last().unwrap();

    for i in 0..(lines.len() - 1) {
        if lines[i].len() == 0 {
            return Err(ParseError::InvalidCharacter.into());
        }

        if *lines[i].last().unwrap() != delimiter {
            return Err(ParseError::InvalidCharacter.into());
        }

        let len = lines[i].len();
        lines[i].truncate(len - 1);
        while lines[i].ends_with(b" ") {
            let len = lines[i].len();
            lines[i].truncate(len - 1);
        }
        lines[i] = lines[i]
            .iter()
            .map(|c| {
                if *c == (header.hard_blank_char() as u8) {
                    b' '
                } else {
                    *c
                }
            })
            .collect();
    }

    let i = lines.len() - 1;
    if lines[i].len() < 2 {
        return Err(ParseError::InvalidCharacter.into());
    }

    let last_del = [delimiter, delimiter];

    if !lines[i].ends_with(&last_del[..]) {
        return Err(ParseError::InvalidCharacter.into());
    }

    let len = lines[i].len();
    lines[i].truncate(len - 2);
    while lines[i].ends_with(b" ") {
        let len = lines[i].len();
        lines[i].truncate(len - 1);
    }
    lines[i] = lines[i]
        .iter()
        .map(|c| {
            if *c == (header.hard_blank_char() as u8) {
                b' '
            } else {
                *c
            }
        })
        .collect();

    Ok(Character { lines })
}

fn read_lines<R: Read>(bread: &mut BufReader<R>, num: usize) -> Result<Vec<Vec<u8>>> {
    let mut lines = Vec::with_capacity(num);

    for _ in 0..(num - 1) {
        lines.push(read_line(bread)?);
    }

    lines.push(read_last_line(bread)?);

    Ok(lines)
}
