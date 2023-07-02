use i_herenow_serde_generate_types as st;
use rayon::prelude::*;
use serde::{self, Deserialize, Serialize};
use st::TypeRoot;
use std::cell::{Ref, RefCell};
use std::io::Write;
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fmt::Debug,
    io::BufReader,
};

#[derive(Codegen, Serialize, Deserialize, Clone, Debug)]
#[serde(transparent)]
#[codegen(tags = "herenow-generator-internal")]
struct LocationID(String);

#[derive(Serialize, Debug, Codegen)]
#[codegen(tags = "herenow-generator-internal")]
struct Input {
    declarations: Vec<InputDeclaration>,
}

#[derive(Serialize, Debug, Codegen)]
#[codegen(tags = "herenow-generator-internal")]
struct InputDeclaration {
    id: String,
    id_location: LocationID,
    #[serde(flatten)]
    attrs: Attrs,
    container_kind: ContainerFormat,
}

#[derive(Deserialize, Debug, Codegen)]
#[codegen(tags = "herenow-generator-internal")]
struct Output {
    errors: Vec<OutputMessage>,
    warnings: Vec<OutputMessage>,
    files: Vec<OutputFile>,
}

#[derive(Deserialize, Debug, Codegen)]
#[codegen(tags = "herenow-generator-internal")]
struct OutputFile {
    path: String,
    source: String,
}

#[derive(Serialize, Deserialize, Debug, Codegen)]
#[codegen(tags = "herenow-generator-internal")]
struct OutputMessage {
    message: String,
    /// Labelled spans
    labels: Vec<(String, LocationID)>,
}

