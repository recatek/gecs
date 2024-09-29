use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use xxhash_rust::xxh3::xxh3_128;

use crate::data::{DataArchetype, DataWorld};
use crate::generate::util::to_snake;

#[allow(non_snake_case)] // Allow for type-like names to make quote!() clearer
#[allow(unused_variables)] // For unused feature-controlled generation elements
pub fn generate_world(world_data: &DataWorld, raw_input: &str) -> TokenStream {
    let world_snake = to_snake(&world_data.name);
    let unique_hash = xxh3_128(raw_input.as_bytes());

    // Module
    let ecs_world_sealed = format_ident!("ecs_{}_sealed", world_snake);

    // Constants and literals
    let WORLD_DATA = world_data.to_base64();

    // Types and traits
    let World = format_ident!("{}", world_data.name);
    let WorldCapacity = format_ident!("{}Capacity", world_data.name);

    let ArchetypeSelectId = format_ident!("ArchetypeSelectId");
    let ArchetypeSelectEntity = format_ident!("ArchetypeSelectEntity");
    let ArchetypeSelectEntityRaw = format_ident!("ArchetypeSelectEntityRaw");

    let ArchetypeSelectInternalWorld =
        format_ident!("__ArchetypeSelectInternal{}", world_data.name);

    let Archetype = world_data
        .archetypes
        .iter()
        .map(|archetype| format_ident!("{}", archetype.name))
        .collect::<Vec<_>>();
    let ArchetypeRaw = world_data
        .archetypes
        .iter()
        .map(|archetype| format_ident!("{}Raw", archetype.name))
        .collect::<Vec<_>>();

    // Variables and fields
    let archetype = world_data
        .archetypes
        .iter()
        .map(|archetype| format_ident!("{}", to_snake(&archetype.name)))
        .collect::<Vec<_>>();
    let num_archetypes = world_data.archetypes.len();

    // Generated subsections
    let section_archetype = world_data
        .archetypes
        .iter()
        .map(|archetype| section_archetype(archetype))
        .collect::<Vec<_>>();
    let with_capacity_param = world_data
        .archetypes
        .iter()
        .map(|archetype| with_capacity_param(archetype))
        .collect::<Vec<_>>();
    let with_capacity_new = world_data
        .archetypes
        .iter()
        .map(|archetype| with_capacity_new(archetype))
        .collect::<Vec<_>>();

    // Macros
    let __impl_ecs_find_unique = format_ident!("__impl_ecs_find_{}", unique_hash);
    let __impl_ecs_find_borrow_unique = format_ident!("__impl_ecs_find_borrow_{}", unique_hash);
    let __impl_ecs_iter_unique = format_ident!("__impl_ecs_iter_{}", unique_hash);
    let __impl_ecs_iter_borrow_unique = format_ident!("__impl_ecs_iter_borrow_{}", unique_hash);
    let __impl_ecs_iter_destroy_unique = format_ident!("__impl_ecs_iter_destroy_{}", unique_hash);

    quote!(
        #( pub use #ecs_world_sealed::#Archetype; )*
        pub use #ecs_world_sealed::{
            #World,
            #WorldCapacity,
            #ArchetypeSelectId,
            #ArchetypeSelectEntity,
            #ArchetypeSelectEntityRaw
        };

        #[doc(hidden)]
        pub use #ecs_world_sealed::{#ArchetypeSelectInternalWorld};

        mod #ecs_world_sealed {
            use super::*;

            use ::std::cell::{Ref, RefMut};

            use ::gecs::__internal::*;

            #(#section_archetype)*

            #[derive(Default)]
            pub struct #World {
                #(
                    pub #archetype: #Archetype,
                )*
            }

            pub struct #WorldCapacity {
                #(
                    pub #with_capacity_param,
                )*
            }

            impl #World {
                /// Creates a new empty world.
                ///
                /// This will allocate for all fixed-size archetypes, but not allocate for
                /// any dynamic archetypes. Dynamic archetypes will begin with 0 capacity.
                pub fn new() -> Self {
                    Self {
                        #( #archetype: #Archetype::new(), )*
                    }
                }

                /// Returns the total number of archetypes in this world.
                pub const fn num_archetypes() -> usize {
                    #num_archetypes
                }

                /// Creates a new world with per-archetype capacities.
                ///
                /// This will allocate all archetypes to the given dynamic capacity. If a
                /// given dynamic capacity is 0, that archetype will not allocate until an
                /// entity is created in it.
                ///
                /// # Panics
                ///
                /// This will panic if given a size that exceeds the maximum possible capacity
                /// value for an archetype (currently `16,777,216`).
                pub fn with_capacity(capacity: #WorldCapacity) -> Self {
                    Self {
                        #( #archetype: #Archetype::#with_capacity_new, )*
                    }
                }


                /// Destroys the given entity and removes it from the world, if it exists.
                ///
                /// Return `true` if the entity was successfully destroyed, or `false` otherwise.
                ///
                /// Unlike other destroy functions, this does not return the entity's components.
                /// If you need the returned components from an `EntityAny`, use the entity's
                /// `resolve` type disambiguator and a match statement to get an `Entity<A>`.
                ///
                /// # Panics
                ///
                /// Panics if the given entity is not of a valid archetype in this world.
                pub fn destroy_any(&mut self, entity: EntityAny) -> bool {
                    match entity.try_into() {
                        #(
                            Ok(#ArchetypeSelectEntity::#Archetype(entity)) =>
                                self.#archetype.destroy(entity).is_some(),
                        )*
                        Err(_) => panic!("invalid entity type"),
                    }
                }
            }

            impl World for #World {}

            #(
                impl WorldHas<#Archetype> for #World {
                    #[inline(always)]
                    fn resolve_create(&mut self, data: <#Archetype as Archetype>::Components)
                        -> Entity<#Archetype>
                    {
                        self.#archetype.create(data)
                    }

                    #[inline(always)]
                    fn resolve_try_create(&mut self, data: <#Archetype as Archetype>::Components)
                        -> Option<Entity<#Archetype>>
                    {
                        self.#archetype.try_create(data)
                    }

                    #[inline(always)]
                    fn resolve_destroy(&mut self, entity: Entity<#Archetype>)
                        -> Option<<#Archetype as Archetype>::Components>
                    {
                        self.#archetype.destroy(entity)
                    }

                    #[inline(always)]
                    fn resolve_archetype(&self) -> &#Archetype {
                        &self.#archetype
                    }

                    #[inline(always)]
                    fn resolve_archetype_mut(&mut self) -> &mut #Archetype {
                        &mut self.#archetype
                    }
                }
            )*

            #[derive(Clone, Copy)]
            pub enum #ArchetypeSelectId {
                #( #Archetype, )*
            }

            #[derive(Clone, Copy)]
            pub enum #ArchetypeSelectEntity {
                #( #Archetype(Entity<#Archetype>), )*
            }

            #[derive(Clone, Copy)]
            pub enum #ArchetypeSelectEntityRaw {
                #( #Archetype(EntityRaw<#Archetype>), )*
            }

            // Combined dispatch table for resolving both entity key types.
            #[doc(hidden)]
            pub enum #ArchetypeSelectInternalWorld {
                #( #Archetype(Entity<#Archetype>), )*
                #( #ArchetypeRaw(EntityRaw<#Archetype>), )*
            }

            // Resolve dispatch implementation ----------------------------------------------------

            #(
                impl From<Entity<#Archetype>> for #ArchetypeSelectId {
                    #[inline(always)]
                    fn from(entity: Entity<#Archetype>) -> Self {
                        #ArchetypeSelectId::#Archetype
                    }
                }

                impl From<Entity<#Archetype>> for #ArchetypeSelectEntity {
                    #[inline(always)]
                    fn from(entity: Entity<#Archetype>) -> Self {
                        #ArchetypeSelectEntity::#Archetype(entity)
                    }
                }

                impl From<&Entity<#Archetype>> for #ArchetypeSelectEntity {
                    #[inline(always)]
                    fn from(entity: &Entity<#Archetype>) -> Self {
                        #ArchetypeSelectEntity::#Archetype(*entity)
                    }
                }

                impl From<EntityRaw<#Archetype>> for #ArchetypeSelectId {
                    #[inline(always)]
                    fn from(entity: EntityRaw<#Archetype>) -> Self {
                        #ArchetypeSelectId::#Archetype
                    }
                }

                impl From<EntityRaw<#Archetype>> for #ArchetypeSelectEntityRaw {
                    #[inline(always)]
                    fn from(entity: EntityRaw<#Archetype>) -> Self {
                        #ArchetypeSelectEntityRaw::#Archetype(entity)
                    }
                }

                impl From<&EntityRaw<#Archetype>> for #ArchetypeSelectEntityRaw {
                    #[inline(always)]
                    fn from(entity: &EntityRaw<#Archetype>) -> Self {
                        #ArchetypeSelectEntityRaw::#Archetype(*entity)
                    }
                }

                impl From<Entity<#Archetype>> for #ArchetypeSelectInternalWorld {
                    #[inline(always)]
                    fn from(entity: Entity<#Archetype>) -> Self {
                        #ArchetypeSelectInternalWorld::#Archetype(entity)
                    }
                }

                impl From<EntityRaw<#Archetype>> for #ArchetypeSelectInternalWorld {
                    #[inline(always)]
                    fn from(entity: EntityRaw<#Archetype>) -> Self {
                        #ArchetypeSelectInternalWorld::#ArchetypeRaw(entity)
                    }
                }

                impl WorldCanResolve<Entity<#Archetype>> for #World {
                    #[inline(always)]
                    fn resolve_destroy(
                        &mut self,
                        entity: Entity<#Archetype>
                    ) -> Option<<#Archetype as Archetype>::Components> {
                        self.archetype_mut::<#Archetype>().destroy(entity)
                    }
                }
            )*

            impl #ArchetypeSelectId {
                #[inline(always)]
                pub fn archetype_id(self) -> ArchetypeId {
                    match self {
                        #(
                            #ArchetypeSelectId::#Archetype => #Archetype::ARCHETYPE_ID,
                        )*
                    }
                }
            }

            impl WorldCanResolve<EntityAny> for #World {
                #[inline(always)]
                fn resolve_destroy(
                    &mut self,
                    entity: EntityAny,
                ) -> bool {
                    match entity.try_into() {
                        #(
                            Ok(#ArchetypeSelectEntity::#Archetype(entity)) =>
                                self.#archetype.destroy(entity).is_some(),
                        )*
                        Err(_) => panic!("invalid entity type"),
                    }
                }
            }

            impl TryFrom<ArchetypeId> for #ArchetypeSelectId {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(id: ArchetypeId) -> Result<Self, EcsError> {
                    match id {
                        #(
                            #Archetype::ARCHETYPE_ID => Ok(#ArchetypeSelectId::#Archetype),
                        )*
                        _ => Err(EcsError::InvalidEntityType),
                    }
                }
            }

            impl TryFrom<EntityAny> for #ArchetypeSelectId {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(entity: EntityAny) -> Result<Self, EcsError> {
                    match entity.archetype_id() {
                        #(
                            #Archetype::ARCHETYPE_ID => Ok(#ArchetypeSelectId::#Archetype),
                        )*
                        _ => Err(EcsError::InvalidEntityType),
                    }
                }
            }

            impl TryFrom<EntityAny> for #ArchetypeSelectEntity {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(entity: EntityAny) -> Result<Self, EcsError> {
                    match entity.archetype_id() {
                        #(
                            #Archetype::ARCHETYPE_ID => {
                                // We can use from_any_unchecked because we just checked the archetype
                                Ok(#ArchetypeSelectEntity::#Archetype(
                                    Entity::<#Archetype>::from_any_unchecked(entity))
                                )
                            },
                        )*
                        _ => Err(EcsError::InvalidEntityType),
                    }
                }
            }

            impl TryFrom<EntityRawAny> for #ArchetypeSelectEntityRaw {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(entity: EntityRawAny) -> Result<Self, EcsError> {
                    match entity.archetype_id() {
                        #(
                            #Archetype::ARCHETYPE_ID => {
                                // We can use from_any_unchecked because we just checked the archetype
                                Ok(#ArchetypeSelectEntityRaw::#Archetype(
                                    EntityRaw::<#Archetype>::from_any_unchecked(entity))
                                )
                            },
                        )*
                        _ => Err(EcsError::InvalidEntityType),
                    }
                }
            }

            impl TryFrom<EntityAny> for #ArchetypeSelectInternalWorld {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(entity: EntityAny) -> Result<Self, EcsError> {
                    match entity.archetype_id() {
                        #(
                            #Archetype::ARCHETYPE_ID => {
                                // We can use from_any_unchecked because we just checked the archetype
                                Ok(#ArchetypeSelectInternalWorld::#Archetype(
                                    Entity::<#Archetype>::from_any_unchecked(entity)
                                ))
                            },
                        )*
                        _ => Err(EcsError::InvalidEntityType),
                    }
                }
            }

            impl TryFrom<EntityRawAny> for #ArchetypeSelectInternalWorld {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(entity: EntityRawAny) -> Result<Self, EcsError> {
                    match entity.archetype_id() {
                        #(
                            #Archetype::ARCHETYPE_ID => {
                                // We can use from_any_unchecked because we just checked the archetype
                                Ok(#ArchetypeSelectInternalWorld::#ArchetypeRaw(
                                    EntityRaw::<#Archetype>::from_any_unchecked(entity))
                                )
                            },
                        )*
                        _ => Err(EcsError::InvalidEntityType),
                    }
                }
            }
        }

        #[macro_export]
        #[doc(hidden)]
        macro_rules! #__impl_ecs_find_unique {
            ($($args:tt)*) => {
                ::gecs::__internal::__impl_ecs_find!(#WORLD_DATA, $($args)*)
            }
        }

        #[macro_export]
        #[doc(hidden)]
        macro_rules! #__impl_ecs_find_borrow_unique {
            ($($args:tt)*) => {
                ::gecs::__internal::__impl_ecs_find_borrow!(#WORLD_DATA, $($args)*)
            }
        }

        #[macro_export]
        #[doc(hidden)]
        macro_rules! #__impl_ecs_iter_unique {
            ($($args:tt)*) => {
                ::gecs::__internal::__impl_ecs_iter!(#WORLD_DATA, $($args)*);
            }
        }

        #[macro_export]
        #[doc(hidden)]
        macro_rules! #__impl_ecs_iter_borrow_unique {
            ($($args:tt)*) => {
                ::gecs::__internal::__impl_ecs_iter_borrow!(#WORLD_DATA, $($args)*);
            }
        }

        #[macro_export]
        #[doc(hidden)]
        macro_rules! #__impl_ecs_iter_destroy_unique {
            ($($args:tt)*) => {
                ::gecs::__internal::__impl_ecs_iter_destroy!(#WORLD_DATA, $($args)*);
            }
        }

        #[doc(inline)]
        pub use #__impl_ecs_find_unique as ecs_find;
        #[doc(inline)]
        pub use #__impl_ecs_find_borrow_unique as ecs_find_borrow;
        #[doc(inline)]
        pub use #__impl_ecs_iter_unique as ecs_iter;
        #[doc(inline)]
        pub use #__impl_ecs_iter_borrow_unique as ecs_iter_borrow;
        #[doc(inline)]
        pub use #__impl_ecs_iter_destroy_unique as ecs_iter_destroy;
    )
}

