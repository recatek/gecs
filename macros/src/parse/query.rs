use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::token::{And, Colon, Comma, Gt, Lt, Mut};
use syn::{Ident, LitStr, Token};

mod kw {
    syn::custom_keyword!(archetype);
    syn::custom_keyword!(cfg);
}

#[derive(Debug)]
pub struct ParseQueryIter {
    pub world_data: String,
    pub world: Ident,
    pub params: Vec<ParseQueryParam>,
    pub body: TokenStream,
}

#[derive(Debug)]
pub struct ParseQueryParam {
    pub name: Ident,
    pub ty: ParseQueryParamType,
}

#[derive(Clone, Debug)]
pub enum ParseQueryParamType {
    Component(Ident),
    MutComponent(Ident),
    Entity(Ident),
    EntityAny,
}

impl Parse for ParseQueryIter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse out the hidden serialized world data
        let world_data = input.parse::<LitStr>()?;
        input.parse::<Comma>()?;

        // Parse out the meta-arguments for the query
        let world = input.parse::<Ident>()?;
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

        // Must be a reference in all cases
        input.parse::<And>()?;
        // The mut is optional
        let is_mut = input.parse::<Option<Mut>>()?.is_some();

        let ty = input.parse::<Ident>()?;
        let ty = match ty.to_string().as_str() {
            "Entity" => {
                if is_mut {
                    Err(syn::Error::new_spanned(
                        ty,
                        "mutable entity access is not allowed",
                    ))
                } else {
                    input.parse::<Lt>()?;
                    let archetype = input.parse::<Ident>()?;
                    input.parse::<Gt>()?;
                    Ok(ParseQueryParamType::Entity(archetype))
                }
            }
            "EntityAny" => {
                if is_mut {
                    Err(syn::Error::new_spanned(
                        ty,
                        "mutable entity access is not allowed",
                    ))
                } else {
                    Ok(ParseQueryParamType::EntityAny)
                }
            }
            _ if is_mut => Ok(ParseQueryParamType::MutComponent(ty)),
            _ => Ok(ParseQueryParamType::Component(ty)),
        }?;

        Ok(Self { name, ty })
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
