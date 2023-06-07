use syn::parse::{Parse, ParseStream};
use syn::token::{Colon, Comma, Gt, Lt, Mut};
use syn::{Expr, Ident, LitStr, Token};

mod kw {
    syn::custom_keyword!(archetype);
    syn::custom_keyword!(cfg);

    syn::custom_keyword!(Entity);
    syn::custom_keyword!(EntityAny);
    syn::custom_keyword!(OneOf);
}

#[derive(Debug)]
pub struct ParseQueryFind {
    pub world_data: String,
    pub world: Expr,
    pub entity: Expr,
    pub params: Vec<ParseQueryParam>,
    pub body: Expr,
}

#[derive(Debug)]
pub struct ParseQueryIter {
    pub world_data: String,
    pub world: Expr,
    pub params: Vec<ParseQueryParam>,
    pub body: Expr,
}

#[derive(Clone, Debug)]
pub struct ParseQueryParam {
    pub name: Ident,
    pub is_mut: bool,
    pub param_type: ParseQueryParamType,
}

#[derive(Clone, Debug)]
pub enum ParseQueryParamType {
    Component(Ident),
    Entity(Ident),
    EntityAny,
    EntityWild, // Entity<_>
    OneOf(Box<[Ident]>),
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

        // Parse the rest of the body, including the braces (if any)
        let body = input.parse::<Expr>()?;

        Ok(Self {
            world_data: world_data.value(),
            world,
            entity,
            params,
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

impl Parse for ParseQueryParam {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse the name and following : token
        let name = parse_param_name(input)?;
        input.parse::<Colon>()?;

        input.parse::<Token![&]>()?;
        let is_mut = input.parse::<Option<Mut>>()?.is_some();
        let check_span = input.span();
        let ty = input.parse::<ParseQueryParamType>()?;

        // Enforce mutability rules
        match ty {
            ParseQueryParamType::Entity(_) | ParseQueryParamType::EntityAny if is_mut => Err(
                syn::Error::new(check_span, "mut entity access is forbidden"),
            ),
            _ => Ok(Self {
                name,
                is_mut,
                param_type: ty,
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
        } else if lookahead.peek(kw::EntityAny) {
            // EntityAny
            input.parse::<kw::EntityAny>()?;
            Ok(ParseQueryParamType::EntityAny)
        } else if lookahead.peek(kw::OneOf) {
            // OneOf<A, B, C>
            input.parse::<kw::OneOf>()?;
            input.parse::<Token![<]>()?;
            let mut result = Vec::<Ident>::new();
            loop {
                let lookahead = input.lookahead1();
                if lookahead.peek(Ident) {
                    result.push(input.parse::<Ident>()?);
                    input.parse::<Option<Token![,]>>()?;
                } else if lookahead.peek(Token![>]) {
                    input.parse::<Token![>]>()?;
                    break Ok(ParseQueryParamType::OneOf(result.into_boxed_slice()));
                } else {
                    break Err(lookahead.error());
                }
            }
        } else if lookahead.peek(Ident) {
            // Component
            Ok(ParseQueryParamType::Component(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

fn parse_params(input: &ParseStream) -> syn::Result<Vec<ParseQueryParam>> {
    let mut result = Vec::<ParseQueryParam>::new();
    loop {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![|]) {
            return Ok(result);
        } else if lookahead.peek(Ident) || lookahead.peek(Token![_]) {
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
