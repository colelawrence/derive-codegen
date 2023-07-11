pub use linkme;
use std::collections::BTreeSet;

#[cfg(feature = "experimental")]
use serde_reflection;

pub mod utils;
pub mod types;

pub struct Context {
    tags: BTreeSet<String>,
    #[cfg(feature = "experimental")]
    tracer: Option<(Vec<(String, types::TypeRoot)>, serde_reflection::Tracer)>,
    untraced: Vec<types::TypeRoot>,
    errors: Vec<String>,
}

impl Context {
    // pub fn trace_type_root<'de, T: serde::Deserialize<'de>>(
    pub fn add_type_root(
        &mut self,
        names_json: &str,
        file_name: &str,
        line: u32,
        tags: &[&str],
    ) -> () {
        if !self.should_include(tags) {
            return;
        }
        let type_root = self.create_type_root(names_json, file_name, line);
        self.untraced.push(type_root);
    }

    fn should_include(&self, tags: &[&str]) -> bool {
        if tags.is_empty() {
            if !self.tags.is_empty() {
                return false;
            }
        } else {
            let mut found = false;
            for tag in tags {
                if self.tags.contains(*tag) {
                    found = true;
                    continue;
                }
            }
            if !found {
                return false;
            }
        }
        return true;
    }

    fn create_type_root(
        &mut self,
        names_json: &str,
        file_name: &str,
        line: u32,
    ) -> types::TypeRoot {
        let mut type_root =
            serde_json::from_str::<types::TypeRoot>(names_json)
                .expect("Incompatible versions of generate & code");

        type_root.file = file_name.to_string();
        type_root.line = line;
        type_root
    }

    #[cfg(features = "experimental")]
    pub fn trace_type_root<T>(
        &mut self,
        names_json: &str,
        file_name: &str,
        line: u32,
        tags: &[&str],
    ) -> () {
        if !self.should_include(tags) {
            return;
        }
        let type_root = self.create_type_root(names_json, file_name, line);

        match &type_root.inner.value {
            types::ContainerFormat::Enum(_) => {
                for types::Spanned {
                    value: (key, _value),
                    ..
                } in type_root.inner.serde_attrs.iter()
                {
                    if &key.value == "tag" || &key.value == "content" {
                        // cannot be traced...
                        eprintln!("We can't trace enums with tag and content!");
                        return;
                    }
                }
            }
            _ => {}
        }

        if let Some((ref mut tracer, ref mut merge)) = self.tracer {
            type TODO = ();
            todo!("Trace simple enabled for serialize only?");
            match tracer.trace_simple_type::<TODO>() {
                Ok((serde_reflection::Format::TypeName(name), _samples)) => {
                    merge.push((name, type_root));
                }
                Ok((other_format, _samples)) => {
                    todo!("handle other format {other_format:?}\nFor type root:{type_root:#?}")
                }
                Err(err) => {
                    self.errors.push(format!("{err:#?}"));
                }
            }
        } else {
            self.untraced.push(type_root);
        }
    }
}

#[linkme::distributed_slice]
pub static CODEGEN_ITEMS: [fn(&mut Context)] = [..];

