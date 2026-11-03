// SPDX-License-Identifier: LGPL-3.0-or-later
// Minecraft 1.21.1 TagParser saved-compound input, independent of 26.3 syntax.
pub trait Target: Sized {
    fn number(kind: u8, bits: u64) -> Self;
    fn string(value: Vec<u16>) -> Self;
    fn compound(value: Vec<(Vec<u16>, Self)>) -> Self;
    // kind 7/11/12 denotes a typed numeric array; 9 denotes a ListTag.
    fn sequence(kind: u8, values: Vec<Self>) -> Self;
}
pub fn parse<T: Target>(text: &[u16]) -> Option<T> {
    let mut parser = Parser {
        text,
        i: 0,
        depth: 0,
    };
    let value = parser.compound()?;
    parser.skip();
    (parser.i == text.len()).then_some(value)
}
struct Parser<'a> {
    text: &'a [u16],
    i: usize,
    depth: usize,
}
fn whitespace(c: u16) -> bool {
    matches!(c, 0x9..=0xd | 0x1c..=0x20 | 0x1680 | 0x2000..=0x2006 | 0x2008..=0x200a | 0x2028..=0x2029 | 0x205f | 0x3000)
}
impl Parser<'_> {
    fn skip(&mut self) {
        while self.text.get(self.i).is_some_and(|&c| whitespace(c)) {
            self.i += 1;
        }
    }
    fn peek(&mut self) -> Option<u16> {
        self.skip();
        self.text.get(self.i).copied()
    }
    fn take(&mut self, c: u8) -> bool {
        if self.peek() == Some(u16::from(c)) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn string(&mut self) -> Option<Vec<u16>> {
        let first = self.peek()?;
        if first == u16::from(b'"') || first == u16::from(b'\'') {
            self.i += 1;
            let mut result = Vec::new();
            while let Some(&c) = self.text.get(self.i) {
                self.i += 1;
                if c == first {
                    return Some(result);
                }
                if c == u16::from(b'\\') {
                    let escaped = *self.text.get(self.i)?;
                    self.i += 1;
                    if escaped != first && escaped != u16::from(b'\\') {
                        return None;
                    }
                    result.push(escaped);
                } else {
                    result.push(c);
                }
            }
            None
        } else {
            let start = self.i;
            while self.text.get(self.i).is_some_and(|&c| {
                c < 128 && ((c as u8).is_ascii_alphanumeric() || b"_-.+".contains(&(c as u8)))
            }) {
                self.i += 1;
            }
            (start != self.i).then(|| self.text[start..self.i].to_vec())
        }
    }
    fn compound<T: Target>(&mut self) -> Option<T> {
        if !self.take(b'{') {
            return None;
        }
        let mut fields = Vec::new();
        if self.take(b'}') {
            return Some(T::compound(fields));
        }
        loop {
            let key = self.string()?;
            if key.is_empty() || !self.take(b':') {
                return None;
            }
            fields.push((key, self.value::<T>()?.0));
            if !self.take(b',') {
                break;
            }
            if self.peek() == Some(u16::from(b'}')) {
                break;
            }
        }
        self.take(b'}').then(|| T::compound(fields))
    }
    fn value<T: Target>(&mut self) -> Option<(T, u8)> {
        // Keep malformed commands bounded, as with native NBT decoding.
        if self.depth >= 512 {
            return None;
        }
        self.depth += 1;
        let result = self.inner();
        self.depth -= 1;
        result
    }
    fn inner<T: Target>(&mut self) -> Option<(T, u8)> {
        match self.peek()? {
            123 => Some((self.compound()?, 10)),
            91 => {
                self.i += 1;
                let array = self.text.get(self.i + 1) == Some(&u16::from(b';'))
                    && !matches!(self.text.get(self.i), Some(34 | 39));
                let (kind, required) = if array {
                    let k = *self.text.get(self.i)?;
                    self.i += 2;
                    match k {
                        66 => (7, 1),
                        73 => (11, 3),
                        76 => (12, 4),
                        _ => return None,
                    }
                } else {
                    (9, 0)
                };
                let mut values = Vec::new();
                let mut element = required;
                if !self.take(b']') {
                    loop {
                        let (v, k) = self.value()?;
                        if element == 0 {
                            element = k;
                        }
                        if k != element {
                            return None;
                        }
                        values.push(v);
                        if !self.take(b',') {
                            break;
                        }
                        if self.peek() == Some(u16::from(b']')) {
                            break;
                        }
                    }
                    if !self.take(b']') {
                        return None;
                    }
                }
                Some((T::sequence(kind, values), kind))
            }
            34 | 39 => Some((T::string(self.string()?), 8)),
            _ => {
                let units = self.string()?;
                let text = String::from_utf16(&units).ok()?;
                if let Some((kind, bits)) = number(&text) {
                    Some((T::number(kind, bits), kind))
                } else {
                    Some((T::string(units), 8))
                }
            }
        }
    }
}
fn integral(text: &str) -> bool {
    let text = text.strip_prefix(['+', '-']).unwrap_or(text);
    text == "0"
        || (!text.starts_with('0') && !text.is_empty() && text.bytes().all(|c| c.is_ascii_digit()))
}
fn decimal(text: &str, dot_required: bool) -> bool {
    let text = text.strip_prefix(['+', '-']).unwrap_or(text);
    let (base, exponent) = text
        .split_once(['e', 'E'])
        .map_or((text, None), |(a, b)| (a, Some(b)));
    if exponent.is_some_and(|e| {
        let e = e.strip_prefix(['+', '-']).unwrap_or(e);
        e.is_empty() || !e.bytes().all(|c| c.is_ascii_digit())
    }) {
        return false;
    }
    let (left, right) = base
        .split_once('.')
        .map_or((base, None), |(a, b)| (a, Some(b)));
    if dot_required && right.is_none() {
        return false;
    }
    (!left.is_empty() || right.is_some_and(|r| !r.is_empty()))
        && left.bytes().all(|c| c.is_ascii_digit())
        && right.is_none_or(|r| r.bytes().all(|c| c.is_ascii_digit()))
}
fn number(text: &str) -> Option<(u8, u64)> {
    let (&suffix, body) = text.as_bytes().split_last()?;
    let prefix = &text[..body.len()];
    // Once a numeric pattern matched, NumberFormatException falls back to a
    // string directly; it does not try another narrower/wider numeric type.
    if matches!(suffix, b'f' | b'F') && decimal(prefix, false) {
        let f = prefix.parse::<f32>().ok()?;
        return Some((5, u64::from(if f == 0.0 { 0 } else { f.to_bits() })));
    }
    if integral(prefix) {
        match suffix {
            b'b' | b'B' => return Some((1, prefix.parse::<i8>().ok()? as u64)),
            b'l' | b'L' => return Some((4, prefix.parse::<i64>().ok()? as u64)),
            b's' | b'S' => return Some((2, prefix.parse::<i16>().ok()? as u64)),
            _ => (),
        }
    }
    if integral(text) {
        return Some((3, text.parse::<i32>().ok()? as u64));
    }
    if matches!(suffix, b'd' | b'D') && decimal(prefix, false) {
        let f = prefix.parse::<f64>().ok()?;
        return Some((6, if f == 0.0 { 0 } else { f.to_bits() }));
    }
    if decimal(text, true) {
        let f = text.parse::<f64>().ok()?;
        return Some((6, if f == 0.0 { 0 } else { f.to_bits() }));
    }
    if text.eq_ignore_ascii_case("true") {
        return Some((1, 1));
    }
    if text.eq_ignore_ascii_case("false") {
        return Some((1, 0));
    }
    None
}
