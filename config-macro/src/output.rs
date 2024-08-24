use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::LitStr;

impl std::ops::Deref for KeyValue {
    type Target = ();

    fn deref(&self) -> &Self::Target {
        todo!()
    }
}

pub fn generate_output(output: HashMap<String, String>) -> TokenStream {
    let kv = output
        .into_iter()
        .map(|(k, v)| KeyValue(k, v))
        .collect::<Vec<_>>();
    eprintln!("{kv:?}");

    quote!(
        #[derive(Debug)]
        pub struct Config {
            pub values: std::collections::HashMap<String, String>,
        }

        impl std::ops::Deref for Config{
            type Target = std::collections::HashMap<String, String>;

            fn deref(&self) -> &Self::Target {
                &self.values
            }

        }

        impl Config {
            pub fn new() -> Self {
                let configuration_values = std::collections::HashMap::from([#(#kv),*]);
                Self{values: configuration_values}
            }
        }
    )
}

#[derive(Debug)]
struct KeyValue(String, String);

impl ToTokens for KeyValue {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let key = LitStr::new(&self.0, Span::call_site());
        let value = LitStr::new(&self.1, Span::call_site());
        tokens.extend(quote!((#key.into(), #value.into())));
    }
}