/// Serde-based serialization format for anonymous "value" types.
/// This is just the path respecting serde names into the container
/// It gets replaced by the knowledge
#[derive(Serialize, Debug, Codegen)]
#[codegen(tags = "herenow-generator-internal")]
enum Format {
    Incomplete {
        debug: String,
    },
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

/// Serde-based serialization format for named "container" types.
/// In Rust, those are enums and structs.
#[derive(Serialize, Debug, Codegen)]
#[codegen(tags = "herenow-generator-internal")]
enum ContainerFormat {
    /// An empty struct, e.g. `struct A`.
    UnitStruct,
    /// A struct with a single unnamed parameter, e.g. `struct A(u16)`
    NewTypeStruct(Box<Format>),
    /// A struct with several unnamed parameters, e.g. `struct A(u16, u32)`
    TupleStruct(Vec<Format>),
    /// A struct with named parameters, e.g. `struct A { a: Foo }`.
    Struct { fields: Vec<NamedField> },
    /// An enum, that is, an enumeration of variants.
    /// Each variant has a unique name and index within the enum.
    Enum {
        repr: EnumRepresentation,
        variants: Vec<NamedVariant>,
    },
}

#[derive(Serialize, Debug, Codegen)]
#[codegen(tags = "herenow-generator-internal")]
struct NamedVariant {
    id: String,
    id_location: LocationID,
    #[serde(flatten)]
    attrs: Attrs,
    variant_format: VariantFormat,
}

#[derive(Serialize, Debug, Codegen)]
#[codegen(tags = "herenow-generator-internal")]
struct NamedField {
    id: String,
    id_location: LocationID,
    #[serde(flatten)]
    attrs: Attrs,
    format: Format,
}

#[derive(Serialize, Debug, Codegen)]
#[codegen(tags = "herenow-generator-internal")]
/// Description of a variant in an enum.
enum VariantFormat {
    /// A variant without parameters, e.g. `A` in `enum X { A }`
    Unit,
    /// A variant with a single unnamed parameter, e.g. `A` in `enum X { A(u16) }`
    NewType(Box<Format>),
    /// A struct with several unnamed parameters, e.g. `A` in `enum X { A(u16, u32) }`
    Tuple(Vec<Format>),
    /// A struct with named parameters, e.g. `A` in `enum X { A { a: Foo } }`
    Struct { fields: Vec<NamedField> },
}

#[derive(Serialize, Debug, Codegen)]
#[codegen(tags = "herenow-generator-internal")]
struct Attrs {
    /// Documentation comments like this one.
    /// Future idea: Pass in tokens with links to other types.
    rust_docs: Option<String>,
    /// e.g. `#[serde(rename = "newName")]`, your generator will need to describe what it supports
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    serde_attrs: BTreeMap<String, (String, LocationID)>,
    /// e.g. `#[serde(transparent)]`, your generator will need to describe what it supports
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    serde_flags: BTreeMap<String, LocationID>,
    /// e.g. `#[codegen(ts_as = "Date")]` - these are customizable for your generator's use cases.
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    codegen_attrs: BTreeMap<String, (String, LocationID)>,
    /// e.g. `#[codegen(hidden)]` - these are customizable for your generator's use cases.
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    codegen_flags: BTreeMap<String, LocationID>,
}

#[derive(Serialize, Debug, Codegen)]
#[codegen(tags = "herenow-generator-internal")]
enum EnumRepresentation {
    /// The default
    /// e.g `{ User: { id: 1200, name: "Smithy" } }`
    External,
    /// e.g `{ id: 1200, name: "Smithy" }`
    Untagged,
    /// e.g `{ type: "User", id: 1200, name: "Smithy" }`
    /// e.g `{ type: "User", content: { id: 1200, name: "Smithy" } }`
    Tagged {
        tag: String,
        tag_location: LocationID,
        content: Option<String>,
        content_location: Option<LocationID>,
    },
}

#[derive(Clone)]
struct TypeRootConverter {
    file_name: String,
    line_number_override: Option<u32>,
    newlines: Vec<usize>,
}
impl TypeRootConverter {
    fn get_ln_col(&self, byte: usize) -> (usize, usize) {
        let index = match self.newlines.binary_search(&byte) {
            Ok(exact_at) => exact_at,
            Err(insert_at) => insert_at - 1,
        };
        if index >= self.newlines.len() || index == 0 {
            return (index, 0);
        } else {
            let newline_byte_offset = *self.newlines.get(index - 1).expect("in bounds");
            if byte < newline_byte_offset {
                panic!("expected newline returned to be before byte (byte: {byte} < newline_at: {newline_byte_offset}) found at index: {index}, all new lines: {:?}", self.newlines)
            }
            (index + 1, byte - newline_byte_offset)
        }
    }
    fn location_id<T>(
        &self,
        st::Spanned {
            bytes: (start, end),
            value,
        }: st::Spanned<T>,
    ) -> (T, LocationID) {
        if let Some(line) = self.line_number_override {
            (
                value,
                LocationID(format!(
                    "L({}:{} #B{}-B{})",
                    &self.file_name, line, start, end
                )),
            )
        } else {
            let (ln, col) = self.get_ln_col(start);
            (
                value,
                LocationID(format!(
                    "L({}:{}:{} #B{}-B{})",
                    &self.file_name, ln, col, start, end
                )),
            )
        }
    }
    fn unname<T>(
        &self,
        st::Named {
            codegen_attrs,
            codegen_flags,
            rust_docs,
            rust_ident,
            serde_attrs,
            serde_flags,
            value,
        }: st::Named<T>,
    ) -> (st::Spanned<String>, T, Attrs) {
        (
            rust_ident,
            value,
            Attrs {
                rust_docs,
                serde_attrs: {
                    let mut bt = BTreeMap::<String, (String, LocationID)>::new();
                    for st::Spanned {
                        bytes,
                        value: (key, value),
                    } in serde_attrs
                    {
                        bt.insert(key.value, self.location_id(value));
                    }
                    bt
                },
                serde_flags: {
                    let mut bt = BTreeMap::<String, LocationID>::new();
                    for flag_span in serde_flags {
                        let (a, b) = self.location_id(flag_span);
                        bt.insert(a, b);
                    }
                    bt
                },
                codegen_attrs: {
                    let mut bt = BTreeMap::<String, (String, LocationID)>::new();
                    for st::Spanned {
                        bytes,
                        value: (key, value),
                    } in codegen_attrs
                    {
                        bt.insert(key.value, self.location_id(value));
                    }
                    bt
                },
                codegen_flags: {
                    let mut bt = BTreeMap::<String, LocationID>::new();
                    for flag_span in codegen_flags {
                        let (a, b) = self.location_id(flag_span);
                        bt.insert(a, b);
                    }
                    bt
                },
            },
        )
    }
    fn format_to_format(&self, format: st::Format) -> Format {
        match format {
            st::Format::Incomplete { debug } => Format::Incomplete { debug },
            st::Format::TypeName(name) => Format::TypeName(name),
            st::Format::Unit => Format::Unit,
            st::Format::Bool => Format::Bool,
            st::Format::I8 => Format::I8,
            st::Format::I16 => Format::I16,
            st::Format::I32 => Format::I32,
            st::Format::I64 => Format::I64,
            st::Format::I128 => Format::I128,
            st::Format::ISIZE => Format::ISIZE,
            st::Format::U8 => Format::U8,
            st::Format::U16 => Format::U16,
            st::Format::U32 => Format::U32,
            st::Format::U64 => Format::U64,
            st::Format::U128 => Format::U128,
            st::Format::USIZE => Format::USIZE,
            st::Format::F32 => Format::F32,
            st::Format::F64 => Format::F64,
            st::Format::Char => Format::Char,
            st::Format::Str => Format::Str,
            st::Format::Bytes => Format::Bytes,
            st::Format::Option(option_format) => {
                Format::Option(Box::new(self.format_to_format(*option_format)))
            }
            st::Format::Never => Format::Never,
            st::Format::Seq(seq_format) => {
                Format::Seq(Box::new(self.format_to_format(*seq_format)))
            }
            st::Format::Map { key, value } => Format::Map {
                key: Box::new(self.format_to_format(*key)),
                value: Box::new(self.format_to_format(*value)),
            },
            st::Format::Tuple(tuple_formats) => Format::Tuple(
                tuple_formats
                    .into_iter()
                    .map(|format| self.format_to_format(format))
                    .collect(),
            ),
            st::Format::TupleArray { content, size } => Format::TupleArray {
                content: Box::new(self.format_to_format(*content)),
                size,
            },
        }
    }

