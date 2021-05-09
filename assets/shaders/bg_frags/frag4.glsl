#version 330
precision mediump float;

uniform float time;

in vec4 o_position;
out vec4 color;

#define R_FACTOR 5.
#define G_FACTOR 0.
#define B_FACTOR 0.

const int MAXITER = 42;

vec3 field(vec3 p) {
    p *= .1;
    float f = .1;
    for (int i = 0; i < 5; i++) {
        p = p.yzx*mat3(.8,.6,0,-.6,.8,0,0,0,1);
        p += vec3(.123,.456,.789)*float(i);
        p = abs(fract(p)-.5);
        p *= 2.0;
        f *= 2.0;
    }
    p *= p;
    return sqrt(p+p.yzx)/f-.002;
}

void main() {
    vec3 dir = vec3(o_position.x, o_position.y, 1.);
    vec3 pos = vec3(.4, .5,time);
    vec3 c = vec3(0);
    for (int i = 0; i < MAXITER; i++) {
        vec3 f2 = field(pos);
        float f = min(min(f2.x,f2.y),f2.z);
        
        pos += dir*f;
        c += float(MAXITER-i)/(f2+.001);
    }
    vec3 color3 = vec3(1.-1./(1.+c*(.09/float(MAXITER*MAXITER))));
    color3 *= color3;
    color = vec4(color3.r*R_FACTOR, color3.g*G_FACTOR, color3.b*B_FACTOR,1.);
}
