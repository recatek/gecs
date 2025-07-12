use std::fmt::Display;

use crate::util;
use proc_macro2::Span;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitInt, Token};

#[derive(Clone, Debug)]
pub struct ParseComponentName {
    pub name: Ident,
    pub generic: Option<ParseComponentGeneric>,
}

#[derive(Clone, Debug)]
pub enum ParseComponentGeneric {
    Placeholder(Token![_]),
    Ident(Ident),
    LitInt(LitInt),
}

impl ParseComponentName {
    pub fn span(&self) -> Span {
        self.name.span()
    }

    pub fn as_snake_name(&self) -> String {
        use ParseComponentGeneric as P;

        match &self.generic {
            None => format!(
                "{}", //.
                util::to_snake(&self.name.to_string())
            ),
            Some(P::Ident(ident)) => format!(
                "{}_{}",
                util::to_snake(&self.name.to_string()),
                util::to_snake(&ident.to_string())
            ),
            Some(P::LitInt(lit)) => format!(
                "{}_{}",
                util::to_snake(&self.name.to_string()),
                util::to_snake(&lit.token().to_string())
            ),
            Some(P::Placeholder(_)) => panic!("placeholder conversion to snake name"),
        }
    }
}

impl Parse for ParseComponentName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;

        // Don't allow special keyword names as component types
        if is_allowed_component_name(&name.to_string()) == false {
            return Err(syn::Error::new_spanned(name, "illegal component name"));
        }

        // Grab the generic argument, if there is one
        let generic = if input.peek(Token![<]) {
            input.parse::<Token![<]>()?;

            let lookahead = input.lookahead1();
            let generic = if lookahead.peek(Token![_]) {
                ParseComponentGeneric::Placeholder(input.parse()?)
            } else if lookahead.peek(Ident) {
                ParseComponentGeneric::Ident(input.parse()?)
            } else if lookahead.peek(LitInt) {
                ParseComponentGeneric::LitInt(input.parse()?)
            } else {
                Err(lookahead.error())?
            };

            input.parse::<Option<Token![,]>>()?;
            input.parse::<Token![>]>()?;

            Some(generic)
        } else {
            None
        };

        Ok(Self { name, generic })
    }
}

impl Display for ParseComponentName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ParseComponentGeneric as P;

        match &self.generic {
            None => write!(f, "{}", self.name,),
            Some(P::Ident(ident)) => write!(f, "{}<{}>", self.name, ident.to_string(),),
            Some(P::LitInt(lit)) => write!(f, "{}<{}>", self.name, lit.token().to_string(),),
            Some(P::Placeholder(_)) => write!(f, "{}<_>", self.name,),
        }
    }
}

impl ToTokens for ParseComponentName {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use ParseComponentGeneric as P;

        self.name.to_tokens(tokens);

        match &self.generic {
            None => {}
            Some(P::Ident(ident)) => tokens.extend(quote::quote! { <#ident> }),
            Some(P::LitInt(lit)) => tokens.extend(quote::quote! { <#lit> }),
            Some(P::Placeholder(_)) => panic!("cannot tokenize placeholder component name"),
        }
    }
}

fn is_allowed_component_name(name: &str) -> bool {
    match name {
        "Entity" => false,
        "EntityAny" => false,
        "OneOf" => false,
        "AnyOf" => false,  // Reserved
        "Option" => false, // Reserved
        _ => true,
    }
}
