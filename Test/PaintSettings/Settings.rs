// Library/PaintSettings: materials read from JSON with their defaults,
// rejected as SimplePaint would, and written back as text that reads the
// same, nested or not.

use paint_settings::{format, parse, to_value};
use simple_paint::Parameters;

fn text(parameters: &Parameters) -> String {
    to_value(parameters).to_json()
}

#[test]
fn a_material_reads_back_as_it_was_written() {
    let material = Parameters {
        base_color_srgb: [0.107, 0.223, 0.578],
        brightness: 0.5,
        shift: 0.25,
        rotation_degrees: 30.0,
        dark_point: 0.1,
        light_point: 0.9,
    };
    let value = json::parse(&text(&material)).unwrap();
    assert_eq!(parse(&value, &Parameters::default()).unwrap(), material);
}

#[test]
fn a_missing_member_keeps_its_default_and_a_wrong_one_says_so() {
    let defaults = Parameters {
        brightness: 0.75,
        ..Parameters::default()
    };
    let value = json::parse(r#"{ "Shift": 0.5 }"#).unwrap();
    let parsed = parse(&value, &defaults).unwrap();
    assert_eq!((parsed.shift, parsed.brightness), (0.5, 0.75));
    for (text, problem) in [
        ("[]", "must be an object with"),
        (r#"{ "Shift": "half" }"#, "Shift must be a number"),
        (r#"{ "BaseColor": [1, 0] }"#, "BaseColor must be a list of 3 numbers"),
        (r#"{ "Gloss": 1 }"#, "Gloss isn't a SimplePaint setting"),
    ] {
        let value = json::parse(text).unwrap();
        let error = parse(&value, &defaults).unwrap_err();
        assert!(error.contains(problem), "{text}: {error}");
    }
    // A value that SimplePaint rejects is rejected here.
    let value = json::parse(r#"{ "Brightness": 7 }"#).unwrap();
    assert!(parse(&value, &defaults).is_err());
}

#[test]
fn the_text_nests_by_indenting_the_lines_after_the_first() {
    let material = Parameters::default();
    let flat = format(&material, 0);
    assert_eq!(flat, text(&material));
    assert!(flat.starts_with("{\n    \"BaseColor\": ["), "{flat}");
    assert!(flat.ends_with("\n}"), "{flat}");
    let nested = format(&material, 4);
    assert_eq!(nested, flat.replace('\n', "\n    "));
    let value = json::parse(&format!("{{ \"Paint\": {nested} }}")).unwrap();
    assert_eq!(parse(value.get("Paint").unwrap(), &Parameters::default()).unwrap(), material);
}
