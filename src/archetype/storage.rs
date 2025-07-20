use std::alloc::{self, Layout};
use std::cell::{Ref, RefCell, RefMut};
use std::marker::PhantomData;
use std::mem::{self, MaybeUninit};
use std::ptr::{self, NonNull};
use std::slice;

use seq_macro::seq;

use crate::archetype::components::*;
use crate::archetype::iter::*;
use crate::archetype::slices::*;
use crate::archetype::slot::{Slot, SlotIndex};
use crate::archetype::view::*;
use crate::entity::{Entity, EntityDirect};
use crate::index::{TrimmedIndex, MAX_DATA_CAPACITY};
use crate::traits::{Archetype, EntityKey, StorageCanResolve};
use crate::util::debug_checked_assume;
use crate::version::ArchetypeVersion;

macro_rules! declare_storage_n {
    (
        $name:ident,
        $borrow:ident,
        $iter:ident,
        $iter_mut:ident,
        $components:ident,
        $slices:ident,
        $view:ident,
        $view_mut:ident,
        $n:literal
    ) => {
        seq!(I in 0..$n {
            pub struct $name<A: Archetype, #(T~I,)*> {
                version: ArchetypeVersion,
                len: usize,
                capacity: usize,
                free_head: SlotIndex,
                slots: DataPtr<Slot>, // Sparse
                // No RefCell here since we never grant mutable access externally
                entities: DataPtr<Entity<A>>,
                #(d~I: RefCell<DataPtr<T~I>>,)*

                #[cfg(feature = "events")]
                created: Vec<Entity<A>>,
                #[cfg(feature = "events")]
                destroyed: Vec<Entity<A>>,
            }

            impl<A: Archetype, #(T~I,)*> $name<A, #(T~I,)*>
            where
                A::Components: $components<#(T~I,)*>,
            {
                #[inline(always)]
                pub fn new() -> Self {
                    Self::with_capacity(0)
                }

                #[inline(always)]
                pub fn with_capacity(capacity: usize) -> Self {
                    const {
                        assert!(
                            mem::size_of::<u32>() <= mem::size_of::<usize>(),
                            "unsupported architecture (usize too small)",
                        );
                    }

                    // Our data indices must be able to fit inside of entity handles
                    if capacity > MAX_DATA_CAPACITY as usize {
                        panic!("capacity may not exceed {}", MAX_DATA_CAPACITY);
                    }

                    let mut slots: DataPtr<Slot> = DataPtr::with_capacity(capacity);
                    // SAFETY: We just allocated the slot array with this capacity.
                    let raw_data = unsafe { slots.raw_data(capacity) };
                    let free_head = Slot::populate_free_list(TrimmedIndex::zero(), raw_data);

                    Self {
                        version: ArchetypeVersion::start(),
                        len: 0,
                        capacity,
                        free_head,
                        slots,
                        entities: DataPtr::with_capacity(capacity),
                        #(d~I: RefCell::new(DataPtr::with_capacity(capacity)),)*

                        #[cfg(feature = "events")]
                        created: Vec::new(), // Shouldn't initially allocate
                        #[cfg(feature = "events")]
                        destroyed: Vec::new(), // Shouldn't initially allocate
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
                    self.capacity
                }

                /// The overall version for this data structure. Used for raw indices.
                #[inline(always)]
                pub const fn version(&self) -> ArchetypeVersion {
                    self.version
                }

                #[cfg(feature = "events")]
                pub fn created(&self) -> &[Entity<A>] {
                    &self.created
                }

                #[cfg(feature = "events")]
                pub fn destroyed(&self) -> &[Entity<A>] {
                    &self.destroyed
                }

                #[cfg(feature = "events")]
                pub fn clear_events(&mut self) {
                    self.created.clear();
                    self.destroyed.clear();
                }

                /// Adds a new entity with the given components to this storage.
                /// Returns a typed entity handle pointing to the added element.
                ///
                /// # Panics
                ///
                /// Panics if the storage can no longer expand to accommodate the new data.
                #[inline(always)]
                pub fn push<D: $components<#(T~I,)*>>(
                    &mut self,
                    data: D,
                ) -> Entity<A> {
                    debug_assert!(self.len <= self.capacity());

                    if self.len >= self.capacity() {
                        // If we're full, we should also be at the end of the slot free list.
                        debug_assert!(self.free_head.is_free_end());

                        if self.grow() == false {
                            panic!("capacity overflow");
                        }
                    }

                    unsafe { self.force_create(data) }
                }

                /// Adds a new entity if there is sufficient spare capacity to store it.
                /// Returns a typed entity handle pointing to the added element.
                ///
                /// Unlike `push` this method will not reallocate when there is insufficient
                /// capacity. Instead, it will return an error along with given components.
                #[inline(always)]
                pub fn push_within_capacity<D: $components<#(T~I,)*>>(
                    &mut self,
                    data: D,
                ) -> Result<Entity<A>, D> {
                    debug_assert!(self.len <= self.capacity());

                    if self.len >= self.capacity() {
                        // If we're full, we should also be at the end of the slot free list.
                        debug_assert!(self.free_head.is_free_end());

                        return Err(data);
                    }

                    Ok(unsafe { self.force_create(data) })
                }

                /// Removes the given entity from storage if it exists there.
                /// Returns the removed entity's components, if any.
                ///
                /// This effectively destroys the entity by invalidating all handles to it.
                ///
                /// # Panics
                ///
                /// This function may panic if a slot's generational version overflows,
                /// in order to protect the safety of entity handle lookups. This is an
                /// extremely unlikely occurrence in nearly all programs -- it would only
                /// happen if the exact same lookup slot was rewritten 4.2 billion times.
                #[inline(always)]
                pub fn destroy<K: EntityKey>(&mut self, entity: K) -> K::DestroyOutput
                where
                    Self: StorageCanResolve<K>
                {
                    <Self as StorageCanResolve<K>>::resolve_destroy(self, entity)
                }

                /// Resolves an entity key to an index in the storage data slices.
                /// This index is guaranteed to be in bounds and point to valid data.
                #[inline(always)]
                pub fn resolve<K: EntityKey>(&self, entity: K) -> Option<usize>
                where
                    Self: StorageCanResolve<K>
                {
                    <Self as StorageCanResolve<K>>::resolve_for(self, entity)
                }

                /// Converts an entity key to direct entity that bypasses slot lookup.
                #[inline(always)]
                pub fn to_direct<K: EntityKey>(&self, entity: K) -> Option<K::DirectOutput>
                where
                    Self: StorageCanResolve<K>
                {
                    <Self as StorageCanResolve<K>>::resolve_direct(self, entity)
                }

                /// Creates a borrow context to accelerate accessing borrowed data for an entity.
                #[inline(always)]
                pub fn begin_borrow<K: EntityKey>(
                    &self,
                    entity: K
                ) -> Option<$borrow<'_, A, #(T~I,)*>>
                where
                    Self: StorageCanResolve<K>
                {
                    self.resolve(entity).map(|index| $borrow { index, source: self })
                }

                /// Creates a borrow context to accelerate accessing borrowed data for an entity.
                /// This version also returns an EntityDirect to the direct dense position.
                #[inline(always)]
                pub fn begin_borrow_direct<K: EntityKey>(
                    &self,
                    entity: K
                ) -> Option<($borrow<'_, A, #(T~I,)*>, EntityDirect<A>)>
                where
                    Self: StorageCanResolve<K>
                {
                    self.resolve(entity).map(|index| unsafe {
                        let dense_index = TrimmedIndex::new_usize(index);
                        debug_assert!(dense_index.is_some());
                        // SAFETY: We know that the index was resolved to a valid trimmed index.
                        let dense_index = dense_index.unwrap_unchecked();
                        let version = self.version();

                        (
                            $borrow { index, source: self },
                            EntityDirect::<A>::new(dense_index, version),
                        )
                    })
                }

                /// Returns an iterator over all of the entities and their data.
                #[inline]
                pub fn iter<'a, V: $view<'a, A, #(T~I,)*> + 'a>(
                    &'a mut self,
                ) -> impl Iterator<Item = V> + use<'a, V, A, #(T~I,)*> {
                    unsafe {
                        // SAFETY: We've initialized all data by this point and won't exceed self.len.
                        $iter {
                            remaining: self.len,
                            ptr_entity: self.entities.ptr_data(),
                            #(ptr_d~I: self.d~I.get_mut().ptr_data(),)*
                            phantom: PhantomData,
                        }
                    }
                }

                /// Returns a mutable iterator over all of the entities and their data.
                #[inline]
                pub fn iter_mut<'a, V: $view_mut<'a, A, #(T~I,)*> + 'a>(
                    &'a mut self,
                ) -> impl Iterator<Item = V> + use<'a, V, A, #(T~I,)*> {
                    unsafe {
                        // SAFETY: We've initialized all data by this point and won't exceed self.len.
                        $iter_mut {
                            remaining: self.len,
                            ptr_entity: self.entities.ptr_data(),
                            #(ptr_d~I: self.d~I.get_mut().ptr_data(),)*
                            phantom: PhantomData,
                        }
                    }
                }

                /// Populates a view struct with our stored data for the given entity key.
                #[inline(always)]
                pub fn get_view<'a, E: $view<'a, A, #(T~I,)*>, K: EntityKey>(
                    &'a mut self,
                    entity: K,
                ) -> Option<E>
                where
                    Self: StorageCanResolve<K>
                {
                    self.resolve(entity).map(|index| unsafe {
                        // SAFETY: We guarantee that if we can resolve, then index < self.len.
                        // SAFETY: We guarantee that the storage is valid up to self.len.
                        E::new(
                            self.entities.slice(self.len).get_unchecked(index),
                            #(self.d~I.get_mut().slice(self.len).get_unchecked(index),)*
                        )
                    })
                }

                /// Populates a view struct with our stored data for the given entity key.
                /// This version also returns an EntityDirect to the direct dense position.
                #[inline(always)]
                pub fn get_view_direct<'a, E: $view<'a, A, #(T~I,)*>, K: EntityKey>(
                    &'a mut self,
                    entity: K,
                ) -> Option<(E, EntityDirect<A>)>
                where
                    Self: StorageCanResolve<K>
                {
                    self.resolve(entity).map(|index| unsafe {
                        let dense_index = TrimmedIndex::new_usize(index);
                        debug_assert!(dense_index.is_some());
                        // SAFETY: We know that the index was resolved to a valid trimmed index.
                        let dense_index = dense_index.unwrap_unchecked();
                        let version = self.version();

                        // SAFETY: We guarantee that if we can resolve, then index < self.len.
                        // SAFETY: We guarantee that the storage is valid up to self.len.
                        (
                            E::new(
                                self.entities.slice(self.len).get_unchecked(index),
                                #(self.d~I.get_mut().slice(self.len).get_unchecked(index),)*
                            ),
                            EntityDirect::<A>::new(dense_index, version),
                        )
                    })
                }

                /// Populates a mutable view struct with our stored data for the given entity key.
                #[inline(always)]
                pub fn get_view_mut<'a, E: $view_mut<'a, A, #(T~I,)*>, K: EntityKey>(
                    &'a mut self,
                    entity: K,
                ) -> Option<E>
                where
                    Self: StorageCanResolve<K>
                {
                    self.resolve(entity).map(|index| unsafe {
                        // SAFETY: We guarantee that if we can resolve, then index < self.len.
                        // SAFETY: We guarantee that the storage is valid up to self.len.
                        E::new(
                            self.entities.slice(self.len).get_unchecked(index),
                            #(self.d~I.get_mut().slice_mut(self.len).get_unchecked_mut(index),)*
                        )
                    })
                }

                /// Populates a mutable view struct with our stored data for the given entity key.
                /// This version also returns an EntityDirect to the direct dense position.
                #[inline(always)]
                pub fn get_view_mut_direct<'a, E: $view_mut<'a, A, #(T~I,)*>, K: EntityKey>(
                    &'a mut self,
                    entity: K,
                ) -> Option<(E, EntityDirect<A>)>
                where
                    Self: StorageCanResolve<K>
                {
                    self.resolve(entity).map(|index| unsafe {
                        let dense_index = TrimmedIndex::new_usize(index);
                        debug_assert!(dense_index.is_some());
                        // SAFETY: We know that the index was resolved to a valid trimmed index.
                        let dense_index = dense_index.unwrap_unchecked();
                        let version = self.version();

                        // SAFETY: We guarantee that if we can resolve, then index < self.len.
                        // SAFETY: We guarantee that the storage is valid up to self.len.
                        (
                            E::new(
                                self.entities.slice(self.len).get_unchecked(index),
                                #(self.d~I.get_mut().slice_mut(self.len).get_unchecked_mut(index),)*
                            ),
                            EntityDirect::<A>::new(dense_index, version),
                        )
                    })
                }

                /// Populates a slice struct with slices to our stored data.
                #[inline(always)]
                pub fn get_all_slices_mut<'a, S: $slices<'a, A, #(T~I,)*>>(&'a mut self,) -> S
                {
                    unsafe {
                        debug_checked_assume!(self.len <= MAX_DATA_CAPACITY as usize);
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
                        debug_checked_assume!(self.len <= MAX_DATA_CAPACITY as usize);
                        // SAFETY: We guarantee that the storage is valid up to self.len.
                        self.entities.slice(self.len)
                    }
                }

                #(
                     /// Gets a slice of the given component index.
                    #[inline(always)]
                    pub fn get_slice_~I(&mut self) -> &[T~I] {
                        unsafe {
                            debug_checked_assume!(self.len <= MAX_DATA_CAPACITY as usize);
                            // SAFETY: We guarantee that the storage is valid up to self.len.
                            self.d~I.get_mut().slice(self.len)
                        }
                    }

                    /// Gets a mutable slice of the given component index.
                    #[inline(always)]
                    pub fn get_slice_mut_~I(&mut self) -> &mut [T~I] {
                        unsafe {
                            debug_checked_assume!(self.len <= MAX_DATA_CAPACITY as usize);
                            // SAFETY: We guarantee that the storage is valid up to self.len.
                            self.d~I.get_mut().slice_mut(self.len)
                        }
                    }

                    /// Borrows the slice of the given component index.
                    #[inline(always)]
                    pub fn borrow_slice_~I(&self) -> Ref<'_, [T~I]> {
                        Ref::map(self.d~I.borrow(), |slice| unsafe {
                            debug_checked_assume!(self.len <= MAX_DATA_CAPACITY as usize);
                            // SAFETY: We guarantee that the storage is valid up to self.len.
                            slice.slice(self.len)
                        })
                    }

                    /// Mutably borrows the slice of the given component index.
                    #[inline(always)]
                    pub fn borrow_slice_mut_~I(&self) -> RefMut<'_, [T~I]> {
                        RefMut::map(self.d~I.borrow_mut(), |slice| unsafe {
                            debug_checked_assume!(self.len <= MAX_DATA_CAPACITY as usize);
                            // SAFETY: We guarantee that the storage is valid up to self.len.
                            slice.slice_mut(self.len)
                        })
                    }
                )*

                /// Resolves the slot index and data index for a given entity.
                /// Both indices are guaranteed to point to valid corresponding cells.
                #[inline(always)]
                fn resolve_entity(&self, entity: Entity<A>) -> Option<(TrimmedIndex, TrimmedIndex)> {
                    debug_assert!(self.len <= self.capacity());

                    // Nothing to resolve if we have nothing stored
                    if self.len == 0 {
                        return None;
                    }

                    // Get the index into the slot array from the entity.
                    let slot_index = entity.slot_index();

                    unsafe {
                        let slot_index_usize: usize = slot_index.into();

                        // NOTE: It's a little silly, but we don't actually know if this entity
                        // was created by this map, so we can't assume internal consistency here.
                        // We'll just have to take the small hit for bounds checking on the index.
                        debug_assert!(slot_index_usize < self.capacity(), "invalid entity handle");
                        if slot_index_usize >= self.capacity() {
                            return None;
                        }

                        // SAFETY: We know that the slot storage is valid up to our capacity.
                        let slots = self.slots.slice(self.capacity());
                        // SAFETY: We know slot_index_usize is within bounds due to the check above.
                        let slot = slots.get_unchecked(slot_index_usize);

                        // NOTE: For similar reasons above, a crossed-wires entity handle from another
                        // world could miraculously have the correct version while pointing to a freed
                        // slot. This could cause some wacky memory access, so we need to allow slots
                        // to be explicitly identified as free or not. Again, this has a small cost.
                        if (slot.version() != entity.version()) || slot.is_free() {
                            return None; // Stale entity handle, fail the lookup
                        }

                        // SAFETY: We know that this is not a free slot due to the check above.
                        let dense_index = slot.index().index_data().unwrap_unchecked();

                        #[cfg(debug_assertions)]
                        {
                            let dense_index_usize: usize = dense_index.into();
                            debug_assert!(dense_index_usize < self.len());

                            let entities = self.entities.slice(self.len());
                            let lookup = entities.get_unchecked(dense_index_usize);
                            debug_assert!(lookup.slot_index() == entity.slot_index());
                            debug_assert!(lookup.version() == entity.version());
                        }

                        Some((slot_index, dense_index))
                    }
                }

                /// Resolves the slot index and data index for a given direct entity.
                /// Both indices are guaranteed to point to valid corresponding cells.
                #[inline(always)]
                fn resolve_direct(&self, entity: EntityDirect<A>) -> Option<(TrimmedIndex, TrimmedIndex)> {
                    debug_assert!(self.len <= self.capacity());

                    // Nothing to resolve if we have nothing stored
                    if self.len == 0 {
                        return None;
                    }

                    // For direct entities, we compare against the storage version
                    if entity.version() != self.version() {
                        return None;
                    }

                    // Get the index into the dense array from the direct entity.
                    let dense_index = entity.dense_index();

                    unsafe {
                        let dense_index_usize: usize = dense_index.into();

                        // NOTE: It's a little silly, but we don't actually know if this entity
                        // was created by this map, so we can't assume internal consistency here.
                        // We'll just have to take the small hit for bounds checking on the index.
                        debug_assert!(dense_index_usize < self.len(), "invalid entity handle");
                        if dense_index_usize >= self.len() {
                            return None;
                        }

                        // SAFETY: We know that the entity storage is valid up to our length.
                        let entities = self.entities.slice(self.len);
                        // SAFETY: We know dense_index_usize is within bounds due to the check above.
                        let lookup = entities.get_unchecked(dense_index_usize);

                        // SAFETY: Entities in the valid dense region point to valid slots.
                        let slot_index = lookup.slot_index();

                        #[cfg(debug_assertions)]
                        {
                            let slot_index_usize: usize = slot_index.into();
                            debug_assert!(slot_index_usize < self.capacity);

                            // SAFETY: We know that the slot storage is valid up to our capacity.
                            let slots = self.slots.slice(self.capacity());
                            // SAFETY: We guarantee that the entity points to a valid slot.
                            let slot = slots.get_unchecked(slot_index_usize);
                            debug_assert!(lookup.version() == slot.version());
                            debug_assert!(slot.is_free() == false);
                        }

                        Some((slot_index, dense_index))
                    }
                }

                /// Grows the storage structure to accommodate more data.
                ///
                /// This wipes the current free list and rebuilds it, including the list head.
                #[inline(always)]
                fn grow(&mut self) -> bool {
                    if self.capacity() >= MAX_DATA_CAPACITY as usize {
                        return false; // Out of room to grow
                    }

                    debug_assert!(self.len == self.capacity);

                    let new_capacity = self.capacity.saturating_add(1).saturating_mul(2);
                    let new_capacity = new_capacity.min(MAX_DATA_CAPACITY as usize);

                    unsafe {
                        // SAFETY: We know new_capacity > self.capacity and is nonzero.
                        self.slots.grow(self.capacity, new_capacity);
                        self.entities.grow(self.capacity, new_capacity);
                        #(self.d~I.get_mut().grow(self.capacity, new_capacity);)*

                        // SAFETY: We know self.len <= MAX_DATA_CAPACITY.
                        let free_start = TrimmedIndex::new_usize(self.len).unwrap_unchecked();
                        // SAFETY: We just grew the slot data array up to new_capacity.
                        let slots = self.slots.raw_data(new_capacity);

                        // Populate the end of the list as the new free list. We are
                        // assuming here that, because we are full, every slot is occupied
                        // and so our free list is entirely empty. Thus, we need a new one.
                        self.free_head = Slot::populate_free_list(free_start, slots);

                        // Update our capacity
                        self.capacity = new_capacity;
                    }

                    // Success!
                    true
                }

                /// Force-pushes an entity's component into the storage and returns a handle.
                ///
                /// # Safety
                ///
                /// It is up to the caller to guarantee the following:
                /// - The storage has enough allocated room for the data.
                #[inline(always)]
                unsafe fn force_create<D: $components<#(T~I,)*>>(&mut self, data: D) -> Entity<A> {
                    debug_assert!(self.len < self.capacity);

                    unsafe {
                        // SAFETY: We will never hit the the free list end if we're below capacity
                        let slot_index = self.free_head.index_free().unwrap_unchecked();
                        // SAFETY: We never let self.len be greater than MAX_DATA_CAPACITY.
                        let dense_index = TrimmedIndex::new_usize(self.len).unwrap_unchecked();

                        // SAFETY: We know that the slot storage is valid up to our capacity.
                        let slots = self.slots.slice_mut(self.capacity());
                        // SAFETY: We know this is not the end of the free list, and we know that
                        // a free list slot index can never be assigned to an out of bounds value.
                        let slot = slots.get_unchecked_mut(Into::<usize>::into(slot_index));

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
                        let data = data.raw_get();
                        self.entities.write(index, entity);
                        #(self.d~I.get_mut().write(index, data.I);)*

                        #[cfg(feature = "events")]
                        {
                            self.created.push(entity);
                        }

                        entity
                    }
                }

                /// Destroys the given slot and data.
                ///
                /// # Safety
                ///
                /// The caller must guarantee that slot_index and dense_index refer to valid
                /// corresponding slot and data cells within range (and implicitly, self.len > 0).
                unsafe fn force_destroy(
                    &mut self,
                    indices: (TrimmedIndex, TrimmedIndex), // (slot_index, dense_index)
                ) -> A::Components {
                    let (slot_index, dense_index) = indices;

                    let result = unsafe {
                        // SAFETY: These are guaranteed by resolve_slot to be in range.
                        let slot_index_usize: usize = slot_index.into();
                        let dense_index_usize: usize = dense_index.into();

                        debug_assert!(self.len > 0);
                        debug_assert!(slot_index_usize <= self.capacity());
                        debug_assert!(dense_index_usize < self.len);

                        let entities = self.entities.slice(self.len);
                        debug_assert!(entities.len() == self.len);

                        // Make sure the entity backtracks to the same slot we're removing now.
                        debug_assert!(entities[dense_index_usize].slot_index() == slot_index);
                        debug_assert_eq!(
                            entities[dense_index_usize].version(),
                            self.slots.slice(self.capacity())[slot_index_usize].version());

                        #[cfg(feature = "events")]
                        {
                            self.destroyed.push(*entities.get_unchecked(dense_index_usize));
                        }

                        // SAFETY: We know self.len > 0 because we got Some from resolve_slot.
                        let last_dense_index = self.len - 1;
                        // SAFETY: We know the entity slice has a length of self.len.
                        let last_entity = *entities.get_unchecked(last_dense_index);
                        // SAFETY: We guarantee that stored entities point to valid slots.
                        let last_slot_index: usize = last_entity.slot_index().into();

                        // Perform the swap_remove on our data to drop the target entity.
                        // SAFETY: We guarantee that non-free slots point to valid dense data.
                        self.entities.swap_remove(dense_index_usize, self.len);
                        let result = <A::Components as $components<#(T~I,)*>>::raw_new(
                            #(self.d~I.get_mut().swap_remove(dense_index_usize, self.len),)*
                        );

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
                            .release(self.free_head);

                        // Advance this storage's overall version (for add/removes).
                        self.version = self.version.next();

                        result
                    };

                    // Update the free list head
                    self.free_head = SlotIndex::new_free(slot_index);
                    self.len -= 1;

                    result
                }
            }

            impl<A: Archetype, #(T~I,)*> StorageCanResolve<Entity<A>> for $name<A, #(T~I,)*>
            where
                A::Components: $components<#(T~I,)*>,
            {
                #[inline(always)]
                fn resolve_for(&self, entity: Entity<A>) -> Option<usize> {
                    // The dense index from resolve_slot is guaranteed to be within bounds.
                    let (_, dense_index) = self.resolve_entity(entity)?;
                    let dense_index_usize = dense_index.into();

                    unsafe {
                        // SAFETY: This is checked when we create and grow.
                        debug_checked_assume!(self.len <= MAX_DATA_CAPACITY as usize);
                        // SAFETY: This is guaranteed by resolve_slot.
                        debug_checked_assume!(self.len >= dense_index_usize);
                    }

                    Some(dense_index_usize)
                }

                #[inline(always)]
                fn resolve_direct(&self, entity: Entity<A>) -> Option<EntityDirect<A>> {
                    let (_, dense_index) = self.resolve_entity(entity)?;
                    Some(EntityDirect::new(dense_index, self.version()))
                }

                #[inline]
                fn resolve_destroy(&mut self, entity: Entity<A>) -> Option<A::Components> {
                    unsafe {
                        // SAFETY: We know that resolve_entity returns valid corresponding slots.
                        Some(self.force_destroy(self.resolve_entity(entity)?))
                    }
                }
            }

            impl<A: Archetype, #(T~I,)*> StorageCanResolve<EntityDirect<A>> for $name<A, #(T~I,)*>
            where
                A::Components: $components<#(T~I,)*>,
            {
                #[inline(always)]
                fn resolve_for(&self, entity: EntityDirect<A>) -> Option<usize> {
                     // The dense index from resolve_slot is guaranteed to be within bounds.
                    let (_, dense_index) = self.resolve_direct(entity)?;
                    let dense_index_usize = dense_index.into();

                    unsafe {
                        // SAFETY: This is checked when we create and grow.
                        debug_checked_assume!(self.len <= MAX_DATA_CAPACITY as usize);
                        // SAFETY: This is guaranteed by resolve_slot.
                        debug_checked_assume!(self.len >= dense_index_usize);
                    }

                    Some(dense_index_usize)
                }

                #[inline(always)]
                fn resolve_direct(&self, entity: EntityDirect<A>) -> Option<EntityDirect<A>> {
                    Some(entity) // Trivially return, as we're already an EntityDirect
                }

                #[inline]
                fn resolve_destroy(&mut self, entity: EntityDirect<A>) -> Option<A::Components> {
                    unsafe {
                        // SAFETY: We know that resolve_direct returns valid corresponding slots.
                        Some(self.force_destroy(self.resolve_direct(entity)?))
                    }
                }
            }

            impl<A: Archetype, #(T~I,)*> Drop
                for $name<A, #(T~I,)*>
            {
                #[inline(always)]
                fn drop(&mut self) {
                    // SAFETY: We guarantee that the storage is valid up to self.len.
                    unsafe {
                        #(self.d~I.get_mut().drop_to(self.len);)*
                        // We don't need to drop the other stuff since it's all trivial.

                        // For dynamic storage we need to deallocate when dropping.
                        self.slots.dealloc(self.capacity);
                        self.entities.dealloc(self.capacity);
                        #(self.d~I.get_mut().dealloc(self.capacity);)*
                    };
                }
            }

            impl<A: Archetype, #(T~I,)*> Default for $name<A, #(T~I,)*>
            where
                A::Components: $components<#(T~I,)*>,
            {
                #[inline(always)]
                fn default() -> Self {
                    $name::new()
                }
            }

            impl<A: Archetype, #(T~I,)*> Clone for $name<A, #(T~I,)*>
            where
                A::Components: $components<#(T~I,)*>,
                #(T~I: Clone,)*
            {
                /// Clones this storage, including all of its data.
                ///
                /// # Panics
                ///
                /// This function will panic if any of its components are mutably borrowed,
                /// or if there is not enough memory available to perform the clone.
                #[inline]
                fn clone(&self) -> Self {
                    #(let ref_d~I = self.d~I.borrow();)*

                    let mut new_slots = DataPtr::with_capacity(self.capacity);
                    let mut new_entities = DataPtr::with_capacity(self.capacity);
                    #(let mut new_d~I = DataPtr::with_capacity(self.capacity);)*

                    unsafe {

                        // SAFETY: We know that the storage is valid up to self.len.
                        let old_slots = self.slots.slice(self.capacity);
                        let old_entities = self.entities.slice(self.len);
                        #(let old_~I = ref_d~I.slice(self.len);)*

                        for idx in 0..self.capacity {
                            // SAFETY: We know that the slot storage is valid up to our capacity,
                            // and the new storage has no data that needs to be dropped first.
                            new_slots.write(idx, old_slots.get_unchecked(idx).clone());
                        }

                        for idx in 0..self.len {
                            // SAFETY: We know that the slot storage is valid up to our len,
                            // and the new storage has no data that needs to be dropped first.
                            new_entities.write(idx, old_entities.get_unchecked(idx).clone());
                            #(new_d~I.write(idx, old_~I.get_unchecked(idx).clone());)*
                        }

                        Self {
                            len: self.len,
                            version: self.version,
                            capacity: self.capacity,
                            free_head: self.free_head,
                            slots: new_slots,
                            entities: new_entities,
                            #(d~I: RefCell::new(new_d~I),)*

                            #[cfg(feature = "events")]
                            created: self.created.clone(),
                            #[cfg(feature = "events")]
                            destroyed: self.destroyed.clone(),
                        }
                    }
                }
            }

            pub struct $borrow<'a, A: Archetype, #(T~I,)*> {
                index: usize,
                source: &'a $name<A, #(T~I,)*>,
            }

            impl<'a, A: Archetype, #(T~I,)*> $borrow<'a, A, #(T~I,)*>
                where A::Components: $components<#(T~I,)*>,
            {
                #[inline(always)]
                pub fn index(&self) -> usize {
                    self.index
                }

                #[inline(always)]
                pub fn entity(&self) -> &Entity<A> {
                    unsafe {
                        // SAFETY: We can only be created with a valid index, and because
                        // we hold a reference to the source, that reference can't have
                        // changed in any way that would have made this index invalid.
                        self.source.get_slice_entities().get_unchecked(self.index)
                    }
                }

                #(
                    /// Borrows the element of the given component index.
                    #[inline(always)]
                    pub fn borrow_component_~I(&self) -> Ref<'_, T~I> {
                        Ref::map(self.source.d~I.borrow(), |slice| unsafe {
                            debug_assert!(self.index < self.source.len);
                            // SAFETY: We can only be created with a valid index, and because
                            // we hold a reference to the source, that reference can't have
                            // changed in any way that would have made this index invalid.
                            // SAFETY: We guarantee that the storage is valid up to self.len.
                            slice.slice(self.source.len).get_unchecked(self.index)
                        })
                    }

                    /// Mutably borrows the element of the given component index.
                    #[inline(always)]
                    pub fn borrow_component_mut_~I(&self) -> RefMut<'_, T~I> {
                        RefMut::map(self.source.d~I.borrow_mut(), |slice| unsafe {
                            debug_assert!(self.index < self.source.len);
                            // SAFETY: We can only be created with a valid index, and because
                            // we hold a reference to the source, that reference can't have
                            // changed in any way that would have made this index invalid.
                            // SAFETY: We guarantee that the storage is valid up to self.len.
                            slice.slice_mut(self.source.len).get_unchecked_mut(self.index)
                        })
                    }
                )*
            }

            impl<'a, A: Archetype, #(T~I,)*> Clone for $borrow<'a, A, #(T~I,)*> {
                #[inline(always)]
                fn clone(&self) -> Self {
                    *self
                }
            }

            impl<'a, A: Archetype, #(T~I,)*> Copy for $borrow<'a, A, #(T~I,)*> {}
        });
    };
}

