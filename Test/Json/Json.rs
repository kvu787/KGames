// Library/Json: parsing, where mistakes are, and writing that parses back.

use json::{Value, object, parse};

#[test]
fn parses_every_kind_of_value() {
    let value = parse(r#" { "a": [1, -2.5e3, true, false, null], "b": "x\n\u00e9\ud83d\ude00" } "#).unwrap();
    assert_eq!(
        value,
        object([
            (
                "a",
                Value::Array(vec![
                    1.0.into(),
                    (-2500.0).into(),
                    true.into(),
                    false.into(),
                    Value::Null
                ])
            ),
            ("b", "x\n\u{e9}\u{1F600}".into()),
        ])
    );
    assert_eq!(value.get("b"), Some(&Value::from("x\n\u{e9}\u{1F600}")));
    assert_eq!(value.get("c"), None);
}

#[test]
fn says_where_a_mistake_is() {
    for (text, line, column, message) in [
        ("{\n  \"a\": 1,\n}", 3, 1, "JSON doesn't allow a comma before '}'"),
        ("[01]", 1, 2, "numbers can't start with 0, like 01"),
        ("\"\\ud800\"", 1, 4, "unpaired UTF-16 surrogate"),
        ("{} x", 1, 4, "expected the end of the file"),
        ("[1 2]", 1, 4, "expected ',' or ']'"),
    ] {
        let error = parse(text).unwrap_err();
        assert_eq!(
            (error.line, error.column, error.message.as_str()),
            (line, column, message),
            "for {text:?}"
        );
    }
}

#[test]
fn fails_on_deep_nesting_instead_of_overflowing_the_stack() {
    let nested = |depth: usize| "[".repeat(depth) + &"]".repeat(depth);
    assert!(parse(&nested(128)).is_ok());
    let error = parse(&nested(129)).unwrap_err();
    assert_eq!(
        (error.line, error.column, error.message.as_str()),
        (1, 129, "arrays and objects are nested too deeply")
    );
    // Deep enough to overflow the stack without the limit.
    assert!(parse(&"[{\"a\":".repeat(100_000)).is_err());
}

#[test]
fn writes_what_parses_back_the_same() {
    let value = object([
        ("Numbers", [0.1, 1.0 / 3.0, -0.0, 1e-300, 123456789.0].into()),
        ("Text", "quote \" backslash \\ tab \t bell \u{7} e \u{e9}".into()),
        ("Nested", object([("Empty", Value::Array(Vec::new())), ("Object", Value::Object(Vec::new()))])),
        ("Rows", Value::Array(vec![object([("Flag", true.into())]), Value::Null])),
    ]);
    let text = value.to_json();
    assert_eq!(parse(&text).unwrap(), value, "{text}");
}

#[test]
fn writes_one_member_a_line() {
    let value = object([("Color", [0.5, 0.25].into()), ("Name", "Body".into())]);
    assert_eq!(
        value.to_json(),
        "{\n    \"Color\": [0.5, 0.25],\n    \"Name\": \"Body\"\n}"
    );
}

#[test]
fn writes_infinities_and_nan_as_null() {
    let value = Value::Array(vec![f64::INFINITY.into(), f64::NAN.into()]);
    assert_eq!(value.to_json(), "[null, null]");
}
