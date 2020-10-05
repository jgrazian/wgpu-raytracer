mod aabb;
mod app;
mod bvh;
mod geometry;
mod globals;
mod material;
mod math;
mod pipelines;
mod traits;

use futures::executor::block_on;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = block_on(app::State::new(&window));

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            state.update();
            state.render();
        }

        Event::MainEventsCleared => window.request_redraw(),

        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !state.input(event) {
                match event {
                    //Closed the window
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    //Resized the window
                    WindowEvent::Resized(physical_size) => state.resize(*physical_size),
                    //Change Window Scale
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size)
                    }
                    //Keyboard
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        _ => {}
    });
}
