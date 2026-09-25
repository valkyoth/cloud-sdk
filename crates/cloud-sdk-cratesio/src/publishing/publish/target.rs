// Bounded ASCII cfg syntax profile. No evaluation, host probing or target lookup.
pub(super) fn valid(s: &str) -> bool {
    if !s.is_ascii() {
        return false;
    }
    if !s.starts_with("cfg(") {
        return !s.is_empty()
            && s.len() <= 256
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b));
    }
    let Some(body) = s.strip_prefix("cfg(").and_then(|s| s.strip_suffix(')')) else {
        return false;
    };
    let mut p = Parser {
        rest: body,
        nodes: 0,
    };
    p.predicate(0) && p.rest.trim().is_empty()
}
struct Parser<'a> {
    rest: &'a str,
    nodes: u16,
}
impl Parser<'_> {
    fn consume(&mut self, s: &str) -> bool {
        self.rest = self.rest.trim_start_matches([' ', '\t']);
        match self.rest.strip_prefix(s) {
            Some(rest) => {
                self.rest = rest;
                true
            }
            None => false,
        }
    }
    fn predicate(&mut self, depth: u8) -> bool {
        if depth >= 8 || self.nodes >= 64 {
            return false;
        }
        self.nodes = self.nodes.saturating_add(1);
        self.rest = self.rest.trim_start_matches([' ', '\t']);
        let n = self
            .rest
            .bytes()
            .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
            .count();
        let Some(name) = self.rest.get(..n) else {
            return false;
        };
        if !name
            .as_bytes()
            .first()
            .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
        {
            return false;
        }
        let Some(rest) = self.rest.get(n..) else {
            return false;
        };
        self.rest = rest;
        if self.consume("=") {
            if !self.consume("\"") {
                return false;
            }
            let Some((value, rest)) = self.rest.split_once('"') else {
                return false;
            };
            if value.chars().any(|c| c.is_control() || c == '\\') {
                return false;
            }
            self.rest = rest;
            return true;
        }
        if !self.consume("(") {
            return true;
        }
        if !matches!(name, "all" | "any" | "not") {
            return false;
        }
        if self.consume(")") {
            return name != "not";
        }
        let mut count = 0u8;
        loop {
            if !self.predicate(depth.saturating_add(1)) {
                return false;
            }
            count = count.saturating_add(1);
            if self.consume(")") {
                return name != "not" || count == 1;
            }
            if !self.consume(",") {
                return false;
            }
            if self.consume(")") {
                return name != "not" || count == 1;
            }
        }
    }
}
