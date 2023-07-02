use linkme::distributed_slice;
use serde::{Deserialize, Serialize};

#[macro_use]
pub extern crate i_herenow_serde_generate_derive;
// re-export macros (note pub)
pub use i_herenow_serde_generate_derive::*;
// re-export types (note pub)
pub use i_herenow_serde_generate_types::*;

#[cfg(test)]
mod test;

mod generate;
