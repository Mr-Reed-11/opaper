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

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution.y;
    float aspect = iResolution.x / iResolution.y;

    float speed = iParams[0].x;
    float gridSpeed = iParams[1].x;
    float sunSize = iParams[2].x;
    float glow = iParams[3].x;
    vec3 skyTop = iParams[4].rgb;
    vec3 skyHorizon = iParams[5].rgb;
    vec3 sunColor = iParams[6].rgb;
    vec3 gridColor = iParams[7].rgb;

    float t = iTime * speed;
    float horizon = 0.42;

    vec3 color = mix(skyHorizon, skyTop, smoothstep(horizon, 1.0, uv.y));

    vec2 starUv = uv * 120.0;
    vec2 cell = floor(starUv);
    vec2 local = fract(starUv) - 0.5;
    float h = hash21(cell);
    float present = step(0.97, h) * step(horizon, uv.y);
    vec2 jitter = vec2(hash21(cell + 2.3), hash21(cell + 9.1)) - 0.5;
    float dist = length(local - jitter * 0.7);
    float star = present * (1.0 - smoothstep(0.0, 0.045, dist));
    color += vec3(0.9, 0.95, 1.0) * star * (0.5 + 0.5 * sin(iTime * 2.0 + h * 30.0));

    vec2 sunCenter = vec2(aspect * 0.5, horizon + sunSize * 0.82);
    float sunDist = length(uv - sunCenter);
    float sunMask = 1.0 - smoothstep(sunSize - 0.006, sunSize, sunDist);
    float bands = step(0.42, fract((sunCenter.y - uv.y) * 26.0 - t * 0.6));
    float bandArea = step(uv.y, sunCenter.y);
    float sunStripes = mix(1.0, bands, bandArea);
    vec3 sunShade = mix(sunColor, skyHorizon, smoothstep(sunCenter.y - sunSize, sunCenter.y + sunSize, uv.y));
    color += sunShade * sunMask * sunStripes;
    color += sunColor * exp(-sunDist * 4.5) * glow * 0.45;

    float below = horizon - uv.y;
    float floorMask = step(0.0, below);
    color = mix(color, skyTop * 0.18, floorMask * 0.9);

    float z = 0.16 / max(below, 0.001);
    float gx = (uv.x - aspect * 0.5) * z * 1.6;
    float fx = fract(gx);
    float dx = min(fx, 1.0 - fx);
    float lineX = 1.0 - smoothstep(0.0, 0.025, dx);
    float hz = z - t * gridSpeed * 3.0;
    float fh = fract(hz);
    float dh = min(fh, 1.0 - fh);
    float lineZ = 1.0 - smoothstep(0.0, 0.035, dh);
    float fade = exp(-z * 0.10);
    color += gridColor * (lineX + lineZ) * fade * floorMask * glow;
    color += gridColor * 0.05 * exp(-below * 10.0) * floorMask;

    color += skyHorizon * exp(-abs(uv.y - horizon) * 70.0) * 0.5;

    float vignette = 1.0 - smoothstep(0.5, 1.25, length(uv - vec2(aspect * 0.5, 0.5)));
    color *= mix(0.6, 1.0, vignette);

    outColor = vec4(color, 1.0);
}
