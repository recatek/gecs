use std::num::NonZeroU32;

use crate::error::EcsError;

const VERSION_START: NonZeroU32 = match NonZeroU32::new(1) {
    Some(v) => v,
    None => [][0],
};
const FREE_BIT: u32 = 0x80000000;

#[derive(Clone, Copy)]
pub(crate) struct Slot {
    index: u32,
    version: NonZeroU32,
}

impl Slot {
    #[inline(always)]
    pub(crate) fn new(next_free: u32) -> Self {
        debug_assert!(next_free <= !FREE_BIT);
        Self {
            index: next_free | FREE_BIT,
            version: VERSION_START,
        }
    }

    /// Gets the slot's index value when representing a data pointer.
    #[inline(always)]
    pub(crate) fn index(&self) -> u32 {
        self.index & (!FREE_BIT)
    }

    /// Returns true if this slot is freed.
    #[inline(always)]
    pub(crate) fn is_free(&self) -> bool {
        (self.index & FREE_BIT) != 0
    }

    /// Get the slot's generational version.
    #[inline(always)]
    pub(crate) fn version(&self) -> NonZeroU32 {
        self.version
    }

    /// Assigns a slot to some data. This does not increment the version.
    #[inline(always)]
    pub(crate) fn assign(&mut self, index_data: u32) {
        self.index = index_data;

        // We increment the version counter on release, not assignment
    }

    /// Releases a slot and increments its version, invalidating all handles.
    /// Returns an `EcsError::VersionOverflow` if the version increment overflows.
    #[inline(always)]
    pub(crate) fn release(&mut self, index_free: u32) -> Result<(), EcsError> {
        debug_assert!(self.is_free() == false);

        self.index = index_free | FREE_BIT;

        // We increment the version counter on release, not assignment
        if let Some(version) = self.version.checked_add(1) {
            self.version = version;
            Ok(())
        } else {
            Err(EcsError::VersionOverflow)
        }
    }
}
