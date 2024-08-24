use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Field, Ident};

#[proc_macro]
pub fn analyze(input: TokenStream) -> TokenStream {
    let _ = syn::parse_macro_input!(input as StructWithComments);

    quote!({}).into()
}

struct StructWithComments {
    ident: Ident,
    fields: Vec<Field>,
    outer_attributes: Vec<Attribute>,
    inner_attributes: Vec<Vec<Attribute>>,
}

impl Parse for StructWithComments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let outer = input.call(Attribute::parse_outer)?;
        let _ = input.parse::<syn::Token![struct]>()?;
        let ident = input.parse::<syn::Ident>()?;

        let content;
        syn::braced!(content in input);

        let mut fields = Vec::new();
        let mut inner_attributes = Vec::new();

        while !content.is_empty() {
            let inner = content.call(Attribute::parse_inner)?;
            inner_attributes.push(inner);

            let field = Field::parse_named(&content)?;
            let _ = content.parse::<syn::Token![,]>()?;

            fields.push(field);
        }

        Ok(Self {
            ident,
            fields,
            outer_attributes: outer,
            inner_attributes,
        })
    }
}
