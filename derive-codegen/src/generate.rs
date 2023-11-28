use i_codegen_code::types as st;
use i_codegen_derive::CodegenInternal;
use rayon::prelude::*;
use serde::{self, Deserialize, Serialize};
use st::TypeRoot;

use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Debug,
    io::BufReader,
};

#[derive(CodegenInternal, Serialize, Deserialize, Clone, Debug)]
#[serde(transparent)]
#[codegen(tags = "derive-codegen-internal")]
struct LocationID(String);

#[derive(Serialize, Debug, CodegenInternal)]
#[codegen(tags = "derive-codegen-internal")]
struct Input {
    declarations: Vec<InputDeclaration>,
    functions: Vec<FunctionDeclaration>,
}

#[derive(Serialize, Debug, CodegenInternal)]
#[codegen(tags = "derive-codegen-internal")]
struct InputDeclaration {
    id: String,
    id_location: LocationID,
    /// Contains generics, docs, and `[codegen]` attr information.
    #[serde(flatten)]
    attrs: Attrs,
    container_kind: ContainerFormat,
}

#[derive(Serialize, Debug, CodegenInternal)]
#[codegen(tags = "derive-codegen-internal")]
struct FunctionDeclaration {
    id: String,
    id_location: LocationID,
    /// Contains generics, docs, and `[codegen]` attr information.
    #[serde(flatten)]
    attrs: Attrs,
    function: FunctionFormat,
}

#[derive(Serialize, Debug, CodegenInternal)]
#[codegen(tags = "derive-codegen-internal")]
struct FunctionFormat {
    /// Whether this function was declared with async
    is_async: bool,
    self_opt: Option<Box<FunctionParameter>>,
    params: Vec<FunctionParameter>,
    return_type: Box<Format>,
}

#[derive(Deserialize, Debug, CodegenInternal)]
#[codegen(tags = "derive-codegen-internal")]
struct Output {
    errors: Vec<OutputMessage>,
    warnings: Vec<OutputMessage>,
    files: Vec<OutputFile>,
}

#[derive(Deserialize, Debug, CodegenInternal)]
#[codegen(tags = "derive-codegen-internal")]
struct OutputFile {
    /// Example: `./some-dir/filename.txt`
    path: String,
    /// Example: `"Hello world"`
    source: String,
}

#[derive(Serialize, Deserialize, Debug, CodegenInternal)]
#[codegen(tags = "derive-codegen-internal")]
struct OutputMessage {
    message: String,
    /// Labelled spans
    labels: Vec<(String, LocationID)>,
}

