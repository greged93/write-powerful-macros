use syn::parse::{Parse, ParseStream};
use syn::Token;

mod kw {
    syn::custom_keyword!(path);
    syn::custom_keyword!(exclude);
}

#[derive(Debug)]
pub(crate) struct ConfigInput {
    pub(crate) path: Option<String>,
    pub(crate) exclude_from: bool,
}

impl Parse for ConfigInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(ConfigInput {
                path: None,
                exclude_from: false,
            });
        }

        let mut path = None;
        let mut exclude_from = false;

        while !input.is_empty() {
            if input.peek(kw::path) {
                let _ = input.parse::<kw::path>().unwrap();
                let _ = input
                    .parse::<Token![=]>()
                    .map_err(|_| syn::Error::new(input.span(), "expected '=' token"))?;

                let lit_path = input
                    .parse::<syn::LitStr>()
                    .map_err(|_| syn::Error::new(input.span(), "expected string literal"))?;

                path = Some(lit_path.value());
            } else if input.peek(kw::exclude) {
                let _ = input.parse::<kw::exclude>().unwrap();
                let _ = input
                    .parse::<Token![=]>()
                    .map_err(|_| syn::Error::new(input.span(), "expected '=' token"))?;

                let exclude = input
                    .parse::<syn::LitStr>()
                    .map_err(|_| syn::Error::new(input.span(), "expected string literal"))?;

                exclude_from = exclude.value() == "from";
            } else {
                return Err(syn::Error::new(
                    input.span(),
                    "expected 'path' or 'exclude', got {}",
                ));
            }
        }

        Ok(ConfigInput { path, exclude_from })
    }
}
