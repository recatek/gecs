use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{format_ident, quote};
use xxhash_rust::xxh3::xxh3_128;

use crate::data::{DataArchetype, DataWorld};
use crate::util;

#[allow(non_snake_case)]
#[allow(unused_variables)] // For unused feature-controlled generation elements
pub fn generate_world(world_data: &DataWorld, raw_input: &str) -> TokenStream {
    let world_snake = util::to_snake(&world_data.name);
    let input_hash = xxh3_128(raw_input.as_bytes());

    // Module
    let ecs_world_sealed = format_ident!("ecs_{}_sealed", world_snake);

    // Constants and literals
    let WORLD_DATA = world_data.to_base64();

    // Types and traits
    let World = format_ident!("{}", world_data.name);
    let WorldCapacity = format_ident!("{}Capacity", world_data.name);
    let __WorldSelectTotal = format_ident!("__{}SelectTotal", world_data.name);

    let Archetype = world_data
        .archetypes
        .iter()
        .map(|archetype| format_ident!("{}", archetype.name))
        .collect::<Vec<_>>();
    let ArchetypeComponents = world_data
        .archetypes
        .iter()
        .map(|archetype| format_ident!("{}Components", archetype.name))
        .collect::<Vec<_>>();
    let ArchetypeDirect = world_data
        .archetypes
        .iter()
        .map(|archetype| format_ident!("{}Direct", archetype.name))
        .collect::<Vec<_>>();

    // Variables and fields
    let archetype = world_data
        .archetypes
        .iter()
        .map(|archetype| format_ident!("{}", util::to_snake(&archetype.name)))
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
    let section_event_iter = section_event_iter(&world_data);
    let section_events = section_events_world(&world_data);

    // Documentation helpers
    let world_doc_archetypes = world_data
        .archetypes
        .iter()
        .map(|archetype| {
            format!(
                "- `{}`: [`{}`],",
                util::to_snake(&archetype.name),
                &archetype.name
            )
        })
        .collect::<Vec<_>>();

    // Macros
    let __expand_ecs_find_hash = format_ident!("__expand_ecs_find_{}", input_hash);
    let __expand_ecs_find_borrow_hash = format_ident!("__expand_ecs_find_borrow_{}", input_hash);
    let __expand_ecs_iter_hash = format_ident!("__expand_ecs_iter_{}", input_hash);
    let __expand_ecs_iter_borrow_hash = format_ident!("__expand_ecs_iter_borrow_{}", input_hash);
    let __expand_ecs_iter_destroy_hash = format_ident!("__expand_ecs_iter_destroy_{}", input_hash);

    quote!(
        pub use #ecs_world_sealed::{
            #World,
            #WorldCapacity,

            SelectArchetype,
            SelectEntity,
            SelectEntityDirect,

            #(
                #Archetype,
                #ArchetypeComponents,
            )*
        };

        #[doc(hidden)]
        pub use #ecs_world_sealed::{#__WorldSelectTotal};

        /// Convenience mod for accessing only archetypes in exports (for blob exports, etc.)
        pub mod archetypes {
            #(
                pub use super::#Archetype;
                pub use super::#ArchetypeComponents;
            )*
        }

        mod #ecs_world_sealed {
            use super::*;

            use ::std::cell::{Ref, RefMut};

            use ::gecs::__internal::*;

            #(#section_archetype)*

            // Will only appear if we have the events feature enabled.
            #section_event_iter

            /// The generated ECS world. See [`World`](gecs::traits::World) for more information.
            ///
            /// Contained archetypes[^1]:
            #(#[doc = #world_doc_archetypes])*
            ///
            /// [^1]: This list may change based on `#[cfg]` state.
            #[derive(Default)]
            pub struct #World {
                #(
                    pub #archetype: #Archetype,
                )*
            }

            /// The capacity constructor for an ECS world.
            ///
            /// Contained archetypes[^1]:
            #(#[doc = #world_doc_archetypes])*
            ///
            /// [^1]: This list may change based on `#[cfg]` state.
            #[derive(Default)]
            pub struct #WorldCapacity {
                #(
                    pub #with_capacity_param,
                )*
            }

            impl World for #World {
                const NUM_ARCHETYPES: usize = #num_archetypes;

                type Capacities = #WorldCapacity;

                // Will only appear if we have the events feature enabled.
                #section_events

                #[inline(always)]
                fn new() -> Self {
                    Self {
                        #( #archetype: #Archetype::new(), )*
                    }
                }

                #[inline(always)]
                fn with_capacity(capacity: #WorldCapacity) -> Self {
                    Self {
                        #( #archetype: #Archetype::#with_capacity_new, )*
                    }
                }
            }

            impl Clone for #World
            where
                #(for<'a> #Archetype: Clone,)*
            {
                /// Clones this world, including all of its data.
                ///
                /// # Panics
                ///
                /// This function will panic if any of its components are mutably borrowed,
                /// or if there is not enough memory available to perform the clone.
                #[inline(always)]
                fn clone(&self) -> Self {
                    Self {
                        #(#archetype: self.#archetype.clone(),)*
                    }
                }
            }

            #(
                impl WorldHas<#Archetype> for #World {
                    #[inline(always)]
                    fn resolve_create(
                        &mut self,
                        data: <#Archetype as Archetype>::Components,
                    ) -> Entity<#Archetype> {
                        self.#archetype.create(data)
                    }

                    #[inline(always)]
                    fn resolve_create_within_capacity(
                        &mut self,
                        data: <#Archetype as Archetype>::Components,
                    ) -> Result<Entity<#Archetype>, <#Archetype as Archetype>::Components> {
                        self.#archetype.create_within_capacity(data)
                    }

                    #[inline(always)]
                    fn resolve_destroy(
                        &mut self,
                        entity: Entity<#Archetype>,
                    ) -> Option<<#Archetype as Archetype>::Components> {
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
            /// See `SelectArchetype` in the `gecs` docs for more information.
            pub enum SelectArchetype {
                #( #Archetype, )*
            }

            #[derive(Clone, Copy)]
            /// See `SelectEntity` in the `gecs` docs for more information.
            pub enum SelectEntity {
                #( #Archetype(Entity<#Archetype>), )*
            }

            #[derive(Clone, Copy)]
            /// See `SelectEntityDirect` in the `gecs` docs for more information.
            pub enum SelectEntityDirect {
                #( #Archetype(EntityDirect<#Archetype>), )*
            }

            // Combined dispatch table for resolving both entity key types.
            /// Used internally by world queries. Not for general use.
            #[doc(hidden)]
            pub enum #__WorldSelectTotal {
                #( #Archetype(Entity<#Archetype>), )*
                #( #ArchetypeDirect(EntityDirect<#Archetype>), )*
            }

            // Resolve dispatch implementation
            #(
                impl From<Entity<#Archetype>> for SelectArchetype {
                    #[inline(always)]
                    fn from(entity: Entity<#Archetype>) -> Self {
                        SelectArchetype::#Archetype
                    }
                }

                impl From<Entity<#Archetype>> for SelectEntity {
                    #[inline(always)]
                    fn from(entity: Entity<#Archetype>) -> Self {
                        SelectEntity::#Archetype(entity)
                    }
                }

                impl From<&Entity<#Archetype>> for SelectEntity {
                    #[inline(always)]
                    fn from(entity: &Entity<#Archetype>) -> Self {
                        SelectEntity::#Archetype(*entity)
                    }
                }

                impl From<EntityDirect<#Archetype>> for SelectArchetype {
                    #[inline(always)]
                    fn from(entity: EntityDirect<#Archetype>) -> Self {
                        SelectArchetype::#Archetype
                    }
                }

                impl From<EntityDirect<#Archetype>> for SelectEntityDirect {
                    #[inline(always)]
                    fn from(entity: EntityDirect<#Archetype>) -> Self {
                        SelectEntityDirect::#Archetype(entity)
                    }
                }

                impl From<&EntityDirect<#Archetype>> for SelectEntityDirect {
                    #[inline(always)]
                    fn from(entity: &EntityDirect<#Archetype>) -> Self {
                        SelectEntityDirect::#Archetype(*entity)
                    }
                }

                impl From<Entity<#Archetype>> for #__WorldSelectTotal {
                    #[inline(always)]
                    fn from(entity: Entity<#Archetype>) -> Self {
                        #__WorldSelectTotal::#Archetype(entity)
                    }
                }

                impl From<EntityDirect<#Archetype>> for #__WorldSelectTotal {
                    #[inline(always)]
                    fn from(entity: EntityDirect<#Archetype>) -> Self {
                        #__WorldSelectTotal::#ArchetypeDirect(entity)
                    }
                }

                impl WorldCanResolve<Entity<#Archetype>> for #World {
                    #[inline(always)]
                    fn resolve_contains(
                        &self,
                        entity: Entity<#Archetype>,
                    ) -> bool {
                        self.archetype::<#Archetype>().contains(entity)
                    }

                    #[inline(always)]
                    fn resolve_direct(
                        &self,
                        entity: Entity<#Archetype>,
                    ) -> Option<EntityDirect<#Archetype>> {
                        self.archetype::<#Archetype>().to_direct(entity)
                    }

                    #[inline(always)]
                    fn resolve_destroy(
                        &mut self,
                        entity: Entity<#Archetype>
                    ) -> Option<<#Archetype as Archetype>::Components> {
                        self.archetype_mut::<#Archetype>().destroy(entity)
                    }
                }

                impl WorldCanResolve<EntityDirect<#Archetype>> for #World {
                    #[inline(always)]
                    fn resolve_contains(
                        &self,
                        entity: EntityDirect<#Archetype>,
                    ) -> bool {
                        self.archetype::<#Archetype>().contains(entity)
                    }

                    #[inline(always)]
                    fn resolve_direct(
                        &self,
                        entity: EntityDirect<#Archetype>,
                    ) -> Option<EntityDirect<#Archetype>> {
                        self.archetype::<#Archetype>().to_direct(entity)
                    }

                    #[inline(always)]
                    fn resolve_destroy(
                        &mut self,
                        entity: EntityDirect<#Archetype>
                    ) -> Option<<#Archetype as Archetype>::Components> {
                        self.archetype_mut::<#Archetype>().destroy(entity)
                    }
                }
            )*

            impl SelectArchetype {
                #[inline(always)]
                pub fn archetype_id(self) -> ArchetypeId {
                    match self {
                        #(
                            SelectArchetype::#Archetype => #Archetype::ARCHETYPE_ID,
                        )*
                    }
                }
            }

            impl WorldCanResolve<EntityAny> for #World {
                #[inline(always)]
                fn resolve_contains(
                    &self,
                    entity: EntityAny,
                ) -> bool {
                    match entity.try_into() {
                        #(
                            Ok(SelectEntity::#Archetype(entity)) =>
                                self.#archetype.contains(entity),
                        )*
                        Err(_) => panic!("invalid entity type"),
                    }
                }

                #[inline(always)]
                fn resolve_direct(
                    &self,
                    entity: EntityAny,
                ) -> Option<EntityDirectAny> {
                    match entity.try_into() {
                        #(
                            Ok(SelectEntity::#Archetype(entity)) =>
                                self.#archetype.to_direct(entity).map(|e| e.into()),
                        )*
                        Err(_) => panic!("invalid entity type"),
                    }
                }

                #[inline(always)]
                fn resolve_destroy(
                    &mut self,
                    entity: EntityAny,
                ) -> Option<()> {
                    match entity.try_into() {
                        #(
                            Ok(SelectEntity::#Archetype(entity)) =>
                                self.#archetype.destroy(entity).map(|_| ()),
                        )*
                        Err(_) => panic!("invalid entity type"),
                    }
                }
            }

            impl WorldCanResolve<EntityDirectAny> for #World {
                #[inline(always)]
                fn resolve_contains(
                    &self,
                    entity: EntityDirectAny,
                ) -> bool {
                    match entity.try_into() {
                        #(
                            Ok(SelectEntityDirect::#Archetype(entity)) =>
                                self.#archetype.contains(entity),
                        )*
                        Err(_) => panic!("invalid entity type"),
                    }
                }

                #[inline(always)]
                fn resolve_direct(
                    &self,
                    entity: EntityDirectAny,
                ) -> Option<EntityDirectAny> {
                    match entity.try_into() {
                        #(
                            Ok(SelectEntityDirect::#Archetype(entity)) =>
                                self.#archetype.to_direct(entity).map(|e| e.into()),
                        )*
                        Err(_) => panic!("invalid entity type"),
                    }
                }

                #[inline(always)]
                fn resolve_destroy(
                    &mut self,
                    entity: EntityDirectAny,
                ) -> Option<()> {
                    match entity.try_into() {
                        #(
                            Ok(SelectEntityDirect::#Archetype(entity)) =>
                                self.#archetype.destroy(entity).map(|_| ()),
                        )*
                        Err(_) => panic!("invalid entity type"),
                    }
                }
            }

            impl TryFrom<ArchetypeId> for SelectArchetype {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(id: ArchetypeId) -> Result<Self, EcsError> {
                    match id {
                        #(
                            #Archetype::ARCHETYPE_ID => Ok(SelectArchetype::#Archetype),
                        )*
                        _ => Err(EcsError::InvalidEntityType),
                    }
                }
            }

            impl TryFrom<EntityAny> for SelectArchetype {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(entity: EntityAny) -> Result<Self, EcsError> {
                    match entity.archetype_id() {
                        #(
                            #Archetype::ARCHETYPE_ID => Ok(SelectArchetype::#Archetype),
                        )*
                        _ => Err(EcsError::InvalidEntityType),
                    }
                }
            }

            impl TryFrom<EntityAny> for SelectEntity {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(entity: EntityAny) -> Result<Self, EcsError> {
                    match entity.archetype_id() {
                        #(
                            #Archetype::ARCHETYPE_ID => {
                                // We can use from_any_unchecked because we just checked the archetype
                                Ok(SelectEntity::#Archetype(
                                    Entity::<#Archetype>::from_any_unchecked(entity))
                                )
                            },
                        )*
                        _ => Err(EcsError::InvalidEntityType),
                    }
                }
            }

            impl TryFrom<EntityDirectAny> for SelectEntityDirect {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(entity: EntityDirectAny) -> Result<Self, EcsError> {
                    match entity.archetype_id() {
                        #(
                            #Archetype::ARCHETYPE_ID => {
                                // We can use from_any_unchecked because we just checked the archetype
                                Ok(SelectEntityDirect::#Archetype(
                                    EntityDirect::<#Archetype>::from_any_unchecked(entity))
                                )
                            },
                        )*
                        _ => Err(EcsError::InvalidEntityType),
                    }
                }
            }

            impl TryFrom<EntityAny> for #__WorldSelectTotal {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(entity: EntityAny) -> Result<Self, EcsError> {
                    match entity.archetype_id() {
                        #(
                            #Archetype::ARCHETYPE_ID => {
                                // We can use from_any_unchecked because we just checked the archetype
                                Ok(#__WorldSelectTotal::#Archetype(
                                    Entity::<#Archetype>::from_any_unchecked(entity)
                                ))
                            },
                        )*
                        _ => Err(EcsError::InvalidEntityType),
                    }
                }
            }

            impl TryFrom<EntityDirectAny> for #__WorldSelectTotal {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(entity: EntityDirectAny) -> Result<Self, EcsError> {
                    match entity.archetype_id() {
                        #(
                            #Archetype::ARCHETYPE_ID => {
                                // We can use from_any_unchecked because we just checked the archetype
                                Ok(#__WorldSelectTotal::#ArchetypeDirect(
                                    EntityDirect::<#Archetype>::from_any_unchecked(entity))
                                )
                            },
                        )*
                        _ => Err(EcsError::InvalidEntityType),
                    }
                }
            }

            impl TryFrom<&EntityAny> for #__WorldSelectTotal {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(entity: &EntityAny) -> Result<Self, EcsError> {
                    (*entity).try_into()
                }
            }

            impl TryFrom<&EntityDirectAny> for #__WorldSelectTotal {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(entity: &EntityDirectAny) -> Result<Self, EcsError> {
                    (*entity).try_into()
                }
            }

            impl<A: Archetype> TryFrom<&Entity<A>> for #__WorldSelectTotal
                where #__WorldSelectTotal: TryFrom<Entity<A>, Error = EcsError>
            {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(entity: &Entity<A>) -> Result<Self, EcsError> {
                    (*entity).try_into()
                }
            }

            impl<A: Archetype> TryFrom<&EntityDirect<A>> for #__WorldSelectTotal
                where #__WorldSelectTotal: TryFrom<EntityDirect<A>, Error = EcsError>
            {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(entity: &EntityDirect<A>) -> Result<Self, EcsError> {
                    (*entity).try_into()
                }
            }

            impl TryFrom<&mut EntityAny> for #__WorldSelectTotal {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(entity: &mut EntityAny) -> Result<Self, EcsError> {
                    (*entity).try_into()
                }
            }

            impl TryFrom<&mut EntityDirectAny> for #__WorldSelectTotal {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(entity: &mut EntityDirectAny) -> Result<Self, EcsError> {
                    (*entity).try_into()
                }
            }

            impl<A: Archetype> TryFrom<&mut Entity<A>> for #__WorldSelectTotal
                where #__WorldSelectTotal: TryFrom<Entity<A>, Error = EcsError>
            {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(entity: &mut Entity<A>) -> Result<Self, EcsError> {
                    (*entity).try_into()
                }
            }

            impl<A: Archetype> TryFrom<&mut EntityDirect<A>> for #__WorldSelectTotal
                where #__WorldSelectTotal: TryFrom<EntityDirect<A>, Error = EcsError>
            {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(entity: &mut EntityDirect<A>) -> Result<Self, EcsError> {
                    (*entity).try_into()
                }
            }
        }

        #[macro_export]
        #[doc(hidden)]
        /// See `ecs_find` in the `gecs` docs for more information.
        macro_rules! #__expand_ecs_find_hash {
            ($($args:tt)*) => {
                ::gecs::__internal::__expand_ecs_find!(#WORLD_DATA, $($args)*)
            }
        }

        #[macro_export]
        #[doc(hidden)]
        /// See `ecs_find_borrow` in the `gecs` docs for more information.
        macro_rules! #__expand_ecs_find_borrow_hash {
            ($($args:tt)*) => {
                ::gecs::__internal::__expand_ecs_find_borrow!(#WORLD_DATA, $($args)*)
            }
        }

        #[macro_export]
        #[doc(hidden)]
        /// See `ecs_iter` in the `gecs` docs for more information.
        macro_rules! #__expand_ecs_iter_hash {
            ($($args:tt)*) => {
                ::gecs::__internal::__expand_ecs_iter!(#WORLD_DATA, $($args)*)
            }
        }

        #[macro_export]
        #[doc(hidden)]
        /// See `ecs_iter_borrow` in the `gecs` docs for more information.
        macro_rules! #__expand_ecs_iter_borrow_hash {
            ($($args:tt)*) => {
                ::gecs::__internal::__expand_ecs_iter_borrow!(#WORLD_DATA, $($args)*)
            }
        }

        #[macro_export]
        #[doc(hidden)]
        /// See `ecs_iter_destroy` in the `gecs` docs for more information.
        macro_rules! #__expand_ecs_iter_destroy_hash {
            ($($args:tt)*) => {
                ::gecs::__internal::__expand_ecs_iter_destroy!(#WORLD_DATA, $($args)*)
            }
        }

        #[doc(inline)]
        pub use #__expand_ecs_find_hash as ecs_find;
        #[doc(inline)]
        pub use #__expand_ecs_find_borrow_hash as ecs_find_borrow;
        #[doc(inline)]
        pub use #__expand_ecs_iter_hash as ecs_iter;
        #[doc(inline)]
        pub use #__expand_ecs_iter_borrow_hash as ecs_iter_borrow;
        #[doc(inline)]
        pub use #__expand_ecs_iter_destroy_hash as ecs_iter_destroy;
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
    let component_index = (0..archetype_data.components.len())
        .into_iter()
        .map(|idx| Literal::usize_unsuffixed(idx))
        .collect::<Vec<_>>();

    // Types and traits
    let Archetype = format_ident!("{}", archetype_data.name);
    let Component = archetype_data
        .components
        .iter()
        .map(|component| {
            let name = &component.name;
            quote!(#name)
        })
        .collect::<Vec<_>>();

    let ArchetypeBorrow = format_ident!("{}Borrow", archetype_data.name);
    let ArchetypeView = format_ident!("{}View", archetype_data.name);
    let ArchetypeViewMut = format_ident!("{}ViewMut", archetype_data.name);
    let ArchetypeSlices = format_ident!("{}Slices", archetype_data.name);
    let ArchetypeComponents = format_ident!("{}Components", archetype_data.name);

    let ComponentsN = format_ident!("Components{}", count_str);
    let ViewN = format_ident!("View{}", count_str);
    let ViewMutN = format_ident!("ViewMut{}", count_str);
    let SlicesN = format_ident!("Slices{}", count_str);
    let StorageN = format_ident!("Storage{}", count_str);
    let BorrowN = format_ident!("Borrow{}", count_str);

    // Function names
    let get_slice = (0..count)
        .into_iter()
        .map(|idx| format_ident!("get_slice_{}", idx.to_string()));
    let get_slice_mut = (0..count)
        .into_iter()
        .map(|idx| format_ident!("get_slice_mut_{}", idx.to_string()));
    let borrow_component = (0..count)
        .into_iter()
        .map(|idx| format_ident!("borrow_component_{}", idx.to_string()));
    let borrow_component_mut = (0..count)
        .into_iter()
        .map(|idx| format_ident!("borrow_component_mut_{}", idx.to_string()));
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
        .map(|component| Ident::new(&component.name.as_snake_name(), Span::call_site()))
        .collect::<Vec<_>>();

    // Generated subsections
    let section_events = section_events_archetype(&archetype_data);

    // Documentation helpers
    let archetype_doc_component_types = archetype_data
        .components
        .iter()
        .map(|component| {
            format!(
                "- [`{}`]", //.
                component.name.to_string()
            )
        })
        .collect::<Vec<_>>();
    let archetype_doc_component_data = archetype_data
        .components
        .iter()
        .map(|component| {
            format!(
                "- {}: [`{}`]",
                &component.name.as_snake_name(),
                &component.name.to_string()
            )
        })
        .collect::<Vec<_>>();

    quote!(
        /// A generated ECS archetype. See [`Archetype`](gecs::traits::Archetype) for more information.
        ///
        /// Contained components[^1]:
        #(#[doc = #archetype_doc_component_types])*
        ///
        /// [^1]: This list may change based on `#[cfg]` state.
        #[derive(Default)]
        #[repr(transparent)]
        pub struct #Archetype {
            #[doc(hidden)]
            pub data: #StorageN<#Archetype, #(#Component),*>,
        }

        impl Archetype for #Archetype {
            #[allow(unconditional_panic)]
            const ARCHETYPE_ID: u8 = #ARCHETYPE_ID;

            type Components = #ArchetypeComponents;

            type Slices<'a> = #ArchetypeSlices<'a>;
            type View<'a> = #ArchetypeView<'a>;
            type ViewMut<'a> = #ArchetypeViewMut<'a>;
            type Borrow<'a> = #ArchetypeBorrow<'a>;

            // Will only appear if we have the events feature enabled.
            #section_events

            #[inline(always)]
            fn new() -> Self {
                Self { data: #StorageN::new() }
            }

            #[inline(always)]
            fn with_capacity(capacity: usize) -> Self {
                Self { data: #StorageN::with_capacity(capacity) }
            }

            #[inline(always)]
            fn len(&self) -> usize {
                self.data.len()
            }

            #[inline(always)]
            fn capacity(&self) -> usize {
                self.data.capacity()
            }

            #[inline(always)]
            fn is_empty(&self) -> bool {
                self.data.is_empty()
            }

            #[inline(always)]
            fn version(&self) -> ArchetypeVersion {
                self.data.version()
            }

            #[inline(always)]
            fn entities(&self) -> &[Entity<#Archetype>] {
                self.data.get_slice_entities()
            }

            #[inline(always)]
            fn create(
                &mut self,
                components: impl Into<Self::Components>,
            ) -> Entity<#Archetype> {
                self.data.push(components.into())
            }

            #[inline(always)]
            fn create_within_capacity(
                &mut self,
                components: impl Into<Self::Components>,
            ) -> Result<Entity<#Archetype>, #ArchetypeComponents> {
                self.data.push_within_capacity(components.into())
            }

            #[inline(always)]
            fn iter(&mut self) -> impl Iterator<Item = #ArchetypeView<'_>> {
                self.data.iter()
            }

            #[inline(always)]
            fn iter_mut(&mut self) -> impl Iterator<Item = #ArchetypeViewMut<'_>> {
                self.data.iter_mut()
            }

            #[inline(always)]
            fn get_all_slices_mut(&mut self) -> #ArchetypeSlices {
                self.data.get_all_slices_mut()
            }
        }

        #(
            impl ArchetypeHas<#Component> for #Archetype {
                const COMPONENT_ID: u8 = #COMPONENT_ID;

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
                fn resolve_extract_components(components: &Self::Components) -> &#Component {
                    &components.#component
                }

                #[inline(always)]
                fn resolve_extract_components_mut(components: &mut Self::Components) -> &mut #Component {
                    &mut components.#component
                }

                #[inline(always)]
                fn resolve_extract_view<'a>(view: &'a Self::View<'_>) -> &'a #Component {
                    &view.#component
                }

                #[inline(always)]
                fn resolve_extract_view_ref<'a>(view: &'a Self::ViewMut<'_>) -> &'a #Component {
                    &view.#component
                }

                #[inline(always)]
                fn resolve_extract_view_mut<'a>(view: &'a mut Self::ViewMut<'_>) -> &'a mut #Component {
                    &mut view.#component
                }

                #[inline(always)]
                fn resolve_extract_borrow<'a>(borrow: &'a Self::Borrow<'_>) -> Ref<'a, #Component> {
                    borrow.0.#borrow_component()
                }

                #[inline(always)]
                fn resolve_extract_borrow_mut<'a>(borrow: &'a Self::Borrow<'_>) -> RefMut<'a, #Component> {
                    borrow.0.#borrow_component_mut()
                }
            }
        )*

        impl Clone for #Archetype
        where
            #(for<'a> #Component: Clone,)*
        {
            /// Clones this archetype, including all of its data.
            ///
            /// # Panics
            ///
            /// This function will panic if any of its components are mutably borrowed,
            /// or if there is not enough memory available to perform the clone.
            #[inline(always)]
            fn clone(&self) -> Self {
                Self {
                    data: self.data.clone(),
                }
            }
        }

        /// Struct for named access to all of the components in an archetype's component tuple.
        pub struct #ArchetypeComponents {
            #(
                pub #component: #Component,
            )*
        }

        impl Components for #ArchetypeComponents {
            type Archetype = #Archetype;
            type Tuple = (#(#Component,)*);

            fn get<C>(&self) -> &C
            where
                Self::Archetype: ArchetypeHas<C>
            {
                <Self::Archetype as ArchetypeHas<C>>::resolve_extract_components(self)
            }

            fn get_mut<C>(&mut self) -> &mut C
            where
                Self::Archetype: ArchetypeHas<C>
            {
                <Self::Archetype as ArchetypeHas<C>>::resolve_extract_components_mut(self)
            }

            fn into_tuple(self) -> Self::Tuple {
                (#(self.#component,)*)
            }
        }

        impl #ComponentsN<#(#Component,)*> for #ArchetypeComponents {
            #[inline(always)]
            fn raw_new(#(#component: #Component,)*) -> Self {
                Self { #(#component,)* }
            }

            #[inline(always)]
            fn raw_get(self) -> (#(#Component,)*) {
                (#(self.#component,)*)
            }
        }

        impl From<(#(#Component,)*)> for #ArchetypeComponents {
            #[inline(always)]
            fn from(components: (#(#Component,)*)) -> #ArchetypeComponents {
                Self {
                    #(
                        #component: components.#component_index,
                    )*
                }
            }
        }

        impl From<#ArchetypeComponents> for (#(#Component,)*) {
            #[inline(always)]
            fn from(components: #ArchetypeComponents) -> (#(#Component,)*) {
                (#(components.#component,)*)
            }
        }

        impl Default for #ArchetypeComponents
        where
            #(for<'a> #Component: Default,)*
        {
            #[inline(always)]
            fn default() -> Self {
                Self {
                    #(#component: Default::default(),)*
                }
            }
        }

        /// Access to all of the stored entity and component data within this archetype.
        /// Each index in these parallel slices refers to the components for a given entity.
        /// Component access is mutable, but entity access is fixed (entities can't be moved).
        ///
        /// Contained components[^1]:
        #(#[doc = #archetype_doc_component_data])*
        ///
        /// [^1]: This list may change based on `#[cfg]` state.
        pub struct #ArchetypeSlices<'a> {
            pub entity: &'a [Entity<#Archetype>],
            #(
                pub #component: &'a mut [#Component],
            )*
        }

        impl<'a> #SlicesN<'a, #Archetype, #(#Component),*> for #ArchetypeSlices<'a> {
            #[inline(always)]
            fn new(
                entity: &'a [Entity<#Archetype>],
                #(#component: &'a mut [#Component]),*
            ) -> Self {
                Self { entity, #(#component),* }
            }
        }

        /// See [`View`](gecs::traits::View) for more information on this type.
        ///
        /// Contained components[^1]:
        #(#[doc = #archetype_doc_component_data])*
        ///
        /// [^1]: This list may change based on `#[cfg]` state.
        pub struct #ArchetypeView<'a> {
            pub entity: &'a Entity<#Archetype>,
            #(
                pub #component: &'a #Component,
            )*
        }

        /// See [`ViewMut`](gecs::traits::ViewMut) for more information on this type.
        ///
        /// Contained components[^1]:
        #(#[doc = #archetype_doc_component_data])*
        ///
        /// [^1]: This list may change based on `#[cfg]` state.
        pub struct #ArchetypeViewMut<'a> {
            pub entity: &'a Entity<#Archetype>,
            #(
                pub #component: &'a mut #Component,
            )*
        }

        impl<'a> View<'a> for #ArchetypeView<'a> {
            type Archetype = #Archetype;

            #[inline(always)]
            fn component<C>(&self) -> &C
            where
                Self::Archetype: ArchetypeHas<C> + 'a
            {
                <Self::Archetype as ArchetypeHas<C>>::resolve_extract_view(self)
            }
        }

        impl<'a> View<'a> for #ArchetypeViewMut<'a> {
            type Archetype = #Archetype;

            #[inline(always)]
            fn component<C>(&self) -> &C
            where
                Self::Archetype: ArchetypeHas<C> + 'a
            {
                <Self::Archetype as ArchetypeHas<C>>::resolve_extract_view_ref(self)
            }
        }

        impl<'a> ViewMut<'a> for #ArchetypeViewMut<'a> {
            #[inline(always)]
            fn component_mut<C>(&mut self) -> &mut C
            where
                Self::Archetype: ArchetypeHas<C> + 'a
            {
                <Self::Archetype as ArchetypeHas<C>>::resolve_extract_view_mut(self)
            }
        }

        impl<'a> #ViewN<'a, #Archetype, #(#Component),*> for #ArchetypeView<'a> {
            #[inline(always)]
            fn new(
                entity: &'a Entity<#Archetype>,
                #(#component: &'a #Component),*
            ) -> Self {
                Self { entity, #(#component),* }
            }
        }

        impl<'a> #ViewMutN<'a, #Archetype, #(#Component),*> for #ArchetypeViewMut<'a> {
            #[inline(always)]
            fn new(
                entity: &'a Entity<#Archetype>,
                #(#component: &'a mut #Component),*
            ) -> Self {
                Self { entity, #(#component),* }
            }
        }

        /// See [`Borrow`](gecs::traits::Borrow) for more information on this type.
        ///
        /// Contained components[^1]:
        #(#[doc = #archetype_doc_component_types])*
        ///
        /// [^1]: This list may change based on `#[cfg]` state.
        #[repr(transparent)]
        #[derive(Clone, Copy)]
        pub struct #ArchetypeBorrow<'a>(#BorrowN<'a, #Archetype, #(#Component),*>);

        impl<'a> Borrow<'a> for #ArchetypeBorrow<'a> {
            type Archetype = #Archetype;

            #[inline(always)]
            fn entity(&self) -> &Entity<#Archetype> {
                self.0.entity()
            }
        }

        impl ArchetypeCanResolve<Entity<#Archetype>> for #Archetype {
            #[inline(always)]
            fn resolve_for(
                &self,
                entity: Entity<#Archetype>,
            ) -> Option<usize> {
                self.data.resolve(entity)
            }

            #[inline(always)]
            fn resolve_direct(
                &self,
                entity: Entity<#Archetype>,
            ) -> Option<EntityDirect<#Archetype>> {
                self.data.to_direct(entity)
            }

            #[inline(always)]
            fn resolve_view(
                &mut self,
                entity: Entity<#Archetype>,
            ) -> Option<<Self as Archetype>::View<'_>> {
                self.data.get_view(entity)
            }

            #[inline(always)]
            fn resolve_view_direct(
                &mut self,
                entity: Entity<#Archetype>,
            ) -> Option<(<Self as Archetype>::View<'_>, EntityDirect<Self>)> {
                self.data.get_view_direct(entity)
            }

            #[inline(always)]
            fn resolve_view_mut(
                &mut self,
                entity: Entity<#Archetype>,
            ) -> Option<<Self as Archetype>::ViewMut<'_>> {
                self.data.get_view_mut(entity)
            }

            #[inline(always)]
            fn resolve_view_mut_direct(
                &mut self,
                entity: Entity<#Archetype>,
            ) -> Option<(<Self as Archetype>::ViewMut<'_>, EntityDirect<Self>)> {
                self.data.get_view_mut_direct(entity)
            }

            #[inline(always)]
            fn resolve_borrow(
                &self,
                entity: Entity<#Archetype>,
            ) -> Option<<Self as Archetype>::Borrow<'_>> {
                self.data.begin_borrow(entity).map(
                    #ArchetypeBorrow
                )
            }

            #[inline(always)]
            fn resolve_borrow_direct(
                &self,
                entity: Entity<#Archetype>,
            ) -> Option<(<Self as Archetype>::Borrow<'_>, EntityDirect<Self>)> {
                self.data.begin_borrow_direct(entity).map(
                    |(borrow, direct)| (#ArchetypeBorrow(borrow), direct)
                )
            }

            #[inline(always)]
            fn resolve_destroy(
                &mut self,
                entity: Entity<#Archetype>,
            ) -> Option<<Self as Archetype>::Components> {
                self.data.destroy(entity)
            }
        }

        impl ArchetypeCanResolve<EntityDirect<#Archetype>> for #Archetype {
            #[inline(always)]
            fn resolve_for(&self, entity: EntityDirect<#Archetype>) -> Option<usize> {
                self.data.resolve(entity)
            }

            #[inline(always)]
            fn resolve_direct(
                &self,
                entity: EntityDirect<#Archetype>,
            ) -> Option<EntityDirect<#Archetype>> {
                self.data.to_direct(entity)
            }

            #[inline(always)]
            fn resolve_view(
                &mut self,
                entity: EntityDirect<#Archetype>,
            ) -> Option<<Self as Archetype>::View<'_>> {
                self.data.get_view(entity)
            }

            #[inline(always)]
            fn resolve_view_direct(
                &mut self,
                entity: EntityDirect<#Archetype>,
            ) -> Option<(<Self as Archetype>::View<'_>, EntityDirect<Self>)> {
                self.data.get_view_direct(entity)
            }

            #[inline(always)]
            fn resolve_view_mut(
                &mut self,
                entity: EntityDirect<#Archetype>,
            ) -> Option<<Self as Archetype>::ViewMut<'_>> {
                self.data.get_view_mut(entity)
            }

            #[inline(always)]
            fn resolve_view_mut_direct(
                &mut self,
                entity: EntityDirect<#Archetype>,
            ) -> Option<(<Self as Archetype>::ViewMut<'_>, EntityDirect<Self>)> {
                self.data.get_view_mut_direct(entity)
            }

            #[inline(always)]
            fn resolve_borrow(
                &self,
                entity: EntityDirect<#Archetype>,
            ) -> Option<<Self as Archetype>::Borrow<'_>> {
                self.data.begin_borrow(entity).map(
                    #ArchetypeBorrow
                )
            }

            #[inline(always)]
            fn resolve_borrow_direct(
                &self,
                entity: EntityDirect<#Archetype>,
            ) -> Option<(<Self as Archetype>::Borrow<'_>, EntityDirect<Self>)> {
                self.data.begin_borrow_direct(entity).map(
                    |(borrow, direct)| (#ArchetypeBorrow(borrow), direct)
                )
            }

            #[inline(always)]
            fn resolve_destroy(
                &mut self,
                entity: EntityDirect<#Archetype>,
            ) -> Option<<Self as Archetype>::Components> {
                self.data.destroy(entity)
            }
        }

        impl ArchetypeCanResolve<EntityAny> for #Archetype {
            #[inline(always)]
            fn resolve_for(&self, entity: EntityAny) -> Option<usize> {
                self.data.resolve(Entity::<Self>::try_from(entity).ok()?)
            }

            #[inline(always)]
            fn resolve_direct(
                &self,
                entity: EntityAny,
            ) -> Option<EntityDirectAny> {
                self.data.resolve_direct(Entity::<Self>::try_from(entity).ok()?).map(Into::into)
            }

            #[inline(always)]
            fn resolve_view(
                &mut self,
                entity: EntityAny,
            ) -> Option<<Self as Archetype>::View<'_>> {
                self.data.get_view(Entity::<Self>::try_from(entity).ok()?)
            }

            #[inline(always)]
            fn resolve_view_direct(
                &mut self,
                entity: EntityAny,
            ) -> Option<(<Self as Archetype>::View<'_>, EntityDirect<Self>)> {
                self.data.get_view_direct(Entity::<Self>::try_from(entity).ok()?)
            }

            #[inline(always)]
            fn resolve_view_mut(
                &mut self,
                entity: EntityAny,
            ) -> Option<<Self as Archetype>::ViewMut<'_>> {
                self.data.get_view_mut(Entity::<Self>::try_from(entity).ok()?)
            }

            #[inline(always)]
            fn resolve_view_mut_direct(
                &mut self,
                entity: EntityAny,
            ) -> Option<(<Self as Archetype>::ViewMut<'_>, EntityDirect<Self>)> {
                self.data.get_view_mut_direct(Entity::<Self>::try_from(entity).ok()?)
            }

            #[inline(always)]
            fn resolve_borrow(
                &self,
                entity: EntityAny,
            ) -> Option<<Self as Archetype>::Borrow<'_>> {
                self.data.begin_borrow(Entity::<Self>::try_from(entity).ok()?).map(
                    #ArchetypeBorrow
                )
            }

            #[inline(always)]
            fn resolve_borrow_direct(
                &self,
                entity: EntityAny,
            ) -> Option<(<Self as Archetype>::Borrow<'_>, EntityDirect<Self>)> {
                self.data.begin_borrow_direct(Entity::<Self>::try_from(entity).ok()?).map(
                    |(borrow, direct)| (#ArchetypeBorrow(borrow), direct)
                )
            }

            #[inline(always)]
            fn resolve_destroy(
                &mut self,
                entity: EntityAny,
            ) -> Option<<Self as Archetype>::Components> {
                self.data.destroy(Entity::<Self>::try_from(entity).ok()?)
            }
        }

        impl ArchetypeCanResolve<EntityDirectAny> for #Archetype {
            #[inline(always)]
            fn resolve_for(&self, entity: EntityDirectAny) -> Option<usize> {
                self.data.resolve(EntityDirect::<Self>::try_from(entity).ok()?)
            }

            #[inline(always)]
            fn resolve_direct(
                &self,
                entity: EntityDirectAny,
            ) -> Option<EntityDirectAny> {
                self.data.resolve_direct(EntityDirect::<Self>::try_from(entity).ok()?).map(Into::into)
            }

            #[inline(always)]
            fn resolve_view(
                &mut self,
                entity: EntityDirectAny,
            ) -> Option<<Self as Archetype>::View<'_>> {
                self.data.get_view(EntityDirect::<Self>::try_from(entity).ok()?)
            }

            #[inline(always)]
            fn resolve_view_direct(
                &mut self,
                entity: EntityDirectAny,
            ) -> Option<(<Self as Archetype>::View<'_>, EntityDirect<Self>)> {
                self.data.get_view_direct(EntityDirect::<Self>::try_from(entity).ok()?)
            }

            #[inline(always)]
            fn resolve_view_mut(
                &mut self,
                entity: EntityDirectAny,
            ) -> Option<<Self as Archetype>::ViewMut<'_>> {
                self.data.get_view_mut(EntityDirect::<Self>::try_from(entity).ok()?)
            }

            #[inline(always)]
            fn resolve_view_mut_direct(
                &mut self,
                entity: EntityDirectAny,
            ) -> Option<(<Self as Archetype>::ViewMut<'_>, EntityDirect<Self>)> {
                self.data.get_view_mut_direct(EntityDirect::<Self>::try_from(entity).ok()?)
            }

            #[inline(always)]
            fn resolve_borrow(
                &self,
                entity: EntityDirectAny,
            ) -> Option<<Self as Archetype>::Borrow<'_>> {
                self.data.begin_borrow(EntityDirect::<Self>::try_from(entity).ok()?).map(
                    #ArchetypeBorrow
                )
            }

            #[inline(always)]
            fn resolve_borrow_direct(
                &self,
                entity: EntityDirectAny,
            ) -> Option<(<Self as Archetype>::Borrow<'_>, EntityDirect<Self>)> {
                self.data.begin_borrow_direct(EntityDirect::<Self>::try_from(entity).ok()?).map(
                    |(borrow, direct)| (#ArchetypeBorrow(borrow), direct)
                )
            }

            #[inline(always)]
            fn resolve_destroy(&mut self, entity: EntityDirectAny) -> Option<<Self as Archetype>::Components> {
                self.data.destroy(EntityDirect::<Self>::try_from(entity).ok()?)
            }
        }
    )
}

