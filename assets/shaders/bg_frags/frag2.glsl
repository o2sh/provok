#version 330
precision highp float;

uniform float time;

in vec4 o_position;
out vec4 color;

void main( void ) {
	vec2 uv = vec2(o_position.x, o_position.y);

	float i0=1.2;
	float i1=0.95;
	float i2=1.5;
	vec2 i4=vec2(0.0,0.0);
	for(int s=0;s<4;s++)
	{
		vec2 r;
		r=vec2(cos(uv.y*i0-i4.y+time/i1),sin(uv.x*i0+i4.x+time/i1))/i2;
		r+=vec2(-r.y,r.x)*0.5;
		uv.xy+=r;
        
		i0*=115.93;
		i1*=729.1;
		i2*=14.7;
		i4+=r.xy*1.0+0.5*time*i1;
	}
	float r=sin(uv.x-time)*0.5+0.5;
	float b=sin(uv.y+time)*0.5+0.5;
	float g=sin((sqrt(uv.x*uv.x+uv.y*uv.y)+time))*0.5+0.5;
	vec3 c=vec3(r,g,b);
	color = vec4(c,1.0);
}