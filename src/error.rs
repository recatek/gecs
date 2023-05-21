use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum EcsError {
    InvalidEntityType,
    VersionOverflow,
}

impl std::error::Error for EcsError {}

impl Display for EcsError {
    #[cold]
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            EcsError::InvalidEntityType => write!(f, "invalid type for entity"),
            EcsError::VersionOverflow => write!(f, "entity version overflow"),
        }
    }
}
