#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
EXAMPLES_DIR="$PROJECT_ROOT/examples"
PLUGIN_DIR="$SCRIPT_DIR/org.opaper.wallpaper"
SHADERS_DIR="$PLUGIN_DIR/contents/shaders"
QSB="${QSB:-/usr/lib/qt6/bin/qsb}"

if ! command -v "$QSB" &>/dev/null; then
    echo "qsb not found. Install qt6-shadertools."
    exit 1
fi

mkdir -p "$SHADERS_DIR"

QT_HEADER='#version 440

layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;

layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    float iTime;
    float iTimeDelta;
    float iFrame;
    float _pad;
    vec4 iResolution;
    vec4 iAudio;
    vec4 iDate;
    vec4 iP0;
    vec4 iP1;
    vec4 iP2;
    vec4 iP3;
    vec4 iP4;
    vec4 iP5;
    vec4 iP6;
    vec4 iP7;
};'

PARAMS_REBUILD='    vec4 iParams[8] = vec4[8](iP0, iP1, iP2, iP3, iP4, iP5, iP6, iP7);'

convert_shader() {
    local input="$1"
    local output="$2"

    python3 -I -c "
import sys, re

with open(sys.argv[1]) as f:
    src = f.read()

# Remove original header (version, layout out, uniform block)
src = re.sub(r'#version\s+\d+\s*\n', '', src)
src = re.sub(r'layout\s*\(location\s*=\s*0\)\s*out\s+vec4\s+outColor\s*;', '', src)
src = re.sub(
    r'layout\s*\([^)]*\)\s*uniform\s+Globals\s*\{[^}]*\}\s*;',
    '',
    src,
    flags=re.DOTALL,
)

# Replace outColor with fragColor
src = src.replace('outColor', 'fragColor')

# Insert iParams rebuild after void main() {
src = re.sub(
    r'(void\s+main\s*\(\s*\)\s*\{)',
    r'\1\n' + sys.argv[3],
    src,
)

# Add qt_Opacity multiply before the last closing brace
lines = src.rstrip().rsplit('}', 1)
src = lines[0] + '    fragColor.a *= qt_Opacity;\n}' + (lines[1] if len(lines) > 1 else '')

# Prepend Qt header
src = sys.argv[2] + '\n' + src.strip() + '\n'

with open(sys.argv[4], 'w') as f:
    f.write(src)
" "$input" "$QT_HEADER" "$PARAMS_REBUILD" "$output"
}

ok=0
fail=0

for dir in "$EXAMPLES_DIR"/*/; do
    name="$(basename "$dir")"
    shader="$dir/shader.frag"
    [ -f "$shader" ] || continue

    qt_shader="$SHADERS_DIR/${name}.frag"
    qsb_file="$SHADERS_DIR/${name}.frag.qsb"

    echo -n "  $name ... "

    if convert_shader "$shader" "$qt_shader" && \
       "$QSB" --glsl "440,300 es" -o "$qsb_file" "$qt_shader" 2>/dev/null; then
        echo "ok"
        rm -f "$qt_shader"
        ok=$((ok + 1))
    else
        echo "FAIL"
        rm -f "$qt_shader" "$qsb_file"
        fail=$((fail + 1))
    fi
done

echo ""
echo "Built: $ok  Failed: $fail"

echo ""
echo "Installing plugin to ~/.local/share/plasma/wallpapers/ ..."
install_dir="$HOME/.local/share/plasma/wallpapers/org.opaper.wallpaper"
mkdir -p "$(dirname "$install_dir")"
rm -rf "$install_dir"
cp -r "$PLUGIN_DIR" "$install_dir"
echo "Done. Restart plasmashell or log out/in to load the plugin."