#[track_caller]
pub fn get_types_by_tags(tags: &[String]) -> Vec<types::TypeRoot> {
    let mut context = Context {
        tags: tags.into_iter().cloned().map(String::from).collect(),
        errors: Vec::new(),
        #[cfg(feature = "experimental")]
        tracer: None,
        // tracer: Some((Tracer::new(TracerConfig::default()), Vec::new())),
        untraced: Vec::new(),
    };
    {
        let mut context = &mut context;
        for gen in CODEGEN_ITEMS {
            gen(&mut context);
            if !context.errors.is_empty() {
                for err in &context.errors {
                    eprintln!("{err}");
                }
            }
            context.errors.clear();
        }
    }

    #[allow(unused_mut)]
    let Context {
        errors,
        untraced: mut type_roots,
        #[cfg(feature = "experimental")]
        tracer,
        tags,
    } = context;

    if !errors.is_empty() {
        eprintln!("Context trace errors for tags {tags:?}:");
        for err in errors {
            eprintln!(" * {err:?}");
        }
    }

    #[cfg(feature = "experimental")]
    if let Some((tracer, merge)) = tracer {
        let registry = tracer.registry().expect("constructing registry");
        eprintln!("{registry:#?}");
        type_roots.extend(types.into_iter().map(|(name, mut type_root)| {
            use types::{ContainerFormat, VariantFormat};
            use serde_reflection as sr;
            let format = registry.get(&name).expect("type exists in registry (if not, maybe alias unsupported)");
            match (&mut type_root.inner.value, format) {
                (ContainerFormat::Struct(ref mut named_formats), sr::ContainerFormat::Struct(ref reflected_named_formats)) => {
                    for named_format in named_formats.iter_mut() {
                        let serialize_name = named_format.serialize_name().to_string();
                        named_format.value.replace_incomplete(
                            reflected_named_formats.iter().find_map(|i| {
                                if &i.name == &serialize_name {
                                    Some(format_to_format(&i.value))
                                } else {
                                    None
                                }
                            }).expect("found matching struct item")
                        );
                    }
                },
                (ContainerFormat::Enum(ref mut enu), sr::ContainerFormat::Enum(ref reflected_enu)) => {
                    for (idx, ref mut enu_variant) in enu.iter_mut() {
                        let reflected_variant = reflected_enu.get(idx).expect("found matching enum new type variant");    
                        match (&mut enu_variant.value, &reflected_variant.value) {
                            (VariantFormat::Unit, sr::VariantFormat::Unit) => {
                                // nothing to replace
                            },
                            (VariantFormat::NewType(ref mut format), sr::VariantFormat::NewType(reflected)) => {
                                format.replace_incomplete(format_to_format(&reflected));
                            },
                            (VariantFormat::Tuple(ref mut formats), sr::VariantFormat::Tuple(reflected_formats)) => {
                                for (format, reflected) in formats.iter_mut().zip(reflected_formats.iter()) {
                                    format.replace_incomplete(format_to_format(&reflected));
                                }
                            },
                            (VariantFormat::Struct(ref mut named_formats), sr::VariantFormat::Struct(reflected_named_formats)) => {
                                for named_format in named_formats.iter_mut() {
                                    let serialize_name = named_format.serialize_name().to_string();
                                    named_format.value.replace_incomplete({
                                        reflected_named_formats.iter().find_map(|i| {
                                            if &i.name == &serialize_name {
                                                Some(format_to_format(&i.value))
                                            } else {
                                                None
                                            }
                                        }).expect("found matching struct item")
                                    });
                                }
                            },
                            (other, reflected_other) => {
                                panic!("Unknown enum combination {other:#?} VERSUS {reflected_other:?}");
                            }
                        };
                    }
                },
                (ContainerFormat::UnitStruct, sr::ContainerFormat::UnitStruct) => {
                    // nothing to fill in
                },
                (ContainerFormat::NewTypeStruct(format), sr::ContainerFormat::NewTypeStruct(reflected_format)) => {
                    format.replace_incomplete(format_to_format(&reflected_format));
                },
                (ContainerFormat::TupleStruct(formats), sr::ContainerFormat::TupleStruct(reflected_formats)) => {
                    for (ref mut format, reflected_format) in formats.iter_mut().zip(reflected_formats.iter()) {
                        format.replace_incomplete(format_to_format(&reflected_format));
                    }
                },
                (named_type, reflected_type) => {
                    panic!("Mismatch between containers (do we need to handle flatten or similar correctly?) {named_type:#?} VERSUS {reflected_type:#?}")
                }
            }
    
            type_root
        }));
    }

    type_roots
}

#[cfg(feature = "experimental")]
fn format_to_format(input: &serde_reflection::Format) -> types::Format {
    use types::Format as IFormat;
    use serde_reflection::Format as SFormat;
    match input {
        SFormat::Variable(_) => unreachable!(),
        SFormat::TypeName(name) => IFormat::TypeName(name.clone()),
        SFormat::Unit => IFormat::Unit,
        SFormat::Bool => IFormat::Bool,
        SFormat::I8 => IFormat::I8,
        SFormat::I16 => IFormat::I16,
        SFormat::I32 => IFormat::I32,
        SFormat::I64 => IFormat::I64,
        SFormat::I128 => IFormat::I128,
        SFormat::U8 => IFormat::U8,
        SFormat::U16 => IFormat::U16,
        SFormat::U32 => IFormat::U32,
        SFormat::U64 => IFormat::U64,
        SFormat::U128 => IFormat::U128,
        SFormat::F32 => IFormat::F32,
        SFormat::F64 => IFormat::F64,
        SFormat::Char => IFormat::Char,
        SFormat::Str => IFormat::Str,
        SFormat::Bytes => IFormat::Bytes,
        SFormat::Option(inner) => IFormat::Option(format_to_format(&inner).into()),
        SFormat::Seq(inner) => IFormat::Seq(format_to_format(&inner).into()),
        SFormat::Map { key, value } => IFormat::Map {
            key: format_to_format(&key).into(),
            value: format_to_format(&value).into(),
        },
        SFormat::Tuple(inner) => IFormat::Tuple(inner.iter().map(format_to_format).collect()),
        SFormat::TupleArray { content, size } => IFormat::TupleArray {
            content: format_to_format(&content).into(),
            size: *size,
        },
    }
}
