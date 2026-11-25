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

float sdLine(vec2 p, vec2 a, vec2 b) {
    vec2 pa = p - a, ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
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
    vec2 d = max(abs(p - c) - ps * 0.38, vec2(0.0));
    return smoothstep(ps * 0.15, 0.0, length(d));
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
    vec2 top = vec2(0.5 * ps, 3.5 * ps);
    vec2 bot = vec2(0.5 * ps, 1.5 * ps);
    return max(drawPx(p, top, ps), drawPx(p, bot, ps));
}

void main() {
    vec2 uv = (gl_FragCoord.xy * 2.0 - iResolution.xy) / iResolution.y;

    float clockSize = iParams[0].x;
    float handWidth = iParams[1].x;
    vec3 faceColor = iParams[2].rgb;
    vec3 handColor = iParams[3].rgb;
    vec3 secondColor = iParams[4].rgb;
    vec3 bgColor = iParams[5].rgb;
    vec3 accentColor = iParams[6].rgb;

    float totalSec = iDate.w;
    float hours = mod(totalSec / 3600.0, 12.0);
    float minutes = mod(totalSec / 60.0, 60.0);
    float seconds = mod(totalSec, 60.0);

    // Angles (clockwise from 12 o'clock)
    float hourAngle = PI / 2.0 - hours / 12.0 * 2.0 * PI;
    float minAngle = PI / 2.0 - minutes / 60.0 * 2.0 * PI;
    float secAngle = PI / 2.0 - seconds / 60.0 * 2.0 * PI;

    // Background
    float wave1 = sin(uv.x * 3.0 + iTime * 0.3) * 0.5 + 0.5;
    float wave2 = sin(uv.y * 2.5 - iTime * 0.2) * 0.5 + 0.5;
    vec3 bg = bgColor + accentColor * wave1 * wave2 * 0.08;
    bg *= 1.0 - length(uv) * 0.4;

    // Clock face
    float dist = length(uv);
    float face = smoothstep(clockSize + 0.003, clockSize - 0.003, dist);
    float rim = smoothstep(clockSize + 0.006, clockSize + 0.002, dist)
              - smoothstep(clockSize + 0.002, clockSize - 0.002, dist);

    vec3 col = mix(bg, faceColor, face);
    col += accentColor * rim * 0.6;

    // O(1) tick rendering via polar coordinates (replaces 72-iteration loops)
    if (dist > clockSize * 0.78 && dist < clockSize * 0.95) {
        float angle = atan(uv.y, uv.x);

        // Hour markers (every 30 degrees)
        float hourSector = PI / 12.0;
        float ha = mod(angle + PI / 2.0 + hourSector, 2.0 * hourSector) - hourSector;
        float hourTickDist = abs(sin(ha) * dist);
        float nearestHour = round(mod((-angle + PI / 2.0) / (2.0 * hourSector), 12.0));
        float hw = (mod(nearestHour, 3.0) == 0.0) ? 0.008 : 0.004;
        float hourMark = smoothstep(hw, hw - 0.002, hourTickDist);
        float inHourZone = step(clockSize * 0.82, dist);
        col = mix(col, handColor, hourMark * inHourZone * face);

        // Minute ticks (every 6 degrees, skip hour positions)
        float minSector = PI / 60.0;
        float ma = mod(angle + PI / 2.0 + minSector, 2.0 * minSector) - minSector;
        float minTickDist = abs(sin(ma) * dist);
        float nearestMin = round(mod((-angle + PI / 2.0) / (2.0 * minSector), 60.0));
        float isHourPos = step(0.5, 1.0 - abs(mod(nearestMin, 5.0)));
        float inMinZone = step(clockSize * 0.88, dist);
        float minMark = smoothstep(0.002, 0.001, minTickDist) * (1.0 - isHourPos);
        col = mix(col, handColor * 0.5, minMark * inMinZone * face);
    }

    // Hour hand
    vec2 hourDir = vec2(cos(hourAngle), sin(hourAngle));
    float hourDist = sdLine(uv, vec2(0.0), hourDir * clockSize * 0.5);
    col = mix(col, handColor, smoothstep(handWidth * 1.3, handWidth * 1.3 - 0.003, hourDist) * face);

    // Minute hand
    vec2 minDir = vec2(cos(minAngle), sin(minAngle));
    float minDist = sdLine(uv, vec2(0.0), minDir * clockSize * 0.72);
    col = mix(col, handColor, smoothstep(handWidth, handWidth - 0.002, minDist) * face);

    // Second hand
    vec2 secDir = vec2(cos(secAngle), sin(secAngle));
    float secDist = sdLine(uv, -secDir * clockSize * 0.15, secDir * clockSize * 0.78);
    col = mix(col, secondColor, smoothstep(0.004, 0.002, secDist) * face);

    // Center dot
    col = mix(col, secondColor, smoothstep(0.015, 0.012, dist) * face);

    // Digital time (HH:MM:SS)
    int ih = int(mod(totalSec / 3600.0, 24.0));
    int im = int(mod(totalSec / 60.0, 60.0));
    int isc = int(mod(totalSec, 60.0));

    float ps = 0.014;
    float dw = 3.0 * ps;
    float cw = 1.0 * ps;
    float gap = 1.5 * ps;
    float totalW = 6.0 * dw + 2.0 * (cw + gap) + 5.0 * gap;
    float dh = 5.0 * ps;

    vec2 tOrig = vec2(-totalW / 2.0, -(clockSize + 0.07 + dh));
    vec2 tp = uv - tOrig;

    // Early out: skip digit rendering if outside the text area
    if (tp.x >= 0.0 && tp.x <= totalW && tp.y >= 0.0 && tp.y <= dh) {
        float ta = 0.0;
        float xo = 0.0;
        ta = max(ta, drawDigit(tp - vec2(xo, 0.0), ih / 10, ps)); xo += dw + gap;
        ta = max(ta, drawDigit(tp - vec2(xo, 0.0), ih - (ih / 10) * 10, ps)); xo += dw + gap;
        ta = max(ta, drawColon(tp - vec2(xo, 0.0), ps)); xo += cw + gap;
        ta = max(ta, drawDigit(tp - vec2(xo, 0.0), im / 10, ps)); xo += dw + gap;
        ta = max(ta, drawDigit(tp - vec2(xo, 0.0), im - (im / 10) * 10, ps)); xo += dw + gap;
        ta = max(ta, drawColon(tp - vec2(xo, 0.0), ps)); xo += cw + gap;
        ta = max(ta, drawDigit(tp - vec2(xo, 0.0), isc / 10, ps)); xo += dw + gap;
        ta = max(ta, drawDigit(tp - vec2(xo, 0.0), isc - (isc / 10) * 10, ps));
        col = mix(col, handColor, ta * 0.9);
    }

    outColor = vec4(col, 1.0);
}
