#version 330
precision highp float;

uniform float time;

in vec4 o_position;
out vec4 color;

#define rot(a) mat2(cos(a),sin(a),-sin(a),cos(a))
#define pi radians(180.)
#define STEPS 150.0
#define MDIST 400.0
#define pmod(p,x) (mod(p, x) - (x)*0.5)
#define rotR 0.01

float box(vec3 p, vec3 b) {
  vec3 q = abs(p) - b;
  return length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0);
}

vec3 glow = vec3(0.);

vec2 map(vec3 p) {
    float t = mod(time, 63.0) + 15.0;
    float t2 = t*1.5;
    t2 = fract(t2)*fract(t2)+floor(t2);
    
    vec3 po = p;
    vec3 p2 = p;
    
    //Center Cubes
    p2.z -= t2*100.0;
    
    float id2 = floor(p2.z/50.0);
    p2.z = pmod(p2.z,50.0);
    
    vec3 ccol = vec3(0);
    
    if(mod(id2,3.0)==0.0) ccol = vec3(1.000,0.059,0.082);
    if(mod(id2,3.0)==1.0) ccol = vec3(0.392,0.102,1.000);
    if(mod(id2,3.0)==2.0) ccol = vec3(0.118,0.224,0.992);
    p2.yx*=rot(0.5*pi*t2*sign(sin(id2*pi+0.1)));
    
    float scl = 0.7;
    
    p2 = abs(p2)-(4.5*abs(1.0+sin(id2*20.0)))*scl;
    p2.xz*=rot(pi/4.0);
    float cb = box(p2,vec3(2.5*scl));
    p2 = abs(p2)-5.0*scl;
    cb = min(cb,box(p2,vec3(2.5*scl)));
    
    glow+=vec3(0.0085/(0.01+cb*cb))*ccol;

    vec2 a = vec2(cb,1.0);

    //Outer Cubes
    p.yx*=rot(p.z*rotR);
    float modd = 12.0;
    vec2 id = floor(p.xz/modd);
    p.z+=t*50.0*(mod(id.x,2.0)*2.0-1.0);
    p.y = abs(p.y)-30.0;
    p.xz=pmod(p.xz,modd);
    p.xy*=rot(t);
    vec2 b = vec2(box(p, vec3(3.7)),2.0);
    a = (a.x<b.x)?a:b;
    glow+=vec3(0.01/(0.01+b.x*b.x));
    
    return a*vec2(0.5,1.0);
}

void main() {
    vec2 uv = vec2(o_position.x, o_position.y);
    vec3 col = vec3(0);
    float t = mod(time,63.0)*2.0+7.5;
    
    vec2 off = vec2(30,-3.0)*rot(t*30.0*rotR);
    
    vec3 ro = vec3(off,t*30.0);
    vec3 lk = vec3(0,0,9999.0);
    vec3 f = normalize(lk-ro);
    vec3 ra = normalize(cross(vec3(0,1,0),f));
    vec3 rd = f*1.0 + uv.x*ra + uv.y*cross(f,ra);
    float dO = 0.0;
    vec3 p = ro;
    vec2 d;

    for(float i = 0.0; i<STEPS; i++) {
        p = ro+rd*dO;
        d = map(p);
        dO += d.x;
        if(d.x < 0.001 || dO > MDIST) {
            break;
        }
    }

    vec3 mix1 = mix(vec3(0.012,0.180,0.745),vec3(0.502,0.000,0.690),sin(p.z*0.2));
    
    if(d.y==2.0){
        glow*=mix1;
        col+=glow*0.12;
    }

    if(d.y==1.0){
        col+=glow*0.2;
    }

    col = mix(col, vec3(0.000,0.161,0.459), clamp(dO/MDIST,0.0,1.0));
    color = vec4(col, 1.0);
}