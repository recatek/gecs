use std::fmt::{Display, Formatter, Result};

/// Error reporting enum for ECS operation failure.
#[derive(Debug)]
pub enum EcsError {
    /// A runtime entity type did not meet the expected type for this operation.
    InvalidEntityType,
}

impl std::error::Error for EcsError {}

impl Display for EcsError {
    #[cold]
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            EcsError::InvalidEntityType => write!(f, "invalid type for entity"),
        }
    }
}
