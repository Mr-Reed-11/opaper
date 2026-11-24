use wgpu::naga::{
    ShaderStage,
    front::glsl::{Frontend, Options},
};

pub const DEFAULT_FRAGMENT: &str = r#"#version 450
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform Globals {
    float iTime;
    float iTimeDelta;
    uint iFrame;
    float _pad;
    vec4 iResolution;
    vec4 iParams[8];
};
void main() {
    vec2 uv = (gl_FragCoord.xy * 2.0 - iResolution.xy) / iResolution.y;
    float wave = sin(uv.x * 4.0 + iTime) * 0.5 + 0.5;
    wave += sin(uv.y * 3.0 - iTime * 0.7) * 0.25;
    outColor = vec4(vec3(0.08, 0.12 + wave * 0.35, 0.45 + wave * 0.4), 1.0);
}
"#;

pub const VERTEX_WGSL: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
    var positions = array<vec2f, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
    return vec4f(positions[index], 0.0, 1.0);
}
"#;

/// Checks version, entry point and GLSL syntax before touching the GPU.
pub fn validate_source(source: &str) -> Result<(), String> {
    if !source.contains("#version 450") && !source.contains("#version 330") {
        return Err("shader must declare #version 450 or #version 330".to_owned());
    }
    if !source.contains("void main") {
        return Err("shader needs a void main() entry point".to_owned());
    }
    let mut frontend = Frontend::default();
    let module = frontend
        .parse(
            &Options {
                stage: ShaderStage::Fragment,
                defines: Default::default(),
            },
            source,
        )
        .map_err(|e| format!("GLSL parse error: {e}"))?;
    if !module.entry_points.iter().any(|ep| ep.name == "main") {
        return Err("shader needs a main entry point".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn accepts_default_shader() {
        validate_source(DEFAULT_FRAGMENT).expect("built-in shader must be valid");
    }

    #[test]
    fn accepts_example_shader() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../examples/plasma-waves/shader.frag");
        let source = fs::read_to_string(path).unwrap();
        validate_source(&source).expect("example shader must be valid");
    }

    #[test]
    fn accepts_nebula_shader() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../examples/nebula-space/shader.frag");
        let source = fs::read_to_string(path).unwrap();
        validate_source(&source).expect("nebula shader must be valid");
    }

    #[test]
    fn accepts_synthwave_shader() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../examples/synthwave-horizon/shader.frag");
        let source = fs::read_to_string(path).unwrap();
        validate_source(&source).expect("synthwave shader must be valid");
    }

    #[test]
    fn rejects_missing_version() {
        let err = validate_source("void main() {}").unwrap_err();
        assert!(err.contains("#version"), "{err}");
    }

    #[test]
    fn rejects_broken_glsl() {
        let err = validate_source("#version 450\nvoid main() { this is not glsl }").unwrap_err();
        assert!(err.contains("GLSL parse error"), "{err}");
    }
}
