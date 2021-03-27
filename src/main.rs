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
use glium::Display;
use std::time::{Duration, Instant};

mod color;
mod config;
mod font;
mod renderer;
use renderer::Renderer;

fn main() -> Fallible<()> {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new().with_inner_size(LogicalSize::new(405., 720.));
    let cb = ContextBuilder::new();
    let display = Display::new(wb, cb, &event_loop)?;

    let renderer = Renderer::new(&display, 405., 720.)?;
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
        renderer.paint(&mut target).unwrap();
        target.finish().unwrap();
    });
}
