use clap::{self, Parser};
mod generate_golang_types_example;
mod generate_typescript_types;
mod print_internal_types;

#[derive(Parser)]
enum Args {
    // Command names are generated from variant names.
    // By default, a CamelCase name will be converted into a lowercase,
    // hyphen-separated name; e.g. `FooBar` becomes `foo-bar`.
    //
    // Names can be explicitly specified using `#[options(name = "...")]`
    /// generate internal typescript types
    GenerateTypescriptTypes(#[command(subcommand)] generate_typescript_types::SubOptions),
    /// generate golang types example
    GenerateGolangTypesExample(#[command(subcommand)] generate_golang_types_example::SubOptions),
    /// print the json of the internal types (this can be useful for testing your generators
    PrintInternalTypes(#[command(subcommand)] print_internal_types::SubOptions),
}

fn main() {
    let args = Args::parse();

    match args {
        Args::GenerateTypescriptTypes(sub) => generate_typescript_types::run(sub),
        Args::GenerateGolangTypesExample(sub) => generate_golang_types_example::run(sub),
        Args::PrintInternalTypes(sub) => print_internal_types::run(sub),
    }
}
