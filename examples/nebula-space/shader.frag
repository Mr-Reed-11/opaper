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

vec3 comet(vec2 uv, float t, float seed, vec3 tint) {
    float cycle = fract(t + seed);
    vec2 head = mix(vec2(1.9, 1.25), vec2(-0.5, -0.3), cycle);
    vec2 motion = normalize(vec2(-0.85, -0.52));
    vec2 side = vec2(-motion.y, motion.x);
    vec2 rel = uv - head;
    float along = dot(rel, -motion);
    float across = abs(dot(rel, side));
    float core = exp(-length(rel) * 26.0);
    float tail = exp(-across * 46.0) * exp(-along * 5.5) * step(0.0, along);
    float fade = sin(cycle * 3.14159265);
    return tint * (core * 1.8 + tail * 0.75) * fade;
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
    float star = present * smoothstep(radius, 0.0, dist);
    float twinkle = 0.55 + 0.45 * sin(iTime * 2.4 + h * 42.0);
    color += vec3(0.85, 0.9, 1.0) * star * twinkle;

    color += comet(uv, iTime * cometSpeed, 0.0, cyan);
    color += comet(uv, iTime * cometSpeed, 0.37, mix(violet, vec3(1.0), 0.35));
    color += comet(uv, iTime * cometSpeed, 0.71, vec3(1.0, 0.78, 0.86));

    float vignette = 1.0 - smoothstep(0.45, 1.15, length(uv - vec2(aspect * 0.5, 0.5)));
    color *= mix(0.55, 1.0, vignette);

    outColor = vec4(color, 1.0);
}
