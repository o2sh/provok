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

pub struct Renderer {
    pub program: glium::Program,
    pub vertex_buffer: VertexBuffer<Vertex>,
    pub index_buffer: IndexBuffer<u32>,
}

impl Renderer {
    pub fn new(display: &Display) -> Fallible<Self> {
        let program = glium::Program::from_source(display, VERTEX_SHADER, FRAGMENT_SHADER, None)?;

        let vertex1 = Vertex { position: [-0.5, -0.5] };
        let vertex2 = Vertex { position: [0.0, 0.5] };
        let vertex3 = Vertex { position: [0.5, -0.25] };
        let shape = vec![vertex1, vertex2, vertex3];

        let vertex_buffer = VertexBuffer::new(display, &shape)?;
        let index_buffer = IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &[0, 1, 2, 1, 3, 2],
        )?;

        Ok(Self { program, vertex_buffer, index_buffer })
    }

    pub fn paint(&self, frame: &mut glium::Frame) -> Fallible<()> {
        frame.clear_color(0.0, 0.0, 1.0, 1.0);

        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32],
            ]
        };

        frame.draw(
            &self.vertex_buffer,
            &self.index_buffer,
            &self.program,
            &uniforms,
            &Default::default(),
        )?;

        Ok(())
    }
}
