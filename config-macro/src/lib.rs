mod input;
mod output;

use crate::input::ConfigInput;
use crate::output::generate_output;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::{emit_error, proc_macro_error};
use quote::quote;
use std::collections::HashMap;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, FieldsNamed};

#[proc_macro_error]
#[proc_macro_attribute]
pub fn config_struct(attr: TokenStream, input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(attr as ConfigInput);
    let ast = parse_macro_input!(input as DeriveInput);

    let yml = get_yam_values(config);
    let additional_fields = get_struct_additional_fields(&yml);
    let initial_values = get_additional_fields_init(&yml);

    let name = ast.ident;
    let attributes = ast.attrs;
    let fields = match ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => named,
        _ => {
            emit_error!(
                name.span(),
                "config_struct macro can only be applied to structures with named fields"
            );
            return quote!().into();
        }
    };
    let fields_initial = fields.iter().map(|f| {
        let ident = &f.ident;
        quote!(#ident: Default::default())
    });
    let fields = fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        quote!(#ident: #ty)
    });

    let output = quote!(
        #(#attributes)*
        pub struct #name {
            #(#fields,)*
            #(#additional_fields,)*
        }

        impl #name {
            pub fn new() -> Self {
                Self {
                    #(#fields_initial,)*
                    #(#initial_values,)*
                }
            }
        }
    );

    output.into()
}

#[proc_macro]
pub fn config(input: TokenStream) -> TokenStream {
    let config_input = parse_macro_input!(input as ConfigInput);
    eprintln!("macro input {config_input:?}");

    let yml = get_yam_values(config_input);
    let output = generate_output(yml);

    output.into()
}

fn get_yam_values(config: ConfigInput) -> HashMap<String, String> {
    let path = config
        .path
        .unwrap_or_else(|| "./configuration/config.yaml".into());

    let file = std::fs::read_to_string(path.clone())
        .unwrap_or_else(|_| panic!("failed to read file with path {path}"));

    serde_yaml::from_str(&file).expect("failed to parse yaml file")
}

fn get_struct_additional_fields(yml: &HashMap<String, String>) -> Vec<proc_macro2::TokenStream> {
    yml.iter()
        .map(|(k, _)| {
            let ident = Ident::new(k, Span::call_site());
            quote!(#ident: String)
        })
        .collect()
}

fn get_additional_fields_init(yml: &HashMap<String, String>) -> Vec<proc_macro2::TokenStream> {
    yml.iter()
        .map(|(k, v)| {
            let ident = Ident::new(k, Span::call_site());
            quote!(#ident: #v.into())
        })
        .collect()
}
