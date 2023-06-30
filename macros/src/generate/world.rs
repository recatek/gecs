use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use xxhash_rust::xxh3::xxh3_128;

use crate::data::{DataArchetype, DataCapacity, DataWorld};

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
    let EntityWorld = format_ident!("Entity{}", world_data.name);
    let EntityWorldExt = format_ident!("Entity{}Ext", world_data.name);
    let EntityRawWorld = format_ident!("EntityRaw{}", world_data.name);
    let EntityRawWorldExt = format_ident!("EntityRaw{}Ext", world_data.name);
    let WorldDispatch = format_ident!("{}DispatchInternal", world_data.name);

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
    let __ecs_find_unique = format_ident!("__ecs_find_{}", unique_hash);
    let __ecs_find_borrow_unique = format_ident!("__ecs_find_borrow_{}", unique_hash);
    let __ecs_iter_unique = format_ident!("__ecs_iter_{}", unique_hash);
    let __ecs_iter_borrow_unique = format_ident!("__ecs_iter_borrow_{}", unique_hash);

    quote!(
        #( pub use #ecs_world_sealed::#Archetype; )*
        pub use #ecs_world_sealed::{#World, #EntityWorld, #EntityRawWorld};

        #[doc(hidden)]
        pub use #ecs_world_sealed::{#WorldDispatch, #EntityWorldExt, #EntityRawWorldExt};

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
                /// Expects a capacity value for every dynamic archetype.
                ///
                /// This will allocate all archetypes to either their fixed size, or the given
                /// dynamic capacity. If a given dynamic capacity is 0, that archetype will not
                /// allocate until an entity is created in it.
                ///
                /// # Panics
                ///
                /// This will panic if given a size that exceeds the maximum possible capacity
                /// value for an archetype (currently `16,777,216`).
                pub fn with_capacity(#(#with_capacity_param)*) -> Self {
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
                pub fn destroy_any(&mut self, entity: EntityAny) -> bool {
                    match entity.into() {
                        #(
                            #EntityWorld::#Archetype(entity) =>
                                self.#archetype.destroy(entity).is_some(),
                        )*
                    }
                }
            }

            impl ArchetypeContainer for #World {}

            #(
                impl HasArchetype<#Archetype> for #World {
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
            pub enum #EntityWorld {
                #( #Archetype(Entity<#Archetype>), )*
            }

            #[derive(Clone, Copy)]
            pub enum #EntityRawWorld {
                #( #Archetype(EntityRaw<#Archetype>), )*
            }

            // Combined dispatch table for resolving both key types.
            #[doc(hidden)]
            pub enum #WorldDispatch {
                #( #Archetype(Entity<#Archetype>), )*
                #( #ArchetypeRaw(EntityRaw<#Archetype>), )*
            }

            pub trait #EntityWorldExt {
                /// Resolve this entity into an enum for accessing its type.
                fn resolve(self) -> #EntityWorld;
            }

            pub trait #EntityRawWorldExt {
                /// Resolve this raw entity into an enum for accessing its type.
                fn resolve(self) -> #EntityRawWorld;
            }

            // Resolve dispatch implementation ----------------------------------------------------

            #(
                impl #EntityWorldExt for Entity<#Archetype> {
                    #[inline(always)]
                    fn resolve(self) -> #EntityWorld {
                        self.into()
                    }
                }

                impl #EntityRawWorldExt for EntityRaw<#Archetype> {
                    #[inline(always)]
                    fn resolve(self) -> #EntityRawWorld {
                        self.into()
                    }
                }

                impl From<Entity<#Archetype>> for #EntityWorld {
                    #[inline(always)]
                    fn from(entity: Entity<#Archetype>) -> Self {
                        #EntityWorld::#Archetype(entity)
                    }
                }

                impl From<EntityRaw<#Archetype>> for #EntityRawWorld {
                    #[inline(always)]
                    fn from(entity: EntityRaw<#Archetype>) -> Self {
                        #EntityRawWorld::#Archetype(entity)
                    }
                }

                impl From<Entity<#Archetype>> for #WorldDispatch {
                    #[inline(always)]
                    fn from(entity: Entity<#Archetype>) -> Self {
                        #WorldDispatch::#Archetype(entity)
                    }
                }

                impl From<EntityRaw<#Archetype>> for #WorldDispatch {
                    #[inline(always)]
                    fn from(entity: EntityRaw<#Archetype>) -> Self {
                        #WorldDispatch::#ArchetypeRaw(entity)
                    }
                }
            )*

            impl #EntityWorldExt for EntityAny {
                #[inline(always)]
                fn resolve(self) -> #EntityWorld {
                    self.into()
                }
            }

            impl #EntityRawWorldExt for EntityRawAny {
                #[inline(always)]
                fn resolve(self) -> #EntityRawWorld {
                    self.into()
                }
            }

            impl From<EntityAny> for #EntityWorld {
                #[inline(always)]
                fn from(entity: EntityAny) -> Self {
                    match entity.archetype_id() {
                        #(
                            #Archetype::ARCHETYPE_ID => {
                                // We can use from_any_unchecked because we just checked the archetype
                                #EntityWorld::#Archetype(entity_from_any_unchecked(entity))
                            },
                        )*
                        _ => panic!("invalid entity type"),
                    }
                }
            }

            impl From<EntityRawAny> for #EntityRawWorld {
                #[inline(always)]
                fn from(entity: EntityRawAny) -> Self {
                    match entity.archetype_id() {
                        #(
                            #Archetype::ARCHETYPE_ID => {
                                // We can use from_any_unchecked because we just checked the archetype
                                #EntityRawWorld::#Archetype(entity_raw_from_any_unchecked(entity))
                            },
                        )*
                        _ => panic!("invalid entity type"),
                    }
                }
            }

            impl From<EntityAny> for #WorldDispatch {
                #[inline(always)]
                fn from(entity: EntityAny) -> Self {
                    match entity.archetype_id() {
                        #(
                            #Archetype::ARCHETYPE_ID => {
                                // We can use from_any_unchecked because we just checked the archetype
                                #WorldDispatch::#Archetype(entity_from_any_unchecked(entity))
                            },
                        )*
                        _ => panic!("invalid entity type"),
                    }
                }
            }

            impl From<EntityRawAny> for #WorldDispatch {
                #[inline(always)]
                fn from(entity: EntityRawAny) -> Self {
                    match entity.archetype_id() {
                        #(
                            #Archetype::ARCHETYPE_ID => {
                                // We can use from_any_unchecked because we just checked the archetype
                                #WorldDispatch::#ArchetypeRaw(entity_raw_from_any_unchecked(entity))
                            },
                        )*
                        _ => panic!("invalid entity type"),
                    }
                }
            }
        }

        #[macro_export]
        #[doc(hidden)]
        macro_rules! #__ecs_find_unique {
            ($($args:tt)*) => {
                ::gecs::__internal::__ecs_find!(#WORLD_DATA, $($args)*)
            }
        }

        #[macro_export]
        #[doc(hidden)]
        macro_rules! #__ecs_find_borrow_unique {
            ($($args:tt)*) => {
                ::gecs::__internal::__ecs_find_borrow!(#WORLD_DATA, $($args)*)
            }
        }

        #[macro_export]
        #[doc(hidden)]
        macro_rules! #__ecs_iter_unique {
            ($($args:tt)*) => {
                ::gecs::__internal::__ecs_iter!(#WORLD_DATA, $($args)*);
            }
        }

        #[macro_export]
        #[doc(hidden)]
        macro_rules! #__ecs_iter_borrow_unique {
            ($($args:tt)*) => {
                ::gecs::__internal::__ecs_iter_borrow!(#WORLD_DATA, $($args)*);
            }
        }

        #[doc(inline)]
        pub use #__ecs_find_unique as ecs_find;
        #[doc(inline)]
        pub use #__ecs_find_borrow_unique as ecs_find_borrow;
        #[doc(inline)]
        pub use #__ecs_iter_unique as ecs_iter;
        #[doc(inline)]
        pub use #__ecs_iter_borrow_unique as ecs_iter_borrow;
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

    let ArchetypeEntries = format_ident!("{}Entries", archetype_data.name);
    let ArchetypeSlices = format_ident!("{}Slices", archetype_data.name);

    let EntriesN = format_ident!("Entries{}", count_str);
    let SlicesN = format_ident!("Slices{}", count_str);
    let ContentArgs = quote!(#Archetype, #(#Component),*);

    let (StorageN, StorageArgs) = match &archetype_data.build_data.as_ref().unwrap().capacity {
        DataCapacity::Fixed(expr) => {
            let StorageFixed = format_ident!("StorageFixed{}", count_str);
            (StorageFixed, quote!(Self, #(#Component,)* { #expr }))
        }
        DataCapacity::Dynamic => {
            let StorageDynamic = format_ident!("StorageDynamic{}", count_str);
            (StorageDynamic, quote!(Self, #(#Component,)*))
        }
    };

    // Generated subsections
    let with_capacity = match &archetype_data.build_data.as_ref().unwrap().capacity {
        DataCapacity::Fixed(_) => quote!(),
        DataCapacity::Dynamic => quote!(
            /// Constructs a new archetype pre-allocated to the given storage capacity.
            ///
            /// If the given capacity would result in zero size, this will not allocate.
            #[inline(always)]
            pub fn with_capacity(capacity: usize) -> Self {
                Self { data: #StorageN::with_capacity(capacity) }
            }
        ),
    };

    // Function names
    let get_slice = (0..count)
        .into_iter()
        .map(|idx| format_ident!("get_slice_{}", idx.to_string()));
    let get_slice_mut = (0..count)
        .into_iter()
        .map(|idx| format_ident!("get_slice_mut_{}", idx.to_string()));
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

            #with_capacity // Only generated for dynamic storage

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
            pub const fn version(&self) -> Version {
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
            pub fn create(&mut self, data: (#(#Component,)*)) -> Entity<#Archetype> {
                self.data.try_push(data).expect("failed to push to full archetype")
            }

            /// Creates a new entity with the given components in the archetype, if there's room.
            ///
            /// Returns a handle for accessing the new entity, or `None` if the archetype is full.
            #[inline(always)]
            pub fn try_create(&mut self, data: (#(#Component,)*)) -> Option<Entity<#Archetype>> {
                self.data.try_push(data)
            }

            /// If the entity exists in the archetype, this returns its dense data slice index.
            /// The returned index is guaranteed to be within bounds of the dense data slices.
            #[inline(always)]
            pub fn resolve<T>(&self, entity: T) -> Option<usize>
            where
                Self: CanResolve<T>
            {
                <Self as CanResolve<T>>::resolve_for(self, entity)
            }

            /// If the entity exists in the archetype, this destroys it and returns its components.
            #[inline(always)]
            pub fn destroy(&mut self, entity: Entity<#Archetype>) -> Option<(#(#Component,)*)> {
                self.data.remove(entity)
            }

            #[inline(always)]
            pub fn get_all_entries_mut<'a, K>(
                &'a mut self,
                entity_key: K
            ) -> Option<(usize, #ArchetypeEntries<'a>)>
            where
                #StorageN<#StorageArgs>: CanResolve<K>
            {
                self.data.get_all_entries_mut(entity_key)
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

        impl ComponentContainer for #Archetype {}

        #(
            impl HasComponent<#Component> for #Archetype {
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
            }
        )*

        impl Archetype for #Archetype {
            #[allow(unconditional_panic)]
            const ARCHETYPE_ID: u8 = #ARCHETYPE_ID;

            type Components = (#(#Component,)*);
            type Entries<'a> = #ArchetypeEntries<'a>;
            type Slices<'a> = #ArchetypeSlices<'a>;

            #[inline(always)]
            fn get_slice_entities(&self) -> &[Entity<#Archetype>] {
                self.data.get_slice_entities()
            }
        }

        pub struct #ArchetypeEntries<'a> {
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

        impl<'a> #EntriesN<'a, #ContentArgs> for #ArchetypeEntries<'a> {
            #[inline(always)]
            fn new(
                entity: &'a Entity<#Archetype>,
                #(#component: &'a mut #Component),*
            ) -> Self {
                Self { entity, #(#component),* }
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

        impl CanResolve<Entity<#Archetype>> for #Archetype {
            #[inline(always)]
            fn resolve_for(&self, entity: Entity<#Archetype>) -> Option<usize> {
                self.data.resolve(entity)
            }
        }

        impl CanResolve<EntityRaw<#Archetype>> for #Archetype {
            #[inline(always)]
            fn resolve_for(&self, entity: EntityRaw<#Archetype>) -> Option<usize> {
                self.data.resolve(entity)
            }
        }
    )
}

#[allow(non_snake_case)]
fn with_capacity_param(archetype_data: &DataArchetype) -> TokenStream {
    let archetype_capacity = format_ident!("capacity_{}", to_snake(&archetype_data.name));
    match archetype_data.build_data.as_ref().unwrap().capacity {
        DataCapacity::Fixed(_) => quote!(),
        DataCapacity::Dynamic => quote!(#archetype_capacity: usize,),
    }
}

#[allow(non_snake_case)]
fn with_capacity_new(archetype_data: &DataArchetype) -> TokenStream {
    let archetype_capacity = format_ident!("capacity_{}", to_snake(&archetype_data.name));
    match archetype_data.build_data.as_ref().unwrap().capacity {
        DataCapacity::Fixed(_) => quote!(new()),
        DataCapacity::Dynamic => quote!(with_capacity(#archetype_capacity)),
    }
}

fn to_snake(name: &String) -> String {
    name.from_case(Case::Pascal).to_case(Case::Snake)
}

fn top_most_ancestor_of_call_site_span() -> String {
    #![allow(dead_code, unstable_name_collisions)]
    /// for code without the `proc_macro_span` feature
    trait ParentSpanPolyfill {
        fn parent(&self) -> Option<::proc_macro::Span> {
            None
        }
    }
    impl ParentSpanPolyfill for ::proc_macro::Span {}

    let mut span = ::proc_macro::Span::call_site();
    while let Some(parent_span) = span.parent() {
        span = parent_span;
    }

    format!("{span:?}")
}
