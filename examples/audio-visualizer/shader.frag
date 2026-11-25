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

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution.xy;

    float barCount = iParams[0].x;
    float sensitivity = iParams[1].x;
    vec3 colorBass = iParams[2].rgb;
    vec3 colorMid = iParams[3].rgb;
    vec3 colorTreble = iParams[4].rgb;
    vec3 bgColor = iParams[5].rgb;

    float level = iAudio.x * sensitivity;

    float barWidth = 1.0 / barCount;
    float barIdx = floor(uv.x / barWidth);
    float barCenter = (barIdx + 0.5) * barWidth;
    float inBar = smoothstep(barWidth * 0.42, barWidth * 0.35, abs(uv.x - barCenter));

    float t = barIdx / barCount;
    vec3 barColor = mix(colorBass, colorMid, smoothstep(0.0, 0.5, t));
    barColor = mix(barColor, colorTreble, smoothstep(0.5, 1.0, t));

    float barHeight = clamp(level, 0.01, 0.92);

    float barMask = inBar * smoothstep(barHeight + 0.005, barHeight - 0.01, uv.y);

    vec3 col = bgColor;
    col = mix(col, barColor, barMask);

    outColor = vec4(col, 1.0);
}
