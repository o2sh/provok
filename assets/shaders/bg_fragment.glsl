#version 330
precision highp float;

in vec4 o_position;

uniform float time;
out vec4 color;

void main() {
    float i0 = 1.0;
    float i1 = 1.0;
    float i2 = 1.0;
    float i4 = 0.0;
    
    vec2 uv = vec2(o_position.x, o_position.y);

    for (int s = 0; s < 7; s++) 
    {
        vec2 r = vec2(
        cos(uv.y * i0 - i4 + time / i1),
        sin(uv.x * i0 - i4 + time / i1)) / i2;
        
        r += vec2(-r.y, r.x) * 0.3;    
        uv += r;
        
        i0 *= 1.93;
        i1 *= 1.15;
        i2 *= 1.7;
        i4 += 0.05 + 0.1 * time * i1;
    }
    
    float r = sin(uv.x - time) * 0.5 + 0.5;
    float b = sin(uv.y + time) * 0.5 + 0.5;
    float g = sin((uv.x + uv.y + sin(time * 0.5)) * 0.5) * 0.5 + 0.5;
    
    color = vec4(r, g, b, 1.0);
}