<p align="center">
  <img src="app/src-tauri/icons/icon.png" width="120" alt="opaper logo" />
</p>

<h1 align="center">opaper</h1>

<p align="center">
  <strong>Animated wallpaper engine for Linux / Wayland</strong><br>
  Open-source alternative to Wallpaper Engine — GLSL shaders running natively on your desktop.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-000000?logo=rust" alt="Rust" />
  <img src="https://img.shields.io/badge/Wayland-FFB800?logo=wayland&logoColor=black" alt="Wayland" />
  <img src="https://img.shields.io/badge/Tauri_2-24C8DB?logo=tauri&logoColor=white" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/Vue_3-4FC08D?logo=vue.js&logoColor=white" alt="Vue 3" />
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="MIT" />
</p>

---

## What is opaper?

opaper renders GLSL fragment shaders as animated wallpapers on Linux desktops using the Wayland layer-shell protocol. It comes with a Tauri-based GUI for browsing and applying wallpapers, and a native KDE Plasma 6 plugin for seamless integration with desktop icons.

### Features

- **GPU-accelerated** — wgpu/Vulkan rendering at 60 fps
- **Click-through** — right-click menus, drag-select and desktop icons work normally
- **10 built-in wallpapers** — ready to use out of the box
- **KDE Plasma plugin** — optional native integration that keeps desktop icons visible
- **Per-monitor wallpapers** — set a different shader on each output
- **D-Bus IPC** — engine exposes `org.opaper.Engine` for scripting and automation
- **Hot-swap** — change wallpapers without restarting the engine
- **Wallpaper format** — simple `manifest.json` + `shader.frag`, up to 8 tweakable parameters

## Built-in wallpapers

| | | |
|---|---|---|
| Aurora Borealis | Digital Rain | Fireflies |
| Geometric Pulse | Lava Lamp | Nebula Space |
| Ocean Deep | Plasma Waves | Rainy Window |
| Synthwave Horizon | | |

Each wallpaper is a folder with a `manifest.json` and a GLSL fragment shader — easy to create your own.

## Quick start

### Dependencies

**Arch / CachyOS:**

```sh
sudo pacman -S --needed rust nodejs npm pkg-config base-devel \
  wayland wayland-protocols libxkbcommon dbus \
  webkit2gtk-4.1 libayatana-appindicator
```

**Debian / Ubuntu:**

```sh
sudo apt install build-essential curl pkg-config libssl-dev file \
  libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev \
  libwayland-dev wayland-protocols libxkbcommon-dev libdbus-1-dev
```

Also install the Tauri CLI:

```sh
cargo install tauri-cli --version '^2' --locked
```

### Install (Arch / CachyOS)

```sh
cd packaging && makepkg -si
```

This installs opaper system-wide with a desktop entry and icon. Launch it from your app menu or run `opaper-launch`.

### Install (any distro)

```sh
make
sudo make install
```

To uninstall: `sudo make uninstall`

### Run without installing

```sh
# Build
cargo build --manifest-path engine/Cargo.toml
cd app && npm install && npm run build && cd ..

# Terminal 1 — engine
cargo run --manifest-path engine/Cargo.toml -- examples/nebula-space

# Terminal 2 — GUI
cd app && npm run tauri dev
```

The GUI shows all wallpapers in a grid. Click one to apply it. Use the monitor selector to target a specific output.

### KDE Plasma plugin (optional)

For KDE Plasma 6 users — renders shaders inside Plasma's own wallpaper layer, so desktop icons stay visible:

```sh
# Requires qt6-shadertools
bash packaging/kde-wallpaper/build.sh
```

The engine auto-detects the plugin and switches to native mode on startup.

## Project structure

```
opaper/
├── engine/          Rust renderer — wgpu, layer-shell, D-Bus service
├── app/             Tauri 2 + Vue 3 + Tailwind CSS frontend
├── examples/        10 built-in wallpapers (manifest.json + shader.frag)
├── packaging/       KDE Plasma wallpaper plugin + build script
├── format/          Wallpaper format schema (JSON Schema v1)
└── docs/            Architecture docs and roadmap
```

## Creating your own wallpaper

Create a folder with two files:

**manifest.json:**

```json
{
  "schema_version": 1,
  "name": "My Wallpaper",
  "author": "you",
  "version": "1.0.0",
  "description": "A cool animated wallpaper",
  "type": "shader",
  "shader": "shader.frag",
  "params": {
    "speed": { "type": "float", "slot": 0, "default": 0.5, "min": 0.0, "max": 2.0, "label": "Speed" },
    "color": { "type": "color", "slot": 1, "default": "#4a90d9", "label": "Base Color" }
  }
}
```

**shader.frag** (GLSL 450):

```glsl
#version 450

layout(location = 0) out vec4 outColor;

layout(std140, binding = 0) uniform Globals {
    float iTime;
    float iTimeDelta;
    uint  iFrame;
    float _pad;
    vec4  iResolution;
    vec4  iParams[8];
};

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution.xy;
    float speed = iParams[0].x;
    vec3 col = iParams[1].rgb;
    col *= 0.5 + 0.5 * sin(iTime * speed + uv.x * 6.28);
    outColor = vec4(col, 1.0);
}
```

Place the folder in `examples/` or `~/.local/share/opaper/wallpapers/` and it will appear in the GUI.

## D-Bus API

The engine registers `org.opaper.Engine` on the session bus:

| Method | Description |
|---|---|
| `SetWallpaper(path)` | Apply wallpaper to all outputs |
| `SetWallpaperOnOutput(path, output)` | Apply to a specific output |
| `ListOutputs()` | List connected outputs |
| `Pause()` / `Resume()` | Pause or resume rendering |
| `Quit()` | Shut down the engine |

```sh
gdbus call --session -d org.opaper.Engine -o /org/opaper/Engine \
  -m org.opaper.Engine.SetWallpaper "$HOME/my-wallpaper"
```

## Requirements

- Linux with a Wayland compositor supporting `zwlr_layer_shell_v1`
- Vulkan-capable GPU
- KDE Plasma 6 (for the optional native plugin)

## License

MIT
