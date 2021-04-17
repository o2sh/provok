#version 330
precision mediump float;
in vec2 position;
in vec2 tex;
in vec4 fg_color;

uniform mat4 projection;

out vec2 o_tex;
out vec4 o_fg_color;

void main() {
    o_tex = tex;
    o_fg_color = fg_color;
    gl_Position = projection * vec4(position, 0.0, 1.0);
}
