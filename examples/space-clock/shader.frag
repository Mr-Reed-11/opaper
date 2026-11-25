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

#define PI 3.14159265
#define TAU 6.28318530

float hash(vec2 p) {
    p = fract(p * vec2(233.34, 851.74));
    p += dot(p, p + 23.45);
    return fract(p.x * p.y);
}

float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1, 0)), f.x),
               mix(hash(i + vec2(0, 1)), hash(i + vec2(1, 1)), f.x), f.y);
}

float fbm(vec2 p) {
    float v = 0.0, a = 0.5;
    for (int i = 0; i < 5; i++) { v += a * noise(p); p *= 2.1; a *= 0.5; }
    return v;
}

float segLine(vec2 p, vec2 a, vec2 b, float w) {
    vec2 pa = p - a, ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return smoothstep(w, 0.0, length(pa - ba * h));
}

float digit7(vec2 p, int d, float size) {
    float w = size * 0.06;
    float s = size;
    float h = size * 1.8;
    float m = size * 0.08;

    vec2 tl = vec2(m, h - m);
    vec2 tr = vec2(s - m, h - m);
    vec2 ml = vec2(m, h * 0.5);
    vec2 mr = vec2(s - m, h * 0.5);
    vec2 bl = vec2(m, m);
    vec2 br = vec2(s - m, m);

    float v = 0.0;
    if (d != 1 && d != 4)                     v = max(v, segLine(p, tl, tr, w));
    if (d != 5 && d != 6)                     v = max(v, segLine(p, tr, mr, w));
    if (d != 2)                               v = max(v, segLine(p, mr, br, w));
    if (d != 1 && d != 4 && d != 7)           v = max(v, segLine(p, bl, br, w));
    if (d == 0 || d == 2 || d == 6 || d == 8) v = max(v, segLine(p, bl, ml, w));
    if (d != 1 && d != 2 && d != 3 && d != 7) v = max(v, segLine(p, tl, ml, w));
    if (d != 0 && d != 1 && d != 7)           v = max(v, segLine(p, ml, mr, w));
    return v;
}

float colon7(vec2 p, float size) {
    float h = size * 1.8;
    float r = size * 0.06;
    float d1 = length(p - vec2(size * 0.2, h * 0.65));
    float d2 = length(p - vec2(size * 0.2, h * 0.35));
    return max(smoothstep(r, 0.0, d1), smoothstep(r, 0.0, d2));
}

void main() {
    vec2 uv = (gl_FragCoord.xy * 2.0 - iResolution.xy) / iResolution.y;

    vec3 ringCol = iParams[2].rgb;
    vec3 glowCol = iParams[3].rgb;
    vec3 clockCol = iParams[4].rgb;

    float level = iAudio.x;

    // --- Nebula ---
    vec3 col = vec3(0.005, 0.005, 0.02);
    vec2 np = uv * 1.2;
    col += vec3(0.15, 0.03, 0.25) * smoothstep(0.25, 0.65, fbm(np + iTime * 0.008)) * 0.7;
    col += vec3(0.03, 0.1, 0.3) * smoothstep(0.3, 0.7, fbm(np * 0.7 + 5.2 + iTime * 0.006)) * 0.5;
    col += vec3(0.05, 0.02, 0.08) * fbm(uv * 2.5 + iTime * 0.003);

    // --- Stars ---
    vec2 starUV = gl_FragCoord.xy / 3.0;
    vec2 cell = floor(starUV);
    vec2 f = fract(starUV) - 0.5;
    float sh = hash(cell);
    if (sh > 0.98) {
        float b = (sh - 0.98) * 50.0;
        float tw = sin(iTime * (1.0 + sh * 3.0) + sh * 50.0) * 0.2 + 0.8;
        col += b * tw * exp(-dot(f, f) * 80.0) * vec3(0.85, 0.9, 1.0);
    }

    // --- Circular rays (rotating) ---
    float dist = length(uv);
    float angle = atan(uv.y, uv.x);
    float rotation = iTime * 0.12;
    float rotAngle = angle - rotation;

    float innerR = 0.2;
    float rayCount = 140.0;
    float sector = TAU / rayCount;

    float rayIdx = floor((rotAngle + PI) / sector);
    float rayAngle = (rayIdx + 0.5) * sector - PI;
    float aDiff = abs(rotAngle - rayAngle);
    if (aDiff > PI) aDiff = TAU - aDiff;

    float rayW = 0.0008 + dist * 0.0003;
    float inRay = smoothstep(rayW, rayW * 0.2, aDiff * dist);

    float rh = hash(vec2(rayIdx * 73.156, rayIdx * 19.73));
    float baseH = 0.08 + rh * 0.25;
    float audioH = level * (0.3 + rh * 0.5);
    float rayH = baseH + audioH;

    float radial = dist - innerR;
    float rayMask = inRay * step(0.0, radial) * smoothstep(rayH, rayH - 0.008, radial);

    float fade = 1.0 - smoothstep(0.0, rayH, radial) * 0.7;
    vec3 rayCol = mix(ringCol, glowCol, clamp(radial / 0.3, 0.0, 1.0));

    col += rayCol * rayMask * fade * 0.9;

    if (radial > 0.0 && radial < rayH + 0.05) {
        float gd = max(radial - rayH, 0.0);
        col += glowCol * inRay * exp(-gd * 25.0) * (0.1 + level * 0.2);
    }

    // --- Inner circle ---
    float faceMask = smoothstep(innerR + 0.003, innerR - 0.01, dist);
    col = mix(col, vec3(0.008, 0.008, 0.025), faceMask * 0.98);

    float ring = smoothstep(innerR + 0.004, innerR, dist)
               - smoothstep(innerR, innerR - 0.004, dist);
    col += ringCol * ring * 0.3;
    col += glowCol * ring * level * 0.4;

    // --- 7-segment clock HH:MM ---
    float totalSec = iDate.w;
    int ih = int(mod(totalSec / 3600.0, 24.0));
    int im = int(mod(totalSec / 60.0, 60.0));

    float sz = 0.055;
    float dgap = sz * 0.25;
    float cw = sz * 0.4;
    float totalW = 4.0 * sz + 2.0 * dgap + cw + 2.0 * dgap;
    float totalH = sz * 1.8;

    vec2 tp = uv - vec2(-totalW / 2.0, -totalH / 2.0);

    if (tp.x >= -0.03 && tp.x <= totalW + 0.03 && tp.y >= -0.03 && tp.y <= totalH + 0.03) {
        float ta = 0.0;
        float xo = 0.0;
        ta = max(ta, digit7(tp - vec2(xo, 0.0), ih / 10, sz)); xo += sz + dgap;
        ta = max(ta, digit7(tp - vec2(xo, 0.0), ih - (ih / 10) * 10, sz)); xo += sz + dgap;
        float blink = step(0.0, sin(iTime * PI));
        ta = max(ta, colon7(tp - vec2(xo, 0.0), sz) * blink); xo += cw + dgap;
        ta = max(ta, digit7(tp - vec2(xo, 0.0), im / 10, sz)); xo += sz + dgap;
        ta = max(ta, digit7(tp - vec2(xo, 0.0), im - (im / 10) * 10, sz));

        vec3 textCol = mix(clockCol, vec3(1.0), 0.2);
        col = mix(col, textCol, ta);
        col += textCol * ta * 0.15;
    }

    col *= 1.0 - length(uv) * 0.25;

    outColor = vec4(col, 1.0);
}
