use std::borrow::Cow;

pub struct Gen {
    /// combination of indent level & item
    items: Vec<(u8, Cow<'static, str>)>,
    kind: GenKind,
}

#[derive(Clone, Copy)]
pub struct GenKind {
    pub item_suffix: &'static str,
    pub item_indent: &'static str,
    pub scope_start: &'static str,
    pub scope_end: &'static str,
}

impl GenKind {
    fn start(&self) -> Gen {
        Gen {
            items: Vec::new(),
            kind: *self,
        }
    }
}

#[test]
fn test() {
    let ts_block = GenKind {
        item_indent: "  ",
        item_suffix: "\n",
        scope_start: "{\n",
        scope_end: "\n}",
    };

    let mut block = ts_block.start();
    block.ad0("str");
    assert_eq!(
        block.to_string(),
        r#"{
      str
    }"#
    );
}

impl Gen {
    pub fn ad0(&mut self, str: impl Into<Cow<'static, str>>) -> &mut Self {
        self.items.push((0, str.into()));
        self
    }
    pub fn ad1(&mut self, str: impl Into<Cow<'static, str>>) -> &mut Self {
        self.items.push((1, str.into()));
        self
    }
    pub fn to_string(&self) -> String {
        let mut buf = String::new();
        buf.push_str(self.kind.scope_start);
        let mut first = true;
        for (item_indent, item_str) in &self.items {
            if !first {
                buf.push_str(self.kind.item_suffix);
            }
            for _ in 0..*item_indent {
                buf.push_str(self.kind.item_indent);
            }
            buf.push_str(&item_str);
            first = false;
        }
        buf.push_str(self.kind.scope_end);
        buf
    }
}