seq!(N in 1..=16 {
    declare_storage_n!(
        Storage~N,
        Borrow~N,
        Iter~N,
        IterMut~N,
        Components~N,
        Slices~N,
        View~N,
        ViewMut~N,
        N
    );
});

#[cfg(feature = "32_components")]
seq!(N in 17..=32 {
    declare_storage_n!(
        Storage~N,
        Borrow~N,
        Iter~N,
        IterMut~N,
        Components~N,
        Slices~N,
        View~N,
        ViewMut~N,
        N
    );
});

pub struct DataPtr<T>(NonNull<MaybeUninit<T>>);

// SAFETY: There's no explicit interior mutability going on here -- this is similar to a Vec-type
// API for the data stored within, so the send- and sync-ness should be the same as T. This is
// similar to the nomicon's implementation of RawVec<T>, which also has these unsafe impls added.
unsafe impl<T> Send for DataPtr<T> where T: Send {}
unsafe impl<T> Sync for DataPtr<T> where T: Sync {}

impl<T> DataPtr<T> {
    /// Allocates a new data array with the given capacity, if any.
    ///
    /// If `T` is zero-sized, or the given capacity is 0, this will not allocate.
    ///
    /// # Panics
    ///
    /// This operation will panic if there is not enough memory to perform the new
    /// allocation, or if the resulting allocation size is greater than `isize::MAX`.
    pub fn with_capacity(capacity: usize) -> Self {
        if (mem::size_of::<T>() == 0) || (capacity == 0) {
            return Self(NonNull::dangling());
        }

        let layout = new_layout::<T>(capacity);

        debug_assert!(capacity > 0);
        debug_assert!(layout.size() > 0);

        unsafe { Self(resolve_ptr(alloc::alloc(layout), layout)) }
    }

