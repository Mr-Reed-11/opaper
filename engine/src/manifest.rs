use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

const SCHEMA_JSON: &str = include_str!("../../format/schema/manifest.schema.json");

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WallpaperKind {
    Shader {
        #[serde(default = "default_shader_file")]
        shader: String,
    },
    Scene {
        #[serde(default)]
        layers: Vec<Layer>,
    },
}

fn default_shader_file() -> String {
    "shader.frag".to_owned()
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Layer {
    pub image: String,
    #[serde(default)]
    pub mask: Option<String>,
    #[serde(default)]
    pub depth: Option<String>,
    #[serde(default)]
    pub flow: Option<String>,
    #[serde(default)]
    pub effects: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Param {
    pub r#type: String,
    pub default: serde_json::Value,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub min: Option<f32>,
    #[serde(default)]
    pub max: Option<f32>,
}

impl Param {
    /// Value packed into the corresponding `iParams[i]` slot.
    pub fn as_uniform(&self) -> Result<[f32; 4], String> {
        match self.r#type.as_str() {
            "float" => {
                let value = self.default.as_f64().ok_or("float param needs a numeric default")?;
                Ok([value as f32, 0.0, 0.0, 1.0])
            }
            "color" => {
                let hex = self.default.as_str().ok_or("color param needs a hex string")?;
                parse_hex_color(hex)
            }
            other => Err(format!("unsupported param type: {other}")),
        }
    }
}

fn parse_hex_color(hex: &str) -> Result<[f32; 4], String> {
    let raw = hex.strip_prefix('#').unwrap_or(hex);
    if raw.len() != 6 {
        return Err(format!("invalid color '{hex}': expected #RRGGBB"));
    }
    let mut out = [0.0; 4];
    for (i, chunk) in raw.as_bytes().chunks(2).enumerate() {
        let text = std::str::from_utf8(chunk).map_err(|_| format!("invalid color '{hex}'"))?;
        out[i] = u8::from_str_radix(text, 16).map_err(|_| format!("invalid color '{hex}'"))?
            as f32
            / 255.0;
    }
    out[3] = 1.0;
    Ok(out)
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Manifest {
    pub schema_version: u32,
    pub name: String,
    pub author: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub params: BTreeMap<String, Param>,
    #[serde(flatten)]
    pub kind: WallpaperKind,
}

#[derive(Debug)]
pub struct ManifestError(pub String);

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ManifestError {}

/// Reads and validates a wallpaper folder.
pub fn load(dir: &Path) -> Result<(Manifest, PathBuf), ManifestError> {
    let path = dir.join("manifest.json");
    let raw = fs::read_to_string(&path)
        .map_err(|e| ManifestError(format!("cannot read {}: {e}", path.display())))?;
    let value: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| ManifestError(format!("invalid JSON in {}: {e}", path.display())))?;

    let schema: serde_json::Value = serde_json::from_str(SCHEMA_JSON)
        .map_err(|e| ManifestError(format!("bundled schema is invalid: {e}")))?;
    let validator = jsonschema::validator_for(&schema)
        .map_err(|e| ManifestError(format!("bundled schema cannot be compiled: {e}")))?;
    if !validator.is_valid(&value) {
        let details: Vec<String> = validator
            .iter_errors(&value)
            .take(3)
            .map(|e| format!("{}: {}", e.instance_path(), e))
            .collect();
        return Err(ManifestError(format!(
            "{} does not match the schema: {}",
            path.display(),
            details.join("; ")
        )));
    }

    let manifest: Manifest = serde_json::from_value(value)
        .map_err(|e| ManifestError(format!("cannot parse manifest: {e}")))?;
    if manifest.params.len() > 8 {
        return Err(ManifestError("at most 8 params are supported in v1".to_owned()));
    }
    for (name, param) in &manifest.params {
        param.as_uniform().map_err(|e| ManifestError(format!("param '{name}': {e}")))?;
    }

    let shader_path = match &manifest.kind {
        WallpaperKind::Shader { shader } => dir.join(shader),
        WallpaperKind::Scene { .. } => {
            return Err(ManifestError("scene wallpapers are not supported yet".to_owned()));
        }
    };
    if !shader_path.is_file() {
        return Err(ManifestError(format!("shader file not found: {}", shader_path.display())));
    }

    Ok((manifest, shader_path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples/plasma-waves")
    }

    #[test]
    fn parses_bundled_example() {
        let (manifest, shader) = load(&example_dir()).expect("example must be valid");
        assert_eq!(manifest.name, "Plasma Waves");
        assert_eq!(manifest.schema_version, 1);
        assert!(shader.ends_with("shader.frag"));
        assert_eq!(manifest.params["speed"].as_uniform().unwrap()[0], 0.6);
        assert_eq!(manifest.params["color_b"].as_uniform().unwrap()[2], 1.0);
    }

    #[test]
    fn rejects_unknown_fields() {
        let json = r#"{
            "schema_version": 1, "name": "x", "author": "x", "version": "1",
            "type": "shader", "shader": "shader.frag", "unknown": true
        }"#;
        let dir = std::env::temp_dir().join(format!("opaper-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("manifest.json"), json).unwrap();
        fs::write(dir.join("shader.frag"), "#version 450").unwrap();
        let err = load(&dir).unwrap_err().to_string();
        assert!(err.contains("does not match the schema"), "{err}");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn rejects_scene_until_milestone_two() {
        let json = r#"{
            "schema_version": 1, "name": "x", "author": "x", "version": "1",
            "type": "scene", "layers": [{"image": "a.png"}]
        }"#;
        let dir = std::env::temp_dir().join(format!("opaper-test-scene-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("manifest.json"), json).unwrap();
        let err = load(&dir).unwrap_err().to_string();
        assert!(err.contains("not supported yet"), "{err}");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn rejects_missing_shader_file() {
        let dir = std::env::temp_dir().join(format!("opaper-test-noshader-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("manifest.json"),
            r#"{"schema_version":1,"name":"x","author":"x","version":"1","type":"shader","shader":"missing.frag"}"#,
        )
        .unwrap();
        let err = load(&dir).unwrap_err().to_string();
        assert!(err.contains("shader file not found"), "{err}");
        fs::remove_dir_all(&dir).unwrap();
    }
}
