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

const PADDING: f32 = 15.;

const ATLAS_SIZE: usize = 8192;

static DEFAULT_INPUT_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/0.json");

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

pub fn compile_shaders(display: &Display) -> Fallible<(glium::Program, glium::Program)> {
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

fn run(input_path: &str) -> Fallible<()> {
    let event_loop = EventLoop::new();
    let (window_width, window_height) = (720., 405.);
    let wb = WindowBuilder::new().with_inner_size(LogicalSize::new(window_width, window_height));
    let cb = ContextBuilder::new();
    let display = Display::new(wb, cb, &event_loop)?;
    let input = Rc::new(Input::new(input_path)?);
    let fontconfig = Rc::new(FontConfiguration::new(input.config.font_size, input.config.dpi)?);
    let programs = compile_shaders(&display)?;
    let mut frame_count = 0;
    let mut draw_word_count = 0;
    let mut time = 0.;
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

        let next_frame_time = Instant::now() + Duration::from_nanos(16_666_667);
        *control_flow = ControlFlow::WaitUntil(next_frame_time);
        let mut target = display.draw();

        paint(
            &fontconfig,
            &display,
            &mut target,
            &programs,
            &input.words,
            window_width,
            window_height,
            &mut draw_word_count,
            &mut time,
            frame_count,
        )
        .unwrap();
        target.finish().unwrap();

        frame_count += 1;
    });
}

fn paint(
    fontconfig: &Rc<FontConfiguration>,
    display: &Display,
    frame: &mut Frame,
    programs: &(Program, Program),
    words: &Vec<Word>,
    window_width: f64,
    window_height: f64,
    draw_word_count: &mut u32,
    time: &mut f32,
    frame_count: u32,
) -> Fallible<()> {
    let (glyph_program, bg_program) = programs;
    let projection = euclid::Transform3D::<f32, f32, f32>::ortho(
        -(window_width as f32) / 2.0,
        window_width as f32 / 2.0,
        window_height as f32 / 2.0,
        -(window_height as f32) / 2.0,
        -1.0,
        1.0,
    )
    .to_arrays();

    let draw_params =
        glium::DrawParameters { blend: glium::Blend::alpha_blending(), ..Default::default() };
    let idx = *draw_word_count as usize % words.len();
    draw_bg(
        display,
        frame,
        bg_program,
        &words[idx],
        time,
        window_width,
        window_height,
        &draw_params,
        &projection,
    )?;
    draw_word(fontconfig, display, frame, glyph_program, &words[idx], &draw_params, &projection)?;
    if frame_count % 30 == 0 {
        *draw_word_count += 1;
    }
    Ok(())
}

fn draw_bg(
    display: &Display,
    frame: &mut Frame,
    program: &Program,
    word: &Word,
    t: &mut f32,
    window_width: f64,
    window_height: f64,
    draw_params: &glium::DrawParameters,
    projection: &[[f32; 4]; 4],
) -> Fallible<()> {
    frame.clear_color(
        word.canvas_color.red as f32 / 255.,
        word.canvas_color.green as f32 / 255.,
        word.canvas_color.blue as f32 / 255.,
        1.0,
    );
    *t += 0.02;
    if *t > 1. {
        *t = 0.;
    }
    let rad = 2.0 * std::f32::consts::PI * *t;

    let mut verts = Vec::new();
    let mut indices = Vec::new();
    let (w, h) = (window_width as f32 / 2. - PADDING, window_height as f32 / 2. - PADDING);

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

    let vertex_buffer = VertexBuffer::dynamic(display, &verts)?;
    let index_buffer =
        IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &indices)?;

    frame.draw(
        &vertex_buffer,
        &index_buffer,
        &program,
        &uniform! {
            projection: *projection,
            rad: rad
        },
        draw_params,
    )?;
    Ok(())
}

fn draw_word(
    fontconfig: &Rc<FontConfiguration>,
    display: &Display,
    frame: &mut Frame,
    program: &Program,
    word: &Word,
    draw_params: &glium::DrawParameters,
    projection: &[[f32; 4]; 4],
) -> Fallible<()> {
    let mut glyph_atlas = GlyphAtlas::new(display, ATLAS_SIZE)?;

    let (mut glyph_vertex_buffer, glyph_index_buffer) =
        compute_glyph_vertices(word, display, &mut glyph_atlas, fontconfig)?;

    if let Some(bg_color) = word.style.bg_color {
        let (bg_glyph_vertex_buffer, bg_glyph_index_buffer) =
            compute_bg_glyph_vertices(bg_color, &mut glyph_vertex_buffer, display)?;
        frame.draw(
            &bg_glyph_vertex_buffer,
            &bg_glyph_index_buffer,
            &program,
            &uniform! {
                projection: *projection,
                draw_bg: true
            },
            draw_params,
        )?;
    }

    let tex = glyph_atlas.atlas.texture();

    frame.draw(
        &glyph_vertex_buffer,
        &glyph_index_buffer,
        &program,
        &uniform! {
            projection: *projection,
            glyph_tex: &*tex,
            draw_bg: false
        },
        draw_params,
    )?;

    Ok(())
}

fn compute_bg_glyph_vertices(
    bg_color: color::RgbColor,
    glyph_vertex_buffer: &mut VertexBuffer<Vertex>,
    display: &Display,
) -> Fallible<(VertexBuffer<Vertex>, IndexBuffer<u32>)> {
    let bg_color = color::to_tuple_rgba(bg_color);
    let mut verts = Vec::new();
    let mut indices = Vec::new();
    let (mut top, mut left, mut bottom, mut right) = (0f32, 0f32, 0f32, 0f32);
    let g_verts = glyph_vertex_buffer.slice_mut(..).unwrap().map_read();
    for v in g_verts.iter() {
        left = left.min(v.position.0);
        right = right.max(v.position.0);
        top = top.min(v.position.1);
        bottom = bottom.max(v.position.1);
    }

    left -= PADDING;
    right += PADDING;
    top -= PADDING;
    bottom += PADDING;

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

    Ok((
        VertexBuffer::dynamic(display, &verts)?,
        IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &indices)?,
    ))
}
fn compute_glyph_vertices(
    word: &Word,
    display: &Display,
    glyph_atlas: &mut GlyphAtlas<SrgbTexture2d>,
    fontconfig: &FontConfiguration,
) -> Fallible<(VertexBuffer<Vertex>, IndexBuffer<u32>)> {
    let mut verts = Vec::new();
    let mut indices = Vec::new();
    let fg_color = color::to_tuple_rgba(word.style.fg_color);

    let font = fontconfig.get_font(&word.style)?;
    let glyph_infos = font.shape(&word.text)?;
    let width = glyph_infos.iter().fold(0., |acc, info| acc + info.x_advance.get() as f32);
    let mut x = -width / 2.;
    let mut y = 0.;
    for glyph_info in &glyph_infos {
        let rasterized_glyph = font.rasterize(glyph_info.glyph_pos)?;
        let glyph = glyph_atlas.load_glyph(rasterized_glyph, &glyph_info)?;

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

    let input_path = matches.value_of("input").unwrap_or(DEFAULT_INPUT_FILE);
    run(input_path)?;
    Ok(())
}
