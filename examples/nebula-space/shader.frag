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

float rand1(float n) {
    return fract(sin(n * 12.9898) * 43758.5453);
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
    float value = 0.0;
    float amplitude = 0.5;
    for (int i = 0; i < 5; i++) {
        value += amplitude * noise(p);
        p *= 2.03;
        amplitude *= 0.5;
    }
    return value;
}

vec2 comet_position(float phase, float seed, float aspect) {
    float angle = rand1(seed + 1.0) * 6.2831853;
    vec2 dir = vec2(cos(angle), sin(angle));
    vec2 side = vec2(-dir.y, dir.x);
    vec2 start = vec2(
        mix(-0.8, aspect + 0.8, rand1(seed + 2.0)),
        mix(-0.4, 1.6, rand1(seed + 3.0))
    );
    float travel = mix(2.4, 4.2, rand1(seed + 4.0));
    float curl = (rand1(seed + 5.0) - 0.5) * 0.9;
    return start + dir * travel * phase + side * curl * sin(phase * 3.14159265);
}

vec3 comet(vec2 uv, float t, float seed, vec3 tint, float aspect) {
    float speed = mix(0.045, 0.14, rand1(seed + 6.0));
    float phase = fract(t * speed + rand1(seed + 7.0));
    vec2 head = comet_position(phase, seed, aspect);
    vec3 glow = vec3(0.0);

    for (int i = 0; i < 12; i++) {
        float f = float(i) / 11.0;
        float past = phase - f * 0.085;
        float valid = step(0.0, past);
        vec2 p = comet_position(max(past, 0.0), seed, aspect);
        float width = mix(52.0, 13.0, f);
        float strength = (1.0 - f) * (1.0 - f) * 0.11 * valid;
        glow += tint * exp(-length(uv - p) * width) * strength;
    }

    glow += tint * exp(-length(uv - head) * 42.0) * 1.5;
    float fade = smoothstep(0.0, 0.06, phase) * (1.0 - smoothstep(0.90, 1.0, phase));
    return glow * fade;
}

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution.y;
    float aspect = iResolution.x / iResolution.y;

    float speed = iParams[0].x;
    float density = iParams[1].x;
    float nebulaScale = iParams[2].x;
    float nebulaIntensity = iParams[3].x;
    float cometSpeed = iParams[4].x;
    vec3 deep = iParams[5].rgb;
    vec3 violet = iParams[6].rgb;
    vec3 cyan = iParams[7].rgb;

    float t = iTime * speed;

    vec3 color = mix(deep * 0.35, deep, smoothstep(-0.1, 1.0, uv.y));
    color += violet * 0.06 * exp(-length(uv - vec2(aspect * 0.7, 0.2)) * 1.6);

    vec2 nebulaUv = uv * nebulaScale;
    float n1 = fbm(nebulaUv + vec2(t * 0.08, -t * 0.05));
    float n2 = fbm(nebulaUv * 1.7 - vec2(t * 0.06, -t * 0.04) + n1);
    float nebula = smoothstep(0.38, 0.95, n1 * 0.65 + n2 * 0.55);
    vec3 nebulaColor = mix(violet, cyan, smoothstep(0.25, 0.8, n2));
    color += nebulaColor * nebula * nebulaIntensity * 0.55;

    vec2 starUv = uv * density;
    vec2 cell = floor(starUv);
    vec2 local = fract(starUv) - 0.5;
    float h = hash21(cell);
    float present = step(0.965, h);
    vec2 jitter = vec2(hash21(cell + 1.7), hash21(cell + 8.3)) - 0.5;
    float dist = length(local - jitter * 0.65);
    float radius = mix(0.025, 0.06, hash21(cell + 4.1));
    float star = present * (1.0 - smoothstep(0.0, radius, dist));
    float twinkle = 0.55 + 0.45 * sin(iTime * 2.4 + h * 42.0);
    color += vec3(0.85, 0.9, 1.0) * star * twinkle;

    float cometTime = iTime * (0.5 + cometSpeed * 6.0);
    color += comet(uv, cometTime, 11.0, cyan, aspect);
    color += comet(uv, cometTime, 29.0, mix(violet, vec3(1.0), 0.35), aspect);
    color += comet(uv, cometTime, 47.0, vec3(1.0, 0.78, 0.86), aspect);
    color += comet(uv, cometTime, 83.0, vec3(0.65, 0.9, 1.0), aspect);
    color += comet(uv, cometTime, 101.0, mix(cyan, vec3(1.0), 0.4), aspect);

    float vignette = 1.0 - smoothstep(0.45, 1.15, length(uv - vec2(aspect * 0.5, 0.5)));
    color *= mix(0.55, 1.0, vignette);

    outColor = vec4(color, 1.0);
}
