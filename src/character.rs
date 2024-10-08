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

/// The FIGcharacter is the representation of a single large FIGfont character.
#[derive(Debug, Clone)]
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

    /// Get the matrix of SubCharacters.
    pub fn lines(&self) -> Cow<'_, Vec<Vec<SubCharacter>>> {
        Cow::Borrowed(&self.lines)
    }

    /// Get the height (number of lines) of FIGcharacter.
    pub fn height(&self) -> usize {
        self.lines.len()
    }

    /// Get the width (number of terminal cells) of FIGcharacter.
    pub fn width(&self) -> usize {
        self.lines.iter().map(|x| x.len()).max().unwrap_or_default()
    }

    /// Get the comment of the FIGcharacter, if any.
    /// Only for codetagged characters.
    pub fn comment(&self) -> Option<Cow<'_, String>> {
        self.comment.as_ref().map(Cow::Borrowed)
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

    if first.is_empty() {
        return Err(ParseError::InvalidCharacter.into());
    }

    let delimiter = *first.last().unwrap();

    {
        let last_i = lines.len() - 1;
        if !lines[last_i].ends_with(&[delimiter][..]) {
            return Err(ParseError::InvalidCharacter.into());
        }

        if lines[last_i].ends_with(&[delimiter, delimiter][..]) {
            let new_len = lines[last_i].len() - 1;
            unsafe { lines[last_i].set_len(new_len) };
        }
    }

    for line in lines.iter_mut() {
        if line.is_empty() {
            return Err(ParseError::InvalidCharacter.into());
        }

        if *line.last().unwrap() != delimiter {
            return Err(ParseError::InvalidCharacter.into());
        }

        let len = line.len();
        line.truncate(len - 1);
    }

    let mut res: Vec<Vec<SubCharacter>> = Vec::with_capacity(lines.len());

    for line in lines {
        res.push(
            SubCharacter::split(&line[..], header.hard_blank_char())
                .ok()
                .ok_or::<Error>(ParseError::InvalidCharacter.into())?,
        );
    }

    let max_len = res
        .iter()
        .map(|line| line.iter().map(|c| c.width()).sum())
        .max()
        .unwrap_or(0);

    res = res
        .into_iter()
        .map(|mut line| {
            if line.len() < max_len {
                for _ in line.len()..max_len {
                    line.push(SubCharacter::Symbol(" ".to_string()));
                }
            }
            line
        })
        .collect();

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
