# Provok

Text Renderer written in Rust using HarfBuzz for shaping, FreeType for rasterization and OpenGL for rendering.

<h3 align="center"><img src="resources/provok.gif"></h3>

## Input

Provok is fed with a [JSON file](./examples/0.json) that consists of an array of word alongside their display parameters (fg_color, boldness, italic, etc.):

```text
{
    "font_size": 50,
    "words": [
        {
            "text": "\"PROVOK\"",
            "canvas_color": "#E24E43",
            "bg_color": "#EFB715",
            "fg_color": "#E24E43",
            "bold": true
        },
...
}

```

## How To Use

First, you need to have installed the [Rust toolchain](https://www.rust-lang.org/tools/install) and [HarfBuzz](https://harfbuzz.github.io) on your machine, then:

```text
git clone https://github.com/o2sh/provok --depth=1
cd provok 
make install
provok
```

You can also provide your own custom input file with the `--input` CLI flag:

```text
provok -i /path/to/input-file
```

You can pick between multiple background effects using the `--fragment` CLI flag:

```text
provok -f NUM (0..4)
```
