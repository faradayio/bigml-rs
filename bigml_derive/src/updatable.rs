//! Implementation of `#[derive(Updatable)]`.

// In this macro, we want `proc_macro2::TokenStream` to manipulate the AST using
// high-level APIs.
use proc_macro2::{Ident, Span, TokenStream};
use syn::{Attribute, DeriveInput, Lit, Meta, MetaNameValue};

/// Do the actual code generation for a `Resource`.
pub(crate) fn derive(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let vis = &ast.vis;
    let update_name = Ident::new(&format!("{}Update", name), Span::call_site());
    let update_comment = format!("An update to `{}`.", name);
    quote! {
        impl Updatable for #name {
            type Update = #update_name;
        }

        #[doc = #update_comment]
        #[derive(Clone, Debug, PartialEq, Serialize)]
        #vis struct #update_name {

        }
    }
}
