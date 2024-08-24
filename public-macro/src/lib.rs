use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro_error::{emit_error, proc_macro_error};
use quote::quote;
use syn::parse::{Parse, Parser};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_macro_input, Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed,
    MetaList, Token,
};

#[proc_macro_error]
#[proc_macro_attribute]
pub fn public(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let excluded_attributes = parse_macro_input!(attr as ExcludeAttributes);

    let name = ast.ident;
    let fields_and_brackets = match ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => parse_named(named, excluded_attributes),
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(FieldsUnnamed { unnamed, .. }),
            ..
        }) => parse_unnamed(unnamed),
        _ => unimplemented!("Only structs are supported"),
    };

    let public_version = quote! {
       pub struct #name #fields_and_brackets
    };
    public_version.into()
}

struct ExcludeAttributes(Vec<String>);

impl Parse for ExcludeAttributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        match input.parse::<MetaList>() {
            Ok(meta) => {
                if meta.path.segments.iter().any(|x| x.ident == "exclude") {
                    let parser = Punctuated::<Ident, Token![,]>::parse_terminated;

                    let excluded = parser.parse(meta.tokens.into())?;
                    let excluded = excluded
                        .into_iter()
                        .map(|ident| ident.to_string())
                        .collect();
                    return Ok(ExcludeAttributes(excluded));
                }
                emit_error!(input.span(), "Expected `exclude` attribute");

                Ok(ExcludeAttributes(vec![]))
            }
            Err(_) => {
                emit_error!(input.span(), "Expected `exclude` attribute");
                Ok(ExcludeAttributes(vec![]))
            }
        }
    }
}

fn parse_named(
    fields: Punctuated<Field, Comma>,
    excluded_attributes: ExcludeAttributes,
) -> proc_macro2::TokenStream {
    let fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        let excluded = excluded_attributes
            .0
            .iter()
            .any(|x| x == &name.as_ref().unwrap().to_string());

        if excluded {
            quote! {
                #name: #ty
            }
        } else {
            quote! {
                pub #name: #ty
            }
        }
    });

    quote! {
        {
            #(#fields),*
        }
    }
}

fn parse_unnamed(fields: Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let fields = fields.iter().map(|f| {
        let ty = &f.ty;
        quote! {
            pub #ty
        }
    });

    quote! {
       (#(#fields),*);
    }
}
