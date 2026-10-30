// A JSON parser and writer, for settings files and test scripts, since KGames
// depends on no crates besides windows-core. The parser follows RFC 8259, and
// says where in the text a mistake is.

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    // Members in the file's order.
    Object(Vec<(String, Value)>),
}

impl Value {
    // For messages like `must be a number, not "1280"`.
    pub fn describe(&self) -> String {
        match self {
            Value::Null => "null".to_string(),
            Value::Bool(value) => value.to_string(),
            Value::Number(value) => value.to_string(),
            Value::String(value) => format!("\"{value}\""),
            Value::Array(values) => format!("a list of {}", values.len()),
            Value::Object(_) => "an object".to_string(),
        }
    }

    // A member of an object, if this is one and it has the member.
    pub fn get(&self, name: &str) -> Option<&Value> {
        match self {
            Value::Object(members) => members
                .iter()
                .find(|(member, _)| member == name)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    // As JSON text, with each object member on its own line, indented by
    // four spaces a level. Arrays of numbers, strings and the like stay on
    // one line. Numbers are written so that they parse back exactly. JSON has
    // no infinities or NaN, so those are written as null.
    pub fn to_json(&self) -> String {
        let mut text = String::new();
        self.write(&mut text, 0);
        text
    }

    fn write(&self, text: &mut String, depth: usize) {
        match self {
            Value::Null => text.push_str("null"),
            Value::Bool(value) => text.push_str(if *value { "true" } else { "false" }),
            Value::Number(value) if value.is_finite() => text.push_str(&format!("{value:?}")),
            Value::Number(_) => text.push_str("null"),
            Value::String(value) => write_string(text, value),
            Value::Array(values) if values.is_empty() => text.push_str("[]"),
            Value::Array(values) => {
                let flat = values
                    .iter()
                    .all(|value| !matches!(value, Value::Array(_) | Value::Object(_)));
                text.push('[');
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        text.push(',');
                    }
                    if !flat {
                        newline(text, depth + 1);
                    } else if index > 0 {
                        text.push(' ');
                    }
                    value.write(text, depth + 1);
                }
                if !flat {
                    newline(text, depth);
                }
                text.push(']');
            }
            Value::Object(members) if members.is_empty() => text.push_str("{}"),
            Value::Object(members) => {
                text.push('{');
                for (index, (name, value)) in members.iter().enumerate() {
                    if index > 0 {
                        text.push(',');
                    }
                    newline(text, depth + 1);
                    write_string(text, name);
                    text.push_str(": ");
                    value.write(text, depth + 1);
                }
                newline(text, depth);
                text.push('}');
            }
        }
    }
}

fn newline(text: &mut String, depth: usize) {
    text.push('\n');
    text.push_str(&"    ".repeat(depth));
}

fn write_string(text: &mut String, value: &str) {
    text.push('"');
    for character in value.chars() {
        match character {
            '"' => text.push_str("\\\""),
            '\\' => text.push_str("\\\\"),
            '\n' => text.push_str("\\n"),
            '\r' => text.push_str("\\r"),
            '\t' => text.push_str("\\t"),
            '\0'..='\u{1F}' => text.push_str(&format!("\\u{:04X}", character as u32)),
            _ => text.push(character),
        }
    }
    text.push('"');
}

// An object with these members, in this order, like
// object([("Shift", 0.5.into()), ("Name", "Body".into())]).
pub fn object<const N: usize>(members: [(&str, Value); N]) -> Value {
    Value::Object(
        members
            .into_iter()
            .map(|(name, value)| (name.to_string(), value))
            .collect(),
    )
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::Number(value as f64)
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Value::Number(value as f64)
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Value::Number(value as f64)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Number(value as f64)
    }
}

impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Value::Number(value as f64)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(value: Option<T>) -> Self {
        value.map_or(Value::Null, Into::into)
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(values: Vec<T>) -> Self {
        Value::Array(values.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<Value>, const N: usize> From<[T; N]> for Value {
    fn from(values: [T; N]) -> Self {
        Value::Array(values.into_iter().map(Into::into).collect())
    }
}

#[derive(Debug)]
pub struct Error {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

pub fn parse(text: &str) -> Result<Value, Error> {
    let mut parser = Parser { text, position: 0 };
    let value = parser.value()?;
    parser.skip_whitespace();
    if parser.position < text.len() {
        return Err(parser.error("expected the end of the file"));
    }
    Ok(value)
}

struct Parser<'a> {
    text: &'a str,
    // A byte offset into text.
    position: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<char> {
        self.text[self.position..].chars().next()
    }

    // Steps past the next character if it's the expected one.
    fn eat(&mut self, expected: char) -> bool {
        let found = self.peek() == Some(expected);
        if found {
            self.position += expected.len_utf8();
        }
        found
    }

    fn skip_whitespace(&mut self) {
        while let Some(' ' | '\t' | '\n' | '\r') = self.peek() {
            self.position += 1;
        }
    }

    // An error at the current position.
    fn error(&self, message: &str) -> Error {
        let before = &self.text[..self.position];
        Error {
            line: before.matches('\n').count() + 1,
            column: before.rsplit('\n').next().unwrap_or("").chars().count() + 1,
            message: message.to_string(),
        }
    }

    fn value(&mut self) -> Result<Value, Error> {
        self.skip_whitespace();
        match self.peek() {
            Some('{') => self.object(),
            Some('[') => self.array(),
            Some('"') => self.string().map(Value::String),
            Some('-' | '0'..='9') => self.number(),
            Some(_) => {
                for (word, value) in [
                    ("true", Value::Bool(true)),
                    ("false", Value::Bool(false)),
                    ("null", Value::Null),
                ] {
                    if self.text[self.position..].starts_with(word) {
                        self.position += word.len();
                        return Ok(value);
                    }
                }
                Err(self.error("expected a value"))
            }
            None => Err(self.error("expected a value, but the file ended")),
        }
    }

    fn object(&mut self) -> Result<Value, Error> {
        self.position += 1;
        let mut members = Vec::new();
        self.skip_whitespace();
        if self.eat('}') {
            return Ok(Value::Object(members));
        }
        loop {
            self.skip_whitespace();
            if self.peek() == Some('}') {
                return Err(self.error("JSON doesn't allow a comma before '}'"));
            }
            if self.peek() != Some('"') {
                return Err(self.error("expected a name in double quotes"));
            }
            let name = self.string()?;
            self.skip_whitespace();
            if !self.eat(':') {
                return Err(self.error("expected ':'"));
            }
            members.push((name, self.value()?));
            self.skip_whitespace();
            if self.eat('}') {
                return Ok(Value::Object(members));
            }
            if !self.eat(',') {
                return Err(self.error("expected ',' or '}'"));
            }
        }
    }

    fn array(&mut self) -> Result<Value, Error> {
        self.position += 1;
        let mut values = Vec::new();
        self.skip_whitespace();
        if self.eat(']') {
            return Ok(Value::Array(values));
        }
        loop {
            self.skip_whitespace();
            if self.peek() == Some(']') {
                return Err(self.error("JSON doesn't allow a comma before ']'"));
            }
            values.push(self.value()?);
            self.skip_whitespace();
            if self.eat(']') {
                return Ok(Value::Array(values));
            }
            if !self.eat(',') {
                return Err(self.error("expected ',' or ']'"));
            }
        }
    }

    fn string(&mut self) -> Result<String, Error> {
        self.position += 1;
        let mut string = String::new();
        loop {
            let start = self.position;
            let Some(character) = self.peek() else {
                return Err(self.error("expected '\"' to end the string"));
            };
            self.position += character.len_utf8();
            match character {
                '"' => return Ok(string),
                '\\' => {
                    let escaped = match self.peek() {
                        Some('"') => '"',
                        Some('\\') => '\\',
                        Some('/') => '/',
                        Some('b') => '\u{8}',
                        Some('f') => '\u{C}',
                        Some('n') => '\n',
                        Some('r') => '\r',
                        Some('t') => '\t',
                        Some('u') => {
                            self.position += 1;
                            string.push(self.unicode_escape()?);
                            continue;
                        }
                        _ => {
                            self.position = start;
                            return Err(self.error("unknown escape sequence"));
                        }
                    };
                    self.position += 1;
                    string.push(escaped);
                }
                // Like a line break. They must be escaped, like \n.
                '\0'..='\u{1F}' => {
                    self.position = start;
                    return Err(self.error("strings can't contain control characters"));
                }
                _ => string.push(character),
            }
        }
    }

    // After "\u": four hex digits of UTF-16. A character outside the Basic
    // Multilingual Plane takes two, a high surrogate then a low one.
    fn unicode_escape(&mut self) -> Result<char, Error> {
        let start = self.position;
        let first = self.hex_digits()?;
        if !(0xD800..0xDC00).contains(&first) {
            // Fails for a low surrogate on its own.
            return char::from_u32(first).ok_or_else(|| self.error("unpaired UTF-16 surrogate"));
        }
        let second = if self.text[self.position..].starts_with("\\u") {
            self.position += 2;
            self.hex_digits()?
        } else {
            0
        };
        if !(0xDC00..0xE000).contains(&second) {
            self.position = start;
            return Err(self.error("unpaired UTF-16 surrogate"));
        }
        Ok(char::from_u32(0x10000 + ((first - 0xD800) << 10) + (second - 0xDC00)).unwrap())
    }

    fn hex_digits(&mut self) -> Result<u32, Error> {
        let digits = self
            .text
            .get(self.position..self.position + 4)
            .filter(|digits| digits.chars().all(|digit| digit.is_ascii_hexdigit()));
        let Some(digits) = digits else {
            return Err(self.error("expected four hex digits after \\u"));
        };
        self.position += 4;
        Ok(u32::from_str_radix(digits, 16).unwrap())
    }

    fn number(&mut self) -> Result<Value, Error> {
        let start = self.position;
        self.eat('-');
        let leading_zero = self.peek() == Some('0');
        match self.digits() {
            0 => return Err(self.error("expected a digit")),
            1 => {}
            _ if leading_zero => {
                self.position = start;
                return Err(self.error("numbers can't start with 0, like 01"));
            }
            _ => {}
        }
        if self.eat('.') && self.digits() == 0 {
            return Err(self.error("expected a digit after '.'"));
        }
        if self.eat('e') || self.eat('E') {
            let _ = self.eat('+') || self.eat('-');
            if self.digits() == 0 {
                return Err(self.error("expected a digit in the exponent"));
            }
        }
        // Rust parses every number that JSON allows.
        Ok(Value::Number(
            self.text[start..self.position].parse().unwrap(),
        ))
    }

    // Steps past a run of digits, and returns how many there were.
    fn digits(&mut self) -> usize {
        let start = self.position;
        while self
            .peek()
            .is_some_and(|character| character.is_ascii_digit())
        {
            self.position += 1;
        }
        self.position - start
    }
}
