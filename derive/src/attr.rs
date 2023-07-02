use syn::{Attribute, LitStr, Result};

/// Find the value of a `#[codegen(tags = "ui,go,...")]` attribute.
pub fn attr_tags(attrs: &[Attribute]) -> Result<Vec<(String, proc_macro2::Span)>> {
    let mut tags = Vec::new();

    for attr in attrs {
        if !attr.path().is_ident("codegen") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("tags") {
                let value = meta.value()?;
                let s: LitStr = value.parse()?;
                let s: String = s.value();
                dbg!(&s);
                for item in s.split(",") {
                    tags.push((item.to_string(), value.span()));
                }
                Ok(())
            } else {
                // Err(meta.error("unsupported attribute"))
                Ok(())
            }
        })?;
    }

    Ok(tags)
}