#[allow(non_snake_case)] // Allow for type-like names to make quote!() clearer
fn section_archetype(archetype_data: &DataArchetype) -> TokenStream {
    let count = archetype_data.components.len();
    let count_str = count.to_string();

    // Constants and literals
    let ARCHETYPE_ID = archetype_data.id;
    let COMPONENT_ID = archetype_data
        .components
        .iter()
        .map(|component| component.id)
        .collect::<Vec<_>>();

    // Types and traits
    let Archetype = format_ident!("{}", archetype_data.name);
    let Component = archetype_data
        .components
        .iter()
        .map(|component| format_ident!("{}", component.name))
        .collect::<Vec<_>>();

    let ArchetypeBorrow = format_ident!("{}Borrow", archetype_data.name);
    let ArchetypeView = format_ident!("{}View", archetype_data.name);
    let ArchetypeSlices = format_ident!("{}Slices", archetype_data.name);

    let ViewN = format_ident!("View{}", count_str);
    let SlicesN = format_ident!("Slices{}", count_str);
    let ContentArgs = quote!(#Archetype, #(#Component),*);

    let StorageN = format_ident!("Storage{}", count_str);
    let BorrowN = format_ident!("Borrow{}", count_str);
    let StorageArgs = quote!(#Archetype, #(#Component,)*);

    let IterArgs = quote!(&Entity<#Archetype>, #(&#Component,)*);
    let IterMutArgs = quote!(&Entity<#Archetype>, #(&mut #Component,)*);

    // Function names
    let get_slice = (0..count)
        .into_iter()
        .map(|idx| format_ident!("get_slice_{}", idx.to_string()));
    let get_slice_mut = (0..count)
        .into_iter()
        .map(|idx| format_ident!("get_slice_mut_{}", idx.to_string()));
    let borrow = (0..count)
        .into_iter()
        .map(|idx| format_ident!("borrow_{}", idx.to_string()));
    let borrow_mut = (0..count)
        .into_iter()
        .map(|idx| format_ident!("borrow_mut_{}", idx.to_string()));
    let borrow_slice = (0..count)
        .into_iter()
        .map(|idx| format_ident!("borrow_slice_{}", idx.to_string()));
    let borrow_slice_mut = (0..count)
        .into_iter()
        .map(|idx| format_ident!("borrow_slice_mut_{}", idx.to_string()));
    let get_id_component = archetype_data
        .components
        .iter()
        .map(|component| format_ident!("get_id_{}", to_snake(&component.name)))
        .collect::<Vec<_>>();

    // Variables/fields
    let component = archetype_data
        .components
        .iter()
        .map(|component| format_ident!("{}", to_snake(&component.name)))
        .collect::<Vec<_>>();

    quote!(
        #[derive(Default)]
        #[repr(transparent)]
        pub struct #Archetype {
            pub data: #StorageN<#StorageArgs>,
        }

        impl #Archetype {
            /// Constructs a new, empty archetype.
            ///
            /// If the archetype uses dynamic storage, this archetype will not allocate until
            /// an entity is added to it. Otherwise, for static storage, the full capacity
            /// will be allocated on creation of the archetype.
            #[inline(always)]
            pub fn new() -> Self {
                Self { data: #StorageN::new() }
            }

            /// Constructs a new archetype pre-allocated to the given storage capacity.
            ///
            /// If the given capacity would result in zero size, this will not allocate.
            #[inline(always)]
            pub fn with_capacity(capacity: usize) -> Self {
                Self { data: #StorageN::with_capacity(capacity) }
            }

            /// Returns the number of entities in the archetype, also referred to as its length.
            #[inline(always)]
            pub fn len(&self) -> usize {
                self.data.len()
            }

            /// Returns the total number of elements the archetype can hold without reallocating.
            /// If the archetype has fixed-sized storage, this is the absolute total capacity.
            ///
            /// Note that the archetype may not be able to me filled to its capacity if it has
            /// had to orphan/leak entity slots due to generational index overflow.
            #[inline(always)]
            pub const fn capacity(&self) -> usize {
                self.data.capacity()
            }

            /// Returns `true` if the archetype contains no elements.
            #[inline(always)]
            pub const fn is_empty(&self) -> bool {
                self.data.is_empty()
            }

            /// Returns the generational version of the archetype. Intended for internal use.
            #[inline(always)]
            pub const fn version(&self) -> ArchetypeVersion {
                self.data.version()
            }

            /// Creates a new entity with the given components in the archetype, if there's room.
            ///
            /// Returns a handle for accessing the new entity.
            ///
            /// # Panics
            ///
            /// Panics if the archetype is full. For a panic-free version, use `try_create`.
            #[inline(always)]
            pub fn create(
                &mut self,
                data: (#(#Component,)*)
            ) -> Entity<#Archetype> {
                self.data.try_push(data).expect("failed to push to full archetype")
            }

            /// Creates a new entity with the given components in the archetype, if there's room.
            ///
            /// Returns a handle for accessing the new entity, or `None` if the archetype is full.
            #[inline(always)]
            pub fn try_create(
                &mut self,
                data: (#(#Component,)*)
            ) -> Option<Entity<#Archetype>> {
                self.data.try_push(data)
            }

            /// If the entity exists in the archetype, this returns its dense data slice index.
            /// The returned index is guaranteed to be within bounds of the dense data slices.
            #[inline(always)]
            pub fn resolve<K: EntityKey>(
                &self,
                entity: K
            ) -> Option<usize>
            where
                #StorageN<#StorageArgs>: StorageCanResolve<K>
            {
                self.data.resolve(entity)
            }

            /// If the entity exists in the archetype, this destroys it and returns its components.
            #[inline(always)]
            pub fn destroy(
                &mut self,
                entity: Entity<#Archetype>
            ) -> Option<(#(#Component,)*)> {
                self.data.remove(entity)
            }

            /// Returns an iterator over all of the entities and their data.
            #[inline(always)]
            pub fn iter(&mut self) -> impl Iterator<Item = (#IterArgs)> {
                self.data.iter()
            }

            /// Returns a mutable iterator over all of the entities and their data.
            #[inline(always)]
            pub fn iter_mut(&mut self) -> impl Iterator<Item = (#IterMutArgs)> {
                self.data.iter_mut()
            }

            /// Begins a borrow context for the given entity on this archetype. This will allow
            /// direct access to that entity's components with runtime borrow checking. This can
            /// be faster than accessing the components as slices, as it will skip bounds checks.
            #[inline(always)]
            pub fn begin_borrow<'a, K: EntityKey>(
                &'a self,
                entity: K,
            ) -> Option<#ArchetypeBorrow<'a>>
            where
                #StorageN<#StorageArgs>: StorageCanResolve<K>
            {
                self.data.begin_borrow(entity).map(#ArchetypeBorrow)
            }

            #[inline(always)]
            pub fn get_view_mut<'a, K: EntityKey>(
                &'a mut self,
                entity_key: K
            ) -> Option<#ArchetypeView<'a>>
            where
                #StorageN<#StorageArgs>: StorageCanResolve<K>
            {
                self.data.get_view_mut(entity_key)
            }

            /// Returns mutable slices to all data for all entities in the archetype. To get the
            /// data index for a specific entity using this function, use the `resolve` function.
            #[inline(always)]
            pub fn get_all_slices_mut(&mut self) -> #ArchetypeSlices {
                self.data.get_all_slices_mut()
            }

            #(
                /// Helper function for getting the compile-time ID for the given component.
                // TODO: Change this to being generic-based once traits can have const fns.
                pub const fn #get_id_component() -> u8 {
                    #COMPONENT_ID
                }
            )*
        }

        #(
            impl ArchetypeHas<#Component> for #Archetype {
                #[inline(always)]
                fn resolve_get_slice(&mut self) -> &[#Component] {
                    self.data.#get_slice()
                }

                #[inline(always)]
                fn resolve_get_slice_mut(&mut self) -> &mut [#Component] {
                    self.data.#get_slice_mut()
                }

                #[inline(always)]
                fn resolve_borrow_slice(&self) -> Ref<[#Component]> {
                    self.data.#borrow_slice()
                }

                #[inline(always)]
                fn resolve_borrow_slice_mut(&self) -> RefMut<[#Component]> {
                    self.data.#borrow_slice_mut()
                }

                #[inline(always)]
                fn resolve_borrow<'a>(borrow: &'a #ArchetypeBorrow<'a>) -> Ref<'a, #Component> {
                    borrow.0.#borrow()
                }

                #[inline(always)]
                fn resolve_borrow_mut<'a>(borrow: &'a #ArchetypeBorrow<'a>) -> RefMut<'a, #Component> {
                    borrow.0.#borrow_mut()
                }
            }
        )*

        impl Archetype for #Archetype {
            #[allow(unconditional_panic)]
            const ARCHETYPE_ID: u8 = #ARCHETYPE_ID;

            type Components = (#(#Component,)*);
            type View<'a> = #ArchetypeView<'a>;
            type Borrow<'a> = #ArchetypeBorrow<'a>;
            type Slices<'a> = #ArchetypeSlices<'a>;

            #[inline(always)]
            fn get_slice_entities(&self) -> &[Entity<#Archetype>] {
                self.data.get_slice_entities()
            }
        }

        #[repr(transparent)]
        #[derive(Clone, Copy)]
        pub struct #ArchetypeBorrow<'a>(#BorrowN<'a, #StorageArgs>);

        impl<'a> #ArchetypeBorrow<'a> {
            #[inline(always)]
            pub fn index(&self) -> usize {
                self.0.index()
            }

            #[inline(always)]
            pub fn entity(&self) -> &Entity<#Archetype> {
                self.0.entity()
            }

            #[inline(always)]
            pub fn borrow<C>(&self) -> Ref<C>
            where
                #Archetype: for<'c> ArchetypeHas<C, Borrow<'c> = #ArchetypeBorrow<'c>>
            {
                #Archetype::resolve_borrow(self)
            }

            #[inline(always)]
            pub fn borrow_mut<C>(&self) -> RefMut<C>
            where
                #Archetype: for<'c> ArchetypeHas<C, Borrow<'c> = #ArchetypeBorrow<'c>>
            {
                #Archetype::resolve_borrow_mut(self)
            }
        }

        pub struct #ArchetypeView<'a> {
            index: usize,
            pub entity: &'a Entity<#Archetype>,
            #(
                pub #component: &'a mut #Component,
            )*
        }

        pub struct #ArchetypeSlices<'a> {
            pub entity: &'a [Entity<#Archetype>],
            #(
                pub #component: &'a mut [#Component],
            )*
        }

        impl<'a> #ArchetypeView<'a> {
            #[inline(always)]
            pub fn index(&self) -> usize {
                self.index
            }

            #[inline(always)]
            pub fn component<C>(&self) -> &C
            where
                Self: ViewHas<C>
            {
                <Self as ViewHas<C>>::resolve_component(self)
            }

            #[inline(always)]
            pub fn component_mut<C>(&mut self) -> &mut C
            where
                Self: ViewHas<C>
            {
                <Self as ViewHas<C>>::resolve_component_mut(self)
            }
        }

        impl<'a> View for #ArchetypeView<'a> {}

        #(
            impl<'a> ViewHas<#Component> for #ArchetypeView<'a> {
                #[inline(always)]
                fn resolve_component(&self) -> &#Component {
                    self.#component
                }

                #[inline(always)]
                fn resolve_component_mut(&mut self) -> &mut #Component {
                    self.#component
                }
            }
        )*

        impl<'a> #ViewN<'a, #ContentArgs> for #ArchetypeView<'a> {
            #[inline(always)]
            fn new(
                index: usize,
                entity: &'a Entity<#Archetype>,
                #(#component: &'a mut #Component),*
            ) -> Self {
                Self { index, entity, #(#component),* }
            }
        }

        impl<'a> #SlicesN<'a, #ContentArgs> for #ArchetypeSlices<'a> {
            #[inline(always)]
            fn new(
                entity: &'a [Entity<#Archetype>],
                #(#component: &'a mut [#Component]),*
            ) -> Self {
                Self { entity, #(#component),* }
            }
        }

        impl<'a> ArchetypeCanResolve<'a, #ArchetypeView<'a>, Entity<#Archetype>> for #Archetype {
            #[inline(always)]
            fn resolve_for(&self, key: Entity<#Archetype>) -> Option<usize> {
                self.data.resolve(key)
            }

            #[inline(always)]
            fn resolve_view(&'a mut self, key: Entity<#Archetype>) -> Option<#ArchetypeView<'a>> {
                self.data.get_view_mut(key)
            }
        }

        impl<'a> ArchetypeCanResolve<'a, #ArchetypeView<'a>, EntityRaw<#Archetype>> for #Archetype {
            #[inline(always)]
            fn resolve_for(&self, key: EntityRaw<#Archetype>) -> Option<usize> {
                self.data.resolve(key)
            }

            #[inline(always)]
            fn resolve_view(&'a mut self, key: EntityRaw<#Archetype>) -> Option<#ArchetypeView<'a>> {
                self.data.get_view_mut(key)
            }
        }
    )
}

#[allow(non_snake_case)]
fn with_capacity_param(archetype_data: &DataArchetype) -> TokenStream {
    let archetype = format_ident!("{}", to_snake(&archetype_data.name));
    quote!(#archetype: usize)
}

#[allow(non_snake_case)]
fn with_capacity_new(archetype_data: &DataArchetype) -> TokenStream {
    let archetype = format_ident!("{}", to_snake(&archetype_data.name));
    quote!(with_capacity(capacity.#archetype))
}
