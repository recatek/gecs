use std::fmt::{Display, Formatter, Result};

/// Error reporting enum for ECS operation failure.
#[derive(Debug)]
pub enum EcsError {
    /// A runtime entity did not meet the expected type for this operation.
    InvalidEntityType,
    /// We failed to construct a locally-valid entity handle from raw data.
    InvalidRawEntity,
}

impl std::error::Error for EcsError {}

impl Display for EcsError {
    #[cold]
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            EcsError::InvalidEntityType => write!(f, "invalid type for entity"),
            EcsError::InvalidRawEntity => write!(f, "invalid raw entity data"),
        }
    }
}
