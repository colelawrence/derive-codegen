use derive_codegen::Generation;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, clap::Parser)]
pub(crate) struct SubOptions {}

pub(crate) fn run(_: SubOptions) {
    let project_root_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("getting manifest directory"))
            .parent()
            .unwrap()
            .to_owned();

    Generation::for_tag("derive-codegen-internal")
        .as_arg_of(
            Command::new("deno")
                .arg("run")
                .arg("./typescript-generator/generate-typescript.ts")
                .current_dir(&project_root_dir),
        )
        .with_output_path("./typescript-generator/codegen-types")
        .write();
}
