#![allow(dead_code)]
#![allow(unused_macros)]
#![allow(unused_imports)]

/// Hints the compiler that the given predicate will always be true.
///
/// # Safety
///
/// If the given predicate is ever not true, this will result in UB.
/// This is checked only in builds with debug assertions enabled.
macro_rules! debug_checked_assume {
    ($ex:expr) => {
        if (!$ex) {
            debug_assert!(false);
            ::std::hint::unreachable_unchecked();
        }
    };
}

/// Hints the compiler that the given line of code can never be reached.
///
/// # Safety
///
/// If this line of code is ever reached, this will result in UB.
/// This is checked only in builds with debug assertions enabled.
macro_rules! debug_checked_unreachable {
    () => {
        debug_assert!(false);
        ::std::hint::unreachable_unchecked();
    };
}

pub(crate) use debug_checked_assume;
pub(crate) use debug_checked_unreachable;
