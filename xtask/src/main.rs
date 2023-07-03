use gumdrop::Options;
mod generate_golang_types_example;
mod generate_typescript_types;
mod print_internal_types;

#[derive(Options)]
enum Command {
    // Command names are generated from variant names.
    // By default, a CamelCase name will be converted into a lowercase,
    // hyphen-separated name; e.g. `FooBar` becomes `foo-bar`.
    //
    // Names can be explicitly specified using `#[options(name = "...")]`
    #[options(help = "generate internal typescript types")]
    GenerateTypescriptTypes(generate_typescript_types::SubOptions),
    #[options(help = "generate golang types example")]
    GenerateGolangTypesExample(generate_golang_types_example::SubOptions),
    #[options(
        help = "print the json of the internal types (this can be useful for testing your generators)"
    )]
    PrintInternalTypes(print_internal_types::SubOptions),
}

// Define options for the program.
#[derive(Options)]
struct MyOptions {
    // Options here can be accepted with any command (or none at all),
    // but they must come before the command name.
    #[options(help = "print help message")]
    help: bool,
    // #[options(help = "be verbose")]
    // verbose: bool,

    // The `command` option will delegate option parsing to the command type,
    // starting at the first free argument.
    #[options(command)]
    command: Option<Command>,
}

fn main() {
    let opts = MyOptions::parse_args_default_or_exit();
    if opts.help {
        println!("{}", opts.self_usage());
        std::process::exit(0);
    }
    let command = if let Some(command) = opts.command {
        command
    } else {
        eprintln!("Sub-command required\n\n{}", opts.self_usage());
        std::process::exit(1);
    };

    match command {
        Command::GenerateTypescriptTypes(sub) => generate_typescript_types::run(sub),
        Command::GenerateGolangTypesExample(sub) => generate_golang_types_example::run(sub),
        Command::PrintInternalTypes(sub) => print_internal_types::run(sub),
    }
}
