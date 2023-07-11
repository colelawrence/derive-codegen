#![allow(unused)]
use i_codegen_derive::CodegenInternal as Codegen;
use serde::{Deserialize, Serialize};

use crate::generate::Generation;

mod random_serde;

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
    insta::assert_snapshot!(Generation::for_tag("docs").to_input_json_pretty(), @r###"
    {
      "declarations": [
        {
          "id": "Basically",
          "id_location": "L(derive-codegen/src/test.rs:65 #B1839-B1848)",
          "rust_docs": "Test doc comment\nSecond line\n```sh\nSome code\n```\nReference to [BasicEnum].",
          "serde_attrs": {
            "rename": [
              "basically",
              "L(derive-codegen/src/test.rs:65 #B1792-B1803)"
            ]
          },
          "codegen_attrs": {
            "tags": [
              "docs",
              "L(derive-codegen/src/test.rs:65 #B1823-B1829)"
            ]
          },
          "container_kind": {
            "Struct": {
              "fields": [
                {
                  "id": "a",
                  "id_location": "L(derive-codegen/src/test.rs:65 #B1875-B1876)",
                  "rust_docs": "Doc comment",
                  "format": "USIZE"
                },
                {
                  "id": "b",
                  "id_location": "L(derive-codegen/src/test.rs:65 #B1889-B1890)",
                  "rust_docs": null,
                  "format": "Str"
                }
              ]
            }
          }
        },
        {
          "id": "BasicallyOther",
          "id_location": "L(derive-codegen/src/test.rs:74 #B1968-B1982)",
          "rust_docs": null,
          "codegen_attrs": {
            "tags": [
              "docs",
              "L(derive-codegen/src/test.rs:74 #B1952-B1958)"
            ]
          },
          "container_kind": {
            "Struct": {
              "fields": [
                {
                  "id": "usize_opt",
                  "id_location": "L(derive-codegen/src/test.rs:74 #B2074-B2083)",
                  "rust_docs": "Other option",
                  "serde_attrs": {
                    "alias": [
                      "usize",
                      "L(derive-codegen/src/test.rs:74 #B2060-B2067)"
                    ],
                    "rename": [
                      "usizeOpt",
                      "L(derive-codegen/src/test.rs:74 #B2027-B2037)"
                    ]
                  },
                  "format": {
                    "Option": "USIZE"
                  }
                },
                {
                  "id": "b",
                  "id_location": "L(derive-codegen/src/test.rs:74 #B2104-B2105)",
                  "rust_docs": null,
                  "format": "Str"
                }
              ]
            }
          }
        },
        {
          "id": "ActionResult",
          "id_location": "L(derive-codegen/src/test.rs:84 #B2183-B2195)",
          "rust_docs": null,
          "codegen_attrs": {
            "tags": [
              "docs",
              "L(derive-codegen/src/test.rs:84 #B2167-B2173)"
            ]
          },
          "container_kind": {
            "Struct": {
              "fields": [
                {
                  "id": "result",
                  "id_location": "L(derive-codegen/src/test.rs:84 #B2202-B2208)",
                  "rust_docs": null,
                  "format": {
                    "TypeName": "Result_OkBasicallyOther_ErrStr"
                  }
                }
              ]
            }
          }
        },
        {
          "id": "Result_OkBasicallyOther_ErrStr",
          "id_location": "L(derive-codegen/src/test.rs:84 #B2210-B2216)",
          "rust_docs": "`Result` is a type that represents either success ([`Ok`]) or failure ([`Err`]).",
          "container_kind": {
            "Enum": {
              "repr": "External",
              "variants": [
                {
                  "id": "Ok",
                  "id_location": "L(derive-codegen/src/test.rs:84 #B2210-B2216)",
                  "rust_docs": "Contains the success value",
                  "variant_format": {
                    "NewType": {
                      "TypeName": "BasicallyOther"
                    }
                  }
                },
                {
                  "id": "Err",
                  "id_location": "L(derive-codegen/src/test.rs:84 #B2210-B2216)",
                  "rust_docs": "Contains the error value",
                  "variant_format": {
                    "NewType": "Str"
                  }
                }
              ]
            }
          }
        }
      ]
    }
    "###);
}
