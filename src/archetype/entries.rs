use seq_macro::seq;

use crate::entity::Entity;
use crate::traits::Archetype;

macro_rules! declare_entries_n {
    ($entries:ident, $n:literal) => {
        seq!(I in 0..$n {
            pub trait $entries<'a, A: Archetype, #(T~I,)*> {
                fn new(index: usize, entity: &'a Entity<A>, #(e~I: &'a mut T~I,)*) -> Self;
            }
        });
    };
}

// Declare entries for up to 16 components.
seq!(N in 1..=16 {
    declare_entries_n!(Entries~N, N);
});

// Declare additional entries for up to 32 components.
#[cfg(feature = "32_components")]
seq!(N in 17..=32 {
    declare_entries_n!(Entries~N, N);
});
