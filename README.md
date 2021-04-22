# Provok

Text Renderer written in Rust using HarfBuzz for shaping, FreeType for rasterization and OpenGL for rendering.

<h3 align="center"><img src="resources/provok.gif"></h3>

## Input

Provok is fed with a [JSON file](./examples/0.json) that consists of an array of words alongside their display parameters (fg_color, boldness, italic, etc.):

```text
{
    "font_size": 50,
    "words": [
        {
            "text": "\"PROVOK\"",
            "canvas_color": "#0A1332",
            "fg_color": "#ff0000",
            "bold": true
        },
...
}

```
