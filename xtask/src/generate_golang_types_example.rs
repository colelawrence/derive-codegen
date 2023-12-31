#![allow(unused)]
use derive_codegen::{Codegen, Generation};
use std::process::Command;

#[derive(Debug, clap::Parser)]
pub(crate) struct SubOptions {}

#[derive(Codegen)]
#[codegen(tags = "simple-go", package = "simple")]
struct SimpleStruct {
    sstr: String,
    sint: i64,
}

#[derive(Codegen)]
#[codegen(tags = "simple-go", package = "simple")]
struct SimpleTupleStruct(u8, i128, isize);

#[derive(Codegen)]
#[codegen(tags = "simple-go", package = "simple")]
enum SimpleEnum {
    VUnit,
    /// awdji VUnit2 has docs
    VUnit2,
    VStr(String),
    VStr2(String),
    /// lkahal VNewTypeStruct has docs
    VNewTypeStruct(SimpleStruct),
    VTuple(String, i64),
    VTupleNested(String, (u8, i128, isize)),
    /// 90uw8d VStruct variant has docs
    VStruct { vfield: String },
}

pub(crate) fn run(options: SubOptions) {
    Generation::for_tag("simple-go")
        .as_arg_of(
            Command::new("deno")
                .arg("run")
                .arg("./golang-generator/generate-go.ts"),
        )
        .with_output_path("./golang-generator/example-gen")
        .write();
}
