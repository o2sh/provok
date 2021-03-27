use failure::Fallible;
use glium::{Display, IndexBuffer, Surface, VertexBuffer};

static VERTEX_SHADER: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/vertex.glsl"));

static FRAGMENT_SHADER: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/fragment.glsl"));

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

pub struct RenderMetrics {
    pub width: f32,
    pub height: f32,
}

impl RenderMetrics {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

pub struct Renderer {
    pub program: glium::Program,
    pub glyph_vertex_buffer: VertexBuffer<Vertex>,
    pub glyph_index_buffer: IndexBuffer<u32>,
    pub render_metrics: RenderMetrics,
}

impl Renderer {
    pub fn new(display: &Display, width: f32, height: f32) -> Fallible<Self> {
        let render_metrics = RenderMetrics::new(width, height);
        let program = glium::Program::from_source(display, VERTEX_SHADER, FRAGMENT_SHADER, None)?;
        let (glyph_vertex_buffer, glyph_index_buffer) = Self::compute_glyph_vertices()?;
        Ok(Self { program, glyph_vertex_buffer, glyph_index_buffer, render_metrics })
    }

    pub fn paint(&self, frame: &mut glium::Frame) -> Fallible<()> {
        frame.clear_color(0.0, 0.0, 1.0, 1.0);

        let projection = euclid::Transform3D::<f32, f32, f32>::ortho(
            -(self.render_metrics.width as f32) / 2.0,
            self.render_metrics.width as f32 / 2.0,
            self.render_metrics.height as f32 / 2.0,
            -(self.render_metrics.height as f32) / 2.0,
            -1.0,
            1.0,
        )
        .to_arrays();

        frame.draw(
            &self.glyph_vertex_buffer,
            &self.glyph_index_buffer,
            &self.program,
            &uniform! {
                projection: projection,
            },
            &Default::default(),
        )?;

        Ok(())
    }

    pub fn compute_glyph_vertices() -> Fallible<(VertexBuffer<Vertex>, IndexBuffer<u32>)> {}
}
