use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::token::{Colon, Comma, Gt, Lt, Mut};
use syn::{Expr, Ident, LitStr, Token};

mod kw {
    syn::custom_keyword!(archetype);
    syn::custom_keyword!(cfg);
}

#[derive(Debug)]
pub struct ParseQueryFind {
    pub world_data: String,
    pub world: Expr,
    pub entity: Expr,
    pub params: Vec<ParseQueryParam>,
    pub body: TokenStream,
}

#[derive(Debug)]
pub struct ParseQueryIter {
    pub world_data: String,
    pub world: Expr,
    pub params: Vec<ParseQueryParam>,
    pub body: TokenStream,
}

#[derive(Debug)]
pub struct ParseQueryParam {
    pub name: Ident,
    pub is_mut: bool,
    pub ty: ParseParamType,
}

#[derive(Clone, Debug)]
pub enum ParseParamType {
    Component(Ident),
    Entity(Ident),
    EntityAny,
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
        let body = input.parse::<TokenStream>()?;

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
        let body = input.parse::<TokenStream>()?;

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
        let ty = input.parse::<ParseParamType>()?;

        // Enforce mutability rules
        match ty {
            ParseParamType::Entity(_) | ParseParamType::EntityAny if is_mut => Err(
                syn::Error::new(check_span, "mut entity access is forbidden"),
            ),
            _ => Ok(Self { name, is_mut, ty }),
        }
    }
}

impl Parse for ParseParamType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty = input.parse::<Ident>()?;
        match ty.to_string().as_str() {
            "Entity" => {
                input.parse::<Lt>()?;
                let archetype = input.parse::<Ident>()?;
                input.parse::<Gt>()?;
                Ok(ParseParamType::Entity(archetype))
            }
            "EntityAny" => Ok(ParseParamType::EntityAny),
            _ => Ok(ParseParamType::Component(ty)),
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
