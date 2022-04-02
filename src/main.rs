#[macro_use]
extern crate glium;

use anyhow::Result;
use clap::{crate_description, crate_name, crate_version, AppSettings, Arg};
use font::FontConfiguration;
use glium::glutin::dpi::LogicalSize;
use glium::glutin::event::Event;
use glium::glutin::event::StartCause;
use glium::glutin::event::WindowEvent;
use glium::glutin::event_loop::ControlFlow;
use glium::glutin::event_loop::EventLoop;
use glium::glutin::window::WindowBuilder;
use glium::glutin::ContextBuilder;
use glium::{BlendingFunction, Display, Frame, LinearBlendingFactor, Surface};
use input::{Input, Word};
use render_state::RenderState;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

mod bitmaps;
mod color;
mod font;
mod glyph_atlas;
mod input;
mod language;
mod render_state;
mod utils;

const FPS: u32 = 60;
static DEFAULT_INPUT_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/0.json");

fn run(input_path: &str, bg_fragment_shader_num: usize) -> Result<()> {
    let event_loop = EventLoop::new();
    let (window_width, window_height) = (720., 405.);
    let wb = WindowBuilder::new().with_inner_size(LogicalSize::new(window_width, window_height));
    let cb = ContextBuilder::new();
    let display = Display::new(wb, cb, &event_loop)?;
    let input = Rc::new(Input::new(input_path)?);
    let fontconfig = Rc::new(FontConfiguration::new(input.config.font_size, input.config.dpi)?);
    let render_state = RefCell::new(RenderState::new(&display, bg_fragment_shader_num)?);
    let mut frame_count = 0;
    let mut count = 0;
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
        let next_frame_time = Instant::now() + Duration::from_micros(1_000_000 / FPS as u64);
        *control_flow = ControlFlow::WaitUntil(next_frame_time);
        let mut target = display.draw();

        paint_screen(
            &fontconfig,
            &render_state,
            &display,
            &mut target,
            &input.words,
            window_width,
            window_height,
            &mut count,
            &mut time,
            frame_count,
        )
        .unwrap();
        target.finish().unwrap();

        frame_count += 1;
    });
}

fn paint_screen(
    fontconfig: &Rc<FontConfiguration>,
    render_state: &RefCell<RenderState>,
    display: &Display,
    frame: &mut Frame,
    words: &[Word],
    window_width: f64,
    window_height: f64,
    count: &mut u32,
    time: &mut f32,
    frame_count: u32,
) -> Result<()> {
    let mut gl_state = render_state.borrow_mut();
    let projection = euclid::Transform3D::<f32, f32, f32>::ortho(
        -(window_width as f32) / 2.0,
        window_width as f32 / 2.0,
        window_height as f32 / 2.0,
        -(window_height as f32) / 2.0,
        -1.0,
        1.0,
    )
    .to_arrays();

    let draw_params_with_alpha = glium::DrawParameters {
        blend: glium::Blend {
            color: BlendingFunction::Addition {
                source: LinearBlendingFactor::SourceAlpha,
                destination: LinearBlendingFactor::OneMinusSourceAlpha,
            },
            alpha: BlendingFunction::Addition {
                source: LinearBlendingFactor::One,
                destination: LinearBlendingFactor::OneMinusSourceAlpha,
            },
            constant_value: (0.0, 0.0, 0.0, 0.0),
        },

        ..Default::default()
    };

    gl_state.compute_bg_vertices(display, window_width, window_height)?;

    if frame_count % 30 == 0 {
        let idx = *count as usize % words.len();
        let w = &words[idx];
        gl_state.word = Some(w.clone());
        gl_state.compute_glyph_vertices(display, fontconfig)?;
        *count += 1;
    }

    *time += 1. / FPS as f32;

    frame.draw(
        gl_state.bg_vertex_buffer.as_ref().unwrap(),
        gl_state.bg_index_buffer.as_ref().unwrap(),
        &gl_state.bg_program,
        &uniform! {
            projection: projection,
            time: *time
        },
        &Default::default(),
    )?;

    gl_state.compute_inner_bg_vertices(display, window_width, window_height)?;

    frame.draw(
        gl_state.inner_bg_vertex_buffer.as_ref().unwrap(),
        gl_state.inner_bg_index_buffer.as_ref().unwrap(),
        &gl_state.glyph_program,
        &uniform! {
            projection: projection,
            draw_bg: true
        },
        &draw_params_with_alpha,
    )?;

    if gl_state.word.as_ref().unwrap().style.bg_color.is_some() {
        frame.draw(
            gl_state.glyph_bg_vertex_buffer.as_ref().unwrap(),
            gl_state.glyph_bg_index_buffer.as_ref().unwrap(),
            &gl_state.glyph_program,
            &uniform! {
                projection: projection,
                draw_bg: true
            },
            &Default::default(),
        )?;
    }

    let tex = gl_state.glyph_atlas.atlas.texture();

    frame.draw(
        gl_state.glyph_vertex_buffer.as_ref().unwrap(),
        gl_state.glyph_index_buffer.as_ref().unwrap(),
        &gl_state.glyph_program,
        &uniform! {
            projection: projection,
            glyph_tex: &*tex,
            draw_bg: false
        },
        &draw_params_with_alpha,
    )?;

    Ok(())
}

fn main() -> Result<()> {
    let matches = clap::Command::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .setting(AppSettings::DeriveDisplayOrder)
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .help("Which input to use.")
                .takes_value(true),
        )
        .arg(
            Arg::new("fragment")
                .short('f')
                .long("fragment")
                .help("Which fragment shader to use.")
                .takes_value(true)
                .validator(|t| {
                    t.parse::<usize>()
                        .map_err(|_t| "must be a number")
                        .map(|_t| ())
                        .map_err(|e| e.to_string())
                }),
        )
        .get_matches();

    let input_path = matches.value_of("input").unwrap_or(DEFAULT_INPUT_FILE);
    let bg_fragment_shader_num: usize =
        matches.value_of("fragment").unwrap_or("0").parse().unwrap();
    run(input_path, bg_fragment_shader_num)?;
    Ok(())
}
