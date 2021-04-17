#version 330
precision mediump float;

in vec2 o_tex;
in vec4 o_fg_color;
in vec4 o_bg_color;

uniform mat4 projection;
uniform bool draw_bg_color;
uniform sampler2D glyph_tex;

out vec4 color;

void main() {
  if (draw_bg_color) {
    color = o_bg_color;
  } else {
    color = texture(glyph_tex, o_tex);
    color.rgb = o_fg_color.rgb;
  }
}