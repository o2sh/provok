#version 330
precision mediump float;
in vec2 position;
in vec2 adjust;
in vec2 tex;
in vec2 underline;
in vec4 bg_color;
in vec4 fg_color;

uniform mat4 projection;
uniform bool bg_and_line_layer;

out vec2 o_tex;
out vec4 o_fg_color;
out vec4 o_bg_color;
out vec2 o_underline;

void main() {
    o_tex = tex;
    o_fg_color = fg_color;
    o_bg_color = bg_color;
    o_underline = underline;

    if (bg_and_line_layer) {
      gl_Position = projection * vec4(position, 0.0, 1.0);
    } else {
      gl_Position = projection * vec4(position + adjust, 0.0, 1.0);
    }
}
