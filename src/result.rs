use std;
use serde_json;
use core;
use crypto;
use crypto::symmetriccipher::SymmetricCipherError;
use std::any::Any;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    JSON(serde_json::Error),
    AES(crypto::symmetriccipher::SymmetricCipherError),
    AddrParse(std::net::AddrParseError),
    JoinError(Box<Any + Send + 'static>),
    RecvError(std::sync::mpsc::RecvError),
    SendError,
    OTPError,
    NoneError,
    NoSpace,
    ToLarge,
    PubKeyNotFound,
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Error::NoSpace, &Error::NoSpace) => true,
            (&Error::ToLarge, &Error::ToLarge) => true,
            _ => false,
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;

pub fn from_option<T>(r: Option<T>) -> Result<T> {
    r.ok_or(Error::NoneError)
}

impl core::convert::From<std::sync::mpsc::RecvError> for Error {
    fn from(e: std::sync::mpsc::RecvError) -> Error {
        Error::RecvError(e)
    }
}

impl core::convert::From<Box<Any + Send + 'static>> for Error {
    fn from(e: Box<Any + Send + 'static>) -> Error {
        Error::JoinError(e)
    }
}

impl core::convert::From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IO(e)
    }
}
impl core::convert::From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::JSON(e)
    }
}
impl core::convert::From<SymmetricCipherError> for Error {
    fn from(e: SymmetricCipherError) -> Error {
        Error::AES(e)
    }
}
impl core::convert::From<std::net::AddrParseError> for Error {
    fn from(e: std::net::AddrParseError) -> Error {
        Error::AddrParse(e)
    }
}

#[cfg(test)]
mod tests {
    use result::Result;
    use result::Error;
    use std::net::SocketAddr;
    use std::sync::mpsc::RecvError;
    use crypto::symmetriccipher::SymmetricCipherError::InvalidPadding;
    use std::thread;
    use std::io;

    fn addr_parse_error() -> Result<()> {
        let _r1: SocketAddr = "12fdfasfsafsadfs".parse()?;
        return Ok(());
    }

    fn join_error() -> Result<()> {
        thread::spawn(|| panic!("hi")).join()?;
        return Ok(());
    }

    #[test]
    fn from_test() {
        assert_matches!(addr_parse_error(), Err(Error::AddrParse(_)));
        assert_matches!(Error::from(InvalidPadding), Error::AES(_));
        assert_matches!(Error::from(RecvError {}), Error::RecvError(_));
        assert_matches!(join_error(), Err(Error::JoinError(_)));
        let ioe = io::Error::new(io::ErrorKind::NotFound, "hi");
        assert_matches!(Error::from(ioe), Error::IO(_));
    }
}
