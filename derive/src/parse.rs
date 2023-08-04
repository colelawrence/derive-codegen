use i_codegen_code::{types as st, utils};
use std::collections::{BTreeMap, HashMap};

// use crate::attr;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Ident, Result};

pub enum DerivationKind {
    Internal,
    External { crate_name: &'static str },
}

/// see [i_codegen_code::Context]
pub fn derive(input: DeriveInput, kind: DerivationKind) -> Result<TokenStream> {
    let ident: &Ident = &input.ident;
    let dummy = Ident::new(
        &format!("_IMPL_HERENOW_GENERATE_FOR_{}", ident),
        Span::call_site(),
    );

    let ctxt = Ctxt::new();

    let container = ast::Container::from_ast(&ctxt, &input, Derive::Serialize)
        .expect("container was derived from AST");

    // Hmmm
    // // must track this in case of errors so we can check them
    // // if we don't consume the errors, we'll get an "unhandled errors" panic whether or not there were errors
    // ctxt.check().unwrap();

    let mut pctxt = ParseContext {
        ctxt: Some(ctxt),
        ident: ident.clone(),
        publish_builtins: Default::default(),
    };

    let container_format = match container.data {
        ast::Data::Enum(ref variants) => pctxt.derive_enum(variants, &container),
        ast::Data::Struct(style, ref fields) => pctxt.derive_struct(style, fields, &container),
    };

    let mut root = st::TypeRoot {
        file: "unknown".to_string(),
        line: 0,
        inner: pctxt.derive_named(container_format, ident, &input.attrs, Some(&container)),
        extras: Vec::new(),
    };

    for (_builtin_id, named_container_format) in pctxt.publish_builtins.drain() {
        root.extras.push(named_container_format);
    }

    let type_root_json: String = serde_json::to_string(&root).expect("serialize type root");
    let type_root_json_lit = syn::LitStr::new(&type_root_json, container.ident.span());

    let q_tags = root
        .inner
        .codegen_attrs
        .iter()
        .flat_map(|attr| {
            if attr.value.0.value == "tags" {
                Some(attr.value.1.value.split(',').into_iter().map(String::from))
            } else {
                None
            }
        })
        .flatten()
        .map(|attr| syn::LitStr::new(&attr, Span::call_site()));

    // This will give a rust analyzer warning because of https://github.com/rust-lang/rust-analyzer/issues/6541
    let i_codegen_code_crate_q = match kind {
        DerivationKind::Internal => Ident::new("i_codegen_code", Span::call_site()),
        DerivationKind::External { crate_name } => Ident::new(&crate_name, Span::call_site()),
    };

    Ok(quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        #[allow(non_snake_case)]
        #[::#i_codegen_code_crate_q::linkme::distributed_slice(::#i_codegen_code_crate_q::CODEGEN_ITEMS)]
        #[linkme(crate = ::#i_codegen_code_crate_q::linkme)]
        fn #dummy(context: &mut ::#i_codegen_code_crate_q::Context) {
            context.add_type_root(#type_root_json_lit, file!(), line!(), &[#(#q_tags,)*]);
        };
    })
}

fn get_doc_comments(attrs: &[syn::Attribute]) -> Option<String> {
    let docs = attrs
        .iter()
        .filter_map(|attr| {
            if let ("doc", syn::Meta::NameValue(name_value)) =
                (path_to_string(&attr.path()).as_str(), &attr.meta)
            {
                if let syn::Expr::Lit(syn::ExprLit {
                    attrs: _,
                    lit: syn::Lit::Str(s),
                }) = &name_value.value
                {
                    Some(s.value())
                } else {
                    eprintln!("Unexpected {:?}", name_value);
                    None
                }
            } else {
                None
            }
        })
        .filter_map(|doc_value| {
            let text = doc_value
                .trim_start_matches("//!")
                .trim_start_matches("///")
                .trim_start_matches("/*!")
                .trim_start_matches("/**")
                .trim_end_matches("*/")
                .trim();
            if text.is_empty() {
                None
            } else {
                Some(text.to_string())
            }
        })
        .collect::<Vec<_>>();

    if docs.is_empty() {
        None
    } else {
        Some(docs.join("\n"))
    }
}