    /// Gets a pointer to the data, assuming that it's initialized.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the following:
    /// - All of the data that this pointer could point to is valid and initialized
    pub unsafe fn ptr_data(&mut self) -> *mut T {
        self.0.cast::<T>().as_ptr()
    }

    /// Gets the raw stored data up to `len` as a mutable `MaybeUninit<T>` slice.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the following:
    /// - This array has at least `len` elements allocated
    pub unsafe fn raw_data(&mut self, len: usize) -> &mut [MaybeUninit<T>] {
        // SAFETY: The caller guarantees that we have at least len elements allocated.
        unsafe { slice::from_raw_parts_mut(self.0.as_ptr(), len) }
    }

    /// Reallocates this array's old data block into a new data block.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the following:
    /// - `capacity >= old_capacity`
    /// - This array has exactly `old_capacity` elements allocated (may be 0)
    ///
    /// # Panics
    ///
    /// This operation will panic if there is not enough memory to perform the new
    /// allocation, or if the resulting allocation size is greater than `isize::MAX`.
    pub unsafe fn grow(&mut self, old_capacity: usize, capacity: usize) {
        debug_assert!(capacity >= old_capacity);

        if (mem::size_of::<T>() == 0) || (capacity == 0) {
            return; // Stay dangling
        }

        let layout = new_layout::<T>(capacity);
        let size = layout.size();
        debug_assert!(size > 0);

        unsafe {
            if old_capacity == 0 {
                // SAFETY: The caller guarantees that capacity > 0.
                self.0 = resolve_ptr(alloc::alloc(layout), layout);
            } else {
                // SAFETY: The caller guarantees that this is allocated.
                let old_ptr = self.0.as_ptr() as *mut u8;
                // SAFETY: We checked that T is not a ZST and old_capacity > 0.
                let old_layout = Layout::array::<T>(old_capacity).unwrap();
                debug_assert!(old_layout.size() > 0);

                // SAFETY: The caller guarantees that capacity > 0.
                self.0 = resolve_ptr(alloc::realloc(old_ptr, old_layout, size), layout);
            }
        }
    }

