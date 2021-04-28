#version 330
precision mediump float;

uniform float rad;
out vec4 color;

void main() {
    color = vec4(
        abs(sin(cos(rad))),
        abs(cos(sin(rad))),
        0.5,
        1.0
    );
}