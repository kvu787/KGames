// The DX12 adapter compiles with the default material count of one and with
// an explicit count, and rejects a count that can't describe a material.

use std::path::Path;

fn compile(entry_point: &str, profile: &str, count: Option<u32>) -> Result<(), String> {
    let source = Path::new(simple_paint::SHADER_DIRECTORY).join("SimplePaint.hlsl");
    let define = count.map(|count| format!("SIMPLE_PAINT_MATERIAL_COUNT={count}"));
    let mut arguments = vec!["-O3", "-Ges", "-WX"];
    if let Some(define) = &define {
        arguments.extend(["-D", define]);
    }
    let output = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "SimplePaint{entry_point}{}.dxil",
        count.map_or("Default".to_string(), |count| count.to_string())
    ));
    shader_compiler::compile(&source, entry_point, profile, &arguments, &output)
}

fn main() {
    for count in [None, Some(3)] {
        compile("VSMain", "vs_6_0", count).unwrap_or_else(|error| panic!("{error}"));
        compile("PSMain", "ps_6_0", count).unwrap_or_else(|error| panic!("{error}"));
    }
    let error = compile("PSMain", "ps_6_0", Some(0)).expect_err("a material count of 0 compiled");
    assert!(
        error.contains("SIMPLE_PAINT_MATERIAL_COUNT must be positive"),
        "a material count of 0 failed for another reason: {error}"
    );
    println!("Both stages compile with the default material count and with 3; 0 is rejected.");
}
