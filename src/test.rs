use std::process::Command;

use i_herenow_serde_generate_code::get_types_by_tag;
use i_herenow_serde_generate_derive::Codegen;
use serde::{Deserialize, Serialize};

use crate::generate::{generate_for_tag, GenerationCommand};

#[derive(Codegen, Deserialize)]
#[codegen(tags = "fsharp")]
struct UnitType;

#[derive(Codegen, Deserialize)]
#[codegen(tags = "fsharp")]
struct Basic {
    a: i32,
    b: String,
}

#[derive(Codegen, Deserialize)]
#[codegen(tags = "fsharp")]
struct BasicNest {
    id: String,
    b: Basic,
}

#[derive(Codegen, Deserialize)]
#[codegen(tags = "fsharp")]
#[serde(tag = "apple", content = "content")]
enum BasicEnum {
    #[serde(rename = "aRename")]
    A(usize),
    B(String),
    C(BasicNest),
    D {
        da: String,
        db: usize,
        dc: Option<usize>,
    },
}

/// These renames don't actually affect the generation.
/// See [Feature request: Allow `#[serde(rename = "...")]` on tuple struct / tuple variant member fields #1510](https://github.com/serde-rs/serde/issues/1510)
#[derive(Codegen, Serialize, Deserialize, Debug, Eq, Clone, PartialEq)]
struct ATuplePartiallyNamed(#[serde(rename = "x")] usize, #[serde(rename = "y")] usize);

#[cfg(test)]
mod test_sers {
    use super::ATuplePartiallyNamed;

    #[test]
    fn test_tuple_partially_named() {
        // insta::assert_display_snapshot!(
        //     serde_json::to_string(&ATuplePartiallyNamed(20, 400)).unwrap(),
        //     ""
        // )
    }
}

/// Test doc comment
/// Second line
/// ```sh
/// Some code
/// ```
/// Reference to [BasicEnum].
#[derive(Codegen, Deserialize)]
#[serde(rename = "basically")]
#[codegen(tags = "docs")]
struct Basically {
    /// Doc comment
    a: usize,
    b: String,
}

#[derive(Codegen, Deserialize)]
#[codegen(tags = "docs")]
struct BasicallyOther {
    /// Other option
    #[serde(rename = "usizeOpt")]
    #[serde(alias = "usize")]
    usize_opt: Option<usize>,
    b: String,
}

#[derive(Codegen, Deserialize)]
#[codegen(tags = "docs")]
struct ActionResult {
    result: Result<BasicallyOther, String>,
}

#[test]
fn test_generate() {
    let tys = get_types_by_tag("docs");
    let inputs = crate::generate::create_input_json_from_type_roots(tys);
    generate_for_tag(
        "herenow-generator-internal",
        GenerationCommand::Arg(
            Command::new("deno")
                .arg("run")
                .arg("./generate-typescript.ts"),
        ),
    );
    panic!("Testing");
    // insta::assert_snapshot!(serde_json::to_string_pretty(&inputs).unwrap());
}
