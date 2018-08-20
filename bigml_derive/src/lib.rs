// `proc_macro` is built into the compiler.
extern crate proc_macro;
// `proc_macro2` wraps `proc_macro` in a way that should be compatible with the
// upcoming `proc_macro` additions, so that we can use the old or new
// `proc_macro` APIs with minimal tweaking.
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use syn::{Attribute, Lit, Meta, MetaNameValue};

/// Derive boilerplate code for a `Resource`.
#[proc_macro_derive(Resource, attributes(api_name))]
pub fn resource_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Rust procedural macros are really limited right now:
    //
    // - We can only parse `TokenStream` using the `syn` library. This is
    //   because `TokenStream` hasn't been fully standardized.
    // - We can only report errors via a panic.
    let input = syn::parse(input).unwrap();
    let gen = resource_derive_impl(&input);
    gen.into()
}

/// Do the actual code generation for a `Resource`.
fn resource_derive_impl(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
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
        let meta = attr.interpret_meta()
            .expect("Invalid `api_name`, try #[api_name = \"my_resource\"]");
        if meta.name() == "api_name" {
            match meta {
                Meta::NameValue(MetaNameValue { lit, .. }) => return lit,
                _ => panic!("Invalid `api_name`, try #[api_name = \"my_resource\"]"),
            }
        }
    }
    panic!("Missing attribute `api_name`, try `#[api_name = \"...\"]`");
}