    fn container_format_to_container_format(
        &self,
        attrs: &Attrs,
        container_format: st::ContainerFormat,
    ) -> ContainerFormat {
        match container_format {
            st::ContainerFormat::UnitStruct => ContainerFormat::UnitStruct,
            st::ContainerFormat::NewTypeStruct(format) => {
                ContainerFormat::NewTypeStruct(Box::new(self.format_to_format(*format)))
            }
            st::ContainerFormat::TupleStruct(formats) => ContainerFormat::TupleStruct(
                formats
                    .into_iter()
                    .map(|format| self.format_to_format(format))
                    .collect(),
            ),
            st::ContainerFormat::Struct(fields) => ContainerFormat::Struct {
                fields: {
                    fields
                        .into_par_iter()
                        .map(|field| {
                            let (id_span, format, attrs) = self.unname(field);
                            let (id, id_location) = self.location_id(id_span);
                            NamedField {
                                attrs,
                                format: self.format_to_format(format),
                                id,
                                id_location,
                            }
                        })
                        .collect()
                },
            },
            st::ContainerFormat::Enum(variants) => ContainerFormat::Enum {
                repr: {
                    if attrs.serde_flags.contains_key("untagged") {
                        EnumRepresentation::Untagged
                    } else {
                        match (
                            attrs.serde_attrs.get("tag").cloned(),
                            attrs.serde_attrs.get("content").cloned(),
                        ) {
                            (Some((tag, tag_location)), Some((content, content_location))) => {
                                EnumRepresentation::Tagged {
                                    tag,
                                    tag_location,
                                    content: Some(content),
                                    content_location: Some(content_location),
                                }
                            }
                            (Some((tag, tag_location)), None) => EnumRepresentation::Tagged {
                                tag,
                                tag_location,
                                content: None,
                                content_location: None,
                            },
                            (None, None) => EnumRepresentation::External,
                            (None, Some(_)) => {
                                // hmm...
                                EnumRepresentation::External
                            }
                        }
                    }
                },
                variants: {
                    variants
                        .into_par_iter()
                        .map(|(_index, named_variant_format)| {
                            let (id_span, variant_format, attrs) =
                                self.unname(named_variant_format);
                            let (id, id_location) = self.location_id(id_span);
                            let variant_format = match variant_format {
                                st::VariantFormat::Unit => VariantFormat::Unit,
                                st::VariantFormat::NewType(format) => {
                                    VariantFormat::NewType(Box::new(self.format_to_format(*format)))
                                }
                                st::VariantFormat::Tuple(formats) => VariantFormat::Tuple(
                                    formats
                                        .into_iter()
                                        .map(|format| self.format_to_format(format))
                                        .collect(),
                                ),
                                st::VariantFormat::Struct(fields) => VariantFormat::Struct {
                                    fields: fields
                                        .into_iter()
                                        .map(|field| {
                                            let (id_span, format, attrs) = self.unname(field);
                                            let (id, id_location) = self.location_id(id_span);
                                            NamedField {
                                                attrs,
                                                format: self.format_to_format(format),
                                                id,
                                                id_location,
                                            }
                                        })
                                        .collect(),
                                },
                            };
                            NamedVariant {
                                id,
                                id_location,
                                attrs,
                                variant_format,
                            }
                        })
                        .collect()
                },
            },
        }
    }
}
pub enum GenerationCommand<'a> {
    PipeInto(&'a mut std::process::Command),
    Arg(&'a mut std::process::Command),
}

#[track_caller]
pub fn generate_for_tag(tag: &str, command: GenerationCommand<'_>) {
    let tys = i_herenow_serde_generate_code::get_types_by_tag(tag);
    let inputs = crate::generate::create_input_json_from_type_roots(tys);
    match command {
        GenerationCommand::PipeInto(mut cmd) => {
            let cmd_str = format!("{cmd:?}");
            let mut child = cmd
                .stdout(std::process::Stdio::piped())
                .stdin(std::process::Stdio::piped())
                .spawn()
                .map_err(|err| format!("Failure executing `{cmd_str}`: {err:?} "))
                .expect("spawning process");

            child
                .stdin
                .as_mut()
                .unwrap()
                .write_all(&serde_json::to_vec(&inputs).unwrap())
                .unwrap();

            let output = child.wait_with_output().expect("failed to wait on child");

            dbg!(output);
        }
        GenerationCommand::Arg(mut cmd) => {
            let cmd_str = format!("{cmd:?} <input-json>");
            let mut child = cmd
                .arg(&serde_json::to_string(&inputs).unwrap())
                .stdout(std::process::Stdio::piped())
                .spawn()
                .map_err(|err| format!("Failure executing `{cmd_str}`: {err:?} "))
                .expect("spawning process");

            let stdout_output = child.wait_with_output().expect("failed to wait on child");
            let stdout_str = String::from_utf8_lossy(&stdout_output.stdout);
            let output = serde_json::from_str::<Output>(&stdout_str)
                .map_err(|err| {
                    format!("Failed to parsed output as JSON of output files: {err:?}, from:\n{stdout_str}")
                })
                .expect("parsing output");
            
            for err in output.errors {
                eprintln!("Output error:\n{err:?}")
            }

            for OutputFile { path, source } in output.files {
                write!(&mut std::fs::File::create(path).expect("open file from output"), "{}", source);
            }
        }
    }
}

pub fn create_input_json_from_type_roots(roots: Vec<st::TypeRoot>) -> serde_json::Value {
    let type_root_converters: HashMap<String, TypeRootConverter> = roots
        .iter()
        .map(|root| root.file.clone())
        .collect::<HashSet<_>>()
        .into_par_iter()
        .map(|file_name| {
            use std::io::Read;
            let mut is_crlf = false;
            let mut newlines = vec![0usize];
            let mut current_byte = 0usize;

            for byte_result in
                BufReader::new(std::fs::File::open(&file_name).expect("opened file")).bytes()
            {
                match byte_result.expect("read next byte") {
                    b'\n' => {
                        newlines.push(current_byte + 1);
                    }
                    b'\r' => {
                        is_crlf = true;
                    }
                    _ => {}
                }
                current_byte += 1;
            }
            (
                file_name.clone(),
                TypeRootConverter {
                    file_name,
                    newlines,
                    line_number_override: None,
                },
            )
        })
        .collect();

    let declarations = roots
        .into_par_iter()
        .flat_map(
            |TypeRoot {
                 extras,
                 file,
                 line,
                 inner,
             }| {
                std::iter::once(inner)
                    .chain(extras)
                    .map(|named_container| {
                        let mut type_root_converter =
                            type_root_converters.get(&file).unwrap().clone();
                        type_root_converter.line_number_override = Some(line);
                        (type_root_converter, named_container)
                    })
                    .collect::<Vec<_>>()
            },
        )
        .map(|(converter, named_container)| {
            let (id_span, container_format, attrs) = converter.unname(named_container);
            let (id, id_location) = converter.location_id(id_span);
            InputDeclaration {
                id,
                id_location,
                container_kind: converter
                    .container_format_to_container_format(&attrs, container_format),
                attrs,
            }
        })
        .collect::<Vec<InputDeclaration>>();

    serde_json::to_value(&Input { declarations }).unwrap()
}
