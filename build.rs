use std::fs;

fn main() {
    let mut compiler = shaderc::Compiler::new().unwrap();
    let mut options = shaderc::CompileOptions::new().unwrap();
    options.set_optimization_level(shaderc::OptimizationLevel::Performance);
    options.set_include_callback(include_glsl);

    // Compute
    let cs_src = include_str!("src/glsl/compute/shader.comp");
    let cs_spirv = compiler
        .compile_into_spirv(
            cs_src,
            shaderc::ShaderKind::Compute,
            "shader.comp",
            "main",
            Some(&options),
        )
        .unwrap();
    fs::write("src/glsl/compute/shader.comp.spv", cs_spirv.as_binary_u8())
        .expect("Unable to write file");

    // Vertex
    let vs_src = include_str!("src/glsl/shader.vert");
    let vs_spirv = compiler
        .compile_into_spirv(
            vs_src,
            shaderc::ShaderKind::Vertex,
            "shader.vert",
            "main",
            Some(&options),
        )
        .unwrap();
    fs::write("src/glsl/shader.vert.spv", vs_spirv.as_binary_u8()).expect("Unable to write file");

    // Fragment
    let fs_src = include_str!("src/glsl/shader.frag");
    let fs_spirv = compiler
        .compile_into_spirv(
            fs_src,
            shaderc::ShaderKind::Fragment,
            "shader.frag",
            "main",
            Some(&options),
        )
        .unwrap();
    fs::write("src/glsl/shader.frag.spv", fs_spirv.as_binary_u8()).expect("Unable to write file");
}

fn include_glsl(
    name: &str,
    _directive: shaderc::IncludeType,
    _source: &str,
    _depth: usize,
) -> Result<shaderc::ResolvedInclude, String> {
    let path = format!["src/glsl/compute/{}", name];

    match fs::read_to_string(&path) {
        Ok(file) => {
            return Ok(shaderc::ResolvedInclude {
                resolved_name: path,
                content: file,
            })
        }
        Err(e) => return Err(e.to_string()),
    }
}
