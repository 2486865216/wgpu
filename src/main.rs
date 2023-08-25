use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wgpu_01::surface::State;

use pollster::block_on;

fn main() {
    //WASM 环境中不能在异步函数里使用 block_on。
    // Future（异步函数的返回对象）必须使用浏览器的执行器来运行。如果你试图使用自己的执行器，一旦遇到没有立即执行的 Future 时代码就会崩溃。
    block_on(run());
}

//现在 run() 是异步的了，main() 需要某种方式来等待它执行完成。我们可以使用 tokio 或 async-std 等异步包，但我打算使用更轻量级的 pollster
pub async fn run() {
    //初始化日志输出
    env_logger::init();

    //初始化窗口
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window).await;

    //运行窗口
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id
            } if window_id == window.id() => if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => *control_flow = ControlFlow::Exit,

                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }

                    WindowEvent::ScaleFactorChanged {new_inner_size, ..} => {
                        // new_inner_size 是 &&mut 类型，因此需要解引用两次
                        state.resize(**new_inner_size);
                    }

                    _ => {}
                }
            }

            Event::RedrawRequested(window_id) if window_id == window.id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // 当展示平面的上下文丢失，就需重新配置
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // 系统内存不足时，程序应该退出。
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // 所有其他错误（过期、超时等）应在下一帧解决
                    Err(e) => eprintln!("{:?}", e),
                }
            }

            /*Event::MainEventsCleared => {
                 // 除非我们手动请求，RedrawRequested 将只会触发一次。
                surface.request_redraw();
            }*/

            _ => {}
        }
    });
}
