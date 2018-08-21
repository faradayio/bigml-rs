// `proc_macro` is built into the compiler.
extern crate proc_macro;
// `proc_macro2` wraps `proc_macro` in a way that should be compatible with the
// upcoming `proc_macro` additions, so that we can use the old or new
// `proc_macro` APIs with minimal tweaking.
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

mod resource;

/// Derive boilerplate code for `Resource`.
#[proc_macro_derive(Resource, attributes(api_name))]
pub fn resource_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Rust procedural macros are really limited right now:
    //
    // - We can only parse `TokenStream` using the `syn` library. This is
    //   because `TokenStream` hasn't been fully standardized.
    // - We can only report errors via a panic.
    let input = syn::parse(input).unwrap();
    let gen = resource::derive(&input);
    gen.into()
}
