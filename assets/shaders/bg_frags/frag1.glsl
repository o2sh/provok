#version 330
precision mediump float;

uniform float time;

in vec4 o_position;
out vec4 color;

#define octaves 11

float random (in vec2 st) {
    return fract(sin(dot(st.xy, vec2(12.9898,78.233)))* 43758.5453123);
}

float noise (in vec2 st) {
    vec2 i = floor(st);
    vec2 f = fract(st);

    float a = random(i);
    float b = random(i + vec2(1.0, 0.0));
    float c = random(i + vec2(0.0, 1.0));
    float d = random(i + vec2(1.0, 1.0));

    vec2 u = f * f * (3.0 - 2.0 * f);

    return mix(a, b, u.x) +
            (c - a)* u.y * (1.0 - u.x) +
            (d - b) * u.x * u.y;
}

float fbm (in vec2 p) {
    float value = 0.0;
    float freq = 1.0;
    float amp = 0.5;    

    for (int i = 0; i < octaves; i++) {
        value += amp * (noise((p - vec2(1.0)) * freq));
        freq *= 1.9;
        amp *= 0.6;
    }
    return value;
}

float pattern(in vec2 p) {
    vec2 offset = vec2(-0.5);

    vec2 aPos = vec2(sin(time * 0.05), sin(time * 0.1)) * 6.;
    vec2 aScale = vec2(3.0); 
    float a = fbm(p * aScale + aPos);

    vec2 bPos = vec2(sin(time * 0.01), sin(time * 0.01)) * 1.;
    vec2 bScale = vec2(0.5); 
    float b = fbm((p + a) * bScale + bPos);
    
    vec2 cPos = vec2(-0.6, -0.5) + vec2(sin(-time * 0.001), sin(time * 0.01)) * 2.;
    vec2 cScale = vec2(3.); 
    float c = fbm((p + b) * cScale + cPos);
    return c;
}


vec3 palette(in float t) {
    vec3 a = vec3(3, 0.5, 0.5);
    vec3 b = vec3(0.45, 0.25, 0.14);
    vec3 c = vec3(1.0 ,1.0, 1.0);
    vec3 d = vec3(0.0, 0.1, 0.2);
    return a + b * cos(9.28318 * (c * t + d));
}

void main() {
    vec2 p = vec2(o_position.x, o_position.y);
    float value = pow(pattern(p), 2.);
    vec3 c = palette(value);
    color = vec4(c, 1.0);
}