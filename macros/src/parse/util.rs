use syn::parse::{Parse, ParseStream};
use syn::token::Comma;
use syn::{Ident, Path};

#[derive(Debug)]
pub struct ParseEcsComponentId {
    pub component: Ident,
    pub archetype: Option<Path>,
}

impl Parse for ParseEcsComponentId {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let component = input.parse::<Ident>()?;

        if input.parse::<Option<Comma>>()?.is_some() {
            let archetype = Some(input.parse::<Path>()?);
            input.parse::<Option<Comma>>()?;

            Ok(Self {
                component,
                archetype,
            })
        } else {
            Ok(Self {
                component,
                archetype: None,
            })
        }
    }
}