#[inline]
fn path_to_string(path: &syn::Path) -> String {
    quote!(#path).to_string()
}

use serde_derive_internals::{ast, Ctxt, Derive};

fn field_type_name(ty: &syn::Type) -> Option<String> {
    use syn::Type::Path;
    match ty {
        Path(syn::TypePath { path, .. }) => match path.segments.last() {
            Some(t) => Some(t.ident.to_string()),
            _ => None,
        },
        _ => None,
    }
}

fn is_phantom(ty: &syn::Type) -> bool {
    match field_type_name(ty) {
        Some(t) => t == "PhantomData",
        _ => false,
    }
}

fn filter_visible<'a>(fields: &'a [ast::Field<'a>]) -> Vec<&'a ast::Field<'a>> {
    let mut content: Vec<&'a ast::Field<'a>> = Vec::with_capacity(fields.len());

    for field in fields {
        if field.attrs.skip_serializing() || is_phantom(field.ty) {
            continue;
        }

        content.push(field);
    }
    content
}

impl<'a> ParseContext {
    pub(crate) fn derive_enum(
        &mut self,
        variants: &[ast::Variant<'a>],
        container: &ast::Container,
    ) -> st::ContainerFormat {
        let mut map = BTreeMap::<u32, st::Named<st::VariantFormat>>::new();
        for (idx, variant) in variants.iter().enumerate() {
            let inner: st::VariantFormat = match variant.style {
                ast::Style::Struct => st::VariantFormat::Struct(
                    self.derive_named_fields_alt(&variant.fields).collect(),
                ),
                ast::Style::Tuple => st::VariantFormat::Tuple(
                    self.derive_fields_tuple_alt(&variant.fields).collect(),
                ),
                ast::Style::Newtype => {
                    st::VariantFormat::NewType(Box::new(self.field_to_format(&variant.fields[0])))
                }
                ast::Style::Unit => st::VariantFormat::Unit,
            };
            map.insert(
                idx as u32,
                self.derive_named(
                    inner,
                    &variant.ident,
                    &variant.original.attrs,
                    Some(container),
                ),
            );
        }
        st::ContainerFormat::Enum(map)
    }

    pub(crate) fn derive_struct(
        &mut self,
        style: ast::Style,
        fields: &[ast::Field<'a>],
        container: &ast::Container,
    ) -> st::ContainerFormat {
        match style {
            ast::Style::Struct => self.derive_struct_named_fields(fields, container),
            ast::Style::Newtype => self.derive_struct_newtype(&fields[0], container),
            ast::Style::Tuple => self.derive_struct_tuple(fields, container),
            ast::Style::Unit => self.derive_struct_unit(),
        }
    }

    fn derive_struct_newtype(
        &mut self,
        field: &ast::Field<'a>,
        _container: &ast::Container,
    ) -> st::ContainerFormat {
        if field.attrs.skip_serializing() {
            return self.derive_struct_unit();
        }

        st::ContainerFormat::NewTypeStruct(Box::new(self.field_to_format(field)))
    }

    fn derive_struct_unit(&self) -> st::ContainerFormat {
        st::ContainerFormat::UnitStruct
    }

    fn derive_struct_named_fields(
        &mut self,
        fields: &[ast::Field<'a>],
        _container: &ast::Container,
    ) -> st::ContainerFormat {
        let fields = filter_visible(fields);
        st::ContainerFormat::Struct(self.derive_named_fields(&fields).collect())
    }

    fn derive_struct_tuple(
        &mut self,
        fields: &[ast::Field<'a>],
        ast_container: &ast::Container,
    ) -> st::ContainerFormat {
        let fields = filter_visible(fields);
        if fields.is_empty() {
            return self.derive_struct_unit();
        }

        if fields.len() == 1 && ast_container.attrs.transparent() {
            return self.derive_struct_newtype(&fields[0], ast_container);
        };

        st::ContainerFormat::TupleStruct(self.derive_fields_tuple(&fields).collect())
    }

    fn derive_named<T>(
        &self,
        value: T,
        ident: &syn::Ident,
        syn_attrs: &[syn::Attribute],
        container: Option<&ast::Container>,
        // serde_name: &serde_derive_internals::attr::Name,
    ) -> st::Named<T> {
        let ident_str = ident.to_string();
        let mut named = st::Named {
            rust_ident: spanned(&[ident.span()], ident_str),
            rust_generics: container
                .map(|c| {
                    c.generics
                        .params
                        .iter()
                        .filter_map(|g| match g {
                            syn::GenericParam::Lifetime(_) => None,
                            syn::GenericParam::Const(_) => None,
                            syn::GenericParam::Type(typ) => {
                                Some(spanned(&[typ.ident.span()], typ.ident.to_string()))
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            rust_docs: get_doc_comments(syn_attrs),
            codegen_attrs: Vec::new(),
            codegen_flags: Vec::new(),
            serde_attrs: Vec::new(),
            serde_flags: Vec::new(),
            value,
        };
        for attr in syn_attrs.iter() {
            if attr.path().is_ident("serde") {
                // // #[repr(align(N))]
                // attr.parse_nested_meta(|meta| {
                //     let content;
                //     syn::parenthesized!(content in meta.input);
                //     let lit: syn::LitInt = content.parse()?;
                //     let n: usize = lit.base10_parse()?;
                //     repr_align = Some(n);
                //     return Ok(());
                // });
                attr.parse_nested_meta(|meta| {
                    let span = meta.input.span();
                    match meta.value() {
                        Ok(value) => {
                            let lit: syn::LitStr = value.parse()?;
                            named.serde_attrs.push(spanned(
                                &[span],
                                (
                                    spanned(&[meta.input.span()], path_to_string(&meta.path)),
                                    spanned(&[lit.span()], lit.value()),
                                ),
                            ))
                        }
                        Err(_other) => {
                            // not a value, assume it's a flag, then?
                            named
                                .serde_flags
                                .push(spanned(&[meta.input.span()], path_to_string(&meta.path)))
                        }
                    }
                    // // #[serde(rename = "new_name")]
                    // if meta.path.is_ident("rename") {
                    //     let value = meta.value()?;
                    //     let lit: syn::LitStr = value.parse()?;
                    //     named.rename = Some(spanned(lit.value(), &[lit.span(), span]))
                    // }
                    // // #[serde(flatten)]
                    // else if meta.path.is_ident("flatten") {
                    //     named.flatten = Some(spanned(true, &[span]))
                    // } else {
                    // }
                    return Ok(());
                })
                .expect("parsed serde attribute");
            } else if attr.path().is_ident("codegen") {
                attr.parse_nested_meta(|meta| {
                    let span = meta.input.span();
                    match meta.value() {
                        Ok(value) => {
                            let lit: syn::LitStr = value.parse()?;
                            named.codegen_attrs.push(spanned(
                                &[span],
                                (
                                    spanned(&[meta.input.span()], path_to_string(&meta.path)),
                                    spanned(&[lit.span()], lit.value()),
                                ),
                            ))
                        }
                        Err(_other) => {
                            // not a value, assume it's a flag, then?
                            named
                                .codegen_flags
                                .push(spanned(&[meta.input.span()], path_to_string(&meta.path)))
                        }
                    }
                    return Ok(());
                })
                .expect("parsed codegen attribute");
            }
        }
        named
    }
}

fn spanned<T>(spans: &[proc_macro2::Span], value: T) -> st::Spanned<T> {
    st::Spanned {
        bytes: spans
            .into_iter()
            .copied()
            .filter_map(|span| {
                let span = format!("{span:?}");
                match utils::parse_span(&span) {
                    Ok(found) => Some(found),
                    Err(_err) => {
                        // eprintln!("Failed creating bytes for spanned. {_err}");
                        None
                    }
                }
            })
            // first
            .next()
            .unwrap_or_default(),
        value,
    }
}

pub(crate) struct ParseContext {
    ctxt: Option<serde_derive_internals::Ctxt>, // serde parse context for error reporting
    #[allow(unused)]
    ident: syn::Ident,      // name of enum struct
    /// Extras to publish like "Duration"
    publish_builtins: HashMap<String, st::Named<st::ContainerFormat>>,
}

impl Drop for ParseContext {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            // must track this in case of errors so we can check them
            // if we don't consume the errors, we'll get an "unhandled errors" panic whether or not there were errors
            if let Some(ctxt) = self.ctxt.take() {
                ctxt.check().expect("no errors")
            }
        }
    }
}

impl<'a> ParseContext {
    // Some helpers

    // fn err_msg<A: quote::ToTokens>(&self, tokens: A, msg: &str) {
    //     if let Some(ref ctxt) = self.ctxt {
    //         ctxt.error_spanned_by(tokens, msg);
    //     } else {
    //         panic!("{}", msg)
    //     }
    // }

    // fn check_flatten(&self, fields: &[&'a ast::Field<'a>], ast_container: &ast::Container) -> bool {
    //     let has_flatten = fields.iter().any(|f| f.attrs.flatten()); // .any(|f| f);
    //     if has_flatten {
    //         self.err_msg(
    //             &self.ident,
    //             &format!(
    //                 "{}: #[serde(flatten)] does not work for derive-codegen.",
    //                 ast_container.ident
    //             ),
    //         );
    //     };
    //     has_flatten
    // }

    fn field_to_format(&mut self, field: &ast::Field<'a>) -> st::Format {
        self.type_to_format(field.ty)
    }

    fn type_to_format(&mut self, ty: &syn::Type) -> st::Format {
        // `type_to_format` recursively calls itself occationally
        // finding a Path which it hands to last_path_element
        // which generates a "simplified" TSType struct which
        // is handed to `generic_to_ts` which possibly "bottoms out"
        // by generating tokens for typescript types.

        use syn::Type as SynType;
        use syn::{
            TypeArray, TypeGroup, TypeParen, TypePath, TypePtr, TypeReference, TypeSlice, TypeTuple,
        };
        match ty {
            SynType::Slice(TypeSlice { elem, .. })
            | SynType::Array(TypeArray { elem, .. })
            | SynType::Ptr(TypePtr { elem, .. }) => self.type_to_seq(elem),
            SynType::Reference(TypeReference { elem, .. }) => self.type_to_format(elem),
            // // fn(a: A,b: B, c:C) -> D
            // BareFn(TypeBareFn { inputs, .. }) => {
            //     self.ctxt
            //         .err_msg(inputs, "we do not support functions");
            //     quote!(any)
            // }
            SynType::Never(..) => st::Format::Never,
            SynType::Tuple(TypeTuple { elems, .. }) => {
                let elems = elems.iter().map(|t| self.type_to_format(t));
                st::Format::Tuple(elems.collect())
            }
            SynType::Path(TypePath { path, .. }) => match last_path_element(&path) {
                Some(ref ts) => self.generic_to_format(ts),
                _ => st::Format::Incomplete {
                    debug: format!("Unknown type path: {path:?}"),
                },
            },
            // TraitObject(TypeTraitObject { bounds, .. })
            // | ImplTrait(TypeImplTrait { bounds, .. }) => {
            //     let elems = bounds
            //         .iter()
            //         .filter_map(|t| match t {
            //             TypeParamBound::Trait(t) => last_path_element(&t.path),
            //             _ => None, // skip lifetime etc.
            //         })
            //         .map(|t| self.generic_to_format(&t));

            //     // TODO check for zero length?
            //     // A + B + C => A & B & C
            //     quote!(#(#elems)&*)
            // }
            SynType::Paren(TypeParen { elem, .. }) | SynType::Group(TypeGroup { elem, .. }) => {
                self.type_to_format(elem)
            }
            SynType::Infer(..) | SynType::Macro(..) | SynType::Verbatim(..) => {
                st::Format::Incomplete {
                    debug: format!("unknown-type: {ty:?}"),
                }
            }
            // Recommended way to test exhaustiveness without breaking API
            #[allow(unknown_lints)]
            #[cfg_attr(test, deny(non_exhaustive_omitted_patterns))]
            _ => st::Format::Incomplete {
                debug: format!("Unknown other type"),
            },
        }
    }

    #[allow(clippy::cognitive_complexity)]
    fn generic_to_format(&mut self, ts: &TypeFormat) -> st::Format {
        let mut to_format = |ty: &syn::Type| self.type_to_format(ty);
        let name = ts.ident.to_string();
        match name.as_ref() {
            "i8" => st::Format::I8,
            "i16" => st::Format::I16,
            "i32" => st::Format::I32,
            "i64" => st::Format::I64,
            "i128" => st::Format::I128,
            "isize" => st::Format::ISIZE,
            "u8" => st::Format::U8,
            "u16" => st::Format::U16,
            "u32" => st::Format::U32,
            "u64" => st::Format::U64,
            "u128" => st::Format::U128,
            "usize" => st::Format::USIZE,
            "f32" => st::Format::F32,
            "f64" => st::Format::F64,
            "char" => st::Format::Char,
            "String" | "str" | "Path" | "PathBuf" => st::Format::Str,
            "bool" => st::Format::Bool,
            "Box" | "Cow" | "Rc" | "Arc" | "Cell" | "RefCell" if ts.args.len() == 1 => {
                to_format(&ts.args[0])
            }
            "Duration" => self.add_builtin::<std::time::Duration>(
                "Duration",
                r#"
A `Duration` type to represent a span of time, typically used for system
timeouts.

Each `Duration` is composed of a whole number of seconds and a fractional part
represented in nanoseconds. If the underlying system does not support
nanosecond-level precision, APIs binding a system timeout will typically round up
the number of nanoseconds.

[`Duration`]s implement many common traits, including [`Add`], [`Sub`], and other
[`ops`] traits. It implements [`Default`] by returning a zero-length `Duration`.

[`ops`]: crate::ops

# Examples

```
use std::time::Duration;

let five_seconds = Duration::new(5, 0);
let five_seconds_and_five_nanos = five_seconds + Duration::new(0, 5);

assert_eq!(five_seconds_and_five_nanos.as_secs(), 5);
assert_eq!(five_seconds_and_five_nanos.subsec_nanos(), 5);

let ten_millis = Duration::from_millis(10);
```

# Formatting `Duration` values

`Duration` intentionally does not have a `Display` impl, as there are a
variety of ways to format spans of time for human readability. `Duration`
provides a `Debug` impl that shows the full precision of the value.

The `Debug` output uses the non-ASCII "µs" suffix for microseconds. If your
program output may appear in contexts that cannot rely on full Unicode
compatibility, you may wish to format `Duration` objects yourself or use a
crate to do so.
"#,
                None,
                || {
                    st::ContainerFormat::Struct(
                        [
                            st::Named::builtin("secs", "", None, st::Format::U64),
                            st::Named::builtin(
                                "nanos",
                                "Always 0 <= nanos < NANOS_PER_SEC",
                                None,
                                st::Format::U32,
                            ),
                        ]
                        .into_iter()
                        .collect(),
                    )
                },
            ),
            "SystemTime" => self.add_builtin::<std::time::SystemTime>(
                "SystemTime",
                r#"A measurement of the system clock, useful for talking to 
external entities like the file system or other processes."#,
                None,
                || {
                    st::ContainerFormat::Struct(
                        [
                            st::Named::builtin("secs_since_epoch", "", None, st::Format::U64),
                            st::Named::builtin(
                                "nanos_since_epoch",
                                "Always 0 <= nanos < NANOS_PER_SEC",
                                None,
                                st::Format::U32,
                            ),
                        ]
                        .into_iter()
                        .collect(),
                    )
                },
            ),
            // std::collections
            "Vec" | "VecDeque" | "LinkedList" if ts.args.len() == 1 => {
                self.type_to_seq(&ts.args[0])
            }
            "HashMap" | "BTreeMap" if ts.args.len() == 2 => {
                let k = to_format(&ts.args[0]);
                let v = to_format(&ts.args[1]);
                st::Format::Map {
                    key: Box::new(k),
                    value: Box::new(v),
                }
            }
            "HashSet" | "BTreeSet" if ts.args.len() == 1 => {
                let k = to_format(&ts.args[0]);
                //quote!(Set<#k>)
                st::Format::Seq(Box::new(k))
            }
            "Option" if ts.args.len() == 1 => {
                let k = to_format(&ts.args[0]);
                st::Format::Option(Box::new(k))
            }
            "Result" if ts.args.len() == 2 => {
                let origin = utils::parse_span(ts.ident.span()).ok();
                let ok = to_format(&ts.args[0]);
                let err = to_format(&ts.args[1]);
                self.add_builtin::<std::result::Result<(), ()>>(
                    &format!("Result_Ok{}_Err{}", ok.as_ident(), err.as_ident()),
                    r#"`Result` is a type that represents either success ([`Ok`]) or failure ([`Err`])."#,
                    origin,
                    || {
                        st::ContainerFormat::Enum(
                            [
                                (0u32, st::Named::builtin("Ok", "Contains the success value", origin, st::VariantFormat::NewType(Box::new(ok)))),
                                (1u32, st::Named::builtin(
                                    "Err",
                                    "Contains the error value",
                                    origin,
                                    st::VariantFormat::NewType(Box::new(err)),
                                )),
                            ]
                            .into_iter()
                            .collect(),
                        )
                    },
                )
            }
            // "Fn" | "FnOnce" | "FnMut" => {
            //     let args = self.derive_syn_types(&ts.args);
            //     if let Some(ref rt) = ts.return_type {
            //         let rt = to_format(rt);
            //         quote! { (#(#args),*) => #rt }
            //     } else {
            //         quote! { (#(#args),*) => undefined }
            //     }
            // }
            _ => {
                let owned = ts.path();
                let path: Vec<&str> = owned.iter().map(|s| s.as_ref()).collect();
                match path[..] {
                    // Check
                    ["chrono", "DateTime"] => st::Format::Str,
                    _ => st::Format::TypeName {
                        ident: ts.ident.to_string().clone(),
                        generics: ts.args.iter().map(to_format).collect(),
                    },
                }
            }
        }
    }

    fn add_builtin<T: std::any::Any>(
        &mut self,
        name: &str,
        docs: &str,
        origin: Option<(usize, usize)>,
        or_with: impl FnOnce() -> st::ContainerFormat,
    ) -> st::Format {
        let name = name.to_string();
        self.publish_builtins
            .entry(name.clone())
            .or_insert_with(|| st::Named::builtin(&name, docs, origin, or_with()));
        st::Format::TypeName {
            ident: name,
            generics: Vec::new(),
        }
    }

    fn type_to_seq(&mut self, elem: &syn::Type) -> st::Format {
        // check for [u8] or Vec<u8>
        // if let Some(ty) = self.get_path(elem) {
        //     if ty.ident == "u8" && is_bytes(&self.field) {
        //         return Format::;
        //     };
        // };
        st::Format::Seq(Box::new(self.type_to_format(elem)))
    }

    fn derive_named_field(&mut self, field: &ast::Field<'a>) -> st::Named<st::Format> {
        match &field.member {
            syn::Member::Named(named) => {
                let format = self.field_to_format(field);
                self.derive_named(format, &named, &field.original.attrs, None)
            }
            syn::Member::Unnamed(_) => todo!("unnamed field"),
        }
    }

    fn derive_named_fields(
        &'a mut self,
        fields: &'a [&'a ast::Field<'a>],
    ) -> impl Iterator<Item = st::Named<st::Format>> + 'a {
        fields.iter().map(move |f| self.derive_named_field(f))
    }

    fn derive_named_fields_alt(
        &'a mut self,
        fields: &'a [ast::Field<'a>],
    ) -> impl Iterator<Item = st::Named<st::Format>> + 'a {
        fields.iter().map(move |f| self.derive_named_field(f))
    }

    fn derive_fields_tuple(
        &'a mut self,
        fields: &'a [&'a ast::Field<'a>],
    ) -> impl Iterator<Item = st::Format> + 'a {
        fields.iter().map(move |f| self.field_to_format(f))
    }

    fn derive_fields_tuple_alt(
        &'a mut self,
        fields: &'a [ast::Field<'a>],
    ) -> impl Iterator<Item = st::Format> + 'a {
        fields.iter().map(move |f| self.field_to_format(f))
    }
}

