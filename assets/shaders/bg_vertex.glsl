#version 330
precision highp float;
in vec2 position;

out vec4 o_position;
uniform mat4 projection;

void main() {
    o_position = projection * vec4(position, 0.0, 1.0);
    gl_Position = o_position;
}
