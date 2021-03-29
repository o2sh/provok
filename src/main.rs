#[macro_use]
extern crate glium;
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
use std::time::{Duration, Instant};

mod cell;
mod color;
mod config;
mod font;
mod line;
mod renderstate;

use config::Config;
use font::FontConfiguration;
use line::Line;
use renderstate::{RenderMetrics, RenderState};

fn main() -> Fallible<()> {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new().with_inner_size(LogicalSize::new(405., 720.));
    let cb = ContextBuilder::new();
    let display = Display::new(wb, cb, &event_loop)?;
    let config = Config::default();
    let fontconfig = FontConfiguration::new(&config);
    let render_metrics = RenderMetrics::new(fontconfig, 405., 720.);
    let render_state = RenderState::new(&display, &render_metrics)?;
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
        paint(&render_state, &render_metrics, &mut target).unwrap();
        target.finish().unwrap();
    });
}

fn paint(
    render_state: &RenderState,
    render_metrics: &RenderMetrics,
    frame: &mut Frame,
) -> Fallible<()> {
    frame.clear_color(0.0, 0.0, 1.0, 1.0);
    render_text(render_state, render_metrics)?;
    let projection = euclid::Transform3D::<f32, f32, f32>::ortho(
        -(render_metrics.win_size.width as f32) / 2.0,
        render_metrics.win_size.width as f32 / 2.0,
        render_metrics.win_size.height as f32 / 2.0,
        -(render_metrics.win_size.height as f32) / 2.0,
        -1.0,
        1.0,
    )
    .to_arrays();

    frame.draw(
        &render_state.glyph_vertex_buffer,
        &render_state.glyph_index_buffer,
        &render_state.program,
        &uniform! {
            projection: projection,
        },
        &Default::default(),
    )?;

    Ok(())
}

fn render_text(render_state: &RenderState, render_metrics: &RenderMetrics) -> Fallible<()> {
    let line = Line::from("ossama");
    Ok(())
}
