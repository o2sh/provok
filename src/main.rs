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
use glium::{Display, Frame, Surface};
use std::rc::Rc;
use std::time::{Duration, Instant};

mod bitmaps;
mod cell;
mod cellcluster;
mod color;
mod font;
mod glyphcache;
mod input;
mod line;
mod quad;
mod renderstate;
mod utils;
mod utilsprites;

use bitmaps::atlas::SpriteSlice;
use bitmaps::Texture2d;
use cell::CellAttributes;
use color::{rgbcolor_to_color, ColorPalette};
use font::FontConfiguration;
use input::{Input, Word};
use line::Line;
use quad::{Quad, VERTICES_PER_CELL};
use renderstate::{RenderMetrics, RenderState};
use utils::PixelLength;

fn run(input_path: &str) -> Fallible<()> {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new().with_inner_size(LogicalSize::new(720., 405.));
    let cb = ContextBuilder::new();
    let display = Display::new(wb, cb, &event_loop)?;
    let input = Rc::new(Input::new(input_path)?);
    let fontconfig = Rc::new(FontConfiguration::new(Rc::clone(&input)));
    let render_metrics = RenderMetrics::new(&fontconfig, 720., 405.);
    let palette = ColorPalette::default();
    let mut render_state = RenderState::new(&display, &render_metrics, &fontconfig)?;
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

        if i == input.words.len() - 1 {
            i = 0;
        } else {
            i += 1;
        }

        paint(
            &mut render_state,
            &render_metrics,
            &palette,
            &fontconfig,
            &mut target,
            &input.words[i],
        )
        .unwrap();
        render_state.recompute_glyph_vertices(&render_metrics, &display).unwrap();
        target.finish().unwrap();
    });
}

fn paint(
    render_state: &mut RenderState,
    render_metrics: &RenderMetrics,
    palette: &ColorPalette,
    fontconfig: &Rc<FontConfiguration>,
    frame: &mut Frame,
    word: &Word,
) -> Fallible<()> {
    frame.clear_color(
        word.canvas_color.red as f32 / 255.,
        word.canvas_color.green as f32 / 255.,
        word.canvas_color.blue as f32 / 255.,
        1.0,
    );
    let style = CellAttributes::from_text_style(&word.style);
    let line = Line::from_text(word.text.as_str(), &style);
    render_text(line, render_state, render_metrics, palette, fontconfig)?;
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
        &render_state.program,
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
        &render_state.program,
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
    line: Line,
    render_state: &mut RenderState,
    render_metrics: &RenderMetrics,
    palette: &ColorPalette,
    fontconfig: &FontConfiguration,
) -> Fallible<()> {
    let cell_width = render_metrics.cell_size.width as f32;
    let num_cols = render_metrics.win_size.width as usize / cell_width as usize;
    let vb = &mut render_state.glyph_vertex_buffer;
    let mut vertices = vb
        .slice_mut(..)
        .ok_or_else(|| failure::err_msg("we're confused about the screen size"))?
        .map();

    let start_pos = ((vertices.len() / VERTICES_PER_CELL) - line.len()) / 2;
    let cell_clusters = line.cluster();

    for cluster in cell_clusters {
        let attrs = &cluster.attrs;
        let style = fontconfig.get_style(attrs);
        let fg_color = palette.resolve_fg(attrs.foreground);
        let bg_color = palette.resolve_bg(attrs.background);

        let fg_color = rgbcolor_to_color(fg_color);
        let bg_color = rgbcolor_to_color(bg_color);

        let glyph_info = {
            let font = fontconfig.resolve_font(&style)?;
            font.shape(&cluster.text)?
        };

        for info in &glyph_info {
            let cell_idx = cluster.byte_to_cell_idx[info.cluster as usize];
            let glyph = render_state.glyph_cache.cached_glyph(info, &style)?;

            let left = (glyph.x_offset + glyph.bearing_x).get() as f32;
            let top = ((PixelLength::new(render_metrics.cell_size.height as f64)
                + render_metrics.descender)
                - (glyph.y_offset + glyph.bearing_y))
                .get() as f32;
            let underline_tex_rect = render_state
                .util_sprites
                .select_sprite(attrs.strikethrough(), attrs.underline())
                .texture_coords();
            for glyph_idx in 0..info.num_cells as usize {
                let cell_idx = start_pos + cell_idx + glyph_idx;

                if cell_idx >= num_cols {
                    break;
                }

                let texture =
                    glyph.texture.as_ref().unwrap_or(&render_state.util_sprites.white_space);

                let slice = SpriteSlice {
                    cell_idx: glyph_idx,
                    num_cells: info.num_cells as usize,
                    cell_width: render_metrics.cell_size.width as usize,
                    scale: glyph.scale as f32,
                    left_offset: left,
                };

                let pixel_rect = slice.pixel_rect(texture);
                let texture_rect = texture.texture.to_texture_coords(pixel_rect);
                let left = if glyph_idx == 0 { left } else { 0.0 };
                let bottom = (pixel_rect.size.height as f32 * glyph.scale as f32) + top
                    - render_metrics.cell_size.height as f32;
                let right =
                    pixel_rect.size.width as f32 + left - render_metrics.cell_size.width as f32;

                let mut quad = Quad::for_cell(cell_idx, &mut vertices);

                quad.set_fg_color(fg_color);
                quad.set_bg_color(bg_color);
                quad.set_texture(texture_rect);
                quad.set_underline(underline_tex_rect);
                quad.set_texture_adjust(left, top, right, bottom);
                quad.set_has_color(glyph.has_color);
            }
        }
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
