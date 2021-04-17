#version 330
precision mediump float;

in vec2 o_tex;
in vec4 o_fg_color;

uniform mat4 projection;
uniform sampler2D glyph_tex;

out vec4 color;

void main() {
    color = texture(glyph_tex, o_tex);
    color.rgb = o_fg_color.rgb;
}