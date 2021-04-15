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
use glium::Program;
use glium::{Display, Frame, Surface};
use std::rc::Rc;
use std::time::{Duration, Instant};

mod bitmaps;
mod color;
mod font;
mod glyphcache;
mod input;
mod language;
mod quad;
mod renderstate;
mod utils;
mod utilsprites;

use bitmaps::{atlas::pixel_rect, Texture2d};
use color::rgbcolor_to_color;
use font::FontConfiguration;
use input::{Input, Word};
use quad::Quad;
use renderstate::{compile_shaders, RenderMetrics, RenderState};
use utils::PixelLength;

fn run(input_path: &str) -> Fallible<()> {
    let event_loop = EventLoop::new();
    let (window_width, window_height) = (720., 405.);
    let wb = WindowBuilder::new().with_inner_size(LogicalSize::new(window_width, window_height));
    let cb = ContextBuilder::new();
    let display = Display::new(wb, cb, &event_loop)?;
    let input = Rc::new(Input::new(input_path)?);
    let fontconfig = Rc::new(FontConfiguration::new(Rc::new(input.config.clone())));
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

        let render_metrics =
            RenderMetrics::new(&fontconfig, &input.words[i].style, window_width, window_height);
        let mut render_state =
            RenderState::new(&display, &render_metrics, &input.words[i].text.chars().count())
                .unwrap();
        paint(
            &mut render_state,
            &render_metrics,
            &fontconfig,
            &mut target,
            &shaders,
            &input.words[i],
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
    render_state: &mut RenderState,
    render_metrics: &RenderMetrics,
    fontconfig: &Rc<FontConfiguration>,
    frame: &mut Frame,
    program: &Program,
    word: &Word,
) -> Fallible<()> {
    frame.clear_color(
        word.canvas_color.red as f32 / 255.,
        word.canvas_color.green as f32 / 255.,
        word.canvas_color.blue as f32 / 255.,
        1.0,
    );
    render_text(word, render_state, render_metrics, fontconfig)?;
    let projection = euclid::Transform3D::<f32, f32, f32>::ortho(
        -(render_metrics.win_size.width as f32) / 2.0,
        render_metrics.win_size.width as f32 / 2.0,
        render_metrics.win_size.height as f32 / 2.0,
        -(render_metrics.win_size.height as f32) / 2.0,
        -1.0,
        1.0,
    )
    .to_arrays();
    let tex = render_state.glyph_cache.atlas.texture();

    let draw_params =
        glium::DrawParameters { blend: glium::Blend::alpha_blending(), ..Default::default() };

    frame.draw(
        &render_state.glyph_vertex_buffer,
        &render_state.glyph_index_buffer,
        &program,
        &uniform! {
            projection: projection,
            glyph_tex: &*tex,
            bg_and_line_layer: true
        },
        &draw_params,
    )?;

    frame.draw(
        &render_state.glyph_vertex_buffer,
        &render_state.glyph_index_buffer,
        &program,
        &uniform! {
            projection: projection,
            glyph_tex: &*tex,
            bg_and_line_layer: false
        },
        &draw_params,
    )?;

    Ok(())
}

fn render_text(
    word: &Word,
    render_state: &mut RenderState,
    render_metrics: &RenderMetrics,
    fontconfig: &FontConfiguration,
) -> Fallible<()> {
    let cell_width = render_metrics.cell_size.width as f32;
    let num_cols = render_metrics.win_size.width as usize / cell_width as usize;
    let vb = &mut render_state.glyph_vertex_buffer;
    let mut vertices = vb
        .slice_mut(..)
        .ok_or_else(|| failure::err_msg("we're confused about the screen size"))?
        .map();
    let fg_color = rgbcolor_to_color(word.style.fg_color);

    let font = fontconfig.resolve_font(&word.style)?;
    let glyph_info = font.shape(&word)?;

    for (cell_idx, info) in glyph_info.iter().enumerate() {
        let glyph = render_state.glyph_cache.cached_glyph(&font, info, &word.style)?;

        let left = (glyph.x_offset + glyph.bearing_x).get() as f32;
        let top = ((PixelLength::new(render_metrics.cell_size.height as f64)
            + render_metrics.descender)
            - (glyph.y_offset + glyph.bearing_y))
            .get() as f32;
        let underline_tex_rect = render_state
            .util_sprites
            .select_sprite(word.style.strikethrough, word.style.underline)
            .texture_coords();

        if cell_idx >= num_cols {
            break;
        }

        let texture = glyph.texture.as_ref().unwrap_or(&render_state.util_sprites.white_space);

        let pixel_rect = pixel_rect(glyph.scale as f32, texture);
        let texture_rect = texture.texture.to_texture_coords(pixel_rect);
        let bottom = (pixel_rect.size.height as f32 * glyph.scale as f32) + top
            - render_metrics.cell_size.height as f32;
        let right = pixel_rect.size.width as f32 + left - render_metrics.cell_size.width as f32;

        let mut quad = Quad::for_cell(cell_idx, &mut vertices);

        quad.set_fg_color(fg_color);
        if let Some(bg_color) = word.style.bg_color {
            let bg_color = rgbcolor_to_color(bg_color);
            quad.set_bg_color(bg_color);
        }
        quad.set_texture(texture_rect);
        quad.set_underline(underline_tex_rect);
        quad.set_texture_adjust(left, top, right, bottom);
        quad.set_has_color(glyph.has_color);
    }
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

    let input_path = matches.value_of("input").unwrap_or("examples/0.json");
    run(input_path)?;
    Ok(())
}
