use crate::entity::ARCHETYPE_ID_BITS;
use crate::util::{debug_checked_assume, num_assert_lt};

pub(crate) const MAX_DATA_CAPACITY: u32 = 1 << (u32::BITS - ARCHETYPE_ID_BITS);
pub(crate) const MAX_DATA_INDEX: u32 = MAX_DATA_CAPACITY - 1;

/// A size-checked index that can always fit in an entity and live slot.
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct DataIndex(u32);

impl DataIndex {
    /// Creates a `DataIndex` pointing to zero.
    #[inline(always)]
    pub(crate) const fn zero() -> Self {
        // Better safe than sorry I guess...
        num_assert_lt!(0, MAX_DATA_INDEX as usize);
        Self(0)
    }

    /// Creates a new `DataIndex` if the given index is within bounds.
    #[inline(always)]
    pub(crate) const fn new(index: u32) -> Option<Self> {
        match index < MAX_DATA_CAPACITY {
            true => Some(Self(index)),
            false => None,
        }
    }

    /// Creates a new `DataIndex` without checking any bounds.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `index < INDEX_MAX`.
    #[inline(always)]
    pub(crate) unsafe fn new_unchecked(index: u32) -> Self {
        debug_assert!(index < MAX_DATA_CAPACITY);
        Self(index)
    }

    /// Gets the raw value of this `DataIndex`.
    ///
    /// The result is guaranteed to be less than `INDEX_MAX`.
    #[inline(always)]
    pub(crate) fn get(self) -> u32 {
        unsafe {
            // SAFETY: This is verified at creation
            debug_checked_assume!(self.0 < MAX_DATA_CAPACITY);
            self.0
        }
    }
}
