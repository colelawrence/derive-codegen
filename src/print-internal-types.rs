#[macro_use]
pub extern crate i_codegen_derive;
use generate::Generation;
use std::process::Command;

mod generate;

fn main() {
    println!(
        "{}",
        Generation::for_tag("derive-codegen-internal").to_input_json_pretty()
    );
}
