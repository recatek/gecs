use std::mem::MaybeUninit;

use crate::index::{DataIndex, MAX_DATA_CAPACITY, MAX_DATA_INDEX};
use crate::util::{num_assert_leq, num_assert_lt};
use crate::version::VersionSlot;

// We use the highest order bit to mark which indices are free list.
// This is necessary because we need to catch if a bad entity handle
// (say, from a different ECS world) tries to access a freed slot and
// treat it as a live data slot, which could cause OOB memory access.
const FREE_BIT: u32 = 1 << 31;

// This is a reserved index marking the end of the free list. It should
// always be bigger than the maximum index we can store in an entity/slot.
// This index has the FREE_BIT baked into it.
const FREE_LIST_END: u32 = (FREE_BIT - 1) | FREE_BIT;

#[inline(always)]
pub(crate) fn populate_free_list(
    free_list_start: DataIndex, // Index in the full slot array of where to begin
    slots: &mut [MaybeUninit<Slot>], // Complete slot array, including old slots
) -> SlotIndex {
    // We need to make sure that MAX_DATA_CAPACITY wouldn't overflow a u32.
    num_assert_leq!(MAX_DATA_CAPACITY as usize, u32::MAX as usize);
    // Make sure we aren't trying to populate more slots than we could store.
    if slots.len() > MAX_DATA_CAPACITY as usize {
        panic!("capacity may not exceed {}", MAX_DATA_CAPACITY);
    }

    if slots.len() > 0 {
        let start_index = free_list_start.get() as usize;
        let last_index = slots.len() - 1;

        debug_assert!(start_index <= slots.len());

        for i in start_index..(slots.len() - 1) {
            unsafe {
                // SAFETY: We know i is less than MAX_DATA_CAPACITY and won't
                // overflow because we only go up to slots.len() - 1, and here
                // we also know that slots.len() <= MAX_DATA_CAPACITY <= u32::MAX.
                let index = DataIndex::new_unchecked(i.wrapping_add(1) as u32);
                let slot = Slot::new_free(SlotIndex::new_free(index));

                // SAFETY: We know that i < slots.len() and is safe to write to.
                slots.get_unchecked_mut(i).write(slot);
            }
        }

        // Set the last slot to point off the end of the free list.
        let last_slot = Slot::new_free(SlotIndex::new_free_end());
        // SAFETY: We know that last_index is valid and less than slots.len().
        unsafe { slots.get_unchecked_mut(last_index).write(last_slot) };

        // Point the free list head to the front of the list.
        SlotIndex::new_free(free_list_start)
    } else {
        // Otherwise, we have nothing, so point the free list head to the end.
        SlotIndex::new_free_end()
    }
}

/// The data index stored in a slot.
///
/// This can point to entity data, or be a member of the slot free list.
#[derive(Clone, Copy)]
pub(crate) struct SlotIndex(u32);

impl SlotIndex {
    /// Creates a new SlotIndex at the end of the free list.
    #[inline(always)]
    pub(crate) fn new_free_end() -> Self {
        Self(FREE_LIST_END)
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

    /// Returns true if this is points to the end of the free list.
    #[inline(always)]
    pub(crate) fn is_free_list_end(&self) -> bool {
        self.0 == FREE_LIST_END
    }

    /// Returns the data index this slot points to.
    /// Will be `None` if the index is a free list index.
    #[inline(always)]
    pub(crate) fn get_data(&self) -> Option<DataIndex> {
        debug_assert!(self.is_free() == false);
        DataIndex::new_u32(self.0)
    }

    /// Returns the free list index this slot points to.
    /// Will be `None` if the index is the free list end.
    #[inline(always)]
    pub(crate) fn get_next_free(&self) -> Option<DataIndex> {
        // This only works if (!FREE_BIT & FREE_LIST_END) is too big to fit in a
        // DataIndex. We test this in verify_free_list_end_is_invalid_data_index.

        debug_assert!(self.is_free());
        DataIndex::new_u32(!FREE_BIT & self.0)
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
    version: VersionSlot,
}

impl Slot {
    #[inline(always)]
    pub(crate) fn new_free(next_free: SlotIndex) -> Self {
        // Make sure that there is room in the index for the free bit.
        num_assert_lt!(MAX_DATA_INDEX as usize, FREE_BIT as usize);

        Self {
            index: next_free,
            version: VersionSlot::start(),
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
    pub(crate) fn version(&self) -> VersionSlot {
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
    assert!(DataIndex::new_u32(!FREE_BIT & FREE_LIST_END).is_none());
}
