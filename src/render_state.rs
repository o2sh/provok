use crate::color;
use crate::font::FontConfiguration;
use crate::glyph_atlas::GlyphAtlas;
use crate::input::Word;
use failure::Fallible;
use glium::texture::SrgbTexture2d;
use glium::Display;
use glium::Program;
use glium::{IndexBuffer, VertexBuffer};

const PADDING: f32 = 15.;

const INNER_BG_ALPHA: f32 = 0.8;

const ATLAS_SIZE: usize = 8192;

static GLYPH_VERTEX_SHADER: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/g_vertex.glsl"));

static GLYPH_FRAGMENT_SHADER: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/g_fragment.glsl"));

static BG_VERTEX_SHADER: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/bg_vertex.glsl"));

static BG_FRAGMENT_SHADER: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/bg_fragment.glsl"));

pub const V_TOP_LEFT: usize = 0;
pub const V_TOP_RIGHT: usize = 1;
pub const V_BOT_LEFT: usize = 2;
pub const V_BOT_RIGHT: usize = 3;

#[derive(Copy, Clone, Default)]
pub struct Vertex {
    pub position: (f32, f32),
    pub tex: (f32, f32),
    pub fg_color: (f32, f32, f32, f32),
    pub bg_color: (f32, f32, f32, f32),
}

implement_vertex!(Vertex, position, tex, fg_color, bg_color);

pub struct RenderState {
    pub glyph_atlas: GlyphAtlas<SrgbTexture2d>,
    pub bg_program: Program,
    pub glyph_program: Program,
    pub glyph_vertex_buffer: Option<VertexBuffer<Vertex>>,
    pub glyph_index_buffer: Option<IndexBuffer<u32>>,
    pub glyph_bg_vertex_buffer: Option<VertexBuffer<Vertex>>,
    pub glyph_bg_index_buffer: Option<IndexBuffer<u32>>,
    pub inner_bg_vertex_buffer: Option<VertexBuffer<Vertex>>,
    pub inner_bg_index_buffer: Option<IndexBuffer<u32>>,
    pub bg_vertex_buffer: Option<VertexBuffer<Vertex>>,
    pub bg_index_buffer: Option<IndexBuffer<u32>>,
    pub word: Option<Word>,
}

impl RenderState {
    pub fn new(display: &Display) -> Fallible<Self> {
        let (glyph_program, bg_program) = compile_shaders(display)?;
        let glyph_atlas = GlyphAtlas::new(display, ATLAS_SIZE)?;
        Ok(Self {
            glyph_atlas,
            bg_program,
            glyph_program,
            glyph_vertex_buffer: None,
            glyph_index_buffer: None,
            glyph_bg_vertex_buffer: None,
            glyph_bg_index_buffer: None,
            inner_bg_vertex_buffer: None,
            inner_bg_index_buffer: None,
            bg_vertex_buffer: None,
            bg_index_buffer: None,
            word: None,
        })
    }

    pub fn compute_glyph_vertices(
        &mut self,
        display: &Display,
        fontconfig: &FontConfiguration,
    ) -> Fallible<()> {
        self.compute_g_vertices(display, fontconfig)?;

        if let Some(bg_color) = self.word.as_ref().unwrap().style.bg_color {
            self.compute_bg_g_vertices(bg_color, display)?;
        }
        Ok(())
    }

