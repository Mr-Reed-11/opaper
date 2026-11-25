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

float hash11(float p) {
    return fract(sin(p * 127.1) * 43758.5453);
}

void main() {
    vec2 uv = (gl_FragCoord.xy * 2.0 - iResolution.xy) / iResolution.y;

    float speed = iParams[0].x;
    float blobCount = iParams[1].x;
    float smoothness = iParams[2].x;
    float glowAmt = iParams[3].x;
    vec3 bgColor = iParams[4].rgb;
    vec3 colorA = iParams[5].rgb;
    vec3 colorB = iParams[6].rgb;
    vec3 colorC = iParams[7].rgb;

    float t = iTime * speed;
    int count = int(blobCount);

    float field = 0.0;
    vec3 colorAccum = vec3(0.0);

    for (int i = 0; i < 12; i++) {
        if (i >= count) break;
        float fi = float(i);
        float h1 = hash11(fi * 13.7);
        float h2 = hash11(fi * 23.1);
        float h3 = hash11(fi * 37.9);

        vec2 center = vec2(
            sin(t * (0.3 + h1 * 0.4) + fi * 2.0) * (0.5 + h2 * 0.4),
            sin(t * (0.2 + h2 * 0.3) + fi * 1.7 + 1.0) * 0.8
        );

        float radius = 0.2 + h3 * 0.15;
        float d = length(uv - center);
        float blob = radius * smoothness / (d + 0.01);

        field += blob;

        vec3 blobColor = mix(colorA, colorB, h1);
        blobColor = mix(blobColor, colorC, h2 * 0.5);
        colorAccum += blobColor * blob;
    }

    colorAccum /= max(field, 0.01);

    float shape = smoothstep(1.8, 2.5, field);
    float edge = smoothstep(1.5, 2.0, field) * (1.0 - smoothstep(2.0, 2.5, field));

    vec3 color = mix(bgColor, colorAccum, shape);
    color += colorAccum * edge * glowAmt * 0.5;
    color += colorAccum * exp(-abs(field - 2.0) * 2.0) * glowAmt * 0.3;

    float bgGlow = 0.0;
    for (int i = 0; i < 12; i++) {
        if (i >= count) break;
        float fi = float(i);
        float h1 = hash11(fi * 13.7);
        float h2 = hash11(fi * 23.1);
        vec2 center = vec2(
            sin(t * (0.3 + h1 * 0.4) + fi * 2.0) * (0.5 + h2 * 0.4),
            sin(t * (0.2 + h2 * 0.3) + fi * 1.7 + 1.0) * 0.8
        );
        bgGlow += exp(-length(uv - center) * 2.0) * 0.08;
    }
    color += mix(colorA, colorB, 0.5) * bgGlow * glowAmt;

    float vignette = 1.0 - smoothstep(0.5, 1.5, length(uv));
    color *= mix(0.4, 1.0, vignette);

    outColor = vec4(color, 1.0);
}
