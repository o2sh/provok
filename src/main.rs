#[macro_use]
extern crate glium;

#[macro_use]
extern crate failure;

use clap::{crate_description, crate_name, crate_version, App, AppSettings, Arg};
use failure::Fallible;
use glium::glutin::dpi::LogicalSize;
use glium::glutin::event::Event;
use glium::glutin::event::StartCause;
use glium::glutin::event::WindowEvent;
use glium::glutin::event_loop::ControlFlow;
use glium::glutin::event_loop::EventLoop;
use glium::glutin::window::WindowBuilder;
use glium::glutin::ContextBuilder;
use glium::texture::SrgbTexture2d;
use glium::Program;
use glium::{Display, Frame, Surface};
use glium::{IndexBuffer, VertexBuffer};
use std::rc::Rc;
use std::time::{Duration, Instant};

mod bitmaps;
mod color;
mod font;
mod glyph_atlas;
mod input;
mod language;
mod utils;

use font::FontConfiguration;
use glyph_atlas::GlyphAtlas;
use input::{Input, Word};

const ATLAS_SIZE: usize = 8192;

static VERTEX_SHADER: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/vertex.glsl"));

static FRAGMENT_SHADER: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/fragment.glsl"));

pub const V_TOP_LEFT: usize = 0;
pub const V_TOP_RIGHT: usize = 1;
pub const V_BOT_LEFT: usize = 2;
pub const V_BOT_RIGHT: usize = 3;

#[derive(Copy, Clone, Default)]
pub struct Vertex {
    pub position: (f32, f32),
    pub tex: (f32, f32),
    pub fg_color: (f32, f32, f32, f32),
}

implement_vertex!(Vertex, position, tex, fg_color);
pub fn compile_shaders(display: &Display) -> Fallible<glium::Program> {
    let glyph_source = glium::program::ProgramCreationInput::SourceCode {
        vertex_shader: VERTEX_SHADER,
        fragment_shader: FRAGMENT_SHADER,
        outputs_srgb: true,
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        transform_feedback_varyings: None,
        uses_point_size: false,
        geometry_shader: None,
    };
    let program = glium::Program::new(display, glyph_source)?;
    Ok(program)
}

fn run(input_path: &str) -> Fallible<()> {
    let event_loop = EventLoop::new();
    let (window_width, window_height) = (720., 405.);
    let wb = WindowBuilder::new().with_inner_size(LogicalSize::new(window_width, window_height));
    let cb = ContextBuilder::new();
    let display = Display::new(wb, cb, &event_loop)?;
    let input = Rc::new(Input::new(input_path)?);
    let fontconfig = Rc::new(FontConfiguration::new(input.config.font_size, input.config.dpi));
    let shaders = compile_shaders(&display)?;
    let mut i = 0;
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                _ => return,
            },
            Event::NewEvents(cause) => match cause {
                StartCause::ResumeTimeReached { .. } => (),
                StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }

        let next_frame_time = Instant::now() + Duration::from_millis(500);
        *control_flow = ControlFlow::WaitUntil(next_frame_time);
        let mut target = display.draw();

        paint(
            &fontconfig,
            &display,
            &mut target,
            &shaders,
            &input.words[i],
            window_width,
            window_height,
        )
        .unwrap();
        target.finish().unwrap();

        i += 1;

        if i == input.words.len() {
            i = 0;
        }
    });
}

fn paint(
    fontconfig: &Rc<FontConfiguration>,
    display: &Display,
    frame: &mut Frame,
    program: &Program,
    word: &Word,
    window_width: f64,
    window_height: f64,
) -> Fallible<()> {
    frame.clear_color(
        word.canvas_color.red as f32 / 255.,
        word.canvas_color.green as f32 / 255.,
        word.canvas_color.blue as f32 / 255.,
        1.0,
    );
    let mut glyph_atlas = GlyphAtlas::new(display, ATLAS_SIZE)?;
    let (glyph_vertex_buffer, glyph_index_buffer) =
        render_text(word, display, &mut glyph_atlas, fontconfig)?;
    let projection = euclid::Transform3D::<f32, f32, f32>::ortho(
        -(window_width as f32) / 2.0,
        window_width as f32 / 2.0,
        window_height as f32 / 2.0,
        -(window_height as f32) / 2.0,
        -1.0,
        1.0,
    )
    .to_arrays();
    let tex = glyph_atlas.atlas.texture();

    let draw_params =
        glium::DrawParameters { blend: glium::Blend::alpha_blending(), ..Default::default() };

    frame.draw(
        &glyph_vertex_buffer,
        &glyph_index_buffer,
        &program,
        &uniform! {
            projection: projection,
            glyph_tex: &*tex,
        },
        &draw_params,
    )?;

    Ok(())
}

fn render_text(
    word: &Word,
    display: &Display,
    glyph_atlas: &mut GlyphAtlas<SrgbTexture2d>,
    fontconfig: &FontConfiguration,
) -> Fallible<(VertexBuffer<Vertex>, IndexBuffer<u32>)> {
    let mut x = 0.;
    let mut y = 0.;
    let mut verts = Vec::new();
    let mut indices = Vec::new();
    let fg_color = color::to_tuple_rgba(word.style.fg_color);

    let font = fontconfig.get_font(&word.style)?;
    let glyph_infos = font.shape(&word)?;

    for glyph_info in &glyph_infos {
        let rasterized_glyph = font.rasterize(glyph_info.glyph_pos)?;
        let glyph = glyph_atlas.load_glyph(rasterized_glyph, &glyph_info)?;

        let x0 = x + (glyph.x_offset + glyph.bearing_x).get() as f32;
        let y0 = y + (glyph.y_offset + glyph.bearing_y).get() as f32;

        let x1 = x0 + glyph.texture.width as f32;
        let y1 = y0 + glyph.texture.height as f32;

        x += glyph_info.x_advance.get() as f32;
        y += glyph_info.y_advance.get() as f32;
        let idx = verts.len() as u32;
        println!("tex_coords: {:?}", glyph.texture.tex_coords);
        verts.push(Vertex {
            position: (x0, y0),
            tex: (glyph.texture.tex_coords.min_x(), glyph.texture.tex_coords.min_y()),
            fg_color,
        });
        verts.push(Vertex {
            position: (x1, y0),
            tex: (glyph.texture.tex_coords.max_x(), glyph.texture.tex_coords.min_y()),
            fg_color,
        });
        verts.push(Vertex {
            position: (x0, y1),
            tex: (glyph.texture.tex_coords.min_x(), glyph.texture.tex_coords.max_y()),
            fg_color,
        });
        verts.push(Vertex {
            position: (x1, y1),
            tex: (glyph.texture.tex_coords.max_x(), glyph.texture.tex_coords.max_y()),
            fg_color,
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

fn main() -> Fallible<()> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::UnifiedHelpMessage)
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .help("Which input to use.")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("input").unwrap_or("examples/0.json");
    run(input_path)?;
    Ok(())
}