#[allow(non_snake_case)]
fn with_capacity_param(archetype_data: &DataArchetype) -> TokenStream {
    let archetype = format_ident!("{}", util::to_snake(&archetype_data.name));
    quote!(#archetype: usize)
}

#[allow(non_snake_case)]
fn with_capacity_new(archetype_data: &DataArchetype) -> TokenStream {
    let archetype = format_ident!("{}", util::to_snake(&archetype_data.name));
    quote!(with_capacity(capacity.#archetype))
}

#[allow(non_snake_case)]
fn section_event_iter(_world_data: &DataWorld) -> TokenStream {
    if cfg!(feature = "events") {
        let Archetype = _world_data
            .archetypes
            .iter()
            .map(|archetype| format_ident!("{}", archetype.name))
            .collect::<Vec<_>>();
        let iter = _world_data
            .archetypes
            .iter()
            .map(|archetype| format_ident!("iter_{}", util::to_snake(&archetype.name)))
            .collect::<Vec<_>>();
        let index = (0.._world_data.archetypes.len()).collect::<Vec<_>>();

        // Increment self.which in every step except the very last
        let mut next = Vec::new();
        if _world_data.archetypes.is_empty() == false {
            for _ in 0.._world_data.archetypes.len() - 1 {
                next.push(quote!(self.which += 1));
            }
            next.push(quote!({}));
        }

        quote!(
            use std::slice::Iter;

            pub struct EcsEventIterator<'a> {
                // We don't actually use archetype IDs since they aren't guaranteed to be
                // sequential -- this is just to make sure that we have a correct max size.
                which: ArchetypeId,

                #(#iter: Iter<'a, Entity<#Archetype>>,)*
            }

            impl<'a> Iterator for EcsEventIterator<'a> {
                type Item = &'a EntityAny;

                #[inline]
                fn next(&mut self) -> Option<Self::Item> {
                    #(
                        if self.which == #index as ArchetypeId {
                            match self.#iter.next() {
                                Some(next) => return Some(next.into()),
                                None => #next,
                            }
                        }
                    )*

                    None
                }

                #[inline]
                fn size_hint(&self) -> (usize, Option<usize>) {
                    let mut min = 0;
                    let mut max = 0;

                    #(
                        if self.which <= #index as ArchetypeId  {
                            let (iter_min, iter_max) = self.#iter.size_hint();
                            min += iter_min;
                            max += match iter_max {
                                Some(iter_max) => iter_max,

                                // This should just compile out due to how slice::Iter works and
                                // inlines. This is here as a backup in case the internals change.
                                None => return (0, None),
                            }
                        }
                    )*

                    (min, Some(max))
                }
            }
        )
    } else {
        quote!()
    }
}

