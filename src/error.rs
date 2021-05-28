#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("parse error")]
    Parse(#[from] ParseError),
    #[error("failed to read file")]
    Io(#[from] std::io::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("not enough data")]
    NotEnoughData,
    #[error("invalid header")]
    InvalidHeader,
    #[error("invalid character")]
    InvalidCharacter,
}
