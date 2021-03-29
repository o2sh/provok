use crate::font::FontConfiguration;
use failure::Fallible;
use glium::{Display, IndexBuffer, VertexBuffer};

pub const VERTICES_PER_CELL: usize = 4;
pub const V_TOP_LEFT: usize = 0;
pub const V_TOP_RIGHT: usize = 1;
pub const V_BOT_LEFT: usize = 2;
pub const V_BOT_RIGHT: usize = 3;

static VERTEX_SHADER: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/vertex.glsl"));

static FRAGMENT_SHADER: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/fragment.glsl"));

#[derive(Copy, Clone, Default)]
pub struct Vertex {
    pub position: (f32, f32),
    pub adjust: (f32, f32),
    pub tex: (f32, f32),
    pub underline: (f32, f32),
    pub bg_color: (f32, f32, f32, f32),
    pub fg_color: (f32, f32, f32, f32),
    pub has_color: f32,
}

implement_vertex!(Vertex, position, adjust, tex, underline, bg_color, fg_color, has_color);

pub struct PixelUnit;
pub type Size = euclid::Size2D<isize, PixelUnit>;

pub struct RenderMetrics {
    pub win_size: Size,
    pub cell_size: Size,
}

impl RenderMetrics {
    pub fn new(fonts: FontConfiguration, width: f32, height: f32) -> Self {
        let win_size = Size::new(width as isize, height as isize);
        let metrics = fonts.default_font_metrics().expect("failed to get font metrics!?");

        let (cell_height, cell_width) =
            (metrics.cell_height.get().ceil() as usize, metrics.cell_width.get().ceil() as usize);
        let cell_size = Size::new(cell_width as isize, cell_height as isize);
        Self { win_size, cell_size }
    }
}

pub struct RenderState {
    pub program: glium::Program,
    pub glyph_vertex_buffer: VertexBuffer<Vertex>,
    pub glyph_index_buffer: IndexBuffer<u32>,
}

impl RenderState {
    pub fn new(display: &Display, render_metrics: &RenderMetrics) -> Fallible<Self> {
        let program = glium::Program::from_source(display, VERTEX_SHADER, FRAGMENT_SHADER, None)?;
        let (glyph_vertex_buffer, glyph_index_buffer) =
            Self::compute_glyph_vertices(&render_metrics, display)?;
        Ok(Self { program, glyph_vertex_buffer, glyph_index_buffer })
    }

    pub fn compute_glyph_vertices(
        render_metrics: &RenderMetrics,
        display: &Display,
    ) -> Fallible<(VertexBuffer<Vertex>, IndexBuffer<u32>)> {
        let cell_width = render_metrics.cell_size.width as f32;
        let cell_height = render_metrics.cell_size.height as f32;
        let mut verts = Vec::new();
        let mut indices = Vec::new();

        let num_cols = render_metrics.win_size.width as usize / cell_width as usize;
        let y_pos = 0.;
        for x in 0..num_cols {
            let x_pos = (render_metrics.win_size.width as f32 / -2.0) + (x as f32 * cell_width);

            let idx = verts.len() as u32;
            verts.push(Vertex { position: (x_pos, y_pos), ..Default::default() });
            verts.push(Vertex { position: (x_pos + cell_width, y_pos), ..Default::default() });
            verts.push(Vertex { position: (x_pos, y_pos + cell_height), ..Default::default() });
            verts.push(Vertex {
                position: (x_pos + cell_width, y_pos + cell_height),
                ..Default::default()
            });

            indices.push(idx + V_TOP_LEFT as u32);
            indices.push(idx + V_TOP_RIGHT as u32);
            indices.push(idx + V_BOT_LEFT as u32);

            indices.push(idx + V_TOP_RIGHT as u32);
            indices.push(idx + V_BOT_LEFT as u32);
            indices.push(idx + V_BOT_RIGHT as u32);
        }

        Ok((
            VertexBuffer::dynamic(display, &verts)?,
            IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &indices)?,
        ))
    }
}
