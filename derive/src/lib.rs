#![allow(
    clippy::module_name_repetitions,
    clippy::needless_pass_by_value,
    clippy::unseparated_literal_suffix
)]

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, ItemFn, Attribute};

mod parse;

// TODO: Support `#[codegen(crate = "wrapping_crate::derive_codegen")]`
// https://github.com/dtolnay/linkme/blob/87e9f68b354421341eccb31c1f0dba0b63cc205d/impl/src/attr.rs#L4-L5
/// Include this struct or enum in a retrievable metadata list in the `derive_codegen` crate.
#[proc_macro_derive(Codegen, attributes(codegen, serde))]
pub fn derive_codegen(input: TokenStream) -> TokenStream {
    parse::derive(
        parse_macro_input!(input as DeriveInput),
        parse::LinkKind::External {
            crate_name: "derive_codegen",
        },
    )
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}

/// Submit functions to your code generator
///
/// example:
/// ```rs
/// #[fn_codegen(tag = "my-tag")]
/// fn my_function(item: i32) -> Result<(), String> {
///   Err("Not implemented".to_string())
/// }
/// #[fn_codegen(tag = "my-tag")]
/// fn my_function(
///     #[codegen(myattr = "something")]
///     item: i32
/// ) -> Result<(), String> {
///   Err("Not implemented".to_string())
/// }
/// ```
#[proc_macro_attribute]
pub fn fn_codegen(
    // TODO: enable parsing this as part of the codegen attributes
    _attribute: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    // Hmm: Could we have a general `#[codegen]` attr macro that auto-detects the kind of item and infers the derive stuff, etc?
    // Would you need to discern when there are multiple `#[codegen]` things? What would happen if the codegen is on a field?
    let generated = parse::item_fn(
        function.clone(),
        parse::LinkKind::External {
            crate_name: "derive_codegen",
        },
    )
    .unwrap_or_else(|err| err.to_compile_error());
    // // check that the reference links
    // output.extend(quote::quote! {
    //     type _ = #ident;
    // });
    proc_macro2::TokenStream::from(quote::quote! {
         #function

         #generated
    })
    .into()
}

/// Necessary to attach attributes outside the context of
/// a derivation.
#[proc_macro_attribute]
pub fn codegen(
    _attributes: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    return item;
}

#[doc(hidden)]
#[proc_macro_derive(CodegenInternal, attributes(codegen, serde))]
pub fn derive_codegen_internal(input: TokenStream) -> TokenStream {
    parse::derive(
        parse_macro_input!(input as DeriveInput),
        parse::LinkKind::Internal,
    )
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}
