pub use generate::{Generation, GenerationCmd};
pub extern crate i_codegen_derive;
// re-export macros (note pub)
#[doc(hidden)]
pub use i_codegen_code::{linkme, Context, CODEGEN_ITEMS};
/// Test documentation
pub use i_codegen_derive::Codegen;
pub use i_codegen_derive::fn_codegen;

#[cfg(test)]
mod test;

mod generate;
