use raw_window_handle::HasRawWindowHandle;
use std::path::Path;
use winit::{dpi::*, event::*, event_loop::*, window::*};

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("mltg d2d")
        .with_inner_size(LogicalSize::new(640, 480))
        .build(&event_loop)?;
    let ctx = mltg::Context::new(mltg::Direct2D::new()?)?;
    let factory = ctx.create_factory();
    let window_size = window.inner_size();
    let mut render_target = ctx.create_render_target(
        window.raw_window_handle(),
        (window_size.width, window_size.height),
    )?;
    let white_brush = factory.create_solid_color_brush((1.0, 1.0, 1.0, 1.0))?;
    let grad = factory.create_gradient_stop_collection(
        mltg::GradientMode::Clamp,
        &[
            (0.0, (1.0, 0.0, 0.0, 1.0)),
            (0.5, (0.0, 1.0, 0.0, 1.0)),
            (1.0, (0.0, 0.0, 1.0, 1.0)),
        ],
    )?;
    let grad_brush = factory.create_linear_gradient_brush((10.0, 10.0), (100.0, 100.0), &grad)?;
    let text_format = factory.create_text_format(
        mltg::Font::File(
            Path::new("./resources/Inconsolata/static/Inconsolata/Inconsolata-Regular.ttf"),
            "Inconsolata",
        ),
        mltg::font_point(14.0),
        None,
        None,
    )?;
    let text = factory.create_text_layout(
        "abcdefghijklmnopqrstuvwxyz",
        &text_format,
        mltg::TextAlignment::Leading,
        None,
    )?;
    let text_size = text.size();
    let image = factory.create_image_from_file("./resources/ferris.png")?;
    let path = factory
        .create_hollow_path()?
        .begin((10.0, 200.0))
        .cubic_bezier_to((80.0, 200.0), (100.0, 300.0), (200.0, 300.0))
        .end(mltg::FigureEnd::Closed)?
        .close()?;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::RedrawRequested(_) => {
                let window_size = window.inner_size().to_logical::<f32>(window.scale_factor());
                let image_size = image.size().cast::<f32>().unwrap() / 4.0;
                ctx.set_scale_factor(window.scale_factor() as _);
                ctx.draw(&render_target, |cmd| {
                    cmd.clear((0.0, 0.0, 0.3, 0.0));
                    cmd.fill(
                        &mltg::Rect::from_points((10.0, 10.0), (100.0, 100.0)),
                        &grad_brush,
                    );
                    cmd.fill(&text.position((50.0, 150.0)), &white_brush);
                    cmd.stroke(
                        &mltg::Rect::new((50.0, 150.0), text_size),
                        &white_brush,
                        2.0,
                        None,
                    );
                    cmd.stroke(&path, &white_brush, 3.0, None);
                    cmd.draw_image(
                        &image,
                        mltg::Rect::new(
                            (window_size.width - image_size.width - 10.0, 10.0),
                            image_size,
                        ),
                        None,
                        mltg::Interpolation::HighQualityCubic,
                    );
                })
                .unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                ctx.resize_target(&mut render_target, (size.width, size.height))
                    .unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