    /// Deallocates this array's data block.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the following:
    /// - This array has exactly `capacity` elements allocated
    pub unsafe fn dealloc(&mut self, capacity: usize) {
        if (mem::size_of::<T>() == 0) || (capacity == 0) {
            return; // Nothing to deallocate
        }

        // SAFETY: We checked that T is not a ZST and capacity > 0.
        let layout = Layout::array::<T>(capacity).unwrap();
        debug_assert!(layout.size() > 0);

        unsafe {
            // SAFETY: We know that old_layout has a nonzero size
            alloc::dealloc(self.0.as_ptr() as *mut u8, layout);
            self.0 = NonNull::dangling();
        }
    }

    /// Writes an element to the given index.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the following:
    /// - This pointer has allocated at least `index` elements
    /// - The element at `index` is currently invalid
    #[inline(always)]
    unsafe fn write(&mut self, index: usize, val: T) {
        unsafe {
            // SAFETY: The caller guarantees that this cell is allocated and invalid.
            (*self.0.as_ptr().add(index)).write(val);
        }
    }

    /// Gets a slice for the range `0..len`.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the following:
    /// - This pointer has allocated at least `len` elements
    /// - All elements the range `0..len` are valid
    #[inline(always)]
    unsafe fn slice(&self, len: usize) -> &[T] {
        unsafe {
            // SAFETY: Casting a `[MaybeUninit<T>]` to a `[T]` is safe because the caller
            // guarantees that this portion of the data is valid and `MaybeUninit<T>` is
            // guaranteed to have the same layout as `T`. The pointer obtained is valid
            // since it refers to memory owned by `slice` which is a reference and thus
            // guaranteed to be valid for reads.
            // Ref: https://doc.rust-lang.org/stable/src/core/mem/maybe_uninit.rs.html#972
            slice::from_raw_parts(self.0.as_ptr() as *const T, len)
        }
    }

