use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::windows::WindowExtWindows,
    window::WindowBuilder,
};

fn main() -> anyhow::Result<()> {
    let _coinit = coinit::init(coinit::APARTMENTTHREADED | coinit::DISABLE_OLE1DDE).unwrap();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("mltg winit d2d")
        .with_inner_size(LogicalSize::new(640, 480))
        .build(&event_loop)?;
    let window_size = window.inner_size();
    let context = mltg::Context::new(mltg::Direct2D::new()?)?;
    let mut back_buffer =
        context.create_render_target(window.hwnd(), (window_size.width, window_size.height))?;
    let image = {
        let factory = context.create_factory();
        factory.create_image("examples/ferris.png")?
    };
    context.set_dpi(window.scale_factor() as f32 * 96.0);
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::RedrawRequested(_) => {
                let window_size = window.inner_size().to_logical::<f32>(window.scale_factor());
                let hw = window_size.width / 2.0;
                let hh = window_size.height / 2.0;
                let image_size = image.size().cast::<f32>().unwrap();
                let size = (hw, image_size.height * hw / image_size.width);
                let ret = context.draw(&back_buffer, |cmd| {
                    cmd.clear([0.0, 0.0, 0.3, 0.0]);
                    cmd.draw_image(
                        &image,
                        mltg::Rect::new((hw - hw / 2.0, hh - size.1 / 2.0), size),
                        None,
                        mltg::Interpolation::HighQualityCubic,
                    );
                });
                match ret {
                    Ok(_) => {
                        back_buffer.present(0, None, None);
                    }
                    Err(e) if e == mltg::ErrorKind::RecreateTarget => {
                        let size = window.inner_size();
                        back_buffer = context
                            .create_render_target(window.hwnd(), (size.width, size.height))
                            .unwrap();
                        window.request_redraw();
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                back_buffer.resize((size.width, size.height)).ok();
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
