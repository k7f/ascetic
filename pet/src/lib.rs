use std::{fmt, io, error::Error};

mod pnml;

pub use pnml::{PNML, PNMLError, Net, Page, Place, Transition, Arc};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PetItemKind {
    Arc,
    Place,
    Transition,
    Page,
    Net,
}

#[derive(Debug)]
pub enum PetError {
    IOFailure(io::Error),
    ParsingFailure(PNMLError),
    ItemNotFound(PetItemKind, String, String),
}

impl fmt::Display for PetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use PetError::*;

        match self {
            IOFailure(err) => err.fmt(f),
            ParsingFailure(err) => err.fmt(f),
            ItemNotFound(item_kind, item_id, parent_id) => match item_kind {
                PetItemKind::Arc => {
                    write!(f, "Arc \"{}\" not found on page \"{}\".", item_id, parent_id)
                }
                PetItemKind::Place => {
                    write!(f, "Place \"{}\" not found on page \"{}\".", item_id, parent_id)
                }
                PetItemKind::Transition => {
                    write!(f, "Transition \"{}\" not found on page \"{}\".", item_id, parent_id)
                }
                PetItemKind::Page => {
                    write!(f, "Page \"{}\" not found in net \"{}\".", item_id, parent_id)
                }
                PetItemKind::Net => {
                    if parent_id.is_empty() {
                        write!(f, "Net \"{}\" not found in PNML.", item_id)
                    } else {
                        write!(f, "Net \"{}\" not found in PNML file \"{}\".", item_id, parent_id)
                    }
                }
            },
        }
    }
}

impl Error for PetError {}

impl From<io::Error> for PetError {
    fn from(error: io::Error) -> Self {
        PetError::IOFailure(error)
    }
}

impl From<PNMLError> for PetError {
    fn from(error: PNMLError) -> Self {
        PetError::ParsingFailure(error)
    }
}
