use std::fmt::Display;

use proc_macro2::Span;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Token};

#[derive(Clone, Debug)]
pub struct ParseComponentName {
    pub name: Ident,
    pub generic: Option<Ident>,
}

impl ParseComponentName {
    pub fn span(&self) -> Span {
        self.name.span()
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
            let generic = input.parse::<Ident>()?;
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
        if let Some(generic) = &self.generic {
            write!(f, "{}<{}>", self.name, generic)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

impl ToTokens for ParseComponentName {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.name.to_tokens(tokens);

        if let Some(generic) = &self.generic {
            tokens.extend(quote::quote! { < #generic > });
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
