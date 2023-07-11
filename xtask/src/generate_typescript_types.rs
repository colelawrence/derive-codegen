use derive_codegen::Generation;
use gumdrop::Options;
use std::path::PathBuf;
use std::process::Command;

#[derive(Options)]
pub(crate) struct SubOptions {
    #[options(help = "show help")]
    help: bool,
}

pub(crate) fn run(options: SubOptions) {
    if options.help {
        println!("{}", SubOptions::usage());
        return;
    }

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
