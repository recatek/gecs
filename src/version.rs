use std::num::NonZeroU32;

#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Version {
    version: NonZeroU32,
}

impl Version {
    #[inline(always)]
    pub(crate) fn start() -> Self {
        // This is a slightly messy hack to create a NonZeroU32 constant.
        const START: NonZeroU32 = match NonZeroU32::new(1) {
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
    pub(crate) fn next(&self) -> Version {
        Version {
            // TODO: Should this be two separate flags for slot vs. archetype wrapping behavior?
            #[cfg(feature = "wrapping_versions")]
            version: NonZeroU32::new(u32::max(self.version.get().wrapping_add(1), 1)).unwrap(),
            #[cfg(not(feature = "wrapping_versions"))]
            version: self.version.checked_add(1).expect("version overflow"),
        }
    }
}
