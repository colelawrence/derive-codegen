use std::{borrow::Cow, collections::BTreeMap, fmt::Debug};

use serde::{self, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TypeRoot {
    #[serde(rename = "f")]
    pub file: String,
    #[serde(rename = "l")]
    pub line: u32,
    #[serde(rename = "i")]
    pub inner: Named<RootItem>,
    /// e.g. built-in types
    #[serde(rename = "e")]
    pub extras: Vec<Named<ContainerFormat>>,
}

/// Containers (structs and enums) or functions (fns with `#[fn_codegen]`).
#[derive(Serialize, Deserialize, Debug)]
pub enum RootItem {
    Container(ContainerFormat),
    Function(FunctionFormat),
}

#[derive(Serialize, Deserialize)]
pub struct Spanned<T> {
    #[serde(rename = "$")]
    pub value: T,
    #[serde(rename = "_")]
    #[serde(skip_serializing_if = "is_null_bytes", default)]
    /// Location in file, byte offset
    pub bytes: (usize, usize),
}

fn is_null_bytes(value: &(usize, usize)) -> bool {
    value.0 == 0 && value.1 == 0
}

impl<T: Debug> Debug for Spanned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("#{:?} ", self.bytes))?;
        self.value.fmt(f)
    }
}

/// Serde-based serialization format for anonymous "value" types.
/// This is just the path respecting serde names into the container
/// It gets replaced by the knowledge
#[derive(Serialize, Deserialize, Debug)]
pub enum Format {
    Incomplete {
        debug: String,
    },
    /// The name of a container.
    TypeName {
        ident: String,
        generics: Vec<Format>,
    },

    // The formats of primitive types
    Unit,
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    ISIZE,
    U8,
    U16,
    U32,
    U64,
    U128,
    USIZE,
    F32,
    F64,
    Char,
    Str,
    Bytes,

    /// The format of `Option<T>`.
    Option(Box<Format>),
    /// Never actually instantiated
    Never,
    /// A sequence, e.g. the format of `Vec<Foo>`.
    Seq(Box<Format>),
    /// A map, e.g. the format of `BTreeMap<K, V>`.
    Map {
        key: Box<Format>,
        value: Box<Format>,
    },

    /// A tuple, e.g. the format of `(Foo, Bar)`.
    Tuple(Vec<Format>),
    /// Alias for `(Foo, ... Foo)`.
    /// E.g. the format of `[Foo; N]`.
    TupleArray {
        content: Box<Format>,
        size: usize,
    },
}

impl Format {
    pub fn is_primitive(&self) -> bool {
        match self {
            // The formats of primitive types
            Format::Unit
            | Format::Bool
            | Format::I8
            | Format::I16
            | Format::I32
            | Format::I64
            | Format::I128
            | Format::ISIZE
            | Format::U8
            | Format::U16
            | Format::U32
            | Format::U64
            | Format::U128
            | Format::USIZE
            | Format::F32
            | Format::F64
            | Format::Char
            | Format::Str
            | Format::Bytes => true,
            _ => false,
        }
    }
    pub fn is_typename(&self) -> Option<(&str, &[Format])> {
        match self {
            Format::TypeName { ident, generics } => Some((&ident, &generics)),
            _ => None,
        }
    }
    pub fn as_ident(&self) -> Cow<'static, str> {
        Cow::Borrowed(match self {
            Format::Incomplete { debug } => todo!("Unknown ident incomplete: {debug}"),
            Format::TypeName { ident, generics } => {
                return Cow::Owned({
                    let mut buf = format!("{ident}");
                    for gen in generics.iter() {
                        buf.push('_');
                        buf.push_str(&gen.as_ident());
                    }
                    buf
                })
            }
            Format::Unit => "Nil",
            Format::Bool => "Bool",
            Format::I8 => "I8",
            Format::I16 => "I16",
            Format::I32 => "I32",
            Format::I64 => "I64",
            Format::I128 => "I128",
            Format::ISIZE => "ISIZE",
            Format::U8 => "U8",
            Format::U16 => "U16",
            Format::U32 => "U32",
            Format::U64 => "U64",
            Format::U128 => "U128",
            Format::USIZE => "USIZE",
            Format::F32 => "F32",
            Format::F64 => "F64",
            Format::Char => "Char",
            Format::Str => "Str",
            Format::Bytes => "Bytes",
            Format::Option(of) => return Cow::Owned(format!("{}_Option", of.as_ident())),
            Format::Never => "Never",
            Format::Seq(of) => return Cow::Owned(format!("{}_List", of.as_ident())),
            Format::Map { key, value } => {
                return Cow::Owned(format!("{}_{}_Map", key.as_ident(), value.as_ident()))
            }
            Format::Tuple(of) => {
                return Cow::Owned(format!(
                    "{}Tuple",
                    of.iter()
                        .flat_map(|v| [v.as_ident(), Cow::Borrowed("_")])
                        .collect::<String>()
                ))
            }
            Format::TupleArray { content, size } => {
                return Cow::Owned(format!("{}_{}_TupleOf", content.as_ident(), size))
            }
        })
    }
    pub fn replace_incomplete(&mut self, replacement: Format) {
        if let Format::Incomplete { .. } = self {
            *self = replacement;
            return;
        }
        if self.is_primitive() || self.is_typename().is_some() {
            return;
        }
        match (self, replacement) {
            (Format::Option(ref mut original), Format::Option(replacement)) => {
                original.replace_incomplete(*replacement);
            }
            (Format::Seq(ref mut original), Format::Seq(replacement)) => {
                original.replace_incomplete(*replacement);
            }
            (
                Format::Map {
                    ref mut key,
                    ref mut value,
                },
                Format::Map {
                    key: replace_key,
                    value: replace_value,
                },
            ) => {
                key.replace_incomplete(*replace_key);
                value.replace_incomplete(*replace_value);
            }
            (Format::Tuple(ref mut original), Format::Tuple(replacement_vec)) => {
                for (original_item, replacement) in original.iter_mut().zip(replacement_vec) {
                    original_item.replace_incomplete(replacement);
                }
            }
            (
                Format::TupleArray {
                    ref mut content, ..
                },
                Format::TupleArray {
                    content: replacement_content,
                    ..
                },
            ) => {
                content.replace_incomplete(*replacement_content);
            }
            (original, replacement) => {
                panic!("Failed to merge original and replacement:\n{original:#?}\nREPLACEMENT\n{replacement:#?}")
            }
        }
    }
}

