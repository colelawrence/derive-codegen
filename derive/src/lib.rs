#![allow(
    clippy::module_name_repetitions,
    clippy::needless_pass_by_value,
    clippy::unseparated_literal_suffix
)]

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod parse;

// TODO: Support `#[codegen(crate = "wrapping_crate::derive_codegen")]`
// https://github.com/dtolnay/linkme/blob/87e9f68b354421341eccb31c1f0dba0b63cc205d/impl/src/attr.rs#L4-L5
/// Include this struct or enum in a retrievable metadata list in the `derive_codegen` crate.
#[proc_macro_derive(Codegen, attributes(codegen, serde))]
pub fn derive_codegen(input: TokenStream) -> TokenStream {
    parse::derive(
        parse_macro_input!(input as DeriveInput),
        parse::DerivationKind::External {
            crate_name: "derive_codegen",
        },
    )
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}

#[doc(hidden)]
#[proc_macro_derive(CodegenInternal, attributes(codegen, serde))]
pub fn derive_codegen_internal(input: TokenStream) -> TokenStream {
    parse::derive(
        parse_macro_input!(input as DeriveInput),
        parse::DerivationKind::Internal,
    )
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}
