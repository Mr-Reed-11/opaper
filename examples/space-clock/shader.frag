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

float hash(float n) { return fract(sin(n) * 43758.5453); }

float starField(vec2 uv, float scale) {
    vec2 cell = floor(uv * scale);
    vec2 f = fract(uv * scale);
    float h = hash(dot(cell, vec2(127.1, 311.7)));
    if (h < 0.96) return 0.0;
    float brightness = smoothstep(0.96, 1.0, h);
    float twinkle = sin(iTime * (1.5 + h * 3.0) + h * 100.0) * 0.3 + 0.7;
    return brightness * twinkle * smoothstep(0.4, 0.0, length(f - 0.5));
}

int digitBits(int d) {
    if (d == 0) return 31599;
    if (d == 1) return 9362;
    if (d == 2) return 29671;
    if (d == 3) return 29647;
    if (d == 4) return 23497;
    if (d == 5) return 31183;
    if (d == 6) return 31215;
    if (d == 7) return 29257;
    if (d == 8) return 31727;
    if (d == 9) return 31695;
    return 0;
}

float drawPx(vec2 p, vec2 c, float ps) {
    vec2 d = max(abs(p - c) - ps * 0.4, vec2(0.0));
    return smoothstep(ps * 0.12, 0.0, length(d));
}

float drawDigit(vec2 p, int d, float ps) {
    if (p.x < 0.0 || p.x >= 3.0 * ps || p.y < 0.0 || p.y >= 5.0 * ps) return 0.0;
    int cx = int(p.x / ps);
    int ry = 4 - int(p.y / ps);
    if (cx > 2) cx = 2;
    if (ry < 0) ry = 0;
    int bits = digitBits(d);
    int idx = 14 - (ry * 3 + cx);
    if (((bits >> idx) & 1) == 0) return 0.0;
    vec2 center = vec2((float(cx) + 0.5) * ps, (float(4 - ry) + 0.5) * ps);
    return drawPx(p, center, ps);
}

float drawColon(vec2 p, float ps) {
    if (p.x < 0.0 || p.x >= ps || p.y < 0.0 || p.y >= 5.0 * ps) return 0.0;
    float a = drawPx(p, vec2(0.5 * ps, 3.5 * ps), ps);
    float b = drawPx(p, vec2(0.5 * ps, 1.5 * ps), ps);
    return max(a, b);
}

void main() {
    vec2 uv = (gl_FragCoord.xy * 2.0 - iResolution.xy) / iResolution.y;

    vec3 ringCol = iParams[2].rgb;
    vec3 glowCol = iParams[3].rgb;
    vec3 clockCol = iParams[4].rgb;

    float level = iAudio.x;
    float bass = iAudio.y;
    float mid = iAudio.z;
    float treble = iAudio.w;

    // --- Space background ---
    vec3 col = vec3(0.01, 0.01, 0.04);
    float nebula1 = sin(uv.x * 2.0 + iTime * 0.02) * cos(uv.y * 1.5 - iTime * 0.015);
    float nebula2 = cos(uv.x * 1.2 - iTime * 0.03) * sin(uv.y * 2.5 + iTime * 0.01);
    col += vec3(0.06, 0.01, 0.1) * (nebula1 * 0.5 + 0.5) * 0.2;
    col += vec3(0.01, 0.04, 0.1) * (nebula2 * 0.5 + 0.5) * 0.15;

    vec2 starUV = gl_FragCoord.xy / iResolution.y;
    col += vec3(0.8, 0.85, 1.0) * starField(starUV, 80.0);
    col += vec3(0.9, 0.9, 1.0) * starField(starUV + 50.0, 150.0) * 0.6;
    col += vec3(1.0, 0.9, 0.8) * starField(starUV + 100.0, 250.0) * 0.4;

    // --- Circular visualizer ---
    float dist = length(uv);
    float angle = atan(uv.y, uv.x);

    float innerR = 0.3;
    float maxBarH = 0.2;
    float barCount = 80.0;
    float sector = 2.0 * PI / barCount;

    float barIdx = floor((angle + PI) / sector);
    float barAngle = (barIdx + 0.5) * sector - PI;

    float angleDiff = abs(angle - barAngle);
    if (angleDiff > PI) angleDiff = 2.0 * PI - angleDiff;
    float inBar = smoothstep(sector * 0.38, sector * 0.28, angleDiff);

    float h = hash(barIdx * 127.53);
    float barH = level * (0.5 + h * 0.5) * maxBarH;
    barH = max(barH, 0.006);

    float radial = dist - innerR;

    // Segmented bars
    float segH = 0.012;
    float segGap = 0.005;
    float segUnit = segH + segGap;
    float segIdx = floor(radial / segUnit);
    float segFrac = fract(radial / segUnit);
    float inSeg = step(0.0, radial) * step(radial, barH) * step(segFrac * segUnit, segH);

    float barMask = inBar * inSeg;

    float t = radial / maxBarH;
    vec3 barColor = mix(ringCol, glowCol, clamp(t, 0.0, 1.0));

    col = mix(col, barColor, barMask);

    // Glow around active bars
    if (dist > innerR && dist < innerR + barH + 0.03) {
        float glowR = abs(dist - (innerR + barH));
        float glow = inBar * exp(-glowR * 40.0) * level * 0.25;
        col += glowCol * glow;
    }

    // --- Inner circle ---
    float faceMask = smoothstep(innerR + 0.002, innerR - 0.005, dist);
    col = mix(col, vec3(0.02, 0.02, 0.06), faceMask * 0.97);

    // Thin ring border
    float ring = smoothstep(innerR + 0.004, innerR + 0.001, dist)
               - smoothstep(innerR + 0.001, innerR - 0.002, dist);
    col += ringCol * ring * 0.6;

    // Audio pulse on ring
    col += glowCol * ring * level * 0.4;

    // --- Digital clock HH:MM ---
    float totalSec = iDate.w;
    int ih = int(mod(totalSec / 3600.0, 24.0));
    int im = int(mod(totalSec / 60.0, 60.0));

    float ps = 0.028;
    float dw = 3.0 * ps;
    float cw = 1.0 * ps;
    float gap = 1.8 * ps;
    float totalW = 4.0 * dw + 1.0 * (cw + gap) + 3.0 * gap;
    float dh = 5.0 * ps;

    vec2 tOrig = vec2(-totalW / 2.0, -dh / 2.0);
    vec2 tp = uv - tOrig;

    if (tp.x >= 0.0 && tp.x <= totalW && tp.y >= 0.0 && tp.y <= dh) {
        float ta = 0.0;
        float xo = 0.0;
        ta = max(ta, drawDigit(tp - vec2(xo, 0.0), ih / 10, ps)); xo += dw + gap;
        ta = max(ta, drawDigit(tp - vec2(xo, 0.0), ih - (ih / 10) * 10, ps)); xo += dw + gap;
        float blink = step(0.0, sin(iTime * PI));
        ta = max(ta, drawColon(tp - vec2(xo, 0.0), ps) * blink); xo += cw + gap;
        ta = max(ta, drawDigit(tp - vec2(xo, 0.0), im / 10, ps)); xo += dw + gap;
        ta = max(ta, drawDigit(tp - vec2(xo, 0.0), im - (im / 10) * 10, ps));

        vec3 textCol = mix(clockCol, glowCol, level * 0.3);
        col = mix(col, textCol, ta * 0.95);

        // Text glow
        col += textCol * ta * 0.1;
    }

    // Vignette
    float vig = 1.0 - length(uv) * 0.5;
    col *= clamp(vig, 0.3, 1.0);

    outColor = vec4(col, 1.0);
}
