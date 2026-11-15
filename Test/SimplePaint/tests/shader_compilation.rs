// The DX12 adapter and the unlit shader compile with the default count of
// one and with an explicit count, and reject a count that can't describe a
// material.

use std::path::Path;

// A shader in the crate's Shaders folder, and the define that sets its count.
const SHADERS: [(&str, &str); 2] = [("SimplePaint", "SIMPLE_PAINT_MATERIAL_COUNT"), ("Unlit", "UNLIT_COLOR_COUNT")];

fn compile(shader: &str, define: &str, entry_point: &str, profile: &str, count: Option<u32>) -> Result<(), String> {
    let source = Path::new(simple_paint::SHADER_DIRECTORY).join(format!("{shader}.hlsl"));
    let define = count.map(|count| format!("{define}={count}"));
    let mut arguments = vec!["-O3", "-Ges", "-WX"];
    if let Some(define) = &define {
        arguments.extend(["-D", define]);
    }
    let output = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "{shader}{entry_point}{}.dxil",
        count.map_or("Default".to_string(), |count| count.to_string())
    ));
    shader_compiler::compile(&source, entry_point, profile, &arguments, &output)
}

fn main() {
    for (shader, define) in SHADERS {
        for count in [None, Some(3)] {
            compile(shader, define, "VSMain", "vs_6_0", count).unwrap_or_else(|error| panic!("{error}"));
            compile(shader, define, "PSMain", "ps_6_0", count).unwrap_or_else(|error| panic!("{error}"));
        }
        let error = compile(shader, define, "PSMain", "ps_6_0", Some(0)).expect_err("a count of 0 compiled");
        assert!(error.contains(&format!("{define} must be positive")), "{shader} failed a count of 0 for another reason: {error}");
        println!("{shader}.hlsl: both stages compile with the default count and with 3; 0 is rejected.");
    }
}
