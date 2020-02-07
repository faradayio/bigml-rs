//! Implementation of `#[derive(Updatable)]`.

// In this macro, we want `proc_macro2::TokenStream` to manipulate the AST using
// high-level APIs.
use proc_macro2::{Ident, Span, TokenStream};
use syn::{Data, DeriveInput, Field, Meta, MetaList, NestedMeta};

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
        #[non_exhaustive]
        #vis struct #update_name {
            #( #update_fields )*
        }
    }
}

/// Find all `#[updatable]` fields in the original struct, and return a list of
/// fields for our `*Update` struct.
fn fields_for_update_type(ast: &DeriveInput) -> Vec<TokenStream> {
    let mut new_fields = vec![];

    if let Data::Struct(ref data_struct) = ast.data {
        for field in &data_struct.fields {
            if let Some(field_opts) = updatable_field_options(field) {
                let attrs = &field_opts.attrs;
                let vis = &field.vis;
                let name = field
                    .ident
                    .as_ref()
                    .expect("Cannot `#[derive(Updatable)]` for tuple struct");
                let ty = &field.ty;
                let comment = format!("New value for `{}` (optional).", name);
                new_fields.push(quote! {
                    #[doc = #comment]
                    #( #attrs )*
                    #vis #name: Option<<#ty as Updatable>::Update>,
                });
            }
        }
    } else {
        panic!("`#[derive(Updatable)]` may only be used on structs");
    }

    new_fields
}

/// Options specified by an `#[updatable(...)]` attribute.
#[derive(Debug, Default)]
struct UpdatableFieldOptions {
    /// Do we want `serde` to flatten this attr into the containing struct for
    /// us? This involves some tweaking.
    flatten: bool,
    /// Attrs to pass through to the generated field.
    attrs: Vec<TokenStream>,
}

/// If the specified structure field is marked with `#[updatable]` or
/// `#[updatable(..)]`, return all relevant information.
fn updatable_field_options(field: &Field) -> Option<UpdatableFieldOptions> {
    let mut updatable = false;
    let mut field_opts = UpdatableFieldOptions::default();
    let mut flatten = false;
    for attr in &field.attrs {
        let meta = attr.parse_meta().expect("unparseable attribute");
        if meta.path().is_ident("updatable") {
            updatable = true;
            match meta {
                // We have `#[updatable]`, do nothing.
                Meta::Path(_) => {}
                // We have `#[updatable(..)]`, look for nested options.
                Meta::List(MetaList {
                    nested: options, ..
                }) => {
                    for option in options {
                        match option {
                            // We have a `flatten` option.
                            NestedMeta::Meta(ref flatten_meta)
                                if flatten_meta.path().is_ident("flatten") =>
                            {
                                if let Meta::Path(_) = flatten_meta {
                                    flatten = true;
                                } else {
                                    panic!(
                                        "#[updatable(flatten)] may not have arguments"
                                    );
                                }
                            }

                            // We have an `attr(..)` option, so extract it and
                            // add to `field_opts.attrs`.
                            //
                            // TODO: Do we want to keep this? It's not being used, but it's
                            // potentially quite useful.
                            NestedMeta::Meta(ref attr_meta)
                                if attr_meta.path().is_ident("attr") =>
                            {
                                match attr_meta {
                                    Meta::List(MetaList {
                                        nested: attr_values,
                                        ..
                                    }) => {
                                        for attr_value in attr_values {
                                            // Wrap in `#[..]`.
                                            field_opts.attrs.push(quote! {
                                                #[ #attr_value ]
                                            });
                                        }
                                    }
                                    _ => {
                                        panic!("cannot parse `#[updatable(attr(..))]`")
                                    }
                                }
                            }
                            _ => {
                                panic!("unexpected option in `#[updatable(..)]`");
                            }
                        }
                    }
                }
                _ => panic!("expected `#[updatable]` or `#[updatable(..)]`"),
            }
        }
    }
    if flatten {
        field_opts.attrs.push(quote! { #[serde(flatten)] });
    } else {
        field_opts.attrs.push(quote! {
            #[serde(skip_serializing_if="Option::is_none")]
        });
    }
    if updatable {
        Some(field_opts)
    } else {
        None
    }
}
