#version 330
precision mediump float;

in vec2 o_tex;
in vec4 o_fg_color;
in vec4 o_bg_color;

uniform mat4 projection;
uniform sampler2D glyph_tex;
uniform bool draw_bg;

out vec4 color;

void main() {
    if (draw_bg) {
        color = o_bg_color;
    } else {
        color = texture(glyph_tex, o_tex);
        color.rgb = o_fg_color.rgb;
    }
}