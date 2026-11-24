# Arquitetura

## Fluxo

```
app (Vue/Tauri) --invoke--> Rust do app --D-Bus--> engine --wgpu--> layer-shell BACKGROUND
```

O frontend não renderiza nada e não fala diretamente com a engine. Ele apenas chama comandos Tauri (`list_outputs`, `set_wallpaper`, `pause`, `resume`, `quit`), que usam um proxy zbus blocking para `org.opaper.Engine`.

## Engine

- `smithay-client-toolkit` gerencia registry, outputs e layer surfaces.
- Cada output recebe uma `LayerSurface` em `Layer::Background`, ancorada nos quatro lados.
- `wgpu` usa um triângulo fullscreen e um fragment shader GLSL 450 compilado via naga.
- `Globals` é um uniform buffer com tempo, frame, resolução e até 8 parâmetros.
- `calloop` integra eventos Wayland, canal de comandos e sinais SIGTERM/SIGINT.
- O manifest é validado contra o schema embutido antes da leitura do shader.

## Identidade

`identity.json` é a única fonte de:

- `productName` e `identifier` do `tauri.conf.json`;
- nome do binário/pacote/pasta `opaper`;
- arquivo `packaging/br.com.nortixcode.opaper.desktop`.

O script `scripts/generate-identity.mjs` regenera os arquivos derivados.

## Tray

O menu é mínimo:

- **Abrir biblioteca**: mostra e foca a janela principal;
- **Pausar/Retomar**: chama `Pause`/`Resume` na engine;
- **Sair**: chama `Quit` na engine e encerra a interface.

Fechar a janela com o botão de fechar chama `prevent_close()` e esconde a janela.

## Ícones

A API de tray do Tauri 2 aceita ICO/PNG, não SVG nem nomes de ícones simbólicos. Os ícones do app são gerados a partir da arte final em `packaging/icons/opaper-1024.png`; o tray usa uma versão monocromática em PNG derivada da mesma arte.
