use std::num::NonZeroU32;

// Starting version number. Must convert to a NonZeroU32.
const VERSION_START: u32 = 1;

#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SlotVersion {
    version: NonZeroU32,
}

#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ArchetypeVersion {
    version: NonZeroU32,
}

impl SlotVersion {
    #[inline(always)]
    pub(crate) fn start() -> Self {
        // This is a slightly messy hack to create a NonZeroU32 constant.
        const START: NonZeroU32 = match NonZeroU32::new(VERSION_START) {
            Some(v) => v,
            None => [][0],
        };

        Self { version: START }
    }

    #[inline(always)]
    pub(crate) fn get(&self) -> NonZeroU32 {
        self.version
    }

    #[inline(always)]
    pub(crate) fn next(&self) -> SlotVersion {
        SlotVersion {
            #[cfg(feature = "wrapping_slot_version")]
            version: NonZeroU32::new(u32::max(self.version.get().wrapping_add(1), VERSION_START))
                .unwrap(),
            #[cfg(not(feature = "wrapping_slot_version"))]
            version: self
                .version //.
                .checked_add(1)
                .expect("slot version overflow"),
        }
    }
}

impl ArchetypeVersion {
    #[inline(always)]
    pub(crate) fn start() -> Self {
        // This is a slightly messy hack to create a NonZeroU32 constant.
        const START: NonZeroU32 = match NonZeroU32::new(VERSION_START) {
            Some(v) => v,
            None => [][0],
        };

        Self { version: START }
    }

    #[inline(always)]
    pub(crate) fn get(&self) -> NonZeroU32 {
        self.version
    }

    #[inline(always)]
    pub(crate) fn next(&self) -> ArchetypeVersion {
        ArchetypeVersion {
            #[cfg(feature = "wrapping_entity_raw_version")]
            version: NonZeroU32::new(u32::max(self.version.get().wrapping_add(1), VERSION_START))
                .unwrap(),
            #[cfg(not(feature = "wrapping_entity_raw_version"))]
            version: self
                .version //.
                .checked_add(1)
                .expect("archetype version overflow"),
        }
    }
}
