use std::collections::HashSet;

use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::token::{Colon, Comma, Gt, Lt, Mut};
use syn::{Expr, Ident, LitStr, Token, Type};

use super::{
    parse_attributes, HasCfgPredicates, ParseAttributeCfg, ParseAttributeData, ParseComponentName,
};

mod kw {
    syn::custom_keyword!(archetype);
    syn::custom_keyword!(cfg);

    syn::custom_keyword!(Entity);
    syn::custom_keyword!(EntityAny);
    syn::custom_keyword!(EntityDirect);
    syn::custom_keyword!(EntityDirectAny);

    syn::custom_keyword!(OneOf);
    syn::custom_keyword!(Option);
    syn::custom_keyword!(With);
    syn::custom_keyword!(Without);
}

#[derive(Debug)]
pub struct ParseQueryFind {
    pub world_data: String,
    pub world: Expr,
    pub entity: Expr,
    pub params: Vec<ParseQueryParam>,
    pub ret: Option<Type>,
    pub body: Expr,
}

#[derive(Debug)]
pub struct ParseQueryIter {
    pub world_data: String,
    pub world: Expr,
    pub params: Vec<ParseQueryParam>,
    pub body: Expr,
}

#[derive(Debug)]
pub struct ParseQueryIterDestroy {
    pub world_data: String,
    pub world: Expr,
    pub params: Vec<ParseQueryParam>,
    pub body: Expr,
}

#[derive(Clone, Debug)]
pub struct ParseQueryParam {
    pub cfgs: Vec<ParseAttributeCfg>,
    pub name: Ident,
    pub is_mut: bool,
    pub param_type: ParseQueryParamType,

    // Set during generation
    pub is_cfg_enabled: bool,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum ParseQueryParamType {
    Component(ParseComponentName), // CompFoo or CompFoo<T>

    // Entity Types
    Entity(Ident),       // Entity<A>
    EntityWild,          // Entity<_>
    EntityAny,           // EntityAny
    EntityDirect(Ident), // EntityDirect<A>
    EntityDirectWild,    // EntityDirect<_>
    EntityDirectAny,     // EntityDirectAny

    // Special Types
    OneOf(Box<[ParseComponentName]>), // OneOf<CompFoo, CompBar<T>>
    Option(Ident),                    // Option<CompFoo> -- TODO: RESERVED
    With(Ident),                      // With<CompFoo> -- TODO: RESERVED
    Without(Ident),                   // Without<CompFoo> -- TODO: RESERVED
}

impl Parse for ParseQueryFind {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse out the hidden serialized world data
        let world_data = input.parse::<LitStr>()?;
        input.parse::<Comma>()?;

        // Parse out the meta-arguments for the query
        let world = input.parse()?;
        input.parse::<Comma>()?;
        let entity = input.parse()?;
        input.parse::<Comma>()?;

        // Parse out the closure arguments
        input.parse::<Token![|]>()?;
        let params = parse_params(&input)?;
        input.parse::<Token![|]>()?;

        // Parse a return type, if there is one
        let ret = match input.parse::<Option<Token![->]>>()? {
            Some(_) => Some(input.parse::<Type>()?),
            None => None,
        };

        // Parse the rest of the body, including the braces (if any)
        let body = input.parse::<Expr>()?;

        Ok(Self {
            world_data: world_data.value(),
            world,
            entity,
            params,
            ret,
            body,
        })
    }
}

impl Parse for ParseQueryIter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse out the hidden serialized world data
        let world_data = input.parse::<LitStr>()?;
        input.parse::<Comma>()?;

        // Parse out the meta-arguments for the query
        let world = input.parse()?;
        input.parse::<Comma>()?;

        // Parse out the closure arguments
        input.parse::<Token![|]>()?;
        let params = parse_params(&input)?;
        input.parse::<Token![|]>()?;

        // Parse the rest of the body, including the braces (if any)
        let body = input.parse::<Expr>()?;

        Ok(Self {
            world_data: world_data.value(),
            world,
            params,
            body,
        })
    }
}

impl Parse for ParseQueryIterDestroy {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse out the hidden serialized world data
        let world_data = input.parse::<LitStr>()?;
        input.parse::<Comma>()?;

        // Parse out the meta-arguments for the query
        let world = input.parse()?;
        input.parse::<Comma>()?;

        // Parse out the closure arguments
        input.parse::<Token![|]>()?;
        let params = parse_params(&input)?;
        input.parse::<Token![|]>()?;

        // Parse the rest of the body, including the braces (if any)
        let body = input.parse::<Expr>()?;

        Ok(Self {
            world_data: world_data.value(),
            world,
            params,
            body,
        })
    }
}

impl Parse for ParseQueryParam {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attributes = Vec::new();

        // Pull out the cfg attributes from those decorating the param
        for attribute in parse_attributes(input)?.drain(..) {
            if let ParseAttributeData::Cfg(cfg) = attribute.data {
                attributes.push(cfg);
            } else {
                return Err(syn::Error::new(
                    attribute.span,
                    "invalid attribute for this position",
                ));
            }
        }

        // Parse the name and following : token
        let name = parse_param_name(input)?;
        input.parse::<Colon>()?;

        input.parse::<Token![&]>()?;
        let is_mut = input.parse::<Option<Mut>>()?.is_some();
        let check_span = input.span();
        let ty = input.parse::<ParseQueryParamType>()?;

