use std::{fmt, error::Error};
use ascetic_vis::piet;

#[derive(Debug)]
pub enum ToyError {
    Fatal(Box<dyn Error>),
    PietFailure(piet::Error),
    MinifbFailure(minifb::Error),
}

impl fmt::Display for ToyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ToyError::*;

        match self {
            Fatal(err) => err.fmt(f),
            PietFailure(err) => err.fmt(f),
            MinifbFailure(err) => err.fmt(f),
        }
    }
}

impl Error for ToyError {}

impl From<Box<dyn Error>> for ToyError {
    fn from(err: Box<dyn Error>) -> Self {
        ToyError::Fatal(err)
    }
}

impl From<piet::Error> for ToyError {
    fn from(err: piet::Error) -> Self {
        ToyError::PietFailure(err)
    }
}

impl From<minifb::Error> for ToyError {
    fn from(err: minifb::Error) -> Self {
        ToyError::MinifbFailure(err)
    }
}
