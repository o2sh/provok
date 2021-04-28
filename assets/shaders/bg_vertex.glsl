#version 330
precision mediump float;
in vec2 position;

uniform mat4 projection;

void main() {
    gl_Position = projection * vec4(position, 0.0, 1.0);
}
