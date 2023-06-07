use std::cell::{Ref, RefCell, RefMut};
use std::mem::MaybeUninit;
use std::ptr;

use paste::paste;

use crate::archetype::slices::*;
use crate::archetype::slot::{Slot, SlotIndex};
use crate::entity::{Entity};
use crate::traits::Archetype;
use crate::util::{debug_checked_assume, num_assert_leq};
use crate::index::{DataIndex, MAX_DATA_CAPACITY};

macro_rules! declare_storage_fixed_n {
    ($n:literal, $($i:literal),+) => {
        paste! {
            pub struct [<StorageFixed$n>]<A: Archetype, $([<T$i>],)+ const N: usize> {
                len: usize,
                free_head: SlotIndex,
                slots: Box<[Slot; N]>, // Sparse
                // No RefCell here since we never grant mutable access externally
                entities: DataFixed<Entity<A>, N>,
                $([<d$i>]: RefCell<DataFixed<[<T$i>], N>>,)+
            }

            impl<A: Archetype, $([<T$i>],)+ const N: usize> [<StorageFixed$n>]<A, $([<T$i>],)+ N>
            {
                #[inline(always)]
                pub fn new() -> Self {
                    // We assume in a lot of places that u32 can trivially convert to usize
                    num_assert_leq!(std::mem::size_of::<u32>(), std::mem::size_of::<usize>());
                    // N must be less than or equal to MAX_DATA_CAPACITY to fit in entities
                    num_assert_leq!(N, MAX_DATA_CAPACITY as usize);

                    let (slots, free_head) = new_slot_array::<N>();

                    Self {
                        len: 0,
                        free_head,
                        slots,
                        entities: DataFixed::new(),
                        $([<d$i>]: RefCell::new(DataFixed::new()),)+
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
                        return None; // Out of space
                    }

                    unsafe {
                        debug_assert!(self.len < MAX_DATA_CAPACITY as usize);

                        // SAFETY: We will never hit the the free list end if we're below capacity
                        // WARNING: This changes if we decide to orphan version-overflow slots!
                        let slot_index = self.free_head.get_next_free().unwrap_unchecked();
                        // SAFETY: We never let self.len be greater than MAX_DATA_CAPACITY.
                        let dense_index = DataIndex::new_unchecked(self.len as u32);

                        // SAFETY: We know this is not the end of the free list, and we know that
                        // a free list slot index can never be assigned to an out of bounds value.
                        let slot = self.slots.get_unchecked_mut(slot_index.get() as usize);

                        // NOTE: Do not change the following order of operations!
                        debug_assert!(slot.is_free());
                        self.free_head = slot.index();
                        slot.assign(dense_index);
                        let entity = Entity::new(slot_index, slot.version());
                        let index = self.len;
                        self.len += 1;

                        // SAFETY: We can't overflow because self.len < N.
                        debug_checked_assume!(index < self.len);

                        // SAFETY: We know that index < N and points to an empty cell.
                        self.entities.write(index, entity);
                        $(self.[<d$i>].get_mut().write(index, data.$i);)+

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

                    let dense_index_usize = dense_index.get() as usize;

                    unsafe {
                        // SAFETY: resolve_slot guarantees this index is valid.
                        // We assume this to help avoid bounds checks later on.
                        debug_checked_assume!(dense_index_usize < self.len);
                    }

                    Some(dense_index_usize)
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
                        let slot_index_usize: usize = slot_index.get() as usize;
                        let dense_index_usize: usize = dense_index.get() as usize;

                        debug_assert!(self.len > 0);
                        debug_assert!(self.len <= N);
                        debug_assert!(slot_index_usize <= N);
                        debug_assert!(dense_index_usize < self.len);

                        let entities = self.entities.slice(self.len);
                        debug_assert!(entities.len() == self.len);
                        debug_assert!(entities[dense_index_usize].index() == entity.index());
                        debug_assert!(entities[dense_index_usize].version() == entity.version());

                        // SAFETY: We know self.len > 0 because we got Some from resolve_slot.
                        let last_dense_index = self.len - 1;
                        // SAFETY: We know the entity slice has a length of self.len.
                        let last_entity = *entities.get_unchecked(last_dense_index);
                        // SAFETY: We guarantee that stored entities point to valid slots.
                        let last_slot_index: usize = last_entity.index().get() as usize;

                        // Perform the swap_remove on our data to drop the target entity.
                        // SAFETY: We guarantee that non-free slots point to valid dense data.
                        self.entities.swap_remove(dense_index_usize, self.len);
                        let result =
                            ($(self.[<d$i>].get_mut().swap_remove(dense_index_usize, self.len),)+);

                        // For consistency with StorageDynamic's remove function
                        let slots = &mut self.slots;

                        // NOTE: Order matters here to support the (target == last) case!
                        // Fix up the slot pointing to the last entity
                        slots
                            .get_unchecked_mut(last_slot_index) // SAFETY: See declaration.
                            .assign(dense_index);
                        // Return the target slot to the free list
                        slots
                            .get_unchecked_mut(slot_index_usize) // SAFETY: See declaration.
                            .release(self.free_head)
                            .expect("slot version overflow"); // TODO: Orphan this slot?

                        result
                    };

                    // TODO: We shouldn't add a slot to the free list if its version number
                    // is maxed out. It would be better to orphan that slot to avoid issues.

                    // Update the free list head
                    self.free_head = SlotIndex::new_free(entity.index());
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
                fn resolve_slot(&self, entity: Entity<A>) -> Option<(DataIndex, DataIndex)> {
                    // Nothing to resolve if we have nothing stored
                    if self.len == 0 {
                        return None;
                    }

                    // Get the index into the slot array from the entity.
                    let slot_index = entity.index();

                    unsafe {
                        let slot_index_usize = slot_index.get() as usize;

                        // NOTE: It's a little silly, but we don't actually know if this entity
                        // was created by this map, so we can't assume internal consistency here.
                        // We'll just have to take the small hit for bounds checking on the index.
                        if slot_index_usize >= N {
                            panic!("entity handle is invalid for this archetype");
                        }

                        // For consistency with StorageDynamic's resolve_slot function.
                        let slots = &self.slots;
                        // SAFETY: We know slot_index_usize is within bounds due to the panic above.
                        let slot = slots.get_unchecked(slot_index_usize);

                        // NOTE: For similar reasons above, a crossed-wires entity handle from another
                        // world could miraculously have the correct version while pointing to a freed
                        // slot. This could cause some wacky memory access, so we need to allow slots
                        // to be explicitly identified as free or not. Again, this has a small cost.
                        if (slot.version() != entity.version()) || slot.is_free() {
                            return None; // Stale entity handle, fail the lookup
                        }

                        // SAFETY: We know that this is not a free slot due to the check above.
                        let dense_index = slot.index().get_data().unwrap_unchecked();

                        Some((slot_index, dense_index))
                    }
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

declare_storage_fixed_n!(1, 0);
declare_storage_fixed_n!(2, 0, 1);
declare_storage_fixed_n!(3, 0, 1, 2);
declare_storage_fixed_n!(4, 0, 1, 2, 3);
declare_storage_fixed_n!(5, 0, 1, 2, 3, 4);
declare_storage_fixed_n!(6, 0, 1, 2, 3, 4, 5);
declare_storage_fixed_n!(7, 0, 1, 2, 3, 4, 5, 6);
declare_storage_fixed_n!(8, 0, 1, 2, 3, 4, 5, 6, 7);
declare_storage_fixed_n!(9, 0, 1, 2, 3, 4, 5, 6, 7, 8);
declare_storage_fixed_n!(10, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
declare_storage_fixed_n!(11, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
declare_storage_fixed_n!(12, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
declare_storage_fixed_n!(13, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
declare_storage_fixed_n!(14, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13);
declare_storage_fixed_n!(15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14);
declare_storage_fixed_n!(16, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);

pub(crate) fn new_slot_array<const N: usize>() -> (Box<[Slot; N]>, SlotIndex) {
    // Prevent u32 overflow when we add 1 to every u32 value between 1 and N-1.
    num_assert_leq!(N, u32::MAX as usize);
    // Make sure we aren't trying to make more slots than we can reference.
    num_assert_leq!(N, MAX_DATA_CAPACITY as usize);

    // TODO: This should be done as a direct heap allocation to avoid stack overflow.
    let mut uninit = Box::new([MaybeUninit::<Slot>::uninit(); N]);

    for i in 0..(N - 1) {
        // SAFETY: We know N is less than MAX_DATA_CAPACITY and won't overflow
        // during u32 addition because of the two compile-time asserts above.
        let index = unsafe { DataIndex::new_unchecked((i as u32) + 1) };
        uninit[i].write(Slot::new_free(SlotIndex::new_free(index)));
    }

    let free_list_head = if N > 0 {
        // Set up the end of the free list with the special end index.
        uninit[N - 1].write(Slot::new_free(SlotIndex::new_free_end()));

        // Point the free list head to the front of the list.
        SlotIndex::new_free(DataIndex::zero())
    } else {
        // Otherwise, we have nothing, so point the free list head to the end.
        SlotIndex::new_free_end()
    };

    // Convert the MaybeUninit slot array to an assumed-init slot array.
    // SAFETY: We have written to every value in the uninitialized array.
    let slots = unsafe { Box::from_raw(Box::into_raw(uninit).cast()) };

    (slots, free_list_head)
}


struct DataFixed<T, const N: usize>(Box<[MaybeUninit<T>; N]>);

impl<T, const N: usize> DataFixed<T, N> {
    /// Creates a new fully uninitialized array.
    #[inline(always)]
    #[rustfmt::skip]
    fn new() -> Self {
        // TODO: This should be done as a direct heap allocation to avoid stack overflow.
        unsafe { 
            // SAFETY: An uninitialized `[MaybeUninit<_>; LEN]` is valid.
            // Ref: https://doc.rust-lang.org/stable/src/core/mem/maybe_uninit.rs.html#350
            Self(Box::new(MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init())) 
        }
    }

    /// Writes an element to the given index.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the following:
    /// - `index <= N`
    /// - The element at `index` is not currently initialized
    #[inline(always)]
    unsafe fn write(&mut self, index: usize, val: T) {
        unsafe {
            // SAFETY: The caller guarantees index <= N.
            self.0.get_unchecked_mut(index).write(val);
        }
    }

    /// Gets a slice for the range `0..len`.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the following:
    /// - All elements in the array in the range `0..len` are initialized
    /// - `len <= N`
    #[inline(always)]
    unsafe fn slice(&self, len: usize) -> &[T] {
        unsafe {
            debug_checked_assume!(len <= N); // SAFETY: The caller guarantees len <= N.

            // SAFETY: Casting `slice` to a `*const [T]` is safe since the caller guarantees that
            // `slice` is initialized, and `MaybeUninit` is guaranteed to have the same layout as
            // `T`. The pointer obtained is valid since it refers to memory owned by `slice` which
            // is a reference and thus guaranteed to be valid for reads.
            // Ref: https://doc.rust-lang.org/stable/src/core/mem/maybe_uninit.rs.html#972
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
        unsafe {
            debug_checked_assume!(len <= N); // SAFETY: The caller guarantees len <= N.

            // SAFETY: Similar to safety notes for `assume_init_slice`, but we have a
            // mutable reference which is also guaranteed to be valid for writes.
            // Ref: https://doc.rust-lang.org/stable/src/core/mem/maybe_uninit.rs.html#994
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
            for i in 0..len {
                let i_ptr = self.0.as_mut_ptr().add(i);
                // SAFETY: The caller guarantees this element is valid.
                ptr::drop_in_place(i_ptr as *mut T);
                ptr::write(i_ptr, MaybeUninit::uninit()); // Hint for Miri
            }
        };
    }
}