    /// Gets a mutable slice for the range `0..len`.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the following:
    /// - This pointer has allocated at least `len` elements
    /// - All elements the range `0..len` are valid
    #[inline(always)]
    unsafe fn slice_mut(&mut self, len: usize) -> &mut [T] {
        unsafe {
            // SAFETY: Similar to safety notes for `slice`, but we have a mutable reference
            // which is also guaranteed to be valid for writes.
            // Ref: https://doc.rust-lang.org/stable/src/core/mem/maybe_uninit.rs.html#994
            slice::from_raw_parts_mut(self.0.as_ptr() as *mut T, len)
        }
    }

    /// Drops the element at `index` and replaces it with the last element in `0..len`.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee the following:
    /// - This pointer has allocated at least `len` elements
    /// - All elements the range `0..len` are valid
    /// - `len > 0`
    /// - `index < len`
    #[inline(always)]
    unsafe fn swap_remove(&mut self, index: usize, len: usize) -> T {
        unsafe {
            debug_assert!(len > 0);
            debug_assert!(index < len);

            // SAFETY: The caller is guaranteeing that the element at index, and
            // the element at len - 1 are both valid. With this guarantee we can
            // safely take the element at index. We then perform a direct pointer
            // copy (we can't assume nonoverlapping here!) from the last element
            // to the one at index. This moves the data, making the data at index
            // valid to the data at last, and the data at last invalid (even if
            // it is still bitwise identical to the data at index).
            let last = len - 1;
            let array_ptr = self.0.as_ptr();
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
    /// - All elements in the range `0..len` are valid
    /// - `len <= N`
    #[inline(always)]
    unsafe fn drop_to(&mut self, len: usize) {
        unsafe {
            for i in 0..len {
                let i_ptr = self.0.as_ptr().add(i);
                // SAFETY: The caller guarantees this element is valid.
                ptr::drop_in_place(i_ptr as *mut T);
                ptr::write(i_ptr, MaybeUninit::uninit()); // Hint for Miri
            }
        };
    }
}

#[inline(always)]
fn new_layout<T>(capacity: usize) -> Layout {
    let layout = Layout::array::<T>(capacity).unwrap();
    assert!(layout.size() <= isize::MAX as usize, "allocation too large");
    layout
}

#[inline(always)]
fn resolve_ptr<T>(ptr: *mut u8, layout: Layout) -> NonNull<T> {
    match NonNull::new(ptr as *mut T) {
        Some(p) => p,
        None => alloc::handle_alloc_error(layout),
    }
}