    pub fn compute_g_vertices(
        &mut self,
        display: &Display,
        fontconfig: &FontConfiguration,
    ) -> Fallible<()> {
        let mut verts = Vec::new();
        let mut indices = Vec::new();
        let word = self.word.as_ref().unwrap();
        let fg_color = color::to_tuple_rgba(word.style.fg_color);

        let font = fontconfig.get_font(&word.style)?;
        let glyph_infos = font.shape(&word.text)?;
        let width = glyph_infos.iter().fold(0., |acc, info| acc + info.x_advance.get() as f32);
        let mut x = -width / 2.;
        let mut y = 0.;
        for glyph_info in &glyph_infos {
            let rasterized_glyph = font.rasterize(glyph_info.glyph_pos)?;
            let glyph = self.glyph_atlas.load_glyph(rasterized_glyph, &glyph_info)?;

            let x0 = x + (glyph.x_offset + glyph.bearing_x).get() as f32;
            let y0 = y - (glyph.y_offset + glyph.bearing_y).get() as f32;

            let x1 = x0 + glyph.texture.width as f32;
            let y1 = y0 + glyph.texture.height as f32;

            x += glyph_info.x_advance.get() as f32;
            y += glyph_info.y_advance.get() as f32;
            let idx = verts.len() as u32;
            verts.push(Vertex {
                position: (x0, y0),
                tex: (glyph.texture.tex_coords.min_x(), glyph.texture.tex_coords.min_y()),
                fg_color,
                ..Default::default()
            });
            verts.push(Vertex {
                position: (x1, y0),
                tex: (glyph.texture.tex_coords.max_x(), glyph.texture.tex_coords.min_y()),
                fg_color,
                ..Default::default()
            });
            verts.push(Vertex {
                position: (x0, y1),
                tex: (glyph.texture.tex_coords.min_x(), glyph.texture.tex_coords.max_y()),
                fg_color,
                ..Default::default()
            });
            verts.push(Vertex {
                position: (x1, y1),
                tex: (glyph.texture.tex_coords.max_x(), glyph.texture.tex_coords.max_y()),
                fg_color,
                ..Default::default()
            });

            indices.push(idx + V_TOP_LEFT as u32);
            indices.push(idx + V_TOP_RIGHT as u32);
            indices.push(idx + V_BOT_LEFT as u32);

            indices.push(idx + V_TOP_RIGHT as u32);
            indices.push(idx + V_BOT_LEFT as u32);
            indices.push(idx + V_BOT_RIGHT as u32);
        }

        self.glyph_vertex_buffer = Some(VertexBuffer::dynamic(display, &verts)?);
        self.glyph_index_buffer =
            Some(IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &indices)?);
        Ok(())
    }

    pub fn compute_bg_g_vertices(
        &mut self,
        bg_color: color::RgbColor,
        display: &Display,
    ) -> Fallible<()> {
        let bg_color = color::to_tuple_rgba(bg_color);
        let mut verts = Vec::new();
        let mut indices = Vec::new();
        let (mut top, mut left, mut bottom, mut right) = (0f32, 0f32, 0f32, 0f32);
        let glyph_vertex_buffer = self.glyph_vertex_buffer.as_mut().unwrap();
        let g_verts = glyph_vertex_buffer.slice_mut(..).unwrap().map_read();
        for v in g_verts.iter() {
            left = left.min(v.position.0);
            right = right.max(v.position.0);
            top = top.min(v.position.1);
            bottom = bottom.max(v.position.1);
        }

        left -= PADDING as f32;
        right += PADDING as f32;
        top -= PADDING as f32;
        bottom += PADDING as f32;

        verts.push(Vertex { position: (left, top), bg_color, ..Default::default() });
        verts.push(Vertex { position: (right, top), bg_color, ..Default::default() });
        verts.push(Vertex { position: (left, bottom), bg_color, ..Default::default() });
        verts.push(Vertex { position: (right, bottom), bg_color, ..Default::default() });

        indices.push(V_TOP_LEFT as u32);
        indices.push(V_TOP_RIGHT as u32);
        indices.push(V_BOT_LEFT as u32);

        indices.push(V_TOP_RIGHT as u32);
        indices.push(V_BOT_LEFT as u32);
        indices.push(V_BOT_RIGHT as u32);

        self.glyph_bg_vertex_buffer = Some(VertexBuffer::dynamic(display, &verts)?);
        self.glyph_bg_index_buffer =
            Some(IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &indices)?);
        Ok(())
    }

    pub fn compute_inner_bg_vertices(
        &mut self,
        display: &Display,
        window_width: f64,
        window_height: f64,
    ) -> Fallible<()> {
        let canvas_color = self.word.as_ref().unwrap().canvas_color;
        let mut bg_color = color::to_tuple_rgba(canvas_color);
        bg_color.3 = INNER_BG_ALPHA;
        let mut verts = Vec::new();
        let mut indices = Vec::new();

        let pad = 1.8 * PADDING;
        let (w, h) = (window_width as f32 / 2., window_height as f32 / 2.);

        let left = -w + pad;
        let right = w - pad;
        let top = -h + pad;
        let bottom = h - pad;

        verts.push(Vertex { position: (left, top), bg_color, ..Default::default() });
        verts.push(Vertex { position: (right, top), bg_color, ..Default::default() });
        verts.push(Vertex { position: (left, bottom), bg_color, ..Default::default() });
        verts.push(Vertex { position: (right, bottom), bg_color, ..Default::default() });

        indices.push(V_TOP_LEFT as u32);
        indices.push(V_TOP_RIGHT as u32);
        indices.push(V_BOT_LEFT as u32);

        indices.push(V_TOP_RIGHT as u32);
        indices.push(V_BOT_LEFT as u32);
        indices.push(V_BOT_RIGHT as u32);

        self.inner_bg_vertex_buffer = Some(VertexBuffer::dynamic(display, &verts)?);
        self.inner_bg_index_buffer =
            Some(IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &indices)?);
        Ok(())
    }

    pub fn compute_bg_vertices(
        &mut self,
        display: &Display,
        window_width: f64,
        window_height: f64,
    ) -> Fallible<()> {
        let mut verts = Vec::new();
        let mut indices = Vec::new();
        let (w, h) = (window_width as f32 / 2., window_height as f32 / 2.);

        verts.push(Vertex { position: (-w, -h), ..Default::default() });
        verts.push(Vertex { position: (w, -h), ..Default::default() });
        verts.push(Vertex { position: (-w, h), ..Default::default() });
        verts.push(Vertex { position: (w, h), ..Default::default() });

        indices.push(V_TOP_LEFT as u32);
        indices.push(V_TOP_RIGHT as u32);
        indices.push(V_BOT_LEFT as u32);

        indices.push(V_TOP_RIGHT as u32);
        indices.push(V_BOT_LEFT as u32);
        indices.push(V_BOT_RIGHT as u32);

        self.bg_vertex_buffer = Some(VertexBuffer::dynamic(display, &verts)?);
        self.bg_index_buffer =
            Some(IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &indices)?);

        Ok(())
    }
}

fn compile_shaders(display: &Display) -> Fallible<(glium::Program, glium::Program)> {
    let glyph_source = glium::program::ProgramCreationInput::SourceCode {
        vertex_shader: GLYPH_VERTEX_SHADER,
        fragment_shader: GLYPH_FRAGMENT_SHADER,
        outputs_srgb: true,
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        transform_feedback_varyings: None,
        uses_point_size: false,
        geometry_shader: None,
    };
    let glyph_program = glium::Program::new(display, glyph_source)?;

    let bg_source = glium::program::ProgramCreationInput::SourceCode {
        vertex_shader: BG_VERTEX_SHADER,
        fragment_shader: BG_FRAGMENT_SHADER,
        outputs_srgb: true,
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        transform_feedback_varyings: None,
        uses_point_size: false,
        geometry_shader: None,
    };
    let bg_program = glium::Program::new(display, bg_source)?;
    Ok((glyph_program, bg_program))
}
