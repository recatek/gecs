use crate::entity::ARCHETYPE_ID_BITS;
use crate::util::debug_checked_assume;

pub(crate) const MAX_DATA_CAPACITY: u32 = 1 << (u32::BITS - ARCHETYPE_ID_BITS);
pub(crate) const MAX_DATA_INDEX: u32 = MAX_DATA_CAPACITY - 1;

/// A size-checked index that can always fit in an entity and live slot.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct TrimmedIndex(u32);

impl TrimmedIndex {
    /// Creates a `TrimmedIndex` pointing to zero.
    #[inline(always)]
    pub(crate) const fn zero() -> Self {
        Self(0)
    }

    /// Creates a new `TrimmedIndex` if the given index is within bounds.
    #[inline(always)]
    pub(crate) const fn new_u32(index: u32) -> Option<Self> {
        match index < MAX_DATA_CAPACITY {
            true => Some(Self(index)),
            false => None,
        }
    }

    /// Creates a new `TrimmedIndex` if the given index is within bounds.
    #[inline(always)]
    pub(crate) const fn new_usize(index: usize) -> Option<Self> {
        match index < MAX_DATA_CAPACITY as usize {
            true => Some(Self(index as u32)),
            false => None,
        }
    }
}

impl From<TrimmedIndex> for u32 {
    fn from(value: TrimmedIndex) -> Self {
        // SAFETY: This is verified at creation
        unsafe { debug_checked_assume!(value.0 < MAX_DATA_CAPACITY) };
        value.0
    }
}

impl From<TrimmedIndex> for usize {
    fn from(value: TrimmedIndex) -> Self {
        // SAFETY: This is verified at creation
        unsafe { debug_checked_assume!(value.0 < MAX_DATA_CAPACITY) };
        value.0.try_into().unwrap()
    }
}
