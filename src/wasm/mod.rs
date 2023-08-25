use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

//引入 wasm-bindgen
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

//接下来，需要告诉 wasm-bindgen 在 WASM 被加载后执行我们的 run() 函数。
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    //初始化日志输出
    //根据是否在 WASM 环境来切换我们正在使用的日志包
    cfg_if::cfg_if! {
        if #[cfg(target_arch="wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("无法初始化日志库");
        } else {
            env_logger::init();
        }
    }

    //初始化窗口
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    //我们需要在应用程序所在的 HTML 网页中添加一个画布
    //Winit 不允许用 CSS 调整大小，所以在 web 环境里我们必须手动设置大小。
    #[cfg(target_arch = "wasm32")] {
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 500));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm_example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            }).expect("无法将画布添加到网页上");
    }

    //运行窗口
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }
            _ => {}
        }
    });
}
