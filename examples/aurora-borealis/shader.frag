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

float fbm(vec2 p) {
    float v = 0.0;
    float a = 0.5;
    for (int i = 0; i < 6; i++) {
        v += a * noise(p);
        p *= 2.1;
        a *= 0.48;
    }
    return v;
}

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution.y;
    float aspect = iResolution.x / iResolution.y;

    float speed = iParams[0].x;
    float intensity = iParams[1].x;
    float curtainScale = iParams[2].x;
    float starDensity = iParams[3].x;
    vec3 skyDark = iParams[4].rgb;
    vec3 skyHorizon = iParams[5].rgb;
    vec3 auroraGreen = iParams[6].rgb;
    vec3 auroraPurple = iParams[7].rgb;

    float t = iTime * speed;

    vec3 color = mix(skyHorizon, skyDark, smoothstep(0.0, 0.9, uv.y));

    vec2 starUv = uv * starDensity;
    vec2 cell = floor(starUv);
    vec2 local = fract(starUv) - 0.5;
    float h = hash21(cell);
    float present = step(0.96, h);
    vec2 jitter = vec2(hash21(cell + 2.3), hash21(cell + 9.1)) - 0.5;
    float dist = length(local - jitter * 0.6);
    float star = present * (1.0 - smoothstep(0.0, 0.04, dist));
    float twinkle = 0.5 + 0.5 * sin(iTime * 2.5 + h * 50.0);
    color += vec3(0.85, 0.9, 1.0) * star * twinkle;

    float auroraY = smoothstep(0.25, 0.75, uv.y) * smoothstep(1.0, 0.75, uv.y);

    for (int i = 0; i < 3; i++) {
        float fi = float(i);
        float offset = fi * 1.3;
        vec2 auv = vec2(uv.x * curtainScale + t * 0.3 + offset, uv.y * 2.0 + fi * 0.5);

        float n = fbm(auv + vec2(t * 0.15, t * 0.08));
        float wave = sin(uv.x * curtainScale * 2.0 + t + n * 4.0 + fi * 2.0) * 0.5 + 0.5;

        float curtain = wave * auroraY;
        curtain *= smoothstep(0.3, 0.6, n);
        curtain = pow(curtain, 1.5);

        float blend = sin(uv.x * 1.5 + t * 0.2 + fi) * 0.5 + 0.5;
        vec3 auroraColor = mix(auroraGreen, auroraPurple, blend);
        auroraColor = mix(auroraColor, vec3(1.0), pow(curtain, 3.0) * 0.4);

        color += auroraColor * curtain * intensity * 0.35;
    }

    color += skyHorizon * 0.15 * exp(-abs(uv.y - 0.08) * 15.0);

    float vignette = 1.0 - smoothstep(0.4, 1.2, length(uv - vec2(aspect * 0.5, 0.5)));
    color *= mix(0.5, 1.0, vignette);

    outColor = vec4(color, 1.0);
}
