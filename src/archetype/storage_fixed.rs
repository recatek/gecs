use std::cell::{Ref, RefCell, RefMut};
use std::mem::MaybeUninit;
use std::ptr;

use seq_macro::seq;

use crate::archetype::slices::*;
use crate::archetype::slot::{self, Slot, SlotIndex};
use crate::entity::Entity;
use crate::index::{DataIndex, MAX_DATA_CAPACITY};
use crate::traits::Archetype;
use crate::util::{debug_checked_assume, num_assert_leq};

macro_rules! declare_storage_fixed_n {
    ($name:ident, $slices:ident, $n:literal) => {
        seq!(I in 0..$n {
            pub struct $name<A: Archetype, #(T~I,)* const N: usize> {
                len: usize,
                free_head: SlotIndex,
                slots: DataFixed<Slot, N>, // Sparse
                // No RefCell here since we never grant mutable access externally
                entities: DataFixed<Entity<A>, N>,
                #(d~I: RefCell<DataFixed<T~I, N>>,)*
            }

            impl<A: Archetype, #(T~I,)* const N: usize> $name<A, #(T~I,)* N>
            {
                #[inline(always)]
                pub fn new() -> Self {
                    // We assume in a lot of places that u32 can trivially convert to usize
                    num_assert_leq!(std::mem::size_of::<u32>(), std::mem::size_of::<usize>());
                    // Our data indices must be able to fit inside of entity handles
                    num_assert_leq!(N, MAX_DATA_CAPACITY as usize);

                    let mut slots: DataFixed<Slot, N> = DataFixed::new();
                    // For consistency with the dynamic version.
                    let raw_data = slots.raw_data();
                    let free_head = slot::populate_free_list(DataIndex::zero(), raw_data);

                    Self {
                        len: 0,
                        free_head,
                        slots,
                        entities: DataFixed::new(),
                        #(d~I: RefCell::new(DataFixed::new()),)*
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
                pub fn try_push(&mut self, data: (#(T~I,)*)) -> Option<Entity<A>> {
                    debug_assert!(self.len <= self.capacity());

                    if self.len >= self.capacity() {
                        // If we're full, we should also be at the end of the slot free list.
                        // WARNING: This changes if we decide to orphan version-overflow slots!
                        debug_assert!(self.free_head.is_free_list_end());

                        return None; // Out of space
                    }

                    unsafe {
                        // SAFETY: We will never hit the the free list end if we're below capacity
                        // WARNING: This changes if we decide to orphan version-overflow slots!
                        let slot_index = self.free_head.get_next_free().unwrap_unchecked();
                        // SAFETY: We never let self.len be greater than MAX_DATA_CAPACITY.
                        let dense_index = DataIndex::new_unchecked(self.len as u32);

                        // SAFETY: We know that the slot storage is valid up to our capacity.
                        let slots = self.slots.slice_mut(self.capacity());
                        // SAFETY: We know this is not the end of the free list, and we know that
                        // a free list slot index can never be assigned to an out of bounds value.
                        let slot = slots.get_unchecked_mut(slot_index.get() as usize);

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
                        #(self.d~I.get_mut().write(index, data.I);)*

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
                ///
                /// # Panics
                ///
                /// This function may panic if a slot's generational version overflows,
                /// in order to protect the safety of entity handle lookups. This is an
                /// extremely unlikely occurrence in nearly all programs -- it would only
                /// happen if the exact same lookup slot was rewritten 4.2 billion times.
                #[inline(always)]
                pub fn remove(&mut self, entity: Entity<A>) -> Option<(#(T~I,)*)> {
                    let (slot_index, dense_index) = match self.resolve_slot(entity) {
                        None => { return None; }
                        Some(found) => found,
                    };

                    let result = unsafe {
                        // SAFETY: These are guaranteed by resolve_slot to be in range.
                        let slot_index_usize: usize = slot_index.get() as usize;
                        let dense_index_usize: usize = dense_index.get() as usize;

                        debug_assert!(self.len > 0);
                        debug_assert!(self.len <= self.capacity());
                        debug_assert!(slot_index_usize <= self.capacity());
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
                            (#(self.d~I.get_mut().swap_remove(dense_index_usize, self.len),)*);

                        // SAFETY: We know that the slot storage is valid up to our capacity.
                        let slots = self.slots.slice_mut(self.capacity());

                        // NOTE: Order matters here to support the (target == last) case!
                        // Fix up the slot pointing to the last entity
                        slots
                            .get_unchecked_mut(last_slot_index) // SAFETY: See declaration.
                            .assign(dense_index);
                        // Return the target slot to the free list
                        slots
                            .get_unchecked_mut(slot_index_usize) // SAFETY: See declaration.
                            .release(self.free_head)
                            .expect("slot version overflow"); // Wow, 4 billion slot writes!
                        // NOTE: We could orphan this slot instead, but this would break the
                        // assumptions we make elsewhere (see the WARNING in try_push, etc.)
                        // that the end of the free list and len == capacity are synonymous.
                        // Enabling slot orphaning would create a lot of edge cases to fix.

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
                pub fn get_all_slices<'a, S: $slices<'a, A, #(T~I,)*>>(&'a mut self,) -> S {
                    unsafe {
                        // SAFETY: We guarantee that the storage is valid up to self.len.
                        S::new(
                            self.entities.slice(self.len),
                            #(self.d~I.get_mut().slice_mut(self.len),)*
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

                #(
                    /// Gets a slice of the given component index.
                    #[inline(always)]
                    pub fn get_slice_~I(&mut self) -> &[T~I] {
                        unsafe {
                            // SAFETY: We guarantee that the storage is valid up to self.len.
                            self.d~I.get_mut().slice(self.len)
                        }
                    }

                    /// Gets a slice of the given component index.
                    #[inline(always)]
                    pub fn get_slice_mut_~I(&mut self) -> &mut [T~I] {
                        unsafe {
                            // SAFETY: We guarantee that the storage is valid up to self.len.
                            self.d~I.get_mut().slice_mut(self.len)
                        }
                    }

                    /// Borrows the slice of the given component index.
                    #[inline(always)]
                    pub fn borrow_slice_~I(&self) -> Ref<[T~I]> {
                        Ref::map(self.d~I.borrow(), |slice| unsafe {
                            // SAFETY: We guarantee that the storage is valid up to self.len.
                            slice.slice(self.len)
                        })
                    }

                    /// Mutably borrows the slice of the given component index.
                    #[inline(always)]
                    pub fn borrow_slice_mut_~I(&self) -> RefMut<[T~I]> {
                        RefMut::map(self.d~I.borrow_mut(), |slice| unsafe {
                            // SAFETY: We guarantee that the storage is valid up to self.len.
                            slice.slice_mut(self.len)
                        })
                    }
                )*

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
                        if slot_index_usize >= self.capacity() {
                            panic!("entity handle is invalid for this archetype");
                        }

                        // SAFETY: We know that the slot storage is valid up to our capacity.
                        let slots = self.slots.slice(self.capacity());
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

            impl<A: Archetype, #(T~I,)* const N: usize> Drop
                for $name<A, #(T~I,)* N>
            {
                #[inline(always)]
                fn drop(&mut self) {
                    // SAFETY: We guarantee that the storage is valid up to self.len.
                    unsafe {
                        #(self.d~I.get_mut().drop_to(self.len);)*
                        // We don't need to drop the other stuff since it's all trivial.
                    };
                }
            }

            impl<A: Archetype, #(T~I,)* const N: usize> Default
                for $name<A, #(T~I,)* N>
            {
                #[inline(always)]
                fn default() -> Self {
                    $name::new()
                }
            }
        });
    };
}

// Declare storage for up to 16 components.
seq!(N in 1..=16 {
    declare_storage_fixed_n!(StorageFixed~N, Slices~N, N);
});

// Declare additional storage for up to 32 components.
#[cfg(feature = "more_components")]
seq!(N in 17..=32 {
    declare_storage_fixed_n!(StorageFixed~N, Slices~N, N);
});

struct DataFixed<T, const N: usize>(Box<[MaybeUninit<T>; N]>);

impl<T, const N: usize> DataFixed<T, N> {
    /// Creates a new fully uninitialized array.
    #[inline(always)]
    #[rustfmt::skip]
    fn new() -> Self {
        let mut v = Vec::with_capacity(N);
        // SAFETY: MaybeUninit is always trivially initialized.
        unsafe { v.set_len(N) };
        Self( v.try_into().unwrap())
    }

    /// Gets the raw stored data as a mutable `MaybeUninit<T>` slice.
    #[inline(always)]
    fn raw_data(&mut self) -> &mut [MaybeUninit<T>] {
        &mut (*self.0)
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
            debug_assert!(index < N);

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
            debug_assert!(len <= N);

            // SAFETY: Casting a `[MaybeUninit<T>]` to a `[T]` is safe because the caller
            // guarantees that this portion of the data is valid and `MaybeUninit<T>` is
            // guaranteed to have the same layout as `T`. The pointer obtained is valid
            // since it refers to memory owned by `slice` which is a reference and thus
            // guaranteed to be valid for reads.
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
            debug_assert!(len <= N);

            // SAFETY: Similar to safety notes for `slice`, but we have a mutable reference
            // which is also guaranteed to be valid for writes.
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
            debug_assert!(len <= N);
            debug_assert!(len > 0);
            debug_assert!(index < N);
            debug_assert!(index < len);

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
            debug_assert!(len <= N);

            for i in 0..len {
                let i_ptr = self.0.as_mut_ptr().add(i);
                // SAFETY: The caller guarantees this element is valid.
                ptr::drop_in_place(i_ptr as *mut T);
                ptr::write(i_ptr, MaybeUninit::uninit()); // Hint for Miri
            }
        };
    }
}
