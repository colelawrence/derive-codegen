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
          "id_location": "L(src/test.rs:64 #B1834-B1843)",
          "rust_docs": "Test doc comment\nSecond line\n```sh\nSome code\n```\nReference to [BasicEnum].",
          "serde_attrs": {
            "rename": [
              "basically",
              "L(src/test.rs:64 #B1787-B1798)"
            ]
          },
          "codegen_attrs": {
            "tags": [
              "docs",
              "L(src/test.rs:64 #B1818-B1824)"
            ]
          },
          "container_kind": {
            "Struct": {
              "fields": [
                {
                  "id": "a",
                  "id_location": "L(src/test.rs:64 #B1870-B1871)",
                  "rust_docs": "Doc comment",
                  "format": "USIZE"
                },
                {
                  "id": "b",
                  "id_location": "L(src/test.rs:64 #B1884-B1885)",
                  "rust_docs": null,
                  "format": "Str"
                }
              ]
            }
          }
        },
        {
          "id": "BasicallyOther",
          "id_location": "L(src/test.rs:73 #B1963-B1977)",
          "rust_docs": null,
          "codegen_attrs": {
            "tags": [
              "docs",
              "L(src/test.rs:73 #B1947-B1953)"
            ]
          },
          "container_kind": {
            "Struct": {
              "fields": [
                {
                  "id": "usize_opt",
                  "id_location": "L(src/test.rs:73 #B2069-B2078)",
                  "rust_docs": "Other option",
                  "serde_attrs": {
                    "alias": [
                      "usize",
                      "L(src/test.rs:73 #B2055-B2062)"
                    ],
                    "rename": [
                      "usizeOpt",
                      "L(src/test.rs:73 #B2022-B2032)"
                    ]
                  },
                  "format": {
                    "Option": "USIZE"
                  }
                },
                {
                  "id": "b",
                  "id_location": "L(src/test.rs:73 #B2099-B2100)",
                  "rust_docs": null,
                  "format": "Str"
                }
              ]
            }
          }
        },
        {
          "id": "ActionResult",
          "id_location": "L(src/test.rs:83 #B2178-B2190)",
          "rust_docs": null,
          "codegen_attrs": {
            "tags": [
              "docs",
              "L(src/test.rs:83 #B2162-B2168)"
            ]
          },
          "container_kind": {
            "Struct": {
              "fields": [
                {
                  "id": "result",
                  "id_location": "L(src/test.rs:83 #B2197-B2203)",
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
          "id_location": "L(src/test.rs:83 #B2205-B2211)",
          "rust_docs": "`Result` is a type that represents either success ([`Ok`]) or failure ([`Err`]).",
          "container_kind": {
            "Enum": {
              "repr": "External",
              "variants": [
                {
                  "id": "Ok",
                  "id_location": "L(src/test.rs:83 #B2205-B2211)",
                  "rust_docs": "Contains the success value",
                  "variant_format": {
                    "NewType": {
                      "TypeName": "BasicallyOther"
                    }
                  }
                },
                {
                  "id": "Err",
                  "id_location": "L(src/test.rs:83 #B2205-B2211)",
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
