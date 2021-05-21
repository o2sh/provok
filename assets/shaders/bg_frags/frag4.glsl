#version 330
precision mediump float;

uniform float time;

in vec4 o_position;
out vec4 color;

#define OCTAVES  32.0

float rand2(vec2 co){
   return fract(cos(dot(co.xy ,vec2(12.9898,78.233))) * 4.5453);
}

float valueNoiseSimple(vec2 vl) {
   float minStep = 1.0 ;

   vec2 grid = floor(vl);
   vec2 gridPnt1 = grid;
   vec2 gridPnt2 = vec2(grid.x, grid.y + minStep);
   vec2 gridPnt3 = vec2(grid.x + minStep, grid.y);
   vec2 gridPnt4 = vec2(gridPnt3.x, gridPnt2.y);

    float s = rand2(gridPnt1);
    float t = rand2(gridPnt3);
    float u = rand2(gridPnt2);
    float v = rand2(gridPnt4);

    float x1 = smoothstep(0., 1., fract(vl.x));
    float interpX1 = mix(s, t, x1);
    float interpX2 = mix(u, v, x1);

    float y = smoothstep(0., 1., fract(vl.y));
    float interpY = mix(interpX1, interpX2, y);

    return interpY;
}

float fractalNoise(vec2 vl) {
    float persistance = 2.0;
    float amplitude = 0.9;
    float rez = 0.0;
    vec2 p = vl;

    for (float i = 0.0; i < OCTAVES; i++) {
        rez += amplitude * valueNoiseSimple(p);
        amplitude /= persistance;
        p *= persistance;
    }
    return rez;
}

float complexFBM(vec2 p) {
    const float NUM_FREQS = 32.0;
    float slow = time / 16.;
    float fast = time / 2.;
    vec2 offset1 = vec2(slow  , 0.);
    vec2 offset2 = vec2(sin(fast)* 0.1, 0.);

    return
        (1. + 0.35) *
        fractalNoise( p + offset1 + fractalNoise(
            p + fractalNoise(
                p + 2. * fractalNoise(p - offset2)
            )
        )
        );
}


void main() {
    vec2 uv = vec2(o_position.x, o_position.y);

    vec3 colour1 = vec3(0.0, 0.1, 0.3);
    vec3 colour2 = vec3(1.0, 0.7, 0.5);

    vec3 rez = mix(colour1, colour2, complexFBM(uv) * 1.5 + uv.y * 0.8 - sin(time * 0.1) * 0.5 - 0.7);

	color = vec4(rez, 1.0);
}