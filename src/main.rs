#[macro_use]
extern crate glium;

#[macro_use]
extern crate failure;

use clap::{crate_description, crate_name, crate_version, App, AppSettings, Arg};
use failure::Fallible;
use font::FontConfiguration;
use glium::glutin::dpi::LogicalSize;
use glium::glutin::event::Event;
use glium::glutin::event::StartCause;
use glium::glutin::event::WindowEvent;
use glium::glutin::event_loop::ControlFlow;
use glium::glutin::event_loop::EventLoop;
use glium::glutin::window::WindowBuilder;
use glium::glutin::ContextBuilder;
use glium::{Display, Frame, Surface};
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

static DEFAULT_INPUT_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/0.json");

fn run(input_path: &str) -> Fallible<()> {
    let event_loop = EventLoop::new();
    let (window_width, window_height) = (720., 405.);
    let wb = WindowBuilder::new().with_inner_size(LogicalSize::new(window_width, window_height));
    let cb = ContextBuilder::new();
    let display = Display::new(wb, cb, &event_loop)?;
    let input = Rc::new(Input::new(input_path)?);
    let fontconfig = Rc::new(FontConfiguration::new(input.config.font_size, input.config.dpi)?);
    let render_state = RefCell::new(RenderState::new(&display)?);
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

        paint_screen(
            &fontconfig,
            &render_state,
            &display,
            &mut target,
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

fn paint_screen(
    fontconfig: &Rc<FontConfiguration>,
    render_state: &RefCell<RenderState>,
    display: &Display,
    frame: &mut Frame,
    words: &Vec<Word>,
    window_width: f64,
    window_height: f64,
    draw_word_count: &mut u32,
    time: &mut f32,
    frame_count: u32,
) -> Fallible<()> {
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

    let draw_params =
        glium::DrawParameters { blend: glium::Blend::alpha_blending(), ..Default::default() };

    let idx = *draw_word_count as usize % words.len();
    let word = &words[idx];

    frame.clear_color(
        word.canvas_color.red as f32 / 255.,
        word.canvas_color.green as f32 / 255.,
        word.canvas_color.blue as f32 / 255.,
        1.0,
    );

    gl_state.compute_bg_vertices(display, window_width, window_height)?;

    if frame_count % 30 == 0 {
        gl_state.compute_glyph_vertices(word, display, fontconfig)?;
        *draw_word_count += 1;
    }

    *time += 0.02;
    if *time > 1. {
        *time = 0.;
    }

    let rad = 2.0 * std::f32::consts::PI * *time;

    frame.draw(
        gl_state.bg_vertex_buffer.as_ref().unwrap(),
        gl_state.bg_index_buffer.as_ref().unwrap(),
        &gl_state.bg_program,
        &uniform! {
            projection: projection,
            rad: rad
        },
        &draw_params,
    )?;

    if gl_state.draw_bg {
        frame.draw(
            gl_state.glyph_bg_vertex_buffer.as_ref().unwrap(),
            gl_state.glyph_bg_index_buffer.as_ref().unwrap(),
            &gl_state.glyph_program,
            &uniform! {
                projection: projection,
                draw_bg: true
            },
            &draw_params,
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
        &draw_params,
    )?;

    Ok(())
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
