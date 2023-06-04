#![allow(dead_code)]
#![allow(unused_macros)]
#![allow(unused_imports)]

pub(crate) struct NumAssert<const L: usize, const R: usize>;

#[allow(clippy::erasing_op)]
impl<const L: usize, const R: usize> NumAssert<L, R> {
    pub const LEQ: usize = R - L;
    pub const LT: usize = R - L - 1;
}

macro_rules! num_assert_leq {
    ($a:expr, $b:expr) => {
        #[allow(path_statements)]
        #[allow(clippy::no_effect)]
        {
            $crate::util::NumAssert::<{ $a }, { $b }>::LEQ;
        }
    };
}

macro_rules! num_assert_lt {
    ($a:expr, $b:expr) => {
        #[allow(path_statements)]
        #[allow(clippy::no_effect)]
        {
            $crate::util::NumAssert::<{ $a }, { $b }>::LT;
        }
    };
}

pub(crate) use num_assert_leq;
pub(crate) use num_assert_lt;

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