#[allow(non_snake_case)]
fn section_events_world(_world_data: &DataWorld) -> TokenStream {
    if cfg!(feature = "events") {
        let iter = _world_data
            .archetypes
            .iter()
            .map(|archetype| format_ident!("iter_{}", util::to_snake(&archetype.name)))
            .collect::<Vec<_>>();
        let archetype = _world_data
            .archetypes
            .iter()
            .map(|archetype| format_ident!("{}", util::to_snake(&archetype.name)))
            .collect::<Vec<_>>();

        quote!(
            #[inline(always)]
            fn iter_created(&self) -> impl Iterator<Item = &EntityAny> {
                EcsEventIterator {
                    which: 0,
                    #(#iter: self.#archetype.data.created().iter(),)*
                }
            }

            #[inline(always)]
            fn iter_destroyed(&self) -> impl Iterator<Item = &EntityAny> {
                EcsEventIterator {
                    which: 0,
                    #(#iter: self.#archetype.data.destroyed().iter(),)*
                }
            }

            #[inline(always)]
            fn clear_events(&mut self) {
                #(self.#archetype.clear_events();)*
            }
        )
    } else {
        quote!()
    }
}

#[allow(non_snake_case)]
fn section_events_archetype(_archetype_data: &DataArchetype) -> TokenStream {
    if cfg!(feature = "events") {
        let Archetype = format_ident!("{}", &_archetype_data.name);

        quote!(
            #[inline(always)]
            fn iter_created(&self) -> impl Iterator<Item = &Entity<#Archetype>> {
                self.data.created().iter()
            }

            #[inline(always)]
            fn iter_destroyed(&self) -> impl Iterator<Item = &Entity<#Archetype>> {
                self.data.destroyed().iter()
            }

            #[inline(always)]
            fn clear_events(&mut self) {
                self.data.clear_events()
            }
        )
    } else {
        quote!()
    }
}
