use crate::error::ParseError;
use crate::result::Result;
use std::io::{BufRead, BufReader, Read};

pub(crate) fn read_line<R: Read>(bread: &mut BufReader<R>) -> Result<String> {
    let mut line = String::new();
    bread.read_line(&mut line)?;

    if line.ends_with("\r\n") {
        line.truncate(line.len() - 2);
    } else if line.ends_with("\n") {
        line.truncate(line.len() - 1);
    } else {
        return Err(ParseError::NotEnoughData.into());
    }

    Ok(line)
}

pub(crate) fn read_last_line<R: Read>(bread: &mut BufReader<R>) -> Result<String> {
    let mut line = String::new();
    bread.read_line(&mut line)?;

    if line.ends_with("\r\n") {
        line.truncate(line.len() - 2);
    } else if line.ends_with("\n") {
        line.truncate(line.len() - 1);
    }

    Ok(line)
}
