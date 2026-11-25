#version 450

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform Globals {
    float iTime;
    float iTimeDelta;
    uint iFrame;
    float _pad;
    vec4 iResolution;
    vec4 iAudio;
    vec4 iDate;
    vec4 iParams[8];
};

void main() {
    vec2 uv = (gl_FragCoord.xy * 2.0 - iResolution.xy) / iResolution.y;

    float speed = iParams[0].x;
    float scale = iParams[1].x;
    vec3 colorA = iParams[2].rgb;
    vec3 colorB = iParams[3].rgb;

    float t = iTime * speed;
    float wave = sin(uv.x * scale + t) * 0.5 + 0.5;
    wave += sin(uv.y * scale * 0.7 - t * 0.8) * 0.25;
    wave += sin((uv.x + uv.y) * scale * 1.3 + t * 1.4) * 0.25;
    wave = clamp(wave * 0.5, 0.0, 1.0);

    vec3 color = mix(colorA, colorB, wave);
    color += pow(wave, 6.0) * 0.35;

    outColor = vec4(color, 1.0);
}
