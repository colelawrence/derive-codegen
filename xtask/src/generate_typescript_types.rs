use derive_codegen::Generation;
use gumdrop::Options;
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

    Generation::for_tag("derive-codegen-internal")
        .as_arg_of(
            Command::new("deno")
                .arg("run")
                .arg("./typescript-generator/generate-typescript.ts"),
        )
        .with_output_path("./typescript-generator/codegen-types")
        .write();
}
