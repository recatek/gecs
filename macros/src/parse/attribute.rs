use proc_macro2::{Span, TokenStream};
use syn::parse::{Parse, ParseStream};
use syn::{bracketed, parenthesized, LitInt, Token};

use super::*;

mod kw {
    syn::custom_keyword!(cfg);

    syn::custom_keyword!(archetype_id);
    syn::custom_keyword!(component_id);
}

pub(super) fn parse_attributes(input: ParseStream) -> syn::Result<Vec<ParseAttribute>> {
    let mut attrs = Vec::new();
    while input.peek(Token![#]) {
        attrs.push(input.parse()?);
    }
    Ok(attrs)
}

#[derive(Debug)]
pub struct ParseAttribute {
    pub span: Span,
    pub data: ParseAttributeData,
}

#[derive(Debug)]
pub enum ParseAttributeData {
    Cfg(ParseAttributeCfg),
    ArchetypeId(ParseAttributeId),
    ComponentId(ParseAttributeId),
}

#[derive(Clone, Debug)]
pub struct ParseAttributeCfg {
    pub predicate: TokenStream,
}

#[derive(Debug)]
pub struct ParseAttributeId {
    pub value: u8,
}

impl Parse for ParseAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![#]>()?;
        let content;
        bracketed!(content in input);

        let span = content.span();
        let lookahead = content.lookahead1();
        let data = if lookahead.peek(kw::cfg) {
            content.parse::<kw::cfg>()?;
            ParseAttributeData::Cfg(content.parse()?)
        } else if lookahead.peek(kw::archetype_id) {
            content.parse::<kw::archetype_id>()?;
            ParseAttributeData::ArchetypeId(content.parse()?)
        } else if lookahead.peek(kw::component_id) {
            content.parse::<kw::component_id>()?;
            ParseAttributeData::ComponentId(content.parse()?)
        } else {
            return Err(lookahead.error());
        };

        Ok(Self { span, data })
    }
}

impl Parse for ParseAttributeId {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args;
        parenthesized!(args in input);

        // Grab the int literal and make sure it's the right type
        let value = args.parse::<LitInt>()?.base10_parse()?;

        Ok(Self { value })
    }
}

impl HasAttributeId for ParseArchetype {
    fn name_to_string(&self) -> String {
        self.name.to_string()
    }

    fn span(&self) -> Span {
        self.name.span()
    }

    fn id(&self) -> Option<u8> {
        self.id
    }
}

impl HasAttributeId for ParseComponent {
    fn name_to_string(&self) -> String {
        self.name.to_string()
    }

    fn span(&self) -> Span {
        self.name.span()
    }

    fn id(&self) -> Option<u8> {
        self.id
    }
}

impl Parse for ParseAttributeCfg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args;
        parenthesized!(args in input);

        // Don't care about parsing the predicate contents
        let predicate = args.parse::<TokenStream>()?;

        Ok(Self { predicate })
    }
}
