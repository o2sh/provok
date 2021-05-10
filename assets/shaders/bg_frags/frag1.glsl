#version 330
precision highp float;

uniform float time;

in vec4 o_position;
out vec4 color;

#define OCTAVES 10

float random(vec2 st) {
	return fract(sin(dot(st.xy, vec2(12.9898,78.233)))*43758.5453123);
}

float noise(vec2 st) {
	vec2 i = floor(st);
	vec2 f = fract(st);
	
	float a = random(i + vec2(0.0, 0.0));
	float b = random(i + vec2(1.0, 0.0));
	float c = random(i + vec2(0.0, 1.0));
	float d = random(i + vec2(1.0, 1.0));
	
	vec2 u = f * f * (3.0 - 2.0 * f);
	
	float result = mix(a, b, u.x) + (c-a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
	
	return result;
	
}

float fbm(vec2 st) {
	float value = 0.0;
	float amplitude = .5;
	float frequency = 0.0;
	
	for (int i = 0; i < OCTAVES; i++) {
		value += amplitude * noise(st);
		st *= 2.0;
		amplitude *= 0.5;
	}
	return value;
}

float orb(vec2 p) {
	return  1.0 - length(p);
}

vec3 gradient(vec2 p) {
	return vec3(0.5, p.x + 1.0 / 2.0, 0.1);
}

void main() {
	vec2 p = vec2(o_position.x, o_position.y);
	vec3 c = vec3(0.0);
	
	c += fbm(p + fbm(p * 2.0) + time * 0.4);
	c += gradient(p);

	color = vec4(c, 1.0);
}