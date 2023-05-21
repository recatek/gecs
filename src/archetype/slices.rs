use paste::paste;

use crate::entity::Entity;
use crate::traits::Archetype;

macro_rules! declare_slices_n {
    ($n:literal, $($i:literal),+) => {
        paste! {
            pub trait [<Slices$n>]<'a, A: Archetype, $( [<T$i>] ),+> {
                fn new(entities: &'a [Entity<A>], $( [<s$i>]: &'a mut [ [<T$i>] ],)+) -> Self;
            }
        }
    };
}

declare_slices_n!(1, 0);
declare_slices_n!(2, 0, 1);
declare_slices_n!(3, 0, 1, 2);
declare_slices_n!(4, 0, 1, 2, 3);
declare_slices_n!(5, 0, 1, 2, 3, 4);
declare_slices_n!(6, 0, 1, 2, 3, 4, 5);
declare_slices_n!(7, 0, 1, 2, 3, 4, 5, 6);
declare_slices_n!(8, 0, 1, 2, 3, 4, 5, 6, 7);
declare_slices_n!(9, 0, 1, 2, 3, 4, 5, 6, 7, 8);
declare_slices_n!(10, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
declare_slices_n!(11, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
declare_slices_n!(12, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
declare_slices_n!(13, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
declare_slices_n!(14, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13);
declare_slices_n!(15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14);
declare_slices_n!(16, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
