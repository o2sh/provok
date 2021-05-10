#version 330
precision mediump float;

uniform float time;

in vec4 o_position;
out vec4 color;

#define MOD3 vec3(.1031,.11369,.13787)
#define grad(x) length(vec2(dFdx(x),dFdy(x)))

vec3 hash33(vec3 p3) {
	p3 = fract(p3 * MOD3);
    p3 += dot(p3, p3.yxz+19.19);
    return -1.0 + 2.0 * fract(vec3((p3.x + p3.y)*p3.z, (p3.x+p3.z)*p3.y, (p3.y+p3.z)*p3.x));
}

float perlin_noise(vec3 p) {
    vec3 pi = floor(p);
    vec3 pf = p - pi;
    
    vec3 w = pf * pf * (3.0 - 2.0 * pf);
    
    return 	mix(
        		mix(
                	mix(dot(pf - vec3(0, 0, 0), hash33(pi + vec3(0, 0, 0))), 
                        dot(pf - vec3(1, 0, 0), hash33(pi + vec3(1, 0, 0))),
                       	w.x),
                	mix(dot(pf - vec3(0, 0, 1), hash33(pi + vec3(0, 0, 1))), 
                        dot(pf - vec3(1, 0, 1), hash33(pi + vec3(1, 0, 1))),
                       	w.x),
                	w.z),
        		mix(
                    mix(dot(pf - vec3(0, 1, 0), hash33(pi + vec3(0, 1, 0))), 
                        dot(pf - vec3(1, 1, 0), hash33(pi + vec3(1, 1, 0))),
                       	w.x),
                   	mix(dot(pf - vec3(0, 1, 1), hash33(pi + vec3(0, 1, 1))), 
                        dot(pf - vec3(1, 1, 1), hash33(pi + vec3(1, 1, 1))),
                       	w.x),
                	w.z),
    			w.y);
}

float fbm(vec3 p) {
    float r = 0.;
    float m = 0.5;
    for (int i=0; i<5;i++){
        r += perlin_noise(p) * m;
        p *= 2.;
        m *= 0.5;
    }
    return r;
}

vec3 hsv2rgb(vec3 c) {
  vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
  return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

void main(void) {
    vec2 uv = vec2(o_position.x, o_position.y);
    float d = length(uv);
    float n = fbm(vec3(uv*0.2,time*0.025) * 5.)*0.5 + 0.5;
    d += n*2.;
    d = pow(d,5.);
    float e = d+.5;
    int m = int(e-.5);
    e = abs(fract(e)-.5)/grad(e);
    float h = fbm(vec3(float(m)*13480.09134,1026.7392,7492.1264)+time*.1)*3.;
    color = vec4(hsv2rgb(vec3(h,1.,1.)),1.) * vec4(clamp(e,0.,1.));
}