/// Serde-based serialization format for anonymous "value" types.
/// This is just the path respecting serde names into the container
/// It gets replaced by the knowledge
#[derive(Serialize, Debug, CodegenInternal)]
#[codegen(tags = "derive-codegen-internal")]
enum Format {
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

/// Serde-based serialization format for named "container" types.
/// In Rust, those are enums and structs.
#[derive(Serialize, Debug, CodegenInternal)]
#[codegen(tags = "derive-codegen-internal")]
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

#[derive(Serialize, Debug, CodegenInternal)]
#[codegen(tags = "derive-codegen-internal")]
struct NamedVariant {
    id: String,
    id_location: LocationID,
    #[serde(flatten)]
    attrs: Attrs,
    variant_format: VariantFormat,
}

#[derive(Serialize, Debug, CodegenInternal)]
#[codegen(tags = "derive-codegen-internal")]
struct NamedField {
    id: String,
    id_location: LocationID,
    #[serde(flatten)]
    attrs: Attrs,
    format: Format,
}

#[derive(Serialize, Debug, CodegenInternal)]
#[codegen(tags = "derive-codegen-internal")]
struct FunctionParameter {
    id: String,
    id_location: LocationID,
    #[serde(flatten)]
    attrs: Attrs,
    format: Format,
}

#[derive(Serialize, Debug, CodegenInternal)]
#[codegen(tags = "derive-codegen-internal")]
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

#[derive(Serialize, Debug, CodegenInternal)]
#[codegen(tags = "derive-codegen-internal")]
struct Attrs {
    /// Documentation comments like this one.
    /// Future idea: Pass in tokens with links to other types.
    rust_docs: Option<String>,
    /// Only specified for enums and structs
    /// Future: Consider whether we should monomorphize on the codegen side...
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    rust_generics: Vec<(String, LocationID)>,
    /// e.g. `#[serde(rename = "newName")]`, your generator will need to describe what it supports
    /// Not applicable to derived functions.
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    serde_attrs: BTreeMap<String, (String, LocationID)>,
    /// e.g. `#[serde(transparent)]`, your generator will need to describe what it supports
    /// Not applicable to derived functions.
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    serde_flags: BTreeMap<String, LocationID>,
    /// e.g. `#[codegen(ts_as = "Date")]` - these are customizable for your generator's use cases.
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    codegen_attrs: BTreeMap<String, (String, LocationID)>,
    /// e.g. `#[codegen(hidden)]` - these are customizable for your generator's use cases.
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    codegen_flags: BTreeMap<String, LocationID>,
}

#[derive(Serialize, Debug, CodegenInternal)]
#[codegen(tags = "derive-codegen-internal")]
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
struct SourceLineNumberIndex {
    newlines: Vec<usize>,
    // TODO: is this crlf useful for something?
    #[allow(unused)]
    is_crlf: bool,
}

impl SourceLineNumberIndex {
    fn new(file: impl std::io::Read) -> Self {
        use std::io::Read;
        let mut newlines = Vec::new();
        let mut is_crlf = false;

        let mut current_byte = 0;
        for byte_result in BufReader::new(file).bytes() {
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

        Self { is_crlf, newlines }
    }

    fn get_ln_col(&self, byte: usize) -> (usize, usize) {
        let index = match self.newlines.binary_search(&byte) {
            Ok(exact_at) => exact_at,
            Err(insert_at) => insert_at - 1,
        };
        if index >= self.newlines.len() || index == 0 {
            (index, 0)
        } else {
            let newline_byte_offset = *self.newlines.get(index).expect("in bounds");
            if byte < newline_byte_offset {
                panic!("expected newline returned to be before byte (byte: {byte} < newline_at: {newline_byte_offset}) found at index: {index}, all new lines: {:?}", self.newlines)
            }
            (index + 1, byte - newline_byte_offset)
        }
    }
}

#[derive(Clone)]
struct TypeRootConverter {
    file_name: String,
    line_number_override: Option<u32>,
    lines: SourceLineNumberIndex,
}

impl TypeRootConverter {
    fn get_ln_col(&self, byte: usize) -> (usize, usize) {
        self.lines.get_ln_col(byte)
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
            rust_generics,
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
                rust_generics: rust_generics
                    .into_iter()
                    .map(|gen| self.location_id(gen))
                    .collect(),
                serde_attrs: {
                    let mut bt = BTreeMap::<String, (String, LocationID)>::new();
                    for st::Spanned {
                        bytes: _,
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
                        bytes: _,
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
            st::Format::TypeName { ident, generics } => Format::TypeName {
                ident,
                generics: generics
                    .into_iter()
                    .map(|format| self.format_to_format(format))
                    .collect(),
            },
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

    fn function_format_to_function_format(
        &self,
        function_format: st::FunctionFormat,
    ) -> FunctionFormat {
        let st::FunctionFormat {
            params: args,
            is_async,
            ret,
            self_opt,
        } = function_format;
        FunctionFormat {
            params: args
                .into_iter()
                .map(|st_named_format| self.named_format_to_function_parameter(st_named_format))
                .collect(),
            is_async,
            self_opt: self_opt
                .map(|selff| Box::new(self.named_format_to_function_parameter(selff))),
            return_type: Box::new(self.format_to_format(*ret)),
        }
    }

    fn named_format_to_named_field(&self, named: st::Named<st::Format>) -> NamedField {
        let (id_span, format, attrs) = self.unname(named);
        let (id, id_location) = self.location_id(id_span);
        NamedField {
            attrs,
            format: self.format_to_format(format),
            id,
            id_location,
        }
    }
    fn named_format_to_function_parameter(
        &self,
        named: st::Named<st::Format>,
    ) -> FunctionParameter {
        let (id_span, format, attrs) = self.unname(named);
        let (id, id_location) = self.location_id(id_span);
        FunctionParameter {
            attrs,
            format: self.format_to_format(format),
            id,
            id_location,
        }
    }
    fn container_format_to_container_format(
        &self,
        // needed to see serde attrs for ensuring correct interpretation
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
                        .map(|field| self.named_format_to_named_field(field))
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

enum GenCommand<'a> {
    PipeInto(&'a mut std::process::Command),
    Arg(&'a mut std::process::Command),
}

#[derive(Clone)]
pub struct Generation {
    tags: Vec<String>,
}

pub struct GenerationCmd<'a> {
    relative_to: Option<PathBuf>,
    selection: &'a Generation,
    command: GenCommand<'a>,
    output_path: Option<PathBuf>,
}

impl Generation {
    pub fn for_tag(tag: &str) -> Self {
        Generation {
            tags: vec![tag.to_string()],
        }
    }

    pub fn include_tag(&mut self, tag: impl Into<String>) -> &mut Self {
        self.tags.push(tag.into());
        self
    }

    pub fn pipe_into<'a>(&'a self, command: &'a mut Command) -> GenerationCmd<'a> {
        GenerationCmd {
            relative_to: command.get_current_dir().map(|dir| dir.to_owned()),
            command: GenCommand::PipeInto(command),
            output_path: None,
            selection: self,
        }
    }

    pub fn as_arg_of<'a>(&'a self, command: &'a mut Command) -> GenerationCmd<'a> {
        GenerationCmd {
            relative_to: command.get_current_dir().map(|dir| dir.to_owned()),
            command: GenCommand::Arg(command),
            output_path: None,
            selection: self,
        }
    }

    pub fn to_input_json_pretty(&self) -> String {
        serde_json::to_string_pretty(&create_input_from_selection(self)).unwrap()
    }

    pub fn to_input_json(&self) -> String {
        serde_json::to_string(&create_input_from_selection(self)).unwrap()
    }
}

#[derive(Debug)]
pub struct GenerationSummary {
    /// Relative to this folder
    pub relative_to: PathBuf,
    /// Paths and their sizes
    pub output_files: Vec<(String, usize)>,
}

impl GenerationSummary {
    pub fn print(self) -> Self {
        eprintln!("{self:?}");
        self
    }
}

impl<'a> GenerationCmd<'a> {
    /// Relative to current directory of teh command passed in
    pub fn with_output_path<P: Into<PathBuf>>(&mut self, path: P) -> &mut Self {
        self.output_path = Some(path.into());
        self
    }

    fn get_output_path(&self) -> PathBuf {
        if let Some(ref rel) = self.relative_to {
            rel.join(self.output_path.clone().unwrap_or_else(|| ".".into()))
        } else {
            self.output_path.clone().unwrap_or_else(|| ".".into())
        }
    }

    #[track_caller]
    pub fn print(&mut self) {
        let output = self.generate();
        for err in &output.warnings {
            eprintln!("Output warning:\n{err:?}")
        }
        for err in &output.errors {
            eprintln!("Output error:\n{err:?}")
        }

        let relative_to = self.get_output_path();
        for output_file in output.files {
            let write_path = relative_to.join(output_file.path);
            println!("\x1b[48:2:255:165:0m{}\x1b[0m", write_path.display());
            println!("{}", output_file.source);
        }
    }

    #[track_caller]
    pub fn write(&mut self) -> GenerationSummary {
        let output = self.generate();
        for err in output.warnings.iter() {
            eprintln!("Output warning:\n{err:?}")
        }
        for err in output.errors.iter() {
            eprintln!("Output error:\n{err:?}")
        }

        let relative_to = self.get_output_path();
        let mut summary = GenerationSummary {
            relative_to: relative_to.clone(),
            output_files: Vec::new(),
        };
        for output_file in output.files.iter() {
            let write_path = relative_to.join(&output_file.path);
            if let Some(parent) = write_path.parent() {
                std::fs::create_dir_all(parent).expect("creating missing directories");
            }

            let mut file = std::fs::File::create(write_path).expect("creating file");
            write!(&mut file, "{}", output_file.source).expect("writing generated file");
            summary
                .output_files
                .push((output_file.path.to_string(), output_file.source.len()));
        }

        summary
    }

    #[track_caller]
    fn generate(&mut self) -> Output {
        let inputs = create_input_from_selection(self.selection);
        let stdout_output = match self.command {
            GenCommand::PipeInto(ref mut cmd) => {
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

                child.wait_with_output().expect("failed to wait on child")
            }
            GenCommand::Arg(ref mut cmd) => {
                let cmd_str = format!("{cmd:?} <input-json>");
                let child = cmd
                    .arg(&serde_json::to_string(&inputs).unwrap())
                    .stdout(std::process::Stdio::piped())
                    .spawn()
                    .map_err(|err| format!("Failure executing `{cmd_str}`: {err:?} "))
                    .expect("spawning process");

                child.wait_with_output().expect("failed to wait on child")
            }
        };

        let stdout_str = String::from_utf8_lossy(&stdout_output.stdout);

        if !stdout_output.status.success() {
            eprintln!("Non-success status from generation");
            if !stdout_str.trim().is_empty() {
                eprintln!("Stdout:\n{stdout_str}");
            }
            std::process::exit(1);
        }

        serde_json::from_str::<Output>(&stdout_str)
            .map_err(|err| {
                format!(
                    "Failed to parsed output as JSON of output files: {err:?}, from:\n{stdout_str}"
                )
            })
            .expect("parsing output")
    }
}

fn create_input_from_selection(selection: &Generation) -> Input {
    let tys = i_codegen_code::get_types_by_tags(&selection.tags);
    let current_directory = std::env::current_dir()
        .expect("getting current directory in order to find source files for line number mapping");
    let type_root_converters: HashMap<String, TypeRootConverter> = tys
        .iter()
        .map(|root| root.file.clone())
        .collect::<HashSet<_>>()
        .into_par_iter()
        .map(|file_name| {
            let file = {
                match std::fs::File::open(&file_name) {
                    Ok(found) => found,
                    Err(_) => {
                        // try going up on directory... hacky...
                        match std::fs::File::open(
                            &current_directory.parent().unwrap().join(&file_name),
                        ) {
                            Ok(file) => file,
                            Err(_err) => {
                                // eprintln!("opening file {file_name:?} (relative to: {current_directory:?}): {_err:?}");
                                return (
                                    file_name.clone(),
                                    TypeRootConverter {
                                        file_name,
                                        line_number_override: None,
                                        lines: SourceLineNumberIndex {
                                            newlines: vec![0usize],
                                            is_crlf: false,
                                        },
                                    },
                                );
                            }
                        }
                    }
                }
            };

            (
                file_name.clone(),
                TypeRootConverter {
                    file_name,
                    line_number_override: None,
                    lines: SourceLineNumberIndex::new(file),
                },
            )
        })
        .collect();

    let mut functions = Vec::new();
    let mut declarations = Vec::<InputDeclaration>::new();
    for TypeRoot {
        extras,
        file,
        line,
        inner,
    } in tys
    {
        let mut converter = type_root_converters.get(&file).unwrap().clone();
        converter.line_number_override = Some(line);
        let (root_id_span, root_item, attrs) = converter.unname(inner);
        let (id, id_location) = converter.location_id(root_id_span);
        match root_item {
            st::RootItem::Container(container_format) => {
                declarations.push(InputDeclaration {
                    id,
                    id_location,
                    container_kind: converter
                        .container_format_to_container_format(&attrs, container_format),
                    attrs,
                });
            }
            st::RootItem::Function(function_format) => {
                functions.push(FunctionDeclaration {
                    id,
                    id_location,
                    function: converter.function_format_to_function_format(function_format),
                    attrs,
                });
            }
        }
        // extra declarations like built-ins
        for extra in extras {
            let (id_span, container_format, attrs) = converter.unname(extra);
            let (id, id_location) = converter.location_id(id_span);
            declarations.push(InputDeclaration {
                id,
                id_location,
                container_kind: converter
                    .container_format_to_container_format(&attrs, container_format),
                attrs,
            });
        }
    }

    Input {
        declarations,
        functions,
    }
}
