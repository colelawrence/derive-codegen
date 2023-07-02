pub fn parse_span<S: std::fmt::Debug + Copy>(
    span: S,
) -> std::result::Result<(usize, usize), String> {
    let mut parser = SpanDebugParser { during: None };
    match parser.parse(span) {
        Some(found) => Ok(found),
        None => Err(format!(
            "failed to parse bytes: {:?}, {:?}",
            parser.during, span
        )),
    }
}

struct SpanDebugParser {
    // idx: usize,
    during: Option<&'static str>,
}

impl SpanDebugParser {
    // TODO: Make this less gross
    // Expects something like `"#0 bytes(2000..2030)"`
    fn parse<S: std::fmt::Debug>(&mut self, span: S) -> Option<(usize, usize)> {
        self.during = Some("start");
        let dbg_span = format!("{span:?}");
        let mut chs = dbg_span.chars().peekable();
        let mut curr = ' ';
        self.during = Some("skipping opener");
        loop {
            curr = chs.next()?;
            if curr == '(' {
                break;
            }
        }
        self.during = Some("parsing first byte");
        let mut start_byte = String::new();
        let start_at = loop {
            curr = chs.next()?;
            if curr == '.' {
                self.during = Some("found .");
                break start_byte.parse::<usize>().ok()?;
            }
            if let '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' = curr {
                start_byte.push(curr);
            } else {
                self.during = Some("start_byte non-digit");
                return None;
            }
        };

        if chs.next()? != '.' {
            self.during = Some("expected second .");
            return None;
        }
        self.during = Some("parsing second byte index");
        let mut end_byte = String::new();
        let end_at = loop {
            curr = chs.next()?;
            if curr == ')' {
                self.during = Some("parsing end bytes");
                break end_byte.parse::<usize>().ok()?;
            }
            if let '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' = curr {
                end_byte.push(curr);
            } else {
                self.during = Some("end_byte non-digit");
                return None;
            }
        };

        Some((start_at, end_at))
    }
}
