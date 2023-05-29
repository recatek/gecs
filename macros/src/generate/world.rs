use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use xxhash_rust::xxh3::xxh3_128;

use crate::data::{DataArchetype, DataCapacity, DataWorld};

#[allow(non_snake_case)] // Allow for type-like names to make quote!() clearer
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
    let Archetype = world_data
        .archetypes
        .iter()
        .map(|archetype| format_ident!("{}", archetype.name))
        .collect::<Vec<_>>();

    // Variables and fields
    let archetype = world_data
        .archetypes
        .iter()
        .map(|archetype| format_ident!("{}", to_snake(&archetype.name)))
        .collect::<Vec<_>>();

    // Generated subsections
    let section_archetype = world_data
        .archetypes
        .iter()
        .enumerate()
        .map(|(index, archetype)| section_archetype(index, archetype))
        .collect::<Vec<_>>();

    // Macros
    let __ecs_find_unique = format_ident!("__ecs_find_{}", unique_hash);
    let __ecs_iter_unique = format_ident!("__ecs_iter_{}", unique_hash);
    let __ecs_find_borrow_unique = format_ident!("__ecs_find_borrow_{}", unique_hash);
    let __ecs_iter_borrow_unique = format_ident!("__ecs_iter_borrow_{}", unique_hash);

    quote!(
        pub use #ecs_world_sealed::{#World, #EntityWorld, #EntityWorldExt};
        #( pub use #ecs_world_sealed::#Archetype; )*

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
                /// Removes a given entity of any type from the world, if it exists.
                ///
                /// Return `true` if the entity was successfully removed, or `false` otherwise.
                ///
                /// Unlike other remove functions, this does not return the removed components.
                /// If you need the returned components from an `EntityAny`, use the entity's
                /// `resolve` type disambiguator and a match statement to get an `Entity<A>`.
                pub fn remove_any(&mut self, entity: EntityAny) -> bool {
                    match entity.into() {
                        #(
                            #EntityWorld::#Archetype(entity) =>
                                self.#archetype.remove(entity).is_some(),
                        )*
                    }
                }
            }

            impl ArchetypeContainer for #World {}

            #(
                impl HasArchetype<#Archetype> for #World {
                    #[inline(always)]
                    fn resolve_len(&self) -> usize {
                        self.#archetype.len()
                    }

                    #[inline(always)]
                    fn resolve_capacity(&self) -> usize {
                        self.#archetype.capacity()
                    }

                    #[inline(always)]
                    fn resolve_is_empty(&self) -> bool {
                        self.#archetype.is_empty()
                    }

                    #[inline(always)]
                    fn resolve_push(&mut self, components: <#Archetype as Archetype>::Components)
                        -> Option<Entity<#Archetype>>
                    {
                        self.#archetype.push(components)
                    }

                    #[inline(always)]
                    fn resolve_remove(&mut self, entity: Entity<#Archetype>)
                        -> Option<<#Archetype as Archetype>::Components>
                    {
                        self.#archetype.remove(entity)
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

            pub trait #EntityWorldExt {
                fn resolve(self) -> #EntityWorld;
            }

            #(
                impl #EntityWorldExt for Entity<#Archetype> {
                    #[inline(always)]
                    fn resolve(self) -> #EntityWorld {
                        self.into()
                    }
                }

                impl From<Entity<#Archetype>> for #EntityWorld {
                    #[inline(always)]
                    fn from(entity: Entity<#Archetype>) -> Self {
                        #EntityWorld::#Archetype(entity)
                    }
                }
            )*

            impl #EntityWorldExt for EntityAny {
                #[inline(always)]
                fn resolve(self) -> #EntityWorld {
                    self.into()
                }
            }

            impl From<EntityAny> for #EntityWorld {
                #[inline(always)]
                fn from(entity: EntityAny) -> Self {
                    match entity.type_id() {
                        #(
                            #Archetype::TYPE_ID => {
                                #EntityWorld::#Archetype(Entity::from_any(entity))
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
        macro_rules! #__ecs_iter_unique {
            ($($args:tt)*) => {
                ::gecs::__internal::__ecs_iter!(#WORLD_DATA, $($args)*);
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
        macro_rules! #__ecs_iter_borrow_unique {
            ($($args:tt)*) => {
                ::gecs::__internal::__ecs_iter_borrow!(#WORLD_DATA, $($args)*);
            }
        }

        #[doc(hidden)]
        pub use #__ecs_find_unique as ecs_find;
        #[doc(hidden)]
        pub use #__ecs_iter_unique as ecs_iter;
        #[doc(hidden)]
        pub use #__ecs_find_borrow_unique as ecs_find_borrow;
        #[doc(hidden)]
        pub use #__ecs_iter_borrow_unique as ecs_iter_borrow;
    )
}

#[allow(non_snake_case)] // Allow for type-like names to make quote!() clearer
fn section_archetype(raw_index: usize, archetype_data: &DataArchetype) -> TokenStream {
    let count = archetype_data.components.len();
    let count_str = count.to_string();
    let index: u8 = raw_index
        .checked_add(1) // The raw index starts at 0, but IDs start at 1
        .expect("archetype index exceeds u8 bounds")
        .try_into()
        .expect("archetype index exceeds u8 bounds");

    // Types and traits
    let Archetype = format_ident!("{}", archetype_data.name);
    let Component = archetype_data
        .components
        .iter()
        .map(|component| format_ident!("{}", component.name))
        .collect::<Vec<_>>();

    let ArchetypeSlices = format_ident!("{}Slices", archetype_data.name);
    let ArchetypeSlicesType = format_ident!("Slices{}", count_str);
    let ArchetypeSlicesArgs = quote!(#Archetype, #(#Component),*);

    let (StorageType, StorageArgs) = match &archetype_data.capacity {
        DataCapacity::Literal(lit) => {
            let StorageFixed = format_ident!("StorageFixed{}", count_str);
            (StorageFixed, quote!(Self, #(#Component,)* #lit))
        }
        DataCapacity::Constant(name) => {
            let StorageFixed = format_ident!("StorageFixed{}", count_str);
            let CAPACITY = format_ident!("{}", name);
            (StorageFixed, quote!(Self, #(#Component,)* #CAPACITY))
        }
        DataCapacity::Dynamic => todo!("dynamic archetype capacity not yet supported"),
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
            data: #StorageType<#StorageArgs>,
        }

        impl #Archetype {
            /// Constructs a new, empty archetype.
            ///
            /// If the archetype uses dynamic storage, this archetype will not allocate until
            /// an entity is pushed into it. Otherwise, for static storage, the full capacity
            /// will be allocated on creation of the archetype.
            #[inline(always)]
            pub fn new() -> Self {
                Self { data: #StorageType::new() }
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

            /// Adds a new entity with the given components to the archetype, if there's space.
            ///
            /// Returns a handle for accessing the new entity, or `None` if the archetype is full.
            #[inline(always)]
            pub fn push(&mut self, components: (#(#Component,)*)) -> Option<Entity<#Archetype>> {
                self.data.push(components)
            }

            /// If the entity exists in the archetype, this returns its dense data slice index.
            /// The returned index is guaranteed to be within bounds of the dense data slices.
            #[inline(always)]
            pub fn resolve(&self, entity: Entity<#Archetype>) -> Option<usize> {
                self.data.resolve(entity)
            }

            /// If the entity exists in the archetype, this removes it and returns its components.
            #[inline(always)]
            pub fn remove(&mut self, entity: Entity<#Archetype>) -> Option<(#(#Component,)*)> {
                self.data.remove(entity)
            }

            /// Returns mutable slices to all data for all entities in the archetype. To get the
            /// data index for a specific entity using this function, use the `resolve` function.
            #[inline(always)]
            pub fn get_all_slices(&mut self) -> #ArchetypeSlices {
                self.data.get_all_slices()
            }
        }

        impl ComponentContainer for #Archetype { }

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
            // See https://stackoverflow.com/questions/66838439 for info on this hack
            #[allow(unconditional_panic)]
            const TYPE_ID: std::num::NonZeroU8 = match std::num::NonZeroU8::new(#index) {
                Some(v) => v,
                None => [][0],
            };

            type Components = (#(#Component,)*);

            #[inline(always)]
            fn get_slice_entities(&self) -> &[Entity<#Archetype>] {
                self.data.get_slice_entities()
            }
        }

        pub struct #ArchetypeSlices<'a> {
            pub entities: &'a [Entity<#Archetype>],
            #(
                pub #component: &'a mut [#Component],
            )*
        }

        impl<'a> #ArchetypeSlicesType<'a, #ArchetypeSlicesArgs> for #ArchetypeSlices<'a> {
            #[inline(always)]
            fn new(
                entities: &'a [Entity<#Archetype>],
                #(#component: &'a mut [#Component]),*
            ) -> Self {
                Self { entities, #(#component),* }
            }
        }
    )
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
