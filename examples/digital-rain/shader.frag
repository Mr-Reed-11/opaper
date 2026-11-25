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

float char_pattern(vec2 uv, float seed) {
    vec2 grid = floor(uv * vec2(3.0, 5.0));
    float h = hash21(grid + seed * 17.3);
    return step(0.42, h);
}

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution.xy;
    float aspect = iResolution.x / iResolution.y;

    float speed = iParams[0].x;
    float density = iParams[1].x;
    float glowAmt = iParams[2].x;
    float fadeLength = iParams[3].x;
    vec3 bgColor = iParams[4].rgb;
    vec3 rainColor = iParams[5].rgb;
    vec3 headColor = iParams[6].rgb;
    vec3 accent = iParams[7].rgb;

    float t = iTime * speed;
    vec3 color = bgColor;

    float columns = density * aspect;
    float rows = density * 2.0;

    for (int layer = 0; layer < 3; layer++) {
        float fl = float(layer);
        float layerScale = 1.0 - fl * 0.25;
        float layerSpeed = 1.0 - fl * 0.2;
        float layerAlpha = 1.0 - fl * 0.35;

        vec2 cellUv = uv * vec2(columns * layerScale, rows * layerScale);
        float col = floor(cellUv.x);

        float colHash = hash21(vec2(col, fl + 100.0));
        float streamSpeed = (0.5 + colHash * 0.8) * layerSpeed;
        float streamOffset = colHash * 50.0;

        float fall = t * streamSpeed * 3.0 + streamOffset;
        float row = floor(cellUv.y - fall);
        vec2 local = fract(cellUv - vec2(0.0, fall));

        float headPos = fract(fall / (rows * layerScale)) * rows * layerScale;
        float distFromHead = cellUv.y - headPos;
        float trail = smoothstep(fadeLength * rows * layerScale * 0.5, 0.0, distFromHead);
        trail *= step(0.0, distFromHead);

        float charSeed = hash21(vec2(col, row) + fl * 7.0);
        float flicker = step(0.97, hash21(vec2(col, row + floor(t * 8.0)) + fl));
        charSeed = mix(charSeed, hash21(vec2(col, row + 999.0)), flicker);

        float ch = char_pattern(local, charSeed);
        float brightness = trail * ch * layerAlpha;

        float headDist = abs(distFromHead);
        float isHead = smoothstep(1.5, 0.0, headDist) * ch;

        vec3 streamColor = mix(rainColor * 0.4, rainColor, trail);
        streamColor = mix(streamColor, headColor, isHead * 0.8);

        color += streamColor * brightness;
        color += rainColor * isHead * glowAmt * 0.15;
    }

    color += accent * 0.08 * (1.0 - uv.y);

    float scanline = sin(gl_FragCoord.y * 1.5) * 0.03;
    color -= scanline;

    float vignette = 1.0 - smoothstep(0.4, 1.3, length((uv - 0.5) * vec2(aspect, 1.0)));
    color *= mix(0.3, 1.0, vignette);

    outColor = vec4(max(color, 0.0), 1.0);
}
