#version 330
precision mediump float;
in vec2 position;
in vec2 tex;
in vec4 bg_color;
in vec4 fg_color;

uniform mat4 projection;
uniform bool draw_bg_color;

out vec2 o_tex;
out vec4 o_fg_color;
out vec4 o_bg_color;

void main() {
    o_tex = tex;
    o_fg_color = fg_color;
    o_bg_color = bg_color;
    gl_Position = projection * vec4(position, 0.0, 1.0);
}
