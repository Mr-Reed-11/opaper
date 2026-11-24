# opaper

Alternativa open source ao Wallpaper Engine, começando por **KDE Plasma 6 / Wayland**.

Neste milestone 1 há uma engine Rust que desenha shaders GLSL em camadas `Bottom` do layer-shell (acima do wallpaper do Plasma) e uma interface Tauri 2 + Vue 3 para controlar a engine por D-Bus.

## Estrutura

- `engine/` — renderer wgpu + layer-shell, parser/validador do manifest e serviço D-Bus.
- `app/` — interface Tauri 2 + Vue 3 + TypeScript + Tailwind v4 + Lucide.
- `format/` — schema JSON do formato `opaper` v1.
- `examples/plasma-waves/`, `examples/nebula-space/`, `examples/synthwave-horizon/` — wallpapers de exemplo.
- `packaging/` — `.desktop`, ícones e arquivos de empacotamento.
- `docs/` — arquitetura e roadmap do milestone 2.
- `identity.json` + `scripts/generate-identity.mjs` — fonte única do identificador `br.com.nortixcode.opaper`.

## Dependências

### Arch / CachyOS

```sh
sudo pacman -S --needed rust nodejs npm pkg-config base-devel \
  wayland wayland-protocols libxkbcommon dbus \
  webkit2gtk-4.1 libayatana-appindicator
```

Instale a CLI do Tauri 2:

```sh
cargo install tauri-cli --version '^2' --locked
```

### Debian / Ubuntu

```sh
sudo apt install build-essential curl pkg-config libssl-dev file \
  libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev \
  libwayland-dev wayland-protocols libxkbcommon-dev libdbus-1-dev
```

## Build e teste

```sh
cargo test --manifest-path engine/Cargo.toml
cargo test --manifest-path app/src-tauri/Cargo.toml
cd app && npm install && npm run build
```

## Uso

Rode a engine com um wallpaper:

```sh
cargo run --manifest-path engine/Cargo.toml -- examples/nebula-space
```

Outros exemplos disponíveis são `examples/plasma-waves` e `examples/synthwave-horizon`.

Em outro terminal, abra a interface:

```sh
cd app && npm run tauri dev
```

A engine publica `org.opaper.Engine` em `/org/opaper/Engine` no barramento de sessão:

```sh
gdbus call --session \
  --dest org.opaper.Engine \
  --object-path /org/opaper/Engine \
  --method org.opaper.Engine.ListOutputs

gdbus call --session \
  --dest org.opaper.Engine \
  --object-path /org/opaper/Engine \
  --method org.opaper.Engine.SetWallpaper \
  /home/vitor/Projetos/outros/opaper/examples/plasma-waves
```

Métodos disponíveis: `SetWallpaper(path)`, `SetWallpaperOnOutput(path, output)`, `ListOutputs()`, `Pause()`, `Resume()` e `Quit()`.

Na interface, use **Monitor** para escolher um output específico ou **Todos os monitores**. Cada monitor pode ter um wallpaper diferente; basta selecionar o monitor, informar a pasta e clicar **Aplicar**.

A interface tem tray com **Abrir biblioteca**, **Pausar/Retomar** e **Sair**. Fechar a janela apenas a esconde; **Sair** encerra a interface e a engine.

## Formato

Um wallpaper é uma pasta com `manifest.json` e os arquivos referenciados por ele. O schema v1 está em `format/schema/manifest.schema.json`. Exemplo mínimo:

```json
{
  "schema_version": 1,
  "name": "Meu Wallpaper",
  "author": "você",
  "version": "1.0.0",
  "type": "shader",
  "shader": "shader.frag",
  "params": {
    "speed": { "type": "float", "slot": 0, "default": 0.5 }
  }
}
```

O fragment shader GLSL recebe:

- `float iTime`
- `float iTimeDelta`
- `uint iFrame`
- `vec4 iResolution`
- `vec4 iParams[8]`

Cada parâmetro do manifest deve declarar `slot` de `0` a `7`. Valores `float` ocupam `.x`; cores `#RRGGBB` ocupam `.rgb` com `.a = 1`.

## Limitações do M1

- Somente Plasma 6 com Wayland e compositor compatível com `zwlr_layer_shell_v1`.
- Somente wallpapers do tipo `shader`; `scene` está no schema para o M2, mas ainda é rejeitado na carga.
- Sem áudio, vídeo, páginas web, loja/workshop ou download da comunidade.
- Até 8 parâmetros por wallpaper.
- Ícones gerados a partir da arte final fornecida para `opaper`; o tray usa uma versão monocromática em PNG.
- A engine precisa estar rodando antes de usar os comandos da interface.

Veja `docs/ARCHITECTURE.md` e `docs/M2.md` para detalhes.
