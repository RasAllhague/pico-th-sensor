use core::convert::Infallible;

use dht_sensor::DhtError;
use display_interface::DisplayError;

pub enum Error {
    Dht(DhtError<Infallible>),
    Display(DisplayError),
    Fmt(core::fmt::Error),
}

impl Error {
    pub fn error_interval(&self) -> u32 {
        match self {
            Error::Dht(_) => 500,
            Error::Display(_) => 1000,
            Error::Fmt(_) => 1500,
        }
    }
}

impl From<DhtError<Infallible>> for Error {
    fn from(value: DhtError<Infallible>) -> Self {
        Error::Dht(value)
    }
}

impl From<DisplayError> for Error {
    fn from(value: DisplayError) -> Self {
        Error::Display(value)
    }
}

impl From<core::fmt::Error> for Error {
    fn from(value: core::fmt::Error) -> Self {
        Error::Fmt(value)
    }
}
