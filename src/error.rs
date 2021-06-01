#[cfg(feature = "zip")]
use zip::result::ZipError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("parse error")]
    Parse(#[from] ParseError),
    #[error("failed to read file")]
    #[cfg(not(feature = "zip"))]
    Io(#[from] std::io::Error),
    #[error("failed to read file")]
    #[cfg(feature = "zip")]
    Io(#[from] Io),
}

#[cfg(feature = "zip")]
#[derive(thiserror::Error, Debug)]
pub enum Io {
    #[error("failed to read file")]
    Std(#[from] std::io::Error),
    #[error("failed to read zip archive")]
    Zip(#[from] ZipError),
}

#[cfg(feature = "zip")]
impl From<std::io::Error> for Error {
    fn from(inner: std::io::Error) -> Self {
        Error::Io(Io::Std(inner))
    }
}

#[cfg(feature = "zip")]
impl From<ZipError> for Error {
    fn from(inner: ZipError) -> Self {
        Error::Io(Io::Zip(inner))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("not enough data")]
    NotEnoughData,
    #[error("invalid header")]
    InvalidHeader,
    #[error("invalid character")]
    InvalidCharacter,
    #[error("invalid font")]
    InvalidFont,
    #[error("invalid extension")]
    InvalidExtension,
}
