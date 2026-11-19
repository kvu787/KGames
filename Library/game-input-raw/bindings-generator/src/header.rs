// GameInput.h's declarations, in the header's own C terms: its types as C
// spells them, and its parameters' SAL annotations as the header writes them,
// for lib.rs to describe as metadata. A declaration of a kind that this
// doesn't know fails, instead of going unbound.

use std::collections::HashMap;

pub struct Header {
    // GAMEINPUT_API_VERSION.
    pub version: i32,
    pub enums: Vec<Enum>,
    pub structs: Vec<Struct>,
    pub typedefs: Vec<Typedef>,
    pub callbacks: Vec<Callback>,
    pub interfaces: Vec<Interface>,
    pub constants: Vec<Constant>,
    pub functions: Vec<Function>,
}

pub struct Enum {
    pub name: String,
    // Whether DEFINE_ENUM_FLAG_OPERATORS makes its values flags.
    pub flags: bool,
    pub members: Vec<(String, i64)>,
}

pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
    // The anonymous union that it ends with, like GameInputForceFeedbackParams'
    // `data`, as the field's name and the union's fields.
    pub union: Option<(String, Vec<Field>)>,
}

pub struct Field {
    pub ty: String,
    pub name: String,
    // The length of an array.
    pub length: Option<usize>,
}

pub struct Typedef {
    pub ty: String,
    pub name: String,
}

pub struct Callback {
    pub name: String,
    pub params: Vec<Param>,
}

pub struct Interface {
    pub name: String,
    pub guid: u128,
    pub methods: Vec<Function>,
}

pub struct Function {
    pub returns: String,
    pub name: String,
    pub params: Vec<Param>,
}

pub struct Param {
    pub ty: String,
    pub name: String,
    // Its SAL annotations, each with its argument, like ("_Out_writes_",
    // Some("stateArrayCount")).
    pub annotations: Vec<(String, Option<String>)>,
}

pub struct Constant {
    pub ty: String,
    pub name: String,
    pub value: Value,
}

pub enum Value {
    Integer(i64),
    Guid(u128),
}

impl Header {
    pub fn parse(text: &str) -> Header {
        let text: String = text.lines().map(|line| line.split("//").next().unwrap()).collect::<Vec<_>>().join("\n");
        let version = text
            .lines()
            .find_map(|line| line.strip_prefix("#define GAMEINPUT_API_VERSION"))
            .expect("no GAMEINPUT_API_VERSION");
        let mut header = Header {
            version: integer(version) as i32,
            enums: Vec::new(),
            structs: Vec::new(),
            typedefs: Vec::new(),
            callbacks: Vec::new(),
            interfaces: Vec::new(),
            constants: Vec::new(),
            functions: Vec::new(),
        };

        let open = "namespace GameInput { namespace v4 {";
        let start = text.find(open).expect("no namespace GameInput::v4") + open.len();
        let body = &text[start..text.rfind("}}").unwrap()];

        // The values of the enums' members and of the constants, which
        // constants are made of.
        let mut values: HashMap<String, i64> = HashMap::new();
        let mut flags = Vec::new();
        for statement in statements(body) {
            let mut statement = statement.as_str();
            // DEFINE_ENUM_FLAG_OPERATORS(X) has no `;`, so it leads the next
            // statement.
            while let Some(rest) = statement.strip_prefix("DEFINE_ENUM_FLAG_OPERATORS(") {
                let (name, rest) = rest.split_once(')').unwrap();
                flags.push(name.to_string());
                statement = rest.trim_start();
            }
            let words: Vec<&str> = statement.split_whitespace().collect();
            match words.as_slice() {
                [] => {}
                ["enum", name, ..] => {
                    let mut members = Vec::new();
                    for member in split(braced(statement), ',') {
                        let (member, value) = member.split_once('=').expect("an enum member without a value");
                        let value = integer(value);
                        values.insert(member.trim().to_string(), value);
                        members.push((member.trim().to_string(), value));
                    }
                    header.enums.push(Enum { name: name.to_string(), flags: false, members });
                }
                ["struct", name, ..] => header.structs.push(parse_struct(name, braced(statement), &values)),
                // A forward declaration.
                ["interface", _] => {}
                ["typedef", "void", "(CALLBACK*", ..] => {
                    let rest = statement.split_once("(CALLBACK*").unwrap().1;
                    let (name, rest) = rest.split_once(')').unwrap();
                    header.callbacks.push(Callback { name: name.trim().to_string(), params: params(rest) });
                }
                ["typedef", ty, name] => header.typedefs.push(Typedef {
                    ty: ty.to_string(),
                    name: name.trim_end_matches(';').to_string(),
                }),
                [first, ..] if first.starts_with("DECLARE_INTERFACE_IID_(") => header.interfaces.push(parse_interface(statement)),
                // IID_IGameInput is IGameInput's GUID, which its declaration
                // has too.
                ["const", "IID", "IID_IGameInput", ..] => {}
                ["const" | "constexpr", ty, name, "=", ..] => {
                    let expression = statement.split_once('=').unwrap().1.trim().trim_end_matches(';');
                    let value = if *ty == "GUID" {
                        Value::Guid(guid_initializer(expression))
                    } else {
                        let value = evaluate(expression, &values);
                        values.insert(name.to_string(), value);
                        Value::Integer(value)
                    };
                    header.constants.push(Constant { ty: ty.to_string(), name: name.to_string(), value });
                }
                ["STDAPI", ..] => header.functions.push(parse_function("HRESULT", &statement["STDAPI".len()..])),
                // GameInputCreate, which calls GameInputInitialize with
                // IID_IGameInput, is inline, so no library has it.
                ["inline", "HRESULT", name, ..] if name.starts_with("GameInputCreate(") => {}
                _ => panic!("GameInput.h declares something that this doesn't know: {statement}"),
            }
        }
        for name in flags {
            header.enums.iter_mut().find(|e| e.name == name).expect("flags of no enum").flags = true;
        }
        header
    }
}

