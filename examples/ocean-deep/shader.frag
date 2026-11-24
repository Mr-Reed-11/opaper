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

float caustic(vec2 p, float t) {
    float v = 0.0;
    vec2 p1 = p + vec2(t * 0.3, t * 0.2);
    vec2 p2 = p * 1.4 - vec2(t * 0.2, t * 0.35);
    vec2 p3 = p * 0.8 + vec2(t * 0.15, -t * 0.1);

    v += sin(p1.x * 3.0 + sin(p1.y * 2.5 + t)) * 0.5 + 0.5;
    v += sin(p2.y * 2.8 + sin(p2.x * 3.2 - t * 0.7)) * 0.5 + 0.5;
    v += sin((p3.x + p3.y) * 2.0 + t * 0.5) * 0.5 + 0.5;

    v = v / 3.0;
    v = pow(v, 2.5);
    return v;
}

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution.y;
    float aspect = iResolution.x / iResolution.y;
    vec2 uvn = gl_FragCoord.xy / iResolution.xy;

    float speed = iParams[0].x;
    float causticScale = iParams[1].x;
    float causticIntensity = iParams[2].x;
    float particleAmt = iParams[3].x;
    vec3 deepColor = iParams[4].rgb;
    vec3 midColor = iParams[5].rgb;
    vec3 lightColor = iParams[6].rgb;
    vec3 causticColor = iParams[7].rgb;

    float t = iTime * speed;

    float depth = uvn.y;
    vec3 color = mix(deepColor, midColor, smoothstep(0.0, 0.5, 1.0 - depth));
    color = mix(color, lightColor * 0.3, smoothstep(0.5, 1.0, 1.0 - depth) * 0.4);

    float lightRay = exp(-depth * 2.5);
    vec2 cuv = uv * causticScale;
    float c1 = caustic(cuv, t);
    float c2 = caustic(cuv * 1.3 + 3.7, t * 0.8);
    float combined = (c1 + c2) * 0.5;
    color += causticColor * combined * causticIntensity * lightRay;

    float rays = 0.0;
    for (int i = 0; i < 4; i++) {
        float fi = float(i);
        float rx = uvn.x + sin(t * 0.1 + fi * 1.5) * 0.15;
        float ray = exp(-abs(rx - (0.2 + fi * 0.2)) * 15.0);
        ray *= (1.0 - depth) * 0.5;
        rays += ray;
    }
    color += lightColor * rays * 0.15;

    for (int i = 0; i < 20; i++) {
        float fi = float(i);
        float seed = fi * 7.31;
        vec2 ppos = vec2(
            fract(hash21(vec2(seed, 1.0)) + t * 0.02 * (hash21(vec2(seed, 5.0)) - 0.5)),
            fract(hash21(vec2(seed, 2.0)) - t * 0.03 * hash21(vec2(seed, 3.0)))
        );
        ppos.x *= aspect;
        float size = mix(0.001, 0.004, hash21(vec2(seed, 4.0)));
        float pdist = length(uv - ppos);
        float particle = exp(-pdist / size) * particleAmt;
        float ptwinkle = 0.6 + 0.4 * sin(t * 3.0 + fi * 2.0);
        color += vec3(0.6, 0.85, 1.0) * particle * ptwinkle * 0.3;
    }

    float wave = sin(uvn.x * 8.0 + t * 0.5) * sin(uvn.x * 5.0 - t * 0.3);
    float surface = smoothstep(0.95, 1.0, 1.0 - depth + wave * 0.02);
    color += lightColor * surface * 0.3;

    float vignette = 1.0 - smoothstep(0.4, 1.2, length(uvn - 0.5) * 1.4);
    color *= mix(0.4, 1.0, vignette);

    outColor = vec4(color, 1.0);
}
