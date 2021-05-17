use std::{fmt, error::Error};
use ascetic_vis::piet;

#[derive(Debug)]
pub enum BoyError {
    Fatal(Box<dyn Error>),
    PietFailure(piet::Error),
    MinifbFailure(minifb::Error),
}

impl fmt::Display for BoyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use BoyError::*;

        match self {
            Fatal(err) => err.fmt(f),
            PietFailure(err) => err.fmt(f),
            MinifbFailure(err) => err.fmt(f),
        }
    }
}

impl Error for BoyError {}

impl From<Box<dyn Error>> for BoyError {
    fn from(err: Box<dyn Error>) -> Self {
        BoyError::Fatal(err)
    }
}

impl From<piet::Error> for BoyError {
    fn from(err: piet::Error) -> Self {
        BoyError::PietFailure(err)
    }
}

impl From<minifb::Error> for BoyError {
    fn from(err: minifb::Error) -> Self {
        BoyError::MinifbFailure(err)
    }
}