// The namespace's declarations, each ending with a `;`, or with the `}` of an
// interface or an inline function, outside any brackets.
fn statements(body: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut depth = 0;
    let mut current = String::new();
    for c in body.chars() {
        match c {
            '{' | '(' => depth += 1,
            '}' | ')' => depth -= 1,
            _ => {}
        }
        current.push(c);
        let declaration = current.trim_start();
        let ends = match c {
            ';' => depth == 0,
            '}' => depth == 0 && (declaration.starts_with("DECLARE_INTERFACE_IID_") || declaration.starts_with("inline ")),
            _ => false,
        };
        if ends {
            statements.push(declaration.trim_end_matches(';').trim().to_string());
            current.clear();
        }
    }
    assert!(current.trim().is_empty(), "GameInput.h ends with an unfinished declaration: {current}");
    statements
}

fn parse_struct(name: &str, body: &str, values: &HashMap<String, i64>) -> Struct {
    let (body, union) = match body.find("union") {
        Some(start) => {
            let union = &body[start..];
            let (_, after) = union.split_once('}').unwrap();
            let field = after.trim().trim_end_matches(';').trim();
            (&body[..start], Some((field.to_string(), fields(braced(union), values))))
        }
        None => (body, None),
    };
    Struct { name: name.to_string(), fields: fields(body, values), union }
}

fn fields(body: &str, values: &HashMap<String, i64>) -> Vec<Field> {
    split(body, ';')
        .into_iter()
        .map(|declaration| {
            // Without the SAL annotations of fields, like _Field_z_ and
            // _Field_range_(1, GAMEINPUT_HAPTIC_MAX_LOCATIONS).
            let (annotations, declaration) = annotations(&declaration);
            assert!(annotations.iter().all(|(name, _)| name.starts_with("_Field_")), "{declaration}");
            let (ty, name) = declaration.rsplit_once(' ').unwrap();
            let (name, length) = match name.split_once('[') {
                Some((name, length)) => {
                    let length = length.trim_end_matches(']');
                    (name, Some(values.get(length).copied().unwrap_or_else(|| integer(length)) as usize))
                }
                None => (name, None),
            };
            Field { ty: ty.trim().to_string(), name: name.to_string(), length }
        })
        .collect()
}

fn parse_interface(statement: &str) -> Interface {
    let declaration = parenthesized(statement);
    let parts: Vec<&str> = declaration.split(',').map(str::trim).collect();
    assert_eq!(parts[1], "IUnknown", "{} doesn't derive from IUnknown", parts[0]);
    let methods = braced(statement)
        .split("PURE;")
        .map(str::trim)
        .filter(|method| !method.is_empty())
        .map(|method| {
            if let Some(rest) = method.strip_prefix("IFACEMETHOD_(") {
                let (returns, rest) = rest.split_once(',').unwrap();
                let (name, rest) = rest.split_once(')').unwrap();
                parse_function(returns.trim(), &format!("{name}{rest}"))
            } else if let Some(rest) = method.strip_prefix("IFACEMETHOD(") {
                let (name, rest) = rest.split_once(')').unwrap();
                parse_function("HRESULT", &format!("{name}{rest}"))
            } else {
                panic!("an interface method that this doesn't know: {method}")
            }
        })
        .collect();
    Interface {
        name: parts[0].to_string(),
        guid: u128::from_str_radix(&parts[2].trim_matches('"').replace('-', ""), 16).unwrap(),
        methods,
    }
}

