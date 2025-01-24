use std::num::NonZeroU32;

// Starting version number. Must convert to a NonZeroU32.
const VERSION_START: NonZeroU32 = NonZeroU32::MIN;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SlotVersion {
    version: NonZeroU32,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArchetypeVersion {
    version: NonZeroU32,
}

impl SlotVersion {
    #[inline(always)]
    pub(crate) fn new(version: NonZeroU32) -> Self {
        Self {
            version, // Direct set
        }
    }

    #[inline(always)]
    pub(crate) fn start() -> Self {
        Self {
            version: VERSION_START,
        }
    }

    #[inline(always)]
    pub(crate) fn get(&self) -> NonZeroU32 {
        self.version
    }

    #[inline(always)]
    pub(crate) fn next(&self) -> SlotVersion {
        SlotVersion {
            #[cfg(feature = "wrapping_version")]
            version: NonZeroU32::new(self.version.get().wrapping_add(1)).unwrap_or(VERSION_START),
            #[cfg(not(feature = "wrapping_version"))]
            version: self.version.checked_add(1).expect("slot version overflow"),
        }
    }
}

impl ArchetypeVersion {
    #[inline(always)]
    pub(crate) fn start() -> Self {
        Self {
            version: VERSION_START,
        }
    }

    #[inline(always)]
    pub(crate) fn get(&self) -> NonZeroU32 {
        self.version
    }

    #[inline(always)]
    pub(crate) fn next(&self) -> ArchetypeVersion {
        ArchetypeVersion {
            #[cfg(feature = "wrapping_version")]
            version: NonZeroU32::new(self.version.get().wrapping_add(1)).unwrap_or(VERSION_START),
            #[cfg(not(feature = "wrapping_version"))]
            version: self.version.checked_add(1).expect("arch version overflow"),
        }
    }
}
