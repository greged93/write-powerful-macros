use proc_macro::TokenStream;
use quote::{quote, TokenStreamExt};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Token};

mod kw {
    syn::custom_keyword!(bucket);
    syn::custom_keyword!(lambda);
    syn::custom_keyword!(mem);
    syn::custom_keyword!(time);
}

#[proc_macro]
pub fn iac(input: TokenStream) -> TokenStream {
    let iac_input: IacInput = parse_macro_input!(input);
    eprintln!("{:?}", iac_input);

    let mut output = quote! {};
    output.into()
}

#[derive(Debug)]
struct IacInput {
    bucket: Option<Bucket>,
    lambda: Option<Lambda>,
}

impl IacInput {
    fn has_resources(&self) -> bool {
        self.bucket.is_some() || self.lambda.is_some()
    }
}

impl Parse for IacInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut bucket: Option<Bucket> = None;
        let mut lambda: Option<Lambda> = None;

        loop {
            if input.peek(kw::bucket) {
                bucket = Some(input.parse()?);
            } else if input.peek(kw::lambda) {
                lambda = Some(input.parse()?)
            } else if !input.is_empty() {
                return Err(syn::Error::new(
                    input.lookahead1().error().span(),
                    "only lambda and bucket are allowed as keywords",
                ));
            } else {
                break;
            }
        }

        if bucket.as_ref().map_or(false, |b| b.has_event) && lambda.is_none() {
            return Err(syn::Error::new(
                input.span(),
                "bucket with event must be followed by lambda",
            ));
        }

        Ok(Self { bucket, lambda })
    }
}

#[derive(Debug)]
struct Bucket {
    name: String,
    has_event: bool,
}

impl Parse for Bucket {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _ = input.parse::<kw::bucket>()?;

        let name = input
            .parse::<syn::Ident>()
            .map_err(|_| syn::Error::new(input.span(), "bucket needs a name"))?
            .to_string();
        let has_event = input.parse::<syn::Token![=>]>().is_ok();

        Ok(Self { name, has_event })
    }
}

#[derive(Debug)]
struct Lambda {
    name: String,
    mem: Option<u16>,
    time: Option<u16>,
}

impl Parse for Lambda {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _ = input.parse::<kw::lambda>()?;

        let mut mem = None;
        let mut time = None;
        let mut name = None;

        let content;
        syn::parenthesized!(content in input);

        let kvs = Punctuated::<KeyValue, Token![,]>::parse_terminated(&content)?;

        for kv in kvs {
            match kv.key.as_str() {
                "mem" => {
                    mem = Some(
                        str::parse(&kv.value)
                            .map_err(|_| syn::Error::new(input.span(), "invalid u16 value"))?,
                    )
                }
                "time" => {
                    time = Some(
                        str::parse(&kv.value)
                            .map_err(|_| syn::Error::new(input.span(), "invalid u16 value"))?,
                    )
                }
                "name" => name = Some(kv.value),
                other => {
                    return Err(syn::Error::new(
                        content.span(),
                        format!("expected 'name', 'mem' or 'time', got {other}"),
                    ))
                }
            }
        }

        let name = name.ok_or_else(|| syn::Error::new(input.span(), "expected name for lambda"))?;

        Ok(Self { name, mem, time })
    }
}

struct KeyValue {
    key: String,
    value: String,
}

impl Parse for KeyValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key = input.parse::<syn::Ident>()?.to_string();
        let _ = input.parse::<Token![=]>()?;
        let value = match key.as_str() {
            "mem" | "time" => input.parse::<syn::LitInt>()?.to_string(),
            "name" => input.parse::<syn::Ident>()?.to_string(),
            other => {
                return Err(syn::Error::new(
                    input.span(),
                    format!("expected 'name', 'mem' or 'time', got {other}"),
                ))
            }
        };

        Ok(Self { key, value })
    }
}
