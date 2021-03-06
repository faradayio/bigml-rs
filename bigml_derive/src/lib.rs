// `proc_macro` is built into the compiler.
extern crate proc_macro;
// `proc_macro2` wraps `proc_macro` in a way that should be compatible with the
// upcoming `proc_macro` additions, so that we can use the old or new
// `proc_macro` APIs with minimal tweaking.
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

// In this file, we want `proc_macro::TokenStream` to interface with the outside
// world.
use proc_macro::TokenStream;

mod resource;
mod updatable;

/// Derive boilerplate code for `Resource`.
#[proc_macro_derive(Resource, attributes(api_name))]
pub fn resource_derive(input: TokenStream) -> TokenStream {
    // Rust procedural macros are really limited right now:
    //
    // - We can only parse `TokenStream` using the `syn` library. This is
    //   because `TokenStream` hasn't been fully standardized.
    // - We can only report errors via a panic.
    let input = syn::parse(input).unwrap();
    let gen = resource::derive(&input);
    gen.into()
}

/// Derive boilerplate code for `Updatable`.
#[proc_macro_derive(Updatable, attributes(updatable))]
pub fn updatable_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse(input).unwrap();
    let gen = updatable::derive(&input);
    gen.into()
}
