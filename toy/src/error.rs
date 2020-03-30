use std::error::Error;
use ascetic_vis::piet;

#[derive(Debug)]
pub enum ToyError {
    Fatal(Box<dyn Error>),
}

impl From<Box<dyn Error>> for ToyError {
    fn from(err: Box<dyn Error>) -> Self {
        ToyError::Fatal(err)
    }
}

impl From<piet::Error> for ToyError {
    fn from(err: piet::Error) -> Self {
        ToyError::Fatal(err.into())
    }
}

impl From<minifb::Error> for ToyError {
    fn from(err: minifb::Error) -> Self {
        ToyError::Fatal(err.into())
    }
}
