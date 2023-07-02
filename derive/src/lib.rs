#![allow(
    clippy::module_name_repetitions,
    clippy::needless_pass_by_value,
    clippy::unseparated_literal_suffix
)]

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Codegen, attributes(codegen, serde))]
pub fn derive_codegen(input: TokenStream) -> TokenStream {
    cod::derive(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

mod attr;
mod cod;
