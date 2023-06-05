use std::array;
use std::cell::{Ref, RefCell, RefMut};
use std::mem::MaybeUninit;
use std::ptr;

use paste::paste;

use crate::archetype::slices::*;
use crate::archetype::slot::Slot;
use crate::entity::{Entity, EntityIndex, MAX_ARCHETYPE_CAPACITY};
use crate::traits::Archetype;
use crate::util::{debug_checked_assume, num_assert_leq, num_assert_lt};

macro_rules! declare_dense_fixed_n {
    ($n:literal, $($i:literal),+) => {
        paste! {
            pub struct [<StorageFixed$n>]<A: Archetype, $([<T$i>],)+ const N: usize> {
                len: usize,
                free_head: u32,
                slots: Box<[Slot; N]>, // Sparse
                // No RefCell here since we never grant mutable access externally
                entities: Box<MaybeUninitArray<Entity<A>, N>>,
                $([<d$i>]: RefCell<Box<MaybeUninitArray<[<T$i>], N>>>,)+
            }

            impl<A: Archetype, $([<T$i>],)+ const N: usize> [<StorageFixed$n>]<A, $([<T$i>],)+ N>
            {
                #[inline(always)]
                pub fn new() -> Self {
                    // We assume in a lot of places that u32 can trivially convert to usize
                    num_assert_leq!(std::mem::size_of::<u32>(), std::mem::size_of::<usize>());
                    // N must be less than or equal to MAX_ARCHETYPE_CAPACITY to fit in entities
                    num_assert_leq!(N, MAX_ARCHETYPE_CAPACITY as usize);
                    // N must be strictly less than u32::MAX because of the last free list index
                    num_assert_lt!(N, u32::MAX as usize);

                    Self {
                        len: 0,
                        free_head: 0,
                        slots: new_free_list_slot_array(),
                        entities: Box::new(MaybeUninitArray::new()),
                        $([<d$i>]: RefCell::new(Box::new(MaybeUninitArray::new())),)+
                    }
                }

                #[inline(always)]
                pub const fn len(&self) -> usize {
                    self.len
                }

                #[inline(always)]
                pub const fn is_empty(&self) -> bool {
                    self.len == 0
                }

                #[inline(always)]
                pub const fn capacity(&self) -> usize {
                    N
                }

                /// Reserves a slot in the map pointing to the given dense index.
                /// Returns an entity handle if successful, or None if we're full.
                #[inline(always)]
                pub fn try_push(&mut self, data: ($([<T$i>],)+)) -> Option<Entity<A>> {
                    if self.len >= N {
                        return None;
                    }

                    // We know that self.len < N, and N <= u32::MAX
                    let dense_index: u32 = self.len as u32;
                    let slot_index: u32 = self.free_head;

                    // This means we're at the end of the free list
                    if (slot_index as usize) >= N {
                        return None;
                    }

                    unsafe {
                        debug_assert!((slot_index as usize) < N);
                        debug_assert!(slot_index < MAX_ARCHETYPE_CAPACITY);

                        // SAFETY: We know that slot_index < N
                        let slot = self.slots.get_unchecked_mut(slot_index as usize);
                        // SAFETY: We know that slot_index < MAX_ARCHETYPE_CAPACITY
                        let entity_index = EntityIndex::new_unchecked(slot_index);

                        // NOTE: Do not change the following order of operations!
                        debug_assert!(slot.is_free());
                        self.free_head = slot.index();
                        slot.assign(dense_index);
                        // SAFETY: We know slot_index fits in a usize because N <= u32::MAX.
                        let entity = Entity::new(entity_index, slot.version());
                        let index = self.len;
                        self.len += 1;

                        // SAFETY: We can't overflow because self.len < N.
                        debug_checked_assume!(index < self.len);

                        // SAFETY: We know that index < N and points to an empty cell.
                        self.entities.slice_mut(self.len)[index] = entity;
                        $(self.[<d$i>].get_mut().slice_mut(self.len)[index] = data.$i;)+

                        Some(entity)
                    }
                }

                /// Resolves an entity to an index in the storage data slices.
                /// This index is guaranteed to be in bounds and point to valid data.
                #[inline(always)]
                pub fn resolve(&self, entity: Entity<A>) -> Option<usize> {
                    let (_, dense_index) = match self.resolve_slot(entity) {
                        None => { return None; }
                        Some(found) => found,
                    };

                    let dense_index = dense_index.try_into().unwrap();

                    unsafe {
                        // SAFETY: resolve_slot guarantees this index is valid.
                        // We assume this to help avoid bounds checks later on.
                        debug_checked_assume!(dense_index < self.len);
                    }

                    Some(dense_index)
                }

                /// Removes the given entity from storage if it exists there.
                /// Returns the removed entity's components, if any.
                #[inline(always)]
                pub fn remove(&mut self, entity: Entity<A>) -> Option<($([<T$i>],)*)> {
                    let (slot_index, dense_index) = match self.resolve_slot(entity) {
                        None => { return None; }
                        Some(found) => found,
                    };

                    let result = unsafe {
                        // SAFETY: These are guaranteed by resolve_slot to be in range.
                        let slot_index_usize: usize = slot_index.try_into().unwrap();
                        let dense_index_usize: usize = dense_index.try_into().unwrap();

                        debug_assert!(self.len > 0);
                        debug_assert!(self.len <= N);
                        debug_assert!(slot_index_usize <= N);
                        debug_assert!(dense_index_usize <= N);
                        debug_assert!(dense_index_usize < self.len);

                        let entities = self.entities.slice(self.len);
                        debug_assert!(entities.len() == self.len);
                        debug_assert!(entities[dense_index_usize].index() == entity.index());
                        debug_assert!(entities[dense_index_usize].version() == entity.version());

                        // SAFETY: We know that self.len > 0 from the early-out check above.
                        let last_dense_index = self.len - 1;
                        // SAFETY: We know the entity slice has a length of self.len.
                        let last_entity = *entities.get_unchecked(last_dense_index);
                        // SAFETY: We guarantee that stored entities point to valid slots.
                        let last_slot_index: usize = last_entity.index().try_into().unwrap();

                        // Perform the swap_remove on our data to drop the target entity.
                        // SAFETY: We guarantee that non-free slots point to valid dense data.
                        self.entities.swap_remove(dense_index_usize, self.len);
                        let result =
                            ($(self.[<d$i>].get_mut().swap_remove(dense_index_usize, self.len),)+);

                        // NOTE: Order matters here to support the (target == last) case!
                        // Fix up the slot pointing to the last entity
                        self.slots
                            .get_unchecked_mut(last_slot_index) // SAFETY: See declaration.
                            .assign(dense_index);
                        // Return the target slot to the free list
                        self.slots
                            .get_unchecked_mut(slot_index_usize) // SAFETY: See declaration.
                            .release(self.free_head)
                            .expect("slot version overflow"); // TODO: Orphan this slot?

                        result
                    };

                    // TODO: We shouldn't add a slot to the free list if its version number
                    // is maxed out. It would be better to orphan that slot to avoid issues.

                    // Update the free list head
                    self.free_head = entity.index();
                    self.len -= 1;

                    Some(result)
                }

                /// Populates a slice struct with slices to our stored data.
                #[inline(always)]
                pub fn get_all_slices<'a, S: [<Slices$n>]<'a, A, $([<T$i>],)+>>(&'a mut self,) -> S {
                    unsafe {
                        // SAFETY: We guarantee that the storage is valid up to self.len.
                        S::new(
                            self.entities.slice(self.len),
                            $(self.[<d$i>].get_mut().slice_mut(self.len)),+
                        )
                    }
                }

                /// Gets a read-only slice of our currently stored entity handles.
                #[inline(always)]
                pub fn get_slice_entities(&self) -> &[Entity<A>] {
                    unsafe {
                        // SAFETY: We guarantee that the storage is valid up to self.len.
                        self.entities.slice(self.len)
                    }
                }

                $(
                    /// Gets a slice of the given component index.
                    #[inline(always)]
                    pub fn [<get_slice_$i>](&mut self) -> &[[<T$i>]] {
                        unsafe {
                            // SAFETY: We guarantee that the storage is valid up to self.len.
                            self.[<d$i>].get_mut().slice(self.len)
                        }
                    }

                    /// Gets a slice of the given component index.
                    #[inline(always)]
                    pub fn [<get_slice_mut_$i>](&mut self) -> &mut [[<T$i>]] {
                        unsafe {
                            // SAFETY: We guarantee that the storage is valid up to self.len.
                            self.[<d$i>].get_mut().slice_mut(self.len)
                        }
                    }

                    /// Borrows the slice of the given component index.
                    #[inline(always)]
                    pub fn [<borrow_slice_$i>](&self) -> Ref<[[<T$i>]]> {
                        Ref::map(self.[<d$i>].borrow(), |slice| unsafe {
                            // SAFETY: We guarantee that the storage is valid up to self.len.
                            slice.slice(self.len)
                        })
                    }

                    /// Mutably borrows the slice of the given component index.
                    #[inline(always)]
                    pub fn [<borrow_slice_mut_$i>](&self) -> RefMut<[[<T$i>]]> {
                        RefMut::map(self.[<d$i>].borrow_mut(), |slice| unsafe {
                            // SAFETY: We guarantee that the storage is valid up to self.len.
                            slice.slice_mut(self.len)
                        })
                    }
                )+

                /// Resolves the slot index and data index for a given entity.
                /// Both indices are guaranteed to point to valid cells.
                #[inline(always)]
                fn resolve_slot(&self, entity: Entity<A>) -> Option<(u32, u32)> {
                    // Nothing to resolve if we have nothing stored
                    debug_assert!(self.len <= N);
                    if self.len == 0 {
                        return None;
                    }

                    // Get the index into the slot array from the entity.
                    let slot_index = entity.index();

                    // NOTE: It's a little silly, but we don't actually know if this entity
                    // was created by this map, so we can't assume internal consistency here.
                    // We'll just have to take the small hit for bounds checking on the index.
                    let slot = self.slots[slot_index as usize];

                    if (slot.version() != entity.version()) || slot.is_free() {
                        return None; // Invalid entity handle, fail the lookup
                    }

                    // Get the index into the dense array from the slot.
                    let dense_index = slot.index();

                    Some((slot_index, dense_index))
                }
            }

            impl<A: Archetype, $([<T$i>],)+ const N: usize> Drop
                for [<StorageFixed$n>]<A, $([<T$i>],)+ N>
            {
                #[inline(always)]
                fn drop(&mut self) {
                    // SAFETY: We guarantee that the storage is valid up to self.len.
                    unsafe {
                        $(self.[<d$i>].get_mut().drop_to(self.len);)+
                        // We don't need to drop the other stuff since it's all trivial.
                    };
                }
            }

            impl<A: Archetype, $([<T$i>],)+ const N: usize> Default
                for [<StorageFixed$n>]<A, $([<T$i>],)+ N>
            {
                #[inline(always)]
                fn default() -> Self {
                    [<StorageFixed$n>]::new()
                }
            }
        }
    };
}

