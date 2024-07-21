use std::mem::MaybeUninit;

use crate::index::{TrimmedIndex, MAX_DATA_INDEX};
use crate::version::SlotVersion;

// We use the highest order bit to mark which indices are free list.
// This is necessary because we need to catch if a bad entity handle
// (say, from a different ECS world) tries to access a freed slot and
// treat it as a live data slot, which could cause OOB memory access.
const FREE_BIT: u32 = 1 << 31;

// This is a reserved index marking the end of the free list. It should
// always be bigger than the maximum index we can store in an entity/slot.
// This index has the FREE_BIT baked into it.
const FREE_LIST_END: u32 = (FREE_BIT - 1) | FREE_BIT;

/// The data index stored in a slot.
///
/// Can point to the dense list (entity data) if the slot is live, or to
/// the sparse list (other slots) if the slot is a member of the free list.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SlotIndex(u32);

impl SlotIndex {
    /// Assigns this index to some non-free data index.
    /// This may be a reassignment of an already live slot.
    #[inline(always)]
    pub(crate) fn new_data(index: TrimmedIndex) -> Self {
        Self(Into::<u32>::into(index))
    }

    /// Assigns this index to some free slot index.
    #[inline(always)]
    pub(crate) fn new_free(index: TrimmedIndex) -> Self {
        // Make sure that there is room in the index for the free bit.
        const { assert!((MAX_DATA_INDEX as usize) < (FREE_BIT as usize)) };

        Self(Into::<u32>::into(index) | FREE_BIT)
    }

    /// Creates a new SlotIndex at the end of the free list.
    #[inline(always)]
    pub(crate) const fn free_end() -> Self {
        Self(FREE_LIST_END)
    }

    /// Returns true if this slot index points to the free list.
    #[inline(always)]
    pub(crate) const fn is_free(&self) -> bool {
        (FREE_BIT & self.0) != 0
    }

    /// Returns true if this is points to the end of the free list.
    #[inline(always)]
    pub(crate) const fn is_free_end(&self) -> bool {
        self.0 == FREE_LIST_END
    }

    /// Returns the data index this slot points to, if valid (e.g. not free).
    #[inline(always)]
    pub(crate) fn index_data(&self) -> Option<TrimmedIndex> {
        debug_assert!(self.is_free() == false);
        match self.is_free_end() {
            true => None,
            // SAFETY: If this isn't free, then we know it must be a valid `TrimmedIndex`
            false => unsafe { Some(TrimmedIndex::new_u32(self.0).unwrap_unchecked()) },
        }
    }

    /// Returns the free list entry this slot points to, if valid (e.g. not free list end).
    #[inline(always)]
    pub(crate) fn index_free(&self) -> Option<TrimmedIndex> {
        debug_assert!(self.is_free());
        match self.is_free_end() {
            true => None,
            // SAFETY: If this isn't the free end, then we know it must be a valid `TrimmedIndex`
            false => unsafe { Some(TrimmedIndex::new_u32(self.0 & !FREE_BIT).unwrap_unchecked()) },
        }
    }
}

// TODO: Seal this
#[derive(Clone, Copy)]
pub struct Slot {
    index: SlotIndex,
    version: SlotVersion,
}

impl Slot {
    pub(crate) fn populate_free_list(
        start: TrimmedIndex, // Index of where the unset section of the slot array begins
        slots: &mut [MaybeUninit<Slot>], // Complete slot array, including old slots
    ) -> SlotIndex {
        if slots.len() > 0 {
            let start_idx = start.into();
            let end_idx = slots.len() - 1;

            // Go to the second-to-last slot
            for idx in start_idx..end_idx {
                let next = TrimmedIndex::new_usize(idx + 1).unwrap();
                let slot = Slot::new_free(SlotIndex::new_free(next));
                slots.get_mut(idx).unwrap().write(slot);
            }

            // Set the last slot to point off the end of the free list.
            let last_slot = Slot::new_free(SlotIndex::free_end());
            slots.get_mut(end_idx).unwrap().write(last_slot);

            // Point the free list head to the front of the list.
            SlotIndex::new_free(start)
        } else {
            // Otherwise, we have nothing, so point the free list head to the end.
            SlotIndex::free_end()
        }
    }

    #[inline(always)]
    pub(crate) fn new_free(next_free: SlotIndex) -> Self {
        debug_assert!(next_free.is_free());

        Self {
            index: next_free,
            version: SlotVersion::start(),
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
    pub(crate) fn version(&self) -> SlotVersion {
        self.version
    }

    /// Assigns a slot to some data. This does not increment the version.
    #[inline(always)]
    pub(crate) fn assign(&mut self, index_data: TrimmedIndex) {
        self.index = SlotIndex::new_data(index_data);

        // NOTE: We increment the version on release, not assignment.
    }

    /// Releases a slot and increments its version, invalidating all handles.
    /// Returns an `EcsError::VersionOverflow` if the version increment overflows.
    #[inline(always)]
    pub(crate) fn release(&mut self, index_next_free: SlotIndex) {
        debug_assert!(self.is_free() == false);
        self.index = index_next_free;
        self.version = self.version.next();
    }
}

// Need to enforce this invariant here just in case.
// If this isn't true, then we can't trust the FREE_LIST_END value.
#[test]
fn verify_free_list_end_is_invalid_data_index() {
    assert!(TrimmedIndex::new_u32(!FREE_BIT & FREE_LIST_END).is_none());
}
