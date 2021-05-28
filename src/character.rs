use std::{
    borrow::Cow,
    io::{BufReader, Read},
};

use crate::{
    error::ParseError,
    header::Header,
    result::Result,
    utils::{read_last_line, read_line},
};

#[derive(Debug)]
pub struct Character {
    lines: Vec<String>,
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

    pub fn lines<'a>(&'a self) -> Cow<'a, Vec<String>> {
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
        .splitn(2, ' ')
        .next()
        .ok_or(ParseError::InvalidCharacter)?;

    let sign: i32 = if code.starts_with('-') {
        code = &code[1..];
        -1
    } else {
        1
    };

    let code = if code.starts_with("0x") || code.starts_with("0X") {
        i32::from_str_radix(&code[2..], 16)
    } else if code.starts_with('0') {
        i32::from_str_radix(&code[1..], 8)
    } else {
        code.parse()
    };

    Ok(code.map_err(|_| ParseError::InvalidCharacter)? * sign)
}

fn read_character<R: Read>(bread: &mut BufReader<R>, header: &Header) -> Result<Character> {
    let mut lines = read_lines(bread, header.height())?;

    let first = &lines[0];

    if first.len() == 0 {
        return Err(ParseError::InvalidCharacter.into());
    }

    let delimiter = first.chars().last().unwrap();

    for i in 0..(lines.len() - 1) {
        if lines[i].len() == 0 {
            return Err(ParseError::InvalidCharacter.into());
        }

        if lines[i].chars().last().unwrap() != delimiter {
            return Err(ParseError::InvalidCharacter.into());
        }

        let len = lines[i].len();
        lines[i].truncate(len - 1);
        while lines[i].ends_with(' ') {
            let len = lines[i].len();
            lines[i].truncate(len - 1);
        }
        lines[i] = lines[i].replace(header.hard_blank_char(), " ");
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
    while lines[i].ends_with(' ') {
        let len = lines[i].len();
        lines[i].truncate(len - 1);
    }
    lines[i] = lines[i].replace(header.hard_blank_char(), " ");

    Ok(Character { lines })
}

fn read_lines<R: Read>(bread: &mut BufReader<R>, num: usize) -> Result<Vec<String>> {
    let mut lines = Vec::with_capacity(num);

    for i in 0..(num - 1) {
        lines.push(read_line(bread)?);
    }

    lines.push(read_last_line(bread)?);

    Ok(lines)
}
