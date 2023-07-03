use derive_codegen::{Codegen, Generation};
use gumdrop::Options;
use std::process::Command;

#[derive(Options)]
pub(crate) struct SubOptions {
    #[options(help = "show help")]
    help: bool,
}

#[derive(Codegen)]
#[codegen(tags = "simple-go", package = "simple")]
struct SimpleStruct {
    sstr: String,
    sint: i64,
}

#[derive(Codegen)]
#[codegen(tags = "simple-go", package = "simple")]
enum SimpleEnum {
    VUnit,
    VStr(String),
    VTuple(String, i64),
    VStruct { vfield: String },
}

pub(crate) fn run(options: SubOptions) {
    if options.help {
        println!("{}", SubOptions::usage());
        return;
    }

    Generation::for_tag("simple-go")
        .as_arg_of(
            Command::new("deno")
                .arg("run")
                .arg("./golang-generator/generate-go.ts"),
        )
        .with_output_path("./golang-generator/codegen-types")
        .write();
}
