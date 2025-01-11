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
    let ArchetypeSelectEntityDirect = format_ident!("ArchetypeSelectEntityDirect");

    let ArchetypeSelectInternalWorld =
        format_ident!("__ArchetypeSelectInternal{}", world_data.name);

    let Archetype = world_data
        .archetypes
        .iter()
        .map(|archetype| format_ident!("{}", archetype.name))
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

    // Documentation helpers
    let world_doc_archetypes = world_data
        .archetypes
        .iter()
        .map(|archetype| format!("- `{}`: [`{}`],", to_snake(&archetype.name), &archetype.name))
        .collect::<Vec<_>>();

    // Macros
    let __expand_ecs_find_unique = format_ident!("__expand_ecs_find_{}", unique_hash);
    let __expand_ecs_find_borrow_unique = format_ident!("__expand_ecs_find_borrow_{}", unique_hash);
    let __expand_ecs_iter_unique = format_ident!("__expand_ecs_iter_{}", unique_hash);
    let __expand_ecs_iter_borrow_unique = format_ident!("__expand_ecs_iter_borrow_{}", unique_hash);
    let __expand_ecs_iter_destroy_unique =
        format_ident!("__expand_ecs_iter_destroy_{}", unique_hash);

    quote!(
        #( pub use #ecs_world_sealed::#Archetype; )*

        pub use #ecs_world_sealed::{
            #World,
            #WorldCapacity,
            #ArchetypeSelectId,
            #ArchetypeSelectEntity,
            #ArchetypeSelectEntityDirect
        };

        #[doc(hidden)]
        pub use #ecs_world_sealed::{#ArchetypeSelectInternalWorld};

        mod #ecs_world_sealed {
            use super::*;

            use ::std::cell::{Ref, RefMut};

            use ::gecs::__internal::*;

            #(#section_archetype)*

            #[doc = r"The generated ECS world. See [`World`](gecs::traits::World) for more information."]
            #[doc = r""]
            #[doc = r"Contained archetypes (may change based on `#[cfg]` state): "]
            #(#[doc = #world_doc_archetypes])*
            #[derive(Default)]
            pub struct #World {
                #(
                    pub #archetype: #Archetype,
                )*
            }

            #[derive(Default)]
            pub struct #WorldCapacity {
                #(
                    pub #with_capacity_param,
                )*
            }

            impl World for #World {
                const NUM_ARCHETYPES: usize = #num_archetypes;

                type Capacities = #WorldCapacity;

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

            #(
                impl WorldHas<#Archetype> for #World {
                    #[inline(always)]
                    fn resolve_create(
                        &mut self,
                        data: <#Archetype as Archetype>::Components,
                    ) -> Entity<#Archetype>
                    {
                        self.#archetype.create(data)
                    }

                    #[inline(always)]
                    fn resolve_create_within_capacity(
                        &mut self,
                        data: <#Archetype as Archetype>::Components,
                    ) -> Result<Entity<#Archetype>, <#Archetype as Archetype>::Components>
                    {
                        self.#archetype.create_within_capacity(data)
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
            pub enum #ArchetypeSelectEntityDirect {
                #( #Archetype(EntityDirect<#Archetype>), )*
            }

            // Combined dispatch table for resolving both entity key types.
            #[doc(hidden)]
            pub enum #ArchetypeSelectInternalWorld {
                #( #Archetype(Entity<#Archetype>), )*
                #( #ArchetypeDirect(EntityDirect<#Archetype>), )*
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

                impl From<EntityDirect<#Archetype>> for #ArchetypeSelectId {
                    #[inline(always)]
                    fn from(entity: EntityDirect<#Archetype>) -> Self {
                        #ArchetypeSelectId::#Archetype
                    }
                }

                impl From<EntityDirect<#Archetype>> for #ArchetypeSelectEntityDirect {
                    #[inline(always)]
                    fn from(entity: EntityDirect<#Archetype>) -> Self {
                        #ArchetypeSelectEntityDirect::#Archetype(entity)
                    }
                }

                impl From<&EntityDirect<#Archetype>> for #ArchetypeSelectEntityDirect {
                    #[inline(always)]
                    fn from(entity: &EntityDirect<#Archetype>) -> Self {
                        #ArchetypeSelectEntityDirect::#Archetype(*entity)
                    }
                }

                impl From<Entity<#Archetype>> for #ArchetypeSelectInternalWorld {
                    #[inline(always)]
                    fn from(entity: Entity<#Archetype>) -> Self {
                        #ArchetypeSelectInternalWorld::#Archetype(entity)
                    }
                }

                impl From<EntityDirect<#Archetype>> for #ArchetypeSelectInternalWorld {
                    #[inline(always)]
                    fn from(entity: EntityDirect<#Archetype>) -> Self {
                        #ArchetypeSelectInternalWorld::#ArchetypeDirect(entity)
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

                impl WorldCanResolve<EntityDirect<#Archetype>> for #World {
                    #[inline(always)]
                    fn resolve_destroy(
                        &mut self,
                        entity: EntityDirect<#Archetype>
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
                ) -> Option<()> {
                    match entity.try_into() {
                        #(
                            Ok(#ArchetypeSelectEntity::#Archetype(entity)) =>
                                self.#archetype.destroy(entity).map(|_| ()),
                        )*
                        Err(_) => panic!("invalid entity type"),
                    }
                }
            }

            impl WorldCanResolve<EntityDirectAny> for #World {
                #[inline(always)]
                fn resolve_destroy(
                    &mut self,
                    entity: EntityDirectAny,
                ) -> Option<()> {
                    match entity.try_into() {
                        #(
                            Ok(#ArchetypeSelectEntityDirect::#Archetype(entity)) =>
                                self.#archetype.destroy(entity).map(|_| ()),
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

            impl TryFrom<EntityDirectAny> for #ArchetypeSelectEntityDirect {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(entity: EntityDirectAny) -> Result<Self, EcsError> {
                    match entity.archetype_id() {
                        #(
                            #Archetype::ARCHETYPE_ID => {
                                // We can use from_any_unchecked because we just checked the archetype
                                Ok(#ArchetypeSelectEntityDirect::#Archetype(
                                    EntityDirect::<#Archetype>::from_any_unchecked(entity))
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

            impl TryFrom<EntityDirectAny> for #ArchetypeSelectInternalWorld {
                type Error = EcsError;

                #[inline(always)]
                fn try_from(entity: EntityDirectAny) -> Result<Self, EcsError> {
                    match entity.archetype_id() {
                        #(
                            #Archetype::ARCHETYPE_ID => {
                                // We can use from_any_unchecked because we just checked the archetype
                                Ok(#ArchetypeSelectInternalWorld::#ArchetypeDirect(
                                    EntityDirect::<#Archetype>::from_any_unchecked(entity))
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
        macro_rules! #__expand_ecs_find_unique {
            ($($args:tt)*) => {
                ::gecs::__internal::__expand_ecs_find!(#WORLD_DATA, $($args)*)
            }
        }

        #[macro_export]
        #[doc(hidden)]
        macro_rules! #__expand_ecs_find_borrow_unique {
            ($($args:tt)*) => {
                ::gecs::__internal::__expand_ecs_find_borrow!(#WORLD_DATA, $($args)*)
            }
        }

        #[macro_export]
        #[doc(hidden)]
        macro_rules! #__expand_ecs_iter_unique {
            ($($args:tt)*) => {
                ::gecs::__internal::__expand_ecs_iter!(#WORLD_DATA, $($args)*)
            }
        }

        #[macro_export]
        #[doc(hidden)]
        macro_rules! #__expand_ecs_iter_borrow_unique {
            ($($args:tt)*) => {
                ::gecs::__internal::__expand_ecs_iter_borrow!(#WORLD_DATA, $($args)*)
            }
        }

        #[macro_export]
        #[doc(hidden)]
        macro_rules! #__expand_ecs_iter_destroy_unique {
            ($($args:tt)*) => {
                ::gecs::__internal::__expand_ecs_iter_destroy!(#WORLD_DATA, $($args)*)
            }
        }

        #[doc(inline)]
        pub use #__expand_ecs_find_unique as ecs_find;
        #[doc(inline)]
        pub use #__expand_ecs_find_borrow_unique as ecs_find_borrow;
        #[doc(inline)]
        pub use #__expand_ecs_iter_unique as ecs_iter;
        #[doc(inline)]
        pub use #__expand_ecs_iter_borrow_unique as ecs_iter_borrow;
        #[doc(inline)]
        pub use #__expand_ecs_iter_destroy_unique as ecs_iter_destroy;
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
    let ArchetypeSelectEntity = format_ident!("ArchetypeSelectEntity");
    let ArchetypeSelectEntityDirect = format_ident!("ArchetypeSelectEntityDirect");

    let ViewN = format_ident!("View{}", count_str);
    let SlicesN = format_ident!("Slices{}", count_str);
    let ContentArgs = quote!(#Archetype, #(#Component),*);

    let StorageN = format_ident!("Storage{}", count_str);
    let BorrowN = format_ident!("Borrow{}", count_str);
    let StorageArgs = quote!(#Archetype, #(#Component,)*);

    let IterArgs = quote!((&'a Entity<#Archetype>, #(&'a #Component,)*));
    let IterMutArgs = quote!((&'a Entity<#Archetype>, #(&'a mut #Component,)*));

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
        .map(|component| format_ident!("{}", to_snake(&component.name)))
        .collect::<Vec<_>>();

    // Documentation helpers
    let archetype_doc_components = archetype_data
        .components
        .iter()
        .map(|component| format!("- [`{}`]", &component.name))
        .collect::<Vec<_>>();

    quote!(
        #[doc = r"A generated ECS archetype. See [`Archetype`](gecs::traits::Archetype) for more information."]
        #[doc = r""]
        #[doc = r"Contained components (may change based on `#[cfg]` state): "]
        #(#[doc = #archetype_doc_components])*
        #[derive(Default)]
        #[repr(transparent)]
        pub struct #Archetype {
            pub data: #StorageN<#StorageArgs>,
        }

        impl Archetype for #Archetype {
            #[allow(unconditional_panic)]
            const ARCHETYPE_ID: u8 = #ARCHETYPE_ID;

            type Components = (#(#Component,)*);
            type View<'a> = #ArchetypeView<'a>;
            type Borrow<'a> = #ArchetypeBorrow<'a>;
            type Slices<'a> = #ArchetypeSlices<'a>;

            type IterArgs<'a> = #IterArgs;
            type IterMutArgs<'a> = #IterMutArgs;

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
            fn create(
                &mut self,
                data: (#(#Component,)*),
            ) -> Entity<#Archetype> {
                self.data.push(data)
            }

            #[inline(always)]
            fn create_within_capacity(
                &mut self,
                data: (#(#Component,)*),
            ) -> Result<Entity<#Archetype>, (#(#Component,)*)> {
                self.data.push_within_capacity(data)
            }

            #[inline(always)]
            fn iter(&mut self) -> impl Iterator<Item = Self::IterArgs<'_>> {
                self.data.iter()
            }

            #[inline(always)]
            fn iter_mut(&mut self) -> impl Iterator<Item = Self::IterMutArgs<'_>> {
                self.data.iter_mut()
            }

            // #[inline(always)]
            // fn begin_borrow<'a, K: EntityKey>(
            //     &'a self,
            //     entity: K,
            // ) -> Option<#ArchetypeBorrow<'a>>
            // where
            //     #StorageN<#StorageArgs>: StorageCanResolve<K>,
            // {
            //     self.data.begin_borrow(entity).map(#ArchetypeBorrow)
            // }
            //
            // #[inline(always)]
            // fn get_view_mut<'a, K: EntityKey>(
            //     &'a mut self,
            //     entity_key: K,
            // ) -> Option<#ArchetypeView<'a>>
            // where
            //     #StorageN<#StorageArgs>: StorageCanResolve<K>,
            // {
            //     self.data.get_view_mut(entity_key)
            // }

            #[inline(always)]
            fn get_all_slices_mut(&mut self) -> #ArchetypeSlices {
                self.data.get_all_slices_mut()
            }

            #[inline(always)]
            fn get_slice_entities(&self) -> &[Entity<#Archetype>] {
                self.data.get_slice_entities()
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
                fn resolve_borrow_component<'a>(borrow: &'a #ArchetypeBorrow<'a>) -> Ref<'a, #Component> {
                    borrow.0.#borrow_component()
                }

                #[inline(always)]
                fn resolve_borrow_component_mut<'a>(borrow: &'a #ArchetypeBorrow<'a>) -> RefMut<'a, #Component> {
                    borrow.0.#borrow_component_mut()
                }
            }
        )*

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
            pub fn borrow_component<C>(&self) -> Ref<C>
            where
                #Archetype: for<'c> ArchetypeHas<C, Borrow<'c> = #ArchetypeBorrow<'c>>
            {
                #Archetype::resolve_borrow_component(self)
            }

            #[inline(always)]
            pub fn borrow_component_mut<C>(&self) -> RefMut<C>
            where
                #Archetype: for<'c> ArchetypeHas<C, Borrow<'c> = #ArchetypeBorrow<'c>>
            {
                #Archetype::resolve_borrow_component_mut(self)
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

        impl ArchetypeCanResolve<Entity<#Archetype>> for #Archetype {
            #[inline(always)]
            fn resolve_for(&self, key: Entity<#Archetype>) -> Option<usize> {
                self.data.resolve(key)
            }

            #[inline(always)]
            fn resolve_view(&mut self, key: Entity<#Archetype>) -> Option<<Self as Archetype>::View<'_>> {
                self.data.get_view_mut(key)
            }

            #[inline(always)]
            fn resolve_borrow(&self, key: Entity<#Archetype>) -> Option<<Self as Archetype>::Borrow<'_>> {
                self.data.begin_borrow(key).map(#ArchetypeBorrow)
            }

            #[inline(always)]
            fn resolve_destroy(&mut self, key: Entity<#Archetype>) -> Option<(#(#Component,)*)> {
                self.data.destroy(key)
            }
        }

        impl ArchetypeCanResolve<EntityDirect<#Archetype>> for #Archetype {
            #[inline(always)]
            fn resolve_for(&self, key: EntityDirect<#Archetype>) -> Option<usize> {
                self.data.resolve(key)
            }

            #[inline(always)]
            fn resolve_view(&mut self, key: EntityDirect<#Archetype>) -> Option<<Self as Archetype>::View<'_>> {
                self.data.get_view_mut(key)
            }

            #[inline(always)]
            fn resolve_borrow(&self, key: EntityDirect<#Archetype>) -> Option<<Self as Archetype>::Borrow<'_>> {
                self.data.begin_borrow(key).map(#ArchetypeBorrow)
            }

            #[inline(always)]
            fn resolve_destroy(&mut self, key: EntityDirect<#Archetype>) -> Option<(#(#Component,)*)> {
                self.data.destroy(key)
            }
        }

        impl ArchetypeCanResolve<EntityAny> for #Archetype {
            #[inline(always)]
            fn resolve_for(&self, key: EntityAny) -> Option<usize> {
                match key.try_into() {
                    Ok(#ArchetypeSelectEntity::#Archetype(entity)) => self.data.resolve(entity),
                    Ok(_) => None, // Wrong archetype ID in the entity
                    Err(_) => panic!("invalid entity type"),
                }
            }

            #[inline(always)]
            fn resolve_view(&mut self, key: EntityAny) -> Option<<Self as Archetype>::View<'_>> {
                match key.try_into() {
                    Ok(#ArchetypeSelectEntity::#Archetype(entity)) => self.data.get_view_mut(entity),
                    Ok(_) => None, // Wrong archetype ID in the entity
                    Err(_) => panic!("invalid entity type"),
                }
            }

            #[inline(always)]
            fn resolve_borrow(&self, key: EntityAny) -> Option<<Self as Archetype>::Borrow<'_>> {
                match key.try_into() {
                    Ok(#ArchetypeSelectEntity::#Archetype(entity)) => self.data.begin_borrow(entity).map(#ArchetypeBorrow),
                    Ok(_) => None, // Wrong archetype ID in the entity
                    Err(_) => panic!("invalid entity type"),
                }
            }

            #[inline(always)]
            fn resolve_destroy(&mut self, key: EntityAny) -> Option<(#(#Component,)*)> {
                match key.try_into() {
                    Ok(#ArchetypeSelectEntity::#Archetype(entity)) => self.data.destroy(entity),
                    Ok(_) => None, // Wrong archetype ID in the entity
                    Err(_) => panic!("invalid entity type"),
                }
            }
        }

        impl ArchetypeCanResolve<EntityDirectAny> for #Archetype {
            #[inline(always)]
            fn resolve_for(&self, key: EntityDirectAny) -> Option<usize> {
                match key.try_into() {
                    Ok(#ArchetypeSelectEntityDirect::#Archetype(entity)) => self.data.resolve(entity),
                    Ok(_) => None, // Wrong archetype ID in the entity
                    Err(_) => panic!("invalid entity type"),
                }
            }

            #[inline(always)]
            fn resolve_view(&mut self, key: EntityDirectAny) -> Option<<Self as Archetype>::View<'_>> {
                match key.try_into() {
                    Ok(#ArchetypeSelectEntityDirect::#Archetype(entity)) => self.data.get_view_mut(entity),
                    Ok(_) => None, // Wrong archetype ID in the entity
                    Err(_) => panic!("invalid entity type"),
                }
            }

            #[inline(always)]
            fn resolve_borrow(&self, key: EntityDirectAny) -> Option<<Self as Archetype>::Borrow<'_>> {
                match key.try_into() {
                    Ok(#ArchetypeSelectEntityDirect::#Archetype(entity)) => self.data.begin_borrow(entity).map(#ArchetypeBorrow),
                    Ok(_) => None, // Wrong archetype ID in the entity
                    Err(_) => panic!("invalid entity type"),
                }
            }

            #[inline(always)]
            fn resolve_destroy(&mut self, key: EntityDirectAny) -> Option<(#(#Component,)*)> {
                match key.try_into() {
                    Ok(#ArchetypeSelectEntityDirect::#Archetype(entity)) => self.data.destroy(entity),
                    Ok(_) => None, // Wrong archetype ID in the entity
                    Err(_) => panic!("invalid entity type"),
                }
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
