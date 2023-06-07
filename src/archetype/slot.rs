use std::num::NonZeroU32;

use crate::error::EcsError;
use crate::index::{DataIndex, MAX_DATA_INDEX};
use crate::util::num_assert_lt;

// This is a slightly messy hack to create a NonZeroU32 constant.
const VERSION_START: NonZeroU32 = match NonZeroU32::new(1) {
    Some(v) => v,
    None => [][0],
};

// We use the highest order bit to mark which indices are free list.
// This is necessary because we need to catch if a bad entity handle
// (say, from a different ECS world) tries to access a freed slot and
// treat it as a live data slot, which could cause OOB memory access.
const FREE_BIT: u32 = 1 << 31;

// This is a reserved index marking the end of the free list. It should
// always be bigger than the maximum index we can store in an entity/slot.
const FREE_LIST_END: u32 = FREE_BIT - 1;

/// The data index stored in a slot.
///
/// This can point to entity data, or be a member of the slot free list.
#[derive(Clone, Copy)]
pub(crate) struct SlotIndex(u32);

impl SlotIndex {
    /// Creates a new SlotIndex at the end of the free list.
    #[inline(always)]
    pub(crate) fn new_free_end() -> Self {
        Self(FREE_BIT | FREE_LIST_END)
    }

    /// Creates a new SlotIndex pointing to another slot in the free list.
    pub(crate) fn new_free(next_free: DataIndex) -> Self {
        Self(FREE_BIT | next_free.get())
    }

    /// Returns true if this slot is freed.
    #[inline(always)]
    pub(crate) fn is_free(&self) -> bool {
        (FREE_BIT & self.0) != 0
    }

    /// Returns the data index this slot points to.
    /// Will be `None` if the index is a free list index.
    #[inline(always)]
    pub(crate) fn get_data(&self) -> Option<DataIndex> {
        debug_assert!(self.is_free() == false);
        DataIndex::new(self.0)
    }

    /// Returns the free list index this slot points to.
    /// Will be `None` if the index is the free list end.
    #[inline(always)]
    pub(crate) fn get_next_free(&self) -> Option<DataIndex> {
        // This only works if FREE_LIST_END is too big to fit in a DataIndex.
        // We also verify this in verify_free_list_end_is_invalid_data_index.
        num_assert_lt!(MAX_DATA_INDEX as usize, FREE_LIST_END as usize);

        debug_assert!(self.is_free());
        DataIndex::new(!FREE_BIT & self.0)
    }

    /// Assigns a slot to some non-free data index.
    /// This may be a reassignment of an already live slot.
    #[inline(always)]
    pub(crate) fn assign_data(&mut self, index_data: DataIndex) {
        self.0 = index_data.get();
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Slot {
    index: SlotIndex,
    version: NonZeroU32,
}

impl Slot {
    #[inline(always)]
    pub(crate) fn new_free(next_free: SlotIndex) -> Self {
        // Make sure that there is room in the index for the free bit.
        num_assert_lt!(MAX_DATA_INDEX as usize, FREE_BIT as usize);

        Self {
            index: next_free,
            version: VERSION_START,
        }
    }

    /// Returns this slot's index. May point to data or a free list entry.
    #[inline(always)]
    pub(crate) fn index(&self) -> SlotIndex {
        self.index
    }

    /// Returns true if this slot is freed.
    #[inline(always)]
    pub(crate) fn is_free(&self) -> bool {
        self.index.is_free()
    }

    /// Get the slot's generational version.
    #[inline(always)]
    pub(crate) fn version(&self) -> NonZeroU32 {
        self.version
    }

    /// Assigns a slot to some data. This does not increment the version.
    #[inline(always)]
    pub(crate) fn assign(&mut self, index_data: DataIndex) {
        self.index.assign_data(index_data);

        // NOTE: We increment the version on release, not assignment.
    }

    /// Releases a slot and increments its version, invalidating all handles.
    /// Returns an `EcsError::VersionOverflow` if the version increment overflows.
    #[inline(always)]
    pub(crate) fn release(&mut self, index_next_free: SlotIndex) -> Result<(), EcsError> {
        debug_assert!(self.is_free() == false);
        self.index = index_next_free;

        // Increment the version to invalidate all previous handles to this slot.
        // This prevents bad access from stale entity handles after moving the data.
        if let Some(version) = self.version.checked_add(1) {
            self.version = version;
            Ok(())
        } else {
            // The version could overflow after u32::MAX rewrites. This is very unlikely,
            // but irrecoverable for this slot if it happens. We can't wrap, since that
            // could make some stale entity handles valid again. We just have to fail.
            Err(EcsError::VersionOverflow)
        }
    }
}

// Need to enforce this invariant here just in case.
// If this isn't true, then we can't trust the FREE_LIST_END value.
#[test]
fn verify_free_list_end_is_invalid_data_index() {
    assert!(DataIndex::new(FREE_LIST_END).is_none());
}
