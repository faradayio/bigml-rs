//! Implementation of `#[derive(Resource)]`.

// In this macro, we want `proc_macro2::TokenStream` to manipulate the AST using
// high-level APIs.
use proc_macro2::TokenStream;
use syn::{Attribute, DeriveInput, Lit, Meta, MetaNameValue};

/// Do the actual code generation for a `Resource`.
pub(crate) fn derive(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let api_name = get_api_name(&ast.attrs);
    quote! {
        impl #impl_generics Resource for #name #ty_generics #where_clause {
            fn id_prefix() -> &'static str {
                concat!(#api_name, "/")
            }

            fn create_path() -> &'static str {
                concat!("/", #api_name)
            }

            fn common(&self) -> &ResourceCommon {
                &self.common
            }

            fn id(&self) -> &Id<Self> {
                &self.resource
            }

            fn status(&self) -> &Status {
                &self.status
            }
        }
    }
}

/// Search for an `#[api_name = "my_resource"]` attribute and return
/// `"my_resource"` as a `Lit` value.
fn get_api_name(attrs: &[Attribute]) -> Lit {
    for attr in attrs {
        // Parse the `#[...]` expression, called a "meta" in Rust's grammar.
        let meta = attr
            .parse_meta()
            .expect("Invalid `api_name`, try #[api_name = \"my_resource\"]");
        if meta.path().is_ident("api_name") {
            match meta {
                Meta::NameValue(MetaNameValue { lit, .. }) => return lit,
                _ => panic!("Invalid `api_name`, try #[api_name = \"my_resource\"]"),
            }
        }
    }
    panic!("Missing attribute `api_name`, try `#[api_name = \"...\"]`");
}
