#version 450

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform Globals {
    float iTime;
    float iTimeDelta;
    uint iFrame;
    float _pad;
    vec4 iResolution;
    vec4 iParams[8];
};

float hash11(float p) {
    return fract(sin(p * 127.1) * 43758.5453);
}

float hash21(vec2 p) {
    p = fract(p * vec2(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    float a = hash21(i);
    float b = hash21(i + vec2(1.0, 0.0));
    float c = hash21(i + vec2(0.0, 1.0));
    float d = hash21(i + vec2(1.0, 1.0));
    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

float treeSilhouette(vec2 uv, float aspect) {
    float ground = 0.15 + noise(vec2(uv.x * 2.0, 0.0)) * 0.08;
    float treeMask = 0.0;

    for (int i = 0; i < 8; i++) {
        float fi = float(i);
        float tx = hash11(fi * 7.3) * aspect;
        float tw = 0.03 + hash11(fi * 11.1) * 0.02;
        float th = 0.3 + hash11(fi * 17.9) * 0.35;

        float trunk = smoothstep(tw, 0.0, abs(uv.x - tx)) * step(uv.y, ground + th);
        float canopyH = ground + th;
        float canopyW = 0.06 + hash11(fi * 3.3) * 0.08;
        float canopy = smoothstep(canopyW, 0.0, length((uv - vec2(tx, canopyH)) * vec2(1.0, 1.8)));
        treeMask = max(treeMask, max(trunk, canopy));
    }

    float groundMask = smoothstep(ground + 0.01, ground - 0.01, uv.y);
    return max(treeMask, groundMask);
}

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution.y;
    float aspect = iResolution.x / iResolution.y;

    float speed = iParams[0].x;
    float fireflyCount = iParams[1].x;
    float glowSize = iParams[2].x;
    float fogAmt = iParams[3].x;
    vec3 skyColor = iParams[4].rgb;
    vec3 groundColor = iParams[5].rgb;
    vec3 fireflyWarm = iParams[6].rgb;
    vec3 fireflyCool = iParams[7].rgb;

    float t = iTime * speed;

    vec3 color = mix(groundColor * 0.3, skyColor, smoothstep(0.0, 0.8, uv.y));

    vec2 starUv = uv * 100.0;
    vec2 cell = floor(starUv);
    vec2 local = fract(starUv) - 0.5;
    float h = hash21(cell);
    float present = step(0.97, h) * smoothstep(0.3, 0.5, uv.y);
    float starDist = length(local - (vec2(hash21(cell + 2.0), hash21(cell + 8.0)) - 0.5) * 0.6);
    color += vec3(0.7, 0.8, 1.0) * present * (1.0 - smoothstep(0.0, 0.03, starDist)) * (0.5 + 0.5 * sin(iTime + h * 40.0));

    float moon = 1.0 - smoothstep(0.04, 0.045, length(uv - vec2(aspect * 0.8, 0.75)));
    color += vec3(0.9, 0.95, 1.0) * moon * 0.6;
    color += vec3(0.4, 0.5, 0.7) * exp(-length(uv - vec2(aspect * 0.8, 0.75)) * 8.0) * 0.15;

    float trees = treeSilhouette(uv, aspect);
    color = mix(color, vec3(0.01, 0.02, 0.01), trees);

    float fogNoise = noise(vec2(uv.x * 3.0 + t * 0.05, uv.y * 2.0));
    float fogMask = smoothstep(0.0, 0.4, uv.y) * (1.0 - smoothstep(0.3, 0.6, uv.y));
    color += vec3(0.1, 0.15, 0.1) * fogNoise * fogMask * fogAmt;

    int count = int(fireflyCount);
    for (int i = 0; i < 60; i++) {
        if (i >= count) break;
        float fi = float(i);
        float seed = fi * 17.31;

        vec2 basePos = vec2(
            hash11(seed) * aspect,
            hash11(seed + 100.0) * 0.5
        );
        vec2 drift = vec2(
            sin(t * (0.3 + hash11(seed + 1.0) * 0.4) + fi * 2.0),
            sin(t * (0.2 + hash11(seed + 2.0) * 0.3) + fi * 1.5 + 1.0)
        ) * 0.05;
        vec2 pos = basePos + drift;

        float pulse = sin(t * (1.5 + hash11(seed + 3.0) * 2.0) + fi * 3.0);
        float brightness = smoothstep(-0.3, 0.8, pulse);
        brightness *= brightness;

        float d = length(uv - pos);
        float glow = exp(-d * (40.0 / glowSize)) * brightness;

        vec3 fColor = mix(fireflyWarm, fireflyCool, hash11(seed + 4.0));
        color += fColor * glow * 0.5;
        color += fColor * exp(-d * (120.0 / glowSize)) * brightness * 0.8;
    }

    float vignette = 1.0 - smoothstep(0.4, 1.2, length(uv - vec2(aspect * 0.5, 0.5)));
    color *= mix(0.4, 1.0, vignette);

    outColor = vec4(color, 1.0);
}
