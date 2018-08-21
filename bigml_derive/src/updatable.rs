//! Implementation of `#[derive(Updatable)]`.

// In this macro, we want `proc_macro2::TokenStream` to manipulate the AST using
// high-level APIs.
use proc_macro2::{Ident, Span, TokenStream};
use syn::{Data, DeriveInput, Field};

/// Do the actual code generation for a `Resource`.
pub(crate) fn derive(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let vis = &ast.vis;
    let update_name = Ident::new(&format!("{}Update", name), Span::call_site());
    let update_comment = format!("An update to `{}`.", name);
    let update_fields = fields_for_update_type(ast);
    quote! {
        impl Updatable for #name {
            type Update = #update_name;
        }

        #[doc = #update_comment]
        #[derive(Clone, Debug, Default, PartialEq, Serialize)]
        #vis struct #update_name {
            #( #update_fields )*

            /// Placeholder to allow for future extension without breaking the
            /// API. Pleae replace this with `#[non_exhaustive]` when it becomes
            /// stable.
            #[serde(skip)]
            _placeholder: (),
        }
    }
}

/// Find all `#[updatable]` fields in the original struct, and return a list of
/// fields for our `*Update` struct.
fn fields_for_update_type(ast: &DeriveInput) -> Vec<TokenStream> {
    let mut new_fields = vec![];

    if let Data::Struct(ref data_struct) = ast.data {
        for field in &data_struct.fields {
            if field_is_updatable(field) {
                let vis = &field.vis;
                let name = field.ident.as_ref()
                    .expect("Cannot `#[derive(Updatable)]` for tuple struct");
                let ty = &field.ty;
                let comment = format!("New value for `{}` (optional).", name);
                new_fields.push(quote! {
                    #[doc = #comment]
                    #[serde(skip_serializing_if="Option::is_none")]
                    #vis #name: Option<<#ty as Updatable>::Update>,
                });
            }
        }
    } else {
        panic!("`#[derive(Updatable)]` may only be used on structs");
    }

    new_fields
}

/// Is the specific structure field marked with `#[updatable]`?
fn field_is_updatable(field: &Field) -> bool {
    // TODO: Make sure `#[updatable]` appears at most once, with the correct
    // syntax.
    field.attrs
        .iter()
        .find(|attr| {
            let meta = attr.interpret_meta().expect("Unparseable attribute");
            meta.name() == "updatable"
        })
        .is_some()
}
