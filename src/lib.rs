#[macro_use]
pub extern crate i_codegen_derive;
// re-export macros (note pub)
/// Test documentation
pub use i_codegen_derive::Codegen;
#[doc(hidden)]
pub use i_codegen_code::{linkme, Context, CODEGEN_ITEMS};

#[cfg(test)]
mod test;

mod generate;
