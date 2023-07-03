

use i_codegen_derive::CodegenInternal as Codegen;
use serde::{Deserialize, Serialize};

use crate::generate::Generation;

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
          "id_location": "L(src/test.rs:65 #B1822-B1831)",
          "rust_docs": "Test doc comment\nSecond line\n```sh\nSome code\n```\nReference to [BasicEnum].",
          "serde_attrs": {
            "rename": [
              "basically",
              "L(src/test.rs:65 #B1775-B1786)"
            ]
          },
          "codegen_attrs": {
            "tags": [
              "docs",
              "L(src/test.rs:65 #B1806-B1812)"
            ]
          },
          "container_kind": {
            "Struct": {
              "fields": [
                {
                  "id": "a",
                  "id_location": "L(src/test.rs:65 #B1858-B1859)",
                  "rust_docs": "Doc comment",
                  "format": "USIZE"
                },
                {
                  "id": "b",
                  "id_location": "L(src/test.rs:65 #B1872-B1873)",
                  "rust_docs": null,
                  "format": "Str"
                }
              ]
            }
          }
        },
        {
          "id": "BasicallyOther",
          "id_location": "L(src/test.rs:74 #B1951-B1965)",
          "rust_docs": null,
          "codegen_attrs": {
            "tags": [
              "docs",
              "L(src/test.rs:74 #B1935-B1941)"
            ]
          },
          "container_kind": {
            "Struct": {
              "fields": [
                {
                  "id": "usize_opt",
                  "id_location": "L(src/test.rs:74 #B2057-B2066)",
                  "rust_docs": "Other option",
                  "serde_attrs": {
                    "alias": [
                      "usize",
                      "L(src/test.rs:74 #B2043-B2050)"
                    ],
                    "rename": [
                      "usizeOpt",
                      "L(src/test.rs:74 #B2010-B2020)"
                    ]
                  },
                  "format": {
                    "Option": "USIZE"
                  }
                },
                {
                  "id": "b",
                  "id_location": "L(src/test.rs:74 #B2087-B2088)",
                  "rust_docs": null,
                  "format": "Str"
                }
              ]
            }
          }
        },
        {
          "id": "ActionResult",
          "id_location": "L(src/test.rs:84 #B2166-B2178)",
          "rust_docs": null,
          "codegen_attrs": {
            "tags": [
              "docs",
              "L(src/test.rs:84 #B2150-B2156)"
            ]
          },
          "container_kind": {
            "Struct": {
              "fields": [
                {
                  "id": "result",
                  "id_location": "L(src/test.rs:84 #B2185-B2191)",
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
          "id_location": "L(src/test.rs:84 #B2193-B2199)",
          "rust_docs": "`Result` is a type that represents either success ([`Ok`]) or failure ([`Err`]).",
          "container_kind": {
            "Enum": {
              "repr": "External",
              "variants": [
                {
                  "id": "Ok",
                  "id_location": "L(src/test.rs:84 #B2193-B2199)",
                  "rust_docs": "Contains the success value",
                  "variant_format": {
                    "NewType": {
                      "TypeName": "BasicallyOther"
                    }
                  }
                },
                {
                  "id": "Err",
                  "id_location": "L(src/test.rs:84 #B2193-B2199)",
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
