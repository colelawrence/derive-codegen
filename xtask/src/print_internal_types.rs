use derive_codegen::Generation;

#[derive(Debug, clap::Parser)]
pub(crate) struct SubOptions {
    /// pretty json
    pretty: bool,
}

pub(crate) fn run(opts: SubOptions) {
    let selection = Generation::for_tag("derive-codegen-internal");
    println!(
        "{}",
        if opts.pretty {
            selection.to_input_json_pretty()
        } else {
            selection.to_input_json()
        }
    );
}
