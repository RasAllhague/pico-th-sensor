use core::convert::Infallible;

use dht_sensor::DhtError;
use display_interface::DisplayError;

pub enum Error {
    Dht(DhtError<Infallible>),
    Display(DisplayError),
    Fmt(core::fmt::Error),
}

impl Error {
    pub const fn error_interval(&self) -> u32 {
        match self {
            Self::Dht(_) => 500,
            Self::Display(_) => 1000,
            Self::Fmt(_) => 1500,
        }
    }
}

impl From<DhtError<Infallible>> for Error {
    fn from(value: DhtError<Infallible>) -> Self {
        Self::Dht(value)
    }
}

impl From<DisplayError> for Error {
    fn from(value: DisplayError) -> Self {
        Self::Display(value)
    }
}

impl From<core::fmt::Error> for Error {
    fn from(value: core::fmt::Error) -> Self {
        Self::Fmt(value)
    }
}
