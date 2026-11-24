# Formato opaper v1

O schema normativo é `schema/manifest.schema.json` (JSON Schema draft 2020-12).

## Convenções

- `schema_version` deve ser `1`.
- `type` é `shader` no M1. O valor `scene` está reservado para o M2.
- `shader` é o nome do arquivo GLSL dentro da pasta do wallpaper.
- `params` é um objeto cujas chaves são os nomes dos parâmetros.
- Cada parâmetro declara `slot` de `0` a `7`; o shader lê esse mesmo índice em `iParams[slot]`.
- Tipos de parâmetro suportados: `float` e `color` (`#RRGGBB`).

## Uniformes

```glsl
layout(set = 0, binding = 0) uniform Globals {
    float iTime;
    float iTimeDelta;
    uint iFrame;
    float _pad;
    vec4 iResolution;
    vec4 iParams[8];
};
```

`iResolution.xy` é o tamanho do output em pixels. `iParams` contém os valores padrão dos parâmetros declarados no manifest.

## Exemplo completo

Veja `../examples/plasma-waves/`.
