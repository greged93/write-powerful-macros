use proc_macro::TokenStream;
use proc_macro_error::emit_error;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Attribute, DeriveInput, Expr, ExprLit, Field, Fields, Ident, Lit,
    MetaNameValue,
};

#[proc_macro_derive(Builder, attributes(rename, builder_defaults))]
pub fn builder(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    // Check if the `builder_defaults` macro is active
    let use_defaults = ast
        .attrs
        .iter()
        .any(|a| a.path().is_ident("builder_defaults"));

    // Construct the builder struct from the ast
    let builder = Builder::try_from(ast).expect("failed to create builder");

    // Fetch the setters, fields and build function for the builder.
    let setters = builder.setters();
    let marker_types = builder.marker_types();
    let builder_fields = builder.fields();
    let build_fn = builder.build_fn(use_defaults);

    // Get the builder name
    let builder_name = &builder.builder_name;

    quote!(
        #[derive(Default)]
        pub struct #builder_name<T = Init> {
            _marker: std::marker::PhantomData<T>,
            #(#builder_fields),*
        }

        #[derive(Default)]
        pub struct Final;
        #[derive(Default)]
        pub struct Init;
        #(#marker_types)*

        #(#setters)*

        impl #builder_name<Final> {
            #build_fn
        }
    )
    .into()
}

struct Builder {
    fields: Vec<Field>,
    struct_name: Ident,
    builder_name: Ident,
}

impl TryFrom<DeriveInput> for Builder {
    type Error = Box<dyn std::error::Error>;

    fn try_from(input: DeriveInput) -> Result<Self, Self::Error> {
        let struct_name = input.ident.clone();
        let builder_name = format_ident!("{}Builder", struct_name);

        match input.data {
            syn::Data::Struct(syn::DataStruct {
                fields: Fields::Named(syn::FieldsNamed { named, .. }),
                ..
            }) => Ok(Builder {
                fields: named.into_iter().collect(),
                struct_name,
                builder_name,
            }),
            _ => {
                emit_error!(input.ident, "builder only available for Struct");
                Err("".into())
            }
        }
    }
}

impl Builder {
    /// Generate the fields for the builder struct
    fn fields(&self) -> Vec<proc_macro2::TokenStream> {
        self.fields
            .iter()
            .map(|f| {
                let name = &f.ident;
                let ty = &f.ty;
                quote!(#name: Option<#ty>)
            })
            .collect()
    }

    /// Generate the marker type identifier for the builder's field
    fn marker_type_ident(&self, field: String) -> Ident {
        format_ident!("{}Of{}", self.builder_name, field)
    }

    /// Generate the marker types for the builder struct
    fn marker_types(&self) -> Vec<proc_macro2::TokenStream> {
        self.fields
            .iter()
            .map(|f| self.marker_type_ident(maybe_ident_to_string(&f.ident)))
            .map(|ty| quote! {pub struct #ty;})
            .collect()
    }

    /// Generate the setters for the builder struct
    fn setters(&self) -> Vec<proc_macro2::TokenStream> {
        let mut fields = self.fields.iter().peekable();
        let builder_name = &self.builder_name;
        let mut marker_type_prev = Ident::new("Init", proc_macro2::Span::call_site());

        let mut setters = Vec::with_capacity(self.fields.len());

        // Generate all the fields in order to rebuild the builder when
        // modifying the marker
        let builder_fields = fields.clone().map(|f| {
            let name = &f.ident;
            quote!(#name: self.#name)
        });

        while let Some(f) = fields.next() {
            let fn_name = format_ident!("with_{}", get_field_name(f));
            let ty = &f.ty;
            let name = &f.ident;
            let marker_type_out = self.marker_type_ident(maybe_ident_to_string(name));

            // Handle the case of the last field separately.
            let (marker_in, marker_out) = if fields.peek().is_none() {
                (quote!(<#marker_type_prev>), quote!(<Final>))
            } else {
                (quote!(<#marker_type_prev>), quote!(<#marker_type_out>))
            };

            let builder_fields = builder_fields.clone();

            let setter = quote!(
                impl #builder_name #marker_in {
                    pub fn #fn_name(mut self, #name: #ty) -> #builder_name #marker_out {
                        self.#name = Some(#name);
                        #builder_name {
                            _marker: std::marker::PhantomData,
                            #(#builder_fields),*
                        }
                    }
                }
            );

            setters.push(setter);
            marker_type_prev = marker_type_out;
        }

        setters
    }

    /// Generate the build function for the builder struct
    ///
    ///  If `use_defaults` is set to true, use `unwrap_or_default()` to
    ///  unwrap the builder's fields
    fn build_fn(&self, use_defaults: bool) -> proc_macro2::TokenStream {
        let fields = self.fields.iter().map(|f| {
            let name = &f.ident;
            if use_defaults {
                quote!(#name: self.#name.take().unwrap_or_default())
            } else {
                let error_message = format!("missing field {}", maybe_ident_to_string(name));
                quote!(#name: self.#name.take().ok_or_else(|| #error_message)?)
            }
        });

        let struct_name = &self.struct_name;

        quote!(
            pub fn build(mut self) -> Result<#struct_name, Box<dyn std::error::Error>> {
                Ok(#struct_name {
                    #(#fields),*
                })
            }
        )
    }
}

/// Get the field name. Field's identifier by default or the
/// rename attribute if it exists.
fn get_field_name(f: &Field) -> String {
    let rename_attr = f.attrs.iter().find(|attr| attr.path().is_ident("rename"));
    rename_attr
        .and_then(get_renamed_field)
        .or_else(|| f.ident.clone())
        .map(|i| i.to_string())
        .unwrap_or_default()
}

/// Get the renamed field from the attribute
fn get_renamed_field(attr: &Attribute) -> Option<Ident> {
    match attr.meta {
        syn::Meta::List(ref list) => {
            let name = list.tokens.to_string();
            Some(Ident::new(&name, name.span()))
        }
        syn::Meta::NameValue(MetaNameValue {
            value:
                Expr::Lit(ExprLit {
                    lit: Lit::Str(ref str),
                    ..
                }),
            ..
        }) => {
            let name = str.value();
            Some(Ident::new(&name, name.span()))
        }
        _ => None,
    }
}

/// Convert an optional ident to a string
fn maybe_ident_to_string(maybe_ident: &Option<Ident>) -> String {
    maybe_ident
        .as_ref()
        .map(|i| i.to_string())
        .unwrap_or_default()
}