declare_dense_fixed_n!(1, 0);
declare_dense_fixed_n!(2, 0, 1);
declare_dense_fixed_n!(3, 0, 1, 2);
declare_dense_fixed_n!(4, 0, 1, 2, 3);
declare_dense_fixed_n!(5, 0, 1, 2, 3, 4);
declare_dense_fixed_n!(6, 0, 1, 2, 3, 4, 5);
declare_dense_fixed_n!(7, 0, 1, 2, 3, 4, 5, 6);
declare_dense_fixed_n!(8, 0, 1, 2, 3, 4, 5, 6, 7);
declare_dense_fixed_n!(9, 0, 1, 2, 3, 4, 5, 6, 7, 8);
declare_dense_fixed_n!(10, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
declare_dense_fixed_n!(11, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
declare_dense_fixed_n!(12, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
declare_dense_fixed_n!(13, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
declare_dense_fixed_n!(14, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13);
declare_dense_fixed_n!(15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14);
declare_dense_fixed_n!(16, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);

#[inline(always)]
fn new_free_list_slot_array<const N: usize>() -> Box<[Slot; N]> {
    num_assert_lt!(N, u32::MAX as usize); // Prevent overflow

    // NOTE: The last slot will point off the end of the free list.
    // In practice, we should never read this value anyway, since
    // we'd only ask to reserve when the dense list has room. However,
    // we may still want to check in the future if we decide to orphan
    // slots with overflowed versions (since we'd then have fewer slots
    // than dense list space). We'll need to revisit this logic if so.
    Box::new(array::from_fn(|i| Slot::new((i as u32) + 1)))
}

struct MaybeUninitArray<T, const N: usize>([MaybeUninit<T>; N]);

impl<T, const N: usize> MaybeUninitArray<T, N> {
    /// Creates a new fully uninitialized array.
    #[inline(always)]
    fn new() -> Self {
        // SAFETY: An uninitialized `[MaybeUninit<_>; LEN]` is valid.
        // Ref: https://doc.rust-lang.org/stable/src/core/mem/maybe_uninit.rs.html#350
        unsafe { Self(MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init()) }
    }

    /// Gets a slice for the range `0..len`.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the following:
    /// - All elements in the array in the range `0..len` are valid data
    /// - `len <= N`
    #[inline(always)]
    unsafe fn slice(&self, len: usize) -> &[T] {
        // SAFETY: Casting `slice` to a `*const [T]` is safe since the caller guarantees that
        // `slice` is initialized, and `MaybeUninit` is guaranteed to have the same layout as `T`.
        // The pointer obtained is valid since it refers to memory owned by `slice` which is a
        // reference and thus guaranteed to be valid for reads.
        // Ref: https://doc.rust-lang.org/stable/src/core/mem/maybe_uninit.rs.html#972
        unsafe {
            debug_checked_assume!(len <= N); // SAFETY: The caller guarantees len <= N.

            &*(self.0.get_unchecked(0..len) as *const [MaybeUninit<T>] as *const [T])
        }
    }

    /// Gets a mutable slice for the range `0..len`.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the following:
    /// - All elements in the range `0..len` are initialized
    /// - `len <= N`
    #[inline(always)]
    unsafe fn slice_mut(&mut self, len: usize) -> &mut [T] {
        // SAFETY: Similar to safety notes for `assume_init_slice`, but we have a
        // mutable reference which is also guaranteed to be valid for writes.
        // Ref: https://doc.rust-lang.org/stable/src/core/mem/maybe_uninit.rs.html#994
        unsafe {
            debug_checked_assume!(len <= N); // SAFETY: The caller guarantees len <= N.

            &mut *(self.0.get_unchecked_mut(0..len) as *mut [MaybeUninit<T>] as *mut [T])
        }
    }

    /// Drops the element at `index` and replaces it with the last element in `0..len`.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the following:
    /// - All elements in the range `0..len` are initialized
    /// - `len <= N`
    /// - `len > 0`
    /// - `index < N`
    /// - `index < len`
    #[inline(always)]
    unsafe fn swap_remove(&mut self, index: usize, len: usize) -> T {
        unsafe {
            // SAFETY: These are all guaranteed by the caller and stated above.
            debug_checked_assume!(len <= N);
            debug_checked_assume!(len > 0);
            debug_checked_assume!(index < N);
            debug_checked_assume!(index < len);

            // SAFETY: The caller is guaranteeing that the element at index, and
            // the element at len - 1 are both valid. With this guarantee we can
            // safely take the element at index. We then perform a direct pointer
            // copy (we can't assume nonoverlapping here!) from the last element
            // to the one at index. This moves the data, making the data at index
            // initialized to the data at last, and the data at last effectively
            // uninitialized (though bitwise identical to the data at index).
            let last = len - 1;
            let array_ptr = self.0.as_mut_ptr();
            let result = ptr::read(array_ptr.add(index)).assume_init();
            ptr::copy(array_ptr.add(last), array_ptr.add(index), 1);
            *array_ptr.add(last) = MaybeUninit::uninit(); // Hint for Miri
            result
        }
    }

    /// Drops all elements in the range `0..len`.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the following:
    /// - All elements in the range `0..len` are initialized
    /// - `len <= N`
    #[inline(always)]
    unsafe fn drop_to(&mut self, len: usize) {
        unsafe {
            // SAFETY: The caller guarantees len <= N.
            debug_checked_assume!(len <= N);

            for v in self.0[..len].iter_mut() {
                // SAFETY: The caller guarantees v is valid.
                v.assume_init_drop();
                *v = MaybeUninit::uninit(); // Hint for Miri
            }
        };
    }
}