/// Serde-based serialization format for named "container" types.
/// In Rust, those are enums and structs.
#[derive(Serialize, Deserialize, Debug)]
pub enum ContainerFormat {
    /// An empty struct, e.g. `struct A`.
    UnitStruct,
    /// A struct with a single unnamed parameter, e.g. `struct A(u16)`
    NewTypeStruct(Box<Format>),
    /// A struct with several unnamed parameters, e.g. `struct A(u16, u32)`
    TupleStruct(Vec<Format>),
    /// A struct with named parameters, e.g. `struct A { a: Foo }`.
    Struct(Vec<Named<Format>>),
    /// An enum, that is, an enumeration of variants.
    /// Each variant has a unique name and index within the enum.
    Enum(BTreeMap<u32, Named<VariantFormat>>),
}

#[derive(Serialize, Deserialize, Debug)]
/// Description of a variant in an enum.
pub enum VariantFormat {
    /// A variant without parameters, e.g. `A` in `enum X { A }`
    Unit,
    /// A variant with a single unnamed parameter, e.g. `A` in `enum X { A(u16) }`
    NewType(Box<Format>),
    /// A struct with several unnamed parameters, e.g. `A` in `enum X { A(u16, u32) }`
    Tuple(Vec<Format>),
    /// A struct with named parameters, e.g. `A` in `enum X { A { a: Foo } }`
    Struct(Vec<Named<Format>>),
}

/// Free standing function item such as `fn start() -> ()`.
#[derive(Serialize, Deserialize, Debug)]
pub struct FunctionFormat {
    pub is_async: bool,
    pub self_opt: Option<Named<Format>>,
    pub params: Vec<Named<Format>>,
    pub ret: Box<Format>,
}

#[derive(Serialize, Deserialize, Debug)]
/// A named value.
/// Used for named parameters or variants.
pub struct Named<T> {
    #[serde(rename = "id")]
    pub rust_ident: Spanned<String>,
    #[serde(rename = "gn")]
    pub rust_generics: Vec<Spanned<String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[serde(rename = "docs")]
    pub rust_docs: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[serde(rename = "sa")]
    pub serde_attrs: Vec<Spanned<(Spanned<String>, Spanned<String>)>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[serde(rename = "sf")]
    pub serde_flags: Vec<Spanned<String>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[serde(rename = "ca")]
    pub codegen_attrs: Vec<Spanned<(Spanned<String>, Spanned<String>)>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[serde(rename = "cf")]
    pub codegen_flags: Vec<Spanned<String>>,
    #[serde(rename = "$")]
    pub value: T,
}

impl<T> Named<T> {
    pub fn builtin(ident: &str, docs: &str, bytes: Option<(usize, usize)>, value: T) -> Self {
        Named {
            rust_ident: Spanned {
                value: ident.to_string(),
                bytes: bytes.unwrap_or_default(),
            },
            rust_generics: Vec::new(),
            rust_docs: {
                let docs = docs.trim();
                if docs.is_empty() {
                    None
                } else {
                    Some(docs.to_string())
                }
            },
            serde_attrs: Vec::new(),
            serde_flags: Vec::new(),
            codegen_attrs: Vec::new(),
            codegen_flags: Vec::new(),
            value,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EnumRepresentation {
    /// The default
    /// e.g `{ User: { id: 1200, name: "Smithy" } }`
    External,
    /// e.g `{ id: 1200, name: "Smithy" }`
    Untagged,
    /// e.g `{ type: "User", id: 1200, name: "Smithy" }`
    /// e.g `{ type: "User", content: { id: 1200, name: "Smithy" } }`
    Tagged {
        tag: Spanned<String>,
        content: Option<Spanned<String>>,
    },
}

impl<T> Named<T> {
    pub fn serialize_name(&self) -> &str {
        self.serde_attrs
            .iter()
            .filter_map(|attr| {
                if attr.value.0.value == "rename" {
                    Some(&attr.value.1.value)
                } else {
                    None
                }
            })
            .last()
            .unwrap_or(&self.rust_ident.value)
    }
}
