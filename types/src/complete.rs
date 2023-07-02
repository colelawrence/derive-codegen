use std::collections::BTreeMap;

use serde::{self, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TypeRoot {
    pub file: String,
    pub inner: Named<ContainerFormat>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpanStr {
    pub text: String,
    /// Location in file, byte offset
    pub bytes: (usize, usize),
}

/// Serde-based serialization format for anonymous "value" types.
/// This is just the path respecting serde names into the container
/// It gets replaced by the knowledge
#[derive(Serialize, Deserialize, Debug)]
pub enum Format {
    /// A format whose value is initially unknown. Used internally for tracing. Not (de)serializable.
    Variable(#[serde(with = "not_implemented")] Variable<Format>),
    /// The name of a container.
    TypeName(String),

    // The formats of primitive types
    Unit,
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Char,
    Str,
    Bytes,

    /// The format of `Option<T>`.
    Option(Box<Format>),
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

#[derive(Serialize, Deserialize, Debug)]
/// A named value.
/// Used for named parameters or variants.
pub struct Named<T> {
    pub name: SpanStr,
    pub rename: Option<String>,
    pub docs: Option<String>,
    pub value: T,
}
