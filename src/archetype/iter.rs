use seq_macro::seq;

use crate::entity::Entity;
use crate::traits::Archetype;

macro_rules! declare_iter_n {
    ($iter:ident, $iter_mut:ident, $n:literal) => {
        seq!(I in 0..$n {
            pub struct $iter<'a, A: Archetype, #(T~I,)*> {
                pub(crate) remaining: usize,
                pub(crate) ptr_entity: *const Entity<A>,
                #(pub(crate) ptr_d~I: *const T~I,)*
                pub(crate) phantom: std::marker::PhantomData<&'a mut A>,
            }

            pub struct $iter_mut<'a, A: Archetype, #(T~I,)*> {
                pub(crate) remaining: usize,
                pub(crate) ptr_entity: *const Entity<A>,
                #(pub(crate) ptr_d~I: *mut T~I,)*
                pub(crate) phantom: std::marker::PhantomData<&'a mut A>,
            }

            impl<'a, A: Archetype + 'a, #(T~I: 'a,)*> Iterator for $iter<'a, A, #(T~I,)*> {
                type Item = (&'a Entity<A>, #(&'a T~I,)*);

                fn next(&mut self) -> Option<Self::Item> {
                    if self.remaining == 0 {
                        return None;
                    }

                    unsafe {
                        let result = (&*self.ptr_entity, #(&*self.ptr_d~I,)*);

                        self.ptr_entity = self.ptr_entity.offset(1);
                        #(self.ptr_d~I = self.ptr_d~I.offset(1);)*

                        self.remaining -= 1;
                        Some(result)
                    }
                }
            }

            impl<'a, A: Archetype + 'a, #(T~I: 'a,)*> Iterator for $iter_mut<'a, A, #(T~I,)*> {
                type Item = (&'a Entity<A>, #(&'a mut T~I,)*);

                fn next(&mut self) -> Option<Self::Item> {
                    if self.remaining == 0 {
                        return None;
                    }

                    unsafe {
                        let result = (&*self.ptr_entity, #(&mut *self.ptr_d~I,)*);

                        self.ptr_entity = self.ptr_entity.offset(1);
                        #(self.ptr_d~I = self.ptr_d~I.offset(1);)*

                        self.remaining -= 1;
                        Some(result)
                    }
                }
            }
        });
    };
}

// Declare entries for up to 16 components.
seq!(N in 1..=16 {
    declare_iter_n!(Iter~N, IterMut~N, N);
});

// Declare additional entries for up to 32 components.
#[cfg(feature = "32_components")]
seq!(N in 17..=32 {
    declare_iter_n!(Iter~N, IterMut~N, N);
});
