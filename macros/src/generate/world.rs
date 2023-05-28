use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::data::{DataArchetype, DataCapacity, DataWorld};

#[allow(non_snake_case)] // Allow for type-like names to make quote!() clearer
pub fn generate_world(world_data: &DataWorld) -> TokenStream {
    // Module
    let ecs_world_sealed = format_ident!("ecs_{}_sealed", to_snake(&world_data.name));

    // Constants and literals
    let WORLD_DATA = world_data.to_base64();

    // Types and traits
    let World = format_ident!("{}", world_data.name);
    let EntityWorld = format_ident!("Entity{}", world_data.name);
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

    quote!(
        pub use #ecs_world_sealed::#World;
        pub use #ecs_world_sealed::#EntityWorld;
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

            impl HasArchetypes for #World {}

            #(
                impl HasArchetype<#Archetype> for #World {
                    #[inline(always)]
                    fn resolve_archetype(&self) -> &#Archetype {
                        &self.#archetype
                    }

                    #[inline(always)]
                    fn resolve_mut_archetype(&mut self) -> &mut #Archetype {
                        &mut self.#archetype
                    }
                }
            )*

            #[derive(Clone, Copy)]
            pub enum #EntityWorld {
                #( #Archetype(Entity<#Archetype>), )*
            }

            #(
                impl From<Entity<#Archetype>> for #EntityWorld {
                    #[inline(always)]
                    fn from(entity: Entity<#Archetype>) -> Self {
                        #EntityWorld::#Archetype(entity)
                    }
                }
            )*

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
        macro_rules! ecs_iter_mut {
            ($($args:tt)*) => {
                ::gecs::__internal::__ecs_iter_mut!(#WORLD_DATA, $($args)*);
            }
        }

        #[macro_export]
        macro_rules! ecs_find_mut {
            ($($args:tt)*) => {
                ::gecs::__internal::__ecs_find_mut!(#WORLD_DATA, $($args)*);
            }
        }

        #[macro_export]
        macro_rules! ecs_iter_borrow {
            ($($args:tt)*) => {
                ::gecs::__internal::__ecs_iter_borrow!(#WORLD_DATA, $($args)*);
            }
        }

        #[macro_export]
        macro_rules! ecs_find_borrow {
            ($($args:tt)*) => {
                ::gecs::__internal::__ecs_find_borrow!(#WORLD_DATA, $($args)*);
            }
        }
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
    let get_mut_slice = (0..count)
        .into_iter()
        .map(|idx| format_ident!("get_mut_slice_{}", idx.to_string()));
    let borrow_slice = (0..count)
        .into_iter()
        .map(|idx| format_ident!("borrow_slice_{}", idx.to_string()));
    let borrow_mut_slice = (0..count)
        .into_iter()
        .map(|idx| format_ident!("borrow_mut_slice_{}", idx.to_string()));

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
            #[inline(always)]
            pub fn new() -> Self {
                Self { data: #StorageType::new() }
            }

            #[inline(always)]
            pub fn len(&self) -> usize {
                self.data.len()
            }

            #[inline(always)]
            pub const fn is_empty(&self) -> bool {
                self.data.is_empty()
            }

            #[inline(always)]
            pub const fn capacity(&self) -> usize {
                self.data.capacity()
            }

            #[inline(always)]
            pub fn push(&mut self, #( #component: #Component ),*) -> Option<Entity<#Archetype>> {
                self.data.push(#( #component ),*)
            }

            #[inline(always)]
            pub fn resolve(&mut self, entity: Entity<#Archetype>) -> Option<usize> {
                self.data.resolve(entity)
            }

            #[inline(always)]
            pub fn remove(&mut self, entity: Entity<#Archetype>) -> bool {
                self.data.remove(entity)
            }

            #[inline(always)]
            pub fn get_mut_slices(&mut self) -> #ArchetypeSlices {
                self.data.get_mut_slices()
            }
        }

        impl HasComponents for #Archetype { }

        #(
            impl HasComponent<#Component> for #Archetype {
                #[inline(always)]
                fn resolve_get_slice(&mut self) -> &[#Component] {
                    self.data.#get_slice()
                }

                #[inline(always)]
                fn resolve_get_mut_slice(&mut self) -> &mut [#Component] {
                    self.data.#get_mut_slice()
                }

                #[inline(always)]
                fn resolve_borrow_slice(&self) -> Ref<[#Component]> {
                    self.data.#borrow_slice()
                }

                #[inline(always)]
                fn resolve_borrow_mut_slice(&self) -> RefMut<[#Component]> {
                    self.data.#borrow_mut_slice()
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
