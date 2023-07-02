#[macro_use]
pub extern crate i_codegen_derive;
use std::process::Command;
use generate::Generation;

mod generate;

fn main() {
    Generation::for_tag("derive-codegen-internal")
        .as_arg_of(
            Command::new("deno")
                .arg("run")
                .arg("./typescript-generator/generate-typescript.ts"),
        )
        .with_output_path("./typescript-generator/codegen-types")
        .write();
}