        // Enforce mutability rules
        match ty {
            ParseQueryParamType::Entity(_)
            | ParseQueryParamType::EntityAny
            | ParseQueryParamType::EntityWild
            | ParseQueryParamType::EntityDirect(_)
            | ParseQueryParamType::EntityDirectAny
            | ParseQueryParamType::EntityDirectWild
                if is_mut =>
            {
                Err(syn::Error::new(
                    check_span,
                    "mut entity access is forbidden",
                ))
            }
            _ => Ok(Self {
                cfgs: attributes,
                name,
                is_mut,
                param_type: ty,
                is_cfg_enabled: true, // Default to true
            }),
        }
    }
}

impl Parse for ParseQueryParamType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::Entity) {
            // Entity<...>
            input.parse::<kw::Entity>()?;
            input.parse::<Lt>()?;

            // Entity<A> or Entity<_>
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![_]) {
                input.parse::<Token![_]>()?;
                input.parse::<Gt>()?;
                Ok(ParseQueryParamType::EntityWild)
            } else if lookahead.peek(Ident) {
                let archetype = input.parse::<Ident>()?;
                input.parse::<Gt>()?;
                Ok(ParseQueryParamType::Entity(archetype))
            } else {
                Err(lookahead.error())
            }
        } else if lookahead.peek(kw::EntityDirect) {
            // EntityDirect<...>
            input.parse::<kw::EntityDirect>()?;
            input.parse::<Lt>()?;

            // EntityDirect<A> or EntityDirect<_>
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![_]) {
                input.parse::<Token![_]>()?;
                input.parse::<Gt>()?;
                Ok(ParseQueryParamType::EntityDirectWild)
            } else if lookahead.peek(Ident) {
                let archetype = input.parse::<Ident>()?;
                input.parse::<Gt>()?;
                Ok(ParseQueryParamType::EntityDirect(archetype))
            } else {
                Err(lookahead.error())
            }
        } else if lookahead.peek(kw::EntityAny) {
            // EntityAny
            input.parse::<kw::EntityAny>()?;
            Ok(ParseQueryParamType::EntityAny)
        } else if lookahead.peek(kw::EntityDirectAny) {
            // EntityDirectAny
            input.parse::<kw::EntityDirectAny>()?;
            Ok(ParseQueryParamType::EntityDirectAny)
        } else if lookahead.peek(kw::OneOf) {
            // OneOf<A, B, C>
            input.parse::<kw::OneOf>()?;
            input.parse::<Token![<]>()?;
            let mut result = Vec::<ParseComponentName>::new();

            loop {
                let lookahead = input.lookahead1();

                if lookahead.peek(Ident) {
                    result.push(input.parse::<ParseComponentName>()?);
                    input.parse::<Option<Token![,]>>()?;
                } else if lookahead.peek(Token![>]) {
                    input.parse::<Token![>]>()?;
                    break Ok(ParseQueryParamType::OneOf(result.into_boxed_slice()));
                } else {
                    break Err(lookahead.error());
                }
            }
        } else if lookahead.peek(kw::Option) {
            Err(syn::Error::new(
                input.span(),
                "reserved special 'Option' not yet implemented",
            ))
        } else if lookahead.peek(kw::With) {
            Err(syn::Error::new(
                input.span(),
                "reserved special 'With' not yet implemented",
            ))
        } else if lookahead.peek(kw::Without) {
            Err(syn::Error::new(
                input.span(),
                "reserved special 'Without' not yet implemented",
            ))
        } else if lookahead.peek(Ident) {
            let name = input.parse::<ParseComponentName>()?;
            Ok(ParseQueryParamType::Component(name))
        } else {
            Err(lookahead.error())
        }
    }
}

impl HasCfgPredicates for ParseQueryFind {
    fn collect_all_cfg_predicates(&self) -> Vec<TokenStream> {
        get_cfg_predicates(&self.params)
    }
}

impl HasCfgPredicates for ParseQueryIter {
    fn collect_all_cfg_predicates(&self) -> Vec<TokenStream> {
        get_cfg_predicates(&self.params)
    }
}

impl HasCfgPredicates for ParseQueryIterDestroy {
    fn collect_all_cfg_predicates(&self) -> Vec<TokenStream> {
        get_cfg_predicates(&self.params)
    }
}

fn parse_params(input: &ParseStream) -> syn::Result<Vec<ParseQueryParam>> {
    let mut result = Vec::<ParseQueryParam>::new();
    loop {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![|]) {
            return Ok(result);
        } else if lookahead.peek(Ident) || lookahead.peek(Token![_]) || lookahead.peek(Token![#]) {
            result.push(input.parse::<ParseQueryParam>()?);
            input.parse::<Option<Token![,]>>()?;
        } else {
            return Err(lookahead.error());
        }
    }
}

fn parse_param_name(input: ParseStream) -> syn::Result<Ident> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![_]) {
        let token = input.parse::<Token![_]>()?;
        Ok(Ident::new("_", token.span))
    } else if lookahead.peek(Ident) {
        Ok(input.parse::<Ident>()?)
    } else {
        Err(lookahead.error())
    }
}

fn get_cfg_predicates(params: &Vec<ParseQueryParam>) -> Vec<TokenStream> {
    let mut filter = HashSet::new();
    let mut result = Vec::new();

    // Filter duplicates while keeping order for determinism
    for param in params.iter() {
        for cfg in param.cfgs.iter() {
            let predicate_tokens = cfg.predicate.clone();
            let predicate_string = predicate_tokens.to_string();

            if filter.insert(predicate_string) {
                result.push(predicate_tokens);
            }
        }
    }

    result
}
