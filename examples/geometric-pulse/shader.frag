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

float sdBox(vec2 p, vec2 b) {
    vec2 d = abs(p) - b;
    return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0);
}

mat2 rot(float a) {
    float c = cos(a);
    float s = sin(a);
    return mat2(c, -s, s, c);
}

void main() {
    vec2 uv = (gl_FragCoord.xy * 2.0 - iResolution.xy) / iResolution.y;

    float speed = iParams[0].x;
    float ringCount = iParams[1].x;
    float rotSpeed = iParams[2].x;
    float pulseIntensity = iParams[3].x;
    vec3 bgColor = iParams[4].rgb;
    vec3 ringColor = iParams[5].rgb;
    vec3 pulseColor = iParams[6].rgb;
    vec3 accentColor = iParams[7].rgb;

    float t = iTime * speed;

    vec3 color = bgColor;

    float dist = length(uv);
    float angle = atan(uv.y, uv.x);

    float pulseWave = sin(dist * 6.0 - t * 3.0) * 0.5 + 0.5;
    pulseWave = pow(pulseWave, 3.0);
    color += pulseColor * pulseWave * exp(-dist * 2.5) * pulseIntensity * 0.3;

    for (int i = 0; i < 20; i++) {
        if (float(i) >= ringCount) break;
        float fi = float(i);
        float radius = 0.1 + fi * 0.12;

        float ringPulse = sin(t * 2.0 - fi * 0.5) * 0.5 + 0.5;
        float actualRadius = radius + ringPulse * 0.02 * pulseIntensity;

        float ring = abs(dist - actualRadius);
        float thickness = 0.003 + ringPulse * 0.002;
        float ringMask = smoothstep(thickness, 0.0, ring);

        float segments = 6.0 + fi * 2.0;
        float segAngle = angle + t * rotSpeed * (mod(fi, 2.0) == 0.0 ? 1.0 : -1.0);
        float seg = cos(segAngle * segments) * 0.5 + 0.5;
        seg = smoothstep(0.3, 0.7, seg);

        vec3 rColor = mix(ringColor, accentColor, fi / ringCount);
        rColor = mix(rColor, pulseColor, ringPulse * 0.3);

        color += rColor * ringMask * seg * (0.5 + ringPulse * 0.5);
    }

    for (int i = 0; i < 6; i++) {
        float fi = float(i);
        float lineAngle = fi * 3.14159265 / 3.0 + t * rotSpeed * 0.5;
        vec2 dir = vec2(cos(lineAngle), sin(lineAngle));
        float lineDist = abs(dot(uv, vec2(-dir.y, dir.x)));
        float line = smoothstep(0.003, 0.0, lineDist) * smoothstep(1.5, 0.1, dist);
        color += accentColor * line * 0.15;
    }

    float centerGlow = exp(-dist * 8.0);
    float centerPulse = sin(t * 2.5) * 0.5 + 0.5;
    color += mix(pulseColor, accentColor, centerPulse) * centerGlow * 0.6;

    vec2 boxUv = uv * rot(t * rotSpeed * 0.3);
    float box1 = abs(sdBox(boxUv, vec2(0.15 + sin(t) * 0.03)));
    float boxLine = smoothstep(0.004, 0.0, box1);
    color += accentColor * boxLine * 0.4;

    vec2 boxUv2 = uv * rot(-t * rotSpeed * 0.2 + 0.785);
    float box2 = abs(sdBox(boxUv2, vec2(0.25 + cos(t * 0.7) * 0.04)));
    float boxLine2 = smoothstep(0.003, 0.0, box2);
    color += ringColor * boxLine2 * 0.3;

    float vignette = 1.0 - smoothstep(0.3, 1.3, dist);
    color *= mix(0.3, 1.0, vignette);

    outColor = vec4(color, 1.0);
}