// `Name(params)`, after the return type.
fn parse_function(returns: &str, declaration: &str) -> Function {
    let (name, rest) = declaration.split_once('(').unwrap();
    Function {
        returns: returns.to_string(),
        name: name.trim().to_string(),
        params: params(&format!("({rest}")),
    }
}

fn params(text: &str) -> Vec<Param> {
    split(parenthesized(text), ',')
        .into_iter()
        .map(|param| {
            // Without a default value, like GAMEINPUT_LIGHTING_GROUP_ORDER_INDEX_ALL's.
            let param = param.split('=').next().unwrap().trim();
            let (annotations, declaration) = annotations(param);
            let (ty, name) = declaration.rsplit_once(' ').unwrap();
            // `IGameInputDevice **device` as `IGameInputDevice** device`.
            let stars = name.chars().take_while(|c| *c == '*').count();
            Param {
                ty: format!("{}{}", ty.trim(), "*".repeat(stars)),
                name: name[stars..].to_string(),
                annotations,
            }
        })
        .collect()
}

// A declaration's leading SAL annotations, with their arguments, and the
// declaration after them, its whitespace collapsed.
fn annotations(declaration: &str) -> (Vec<(String, Option<String>)>, String) {
    let mut annotations = Vec::new();
    let mut rest = declaration.trim_start();
    while rest.starts_with('_') {
        let end = rest.find(|c: char| c == '(' || c.is_whitespace()).unwrap();
        let name = rest[..end].to_string();
        if rest[end..].starts_with('(') {
            let argument = parenthesized(&rest[end..]);
            annotations.push((name, Some(argument.trim().to_string())));
            rest = rest[end + argument.len() + 2..].trim_start();
        } else {
            annotations.push((name, None));
            rest = rest[end..].trim_start();
        }
    }
    (annotations, rest.split_whitespace().collect::<Vec<_>>().join(" "))
}

// What's inside a declaration's first braces.
fn braced(text: &str) -> &str {
    inside(text, '{', '}')
}

// What's inside a declaration's first parentheses.
fn parenthesized(text: &str) -> &str {
    inside(text, '(', ')')
}

fn inside(text: &str, open: char, close: char) -> &str {
    let start = text.find(open).unwrap_or_else(|| panic!("no {open} in {text}")) + 1;
    let mut depth = 1;
    for (index, c) in text[start..].char_indices() {
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                return &text[start..start + index];
            }
        }
    }
    panic!("no {close} for the {open} in {text}")
}

// The parts of a list, split at separators outside any brackets.
fn split(text: &str, separator: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut current = String::new();
    for c in text.chars() {
        match c {
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => depth -= 1,
            _ => {}
        }
        if c == separator && depth == 0 {
            parts.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(c);
        }
    }
    parts.push(current.trim().to_string());
    parts.retain(|part| !part.is_empty());
    parts
}

// A C integer literal, like 8, -1, 0x00000020 or 0x838A0001L.
fn integer(text: &str) -> i64 {
    let text = text.trim().trim_end_matches('L');
    match text.strip_prefix("0x") {
        Some(hex) => i64::from_str_radix(hex, 16).unwrap(),
        None => text.parse().unwrap_or_else(|_| panic!("not an integer: {text}")),
    }
}

// A constant's value: a literal, _HRESULT_TYPEDEF_ of one, or enum members
// and constants joined with `|`.
fn evaluate(expression: &str, values: &HashMap<String, i64>) -> i64 {
    if let Some(literal) = expression.strip_prefix("_HRESULT_TYPEDEF_(") {
        return integer(literal.trim_end_matches(')'));
    }
    expression
        .split('|')
        .map(|part| {
            let part = part.trim();
            values.get(part).copied().unwrap_or_else(|| integer(part))
        })
        .fold(0, |left, right| left | right)
}

// `{ 0x08c707c2, 0x66bb, 0x406c, { 0xa8, ... } }`.
fn guid_initializer(text: &str) -> u128 {
    let numbers: Vec<i64> = text.split([',', '{', '}']).map(str::trim).filter(|part| !part.is_empty()).map(integer).collect();
    assert_eq!(numbers.len(), 11, "not a GUID: {text}");
    let mut guid = (numbers[0] as u128) << 96 | (numbers[1] as u128) << 80 | (numbers[2] as u128) << 64;
    for (index, byte) in numbers[3..].iter().enumerate() {
        guid |= (*byte as u128) << (56 - 8 * index);
    }
    guid
}
