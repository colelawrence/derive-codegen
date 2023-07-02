use serde::{Deserialize, Serialize};

#[macro_use]
pub extern crate i_codegen_derive;
// re-export macros (note pub)
pub use i_codegen_derive::*;
// re-export types (note pub)
#[doc(hidden)]
pub use i_codegen_code::{linkme, Context, CODEGEN_ITEMS};
pub use i_codegen_types::*;

#[cfg(test)]
mod test;

mod generate;
