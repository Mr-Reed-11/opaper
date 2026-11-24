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

float hash21(vec2 p) {
    p = fract(p * vec2(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

float hash11(float p) {
    return fract(sin(p * 127.1) * 43758.5453);
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

vec2 raindrop(vec2 uv, float t, float seed) {
    float h1 = hash11(seed);
    float h2 = hash11(seed + 100.0);
    float h3 = hash11(seed + 200.0);

    float dropSpeed = 0.15 + h1 * 0.25;
    float startX = h2;
    float wobble = sin(t * 2.0 + seed * 5.0) * 0.01 * h3;

    float y = fract(1.0 - t * dropSpeed - h3);
    float x = startX + wobble + sin(y * 10.0 + seed) * 0.005;

    return vec2(x, y);
}

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution.xy;
    float aspect = iResolution.x / iResolution.y;

    float speed = iParams[0].x;
    float dropDensity = iParams[1].x;
    float blurAmount = iParams[2].x;
    float trailLength = iParams[3].x;
    vec3 bgDark = iParams[4].rgb;
    vec3 bgLight = iParams[5].rgb;
    vec3 dropColor = iParams[6].rgb;
    vec3 bokehColor = iParams[7].rgb;

    float t = iTime * speed;

    vec3 color = mix(bgDark, bgLight, noise(uv * 3.0 + t * 0.05) * 0.5);

    float bokehLayer = 0.0;
    for (int i = 0; i < 15; i++) {
        float fi = float(i);
        vec2 pos = vec2(
            hash11(fi * 7.3) * aspect,
            hash11(fi * 13.1)
        );
        float size = 0.03 + hash11(fi * 3.7) * 0.05;
        float pulse = 0.7 + 0.3 * sin(t * 0.5 + fi * 2.0);
        float d = length((uv - pos) * vec2(1.0, aspect));
        float bokeh = smoothstep(size, size * 0.3, d) * pulse;
        float hue = hash11(fi * 17.0);
        vec3 bColor = mix(bokehColor, dropColor, hue);
        color += bColor * bokeh * 0.15 * blurAmount;
        bokehLayer += bokeh;
    }

    int dropCount = int(dropDensity * 15.0);
    float dropField = 0.0;
    vec3 dropAccum = vec3(0.0);

    for (int i = 0; i < 60; i++) {
        if (i >= dropCount) break;
        float fi = float(i);
        float seed = fi * 31.7;
        vec2 drop = raindrop(uv, t, seed);

        float dx = (uv.x - drop.x) * aspect;
        float dy = uv.y - drop.y;

        float radius = 0.004 + hash11(seed + 50.0) * 0.006;
        float d = length(vec2(dx, dy));
        float dropShape = smoothstep(radius, radius * 0.2, d);

        float trailDy = dy;
        float trailMask = smoothstep(0.0, trailLength * 0.08, trailDy) * smoothstep(trailLength * 0.15, 0.0, trailDy);
        float trailWidth = smoothstep(radius * 2.0, 0.0, abs(dx));
        float trail = trailMask * trailWidth * 0.4;

        float brightness = dropShape + trail;
        dropField += brightness;

        float refract = exp(-d * 80.0) * 0.03;
        vec2 rUv = uv + vec2(dx, dy) * refract;
        float n = noise(rUv * 8.0 + t * 0.1);
        vec3 refractColor = mix(bgLight, dropColor, n * 0.5) * 1.5;

        dropAccum += refractColor * brightness;
    }

    color = mix(color, dropAccum / max(dropField, 1.0), min(dropField, 1.0));
    color += dropColor * dropField * 0.1;

    float foggy = noise(uv * 2.0 + vec2(t * 0.02, 0.0));
    color = mix(color, bgLight * 0.5, foggy * blurAmount * 0.2);

    float vignette = 1.0 - smoothstep(0.3, 1.0, length(uv - 0.5) * 1.5);
    color *= mix(0.3, 1.0, vignette);

    outColor = vec4(color, 1.0);
}
