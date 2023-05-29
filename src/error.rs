use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum EcsError {
    /// A runtime entity type did not meet the expected type for this operation.
    InvalidEntityType,

    /// A generational index version overflowed. This could lead to erroneous
    /// behavior, as we rely on generational indices to detect stale entity keys.
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