struct TypeFormat {
    ident: syn::Ident,
    args: Vec<syn::Type>,
    path: Vec<syn::Ident>, // full path
    #[allow(unused)]
    return_type: Option<syn::Type>, // only if function
}

impl TypeFormat {
    fn path(&self) -> Vec<String> {
        self.path.iter().map(|i| i.to_string()).collect() // hold the memory
    }
}

fn last_path_element(path: &syn::Path) -> Option<TypeFormat> {
    let fullpath = path
        .segments
        .iter()
        .map(|s| s.ident.clone())
        .collect::<Vec<_>>();
    match path.segments.last() {
        Some(t) => {
            let ident = t.ident.clone();
            let args = match &t.arguments {
                syn::PathArguments::AngleBracketed(ref path) => &path.args,
                // closures Fn(A,B) -> C
                syn::PathArguments::Parenthesized(ref path) => {
                    let args: Vec<_> = path.inputs.iter().cloned().collect();
                    let ret = return_type(&path.output);
                    return Some(TypeFormat {
                        ident,
                        args,
                        path: fullpath,
                        return_type: ret,
                    });
                }
                syn::PathArguments::None => {
                    return Some(TypeFormat {
                        ident,
                        args: vec![],
                        path: fullpath,
                        return_type: None,
                    });
                }
            };
            // ignore lifetimes
            let args = args
                .iter()
                .filter_map(|p| match p {
                    syn::GenericArgument::Type(t) => Some(t),
                    syn::GenericArgument::AssocType(t) => Some(&t.ty),
                    syn::GenericArgument::Constraint(..) => None,
                    syn::GenericArgument::Const(..) => None,
                    _ => None, // lifetimes, expr, constraints A : B ... skip!
                })
                .cloned()
                .collect::<Vec<_>>();

            Some(TypeFormat {
                ident,
                path: fullpath,
                args,
                return_type: None,
            })
        }
        None => None,
    }
}

fn return_type(rt: &syn::ReturnType) -> Option<syn::Type> {
    match rt {
        syn::ReturnType::Default => None, // e.g. undefined
        syn::ReturnType::Type(_, tp) => Some(*tp.clone()),
    }
}
