use windows::Win32::System::Com::{
    CoInitializeEx, CoUninitialize, COINIT_DISABLE_OLE1DDE, COINIT_MULTITHREADED,
};
use witas::ToLogical;

async fn main_loop() -> anyhow::Result<()> {
    let (window, mut rx) = witas::Window::builder()
        .title("mltg witas")
        .inner_size(witas::LogicalSize::new(640, 480))
        .build()
        .await?;
    let ctx = mltg::Context::new(mltg::Direct2D::new()?)?;
    let factory = ctx.create_factory();
    let window_size = window.inner_size().await.unwrap();
    let render_target =
        ctx.create_render_target(window.raw_handle(), (window_size.width, window_size.height))?;
    let image = factory.create_image_from_file("./resources/ferris.png")?;
    loop {
        let event = rx.recv().await;
        match event {
            witas::Event::Draw(_) => {
                let Some(window_size) = window
                    .inner_size()
                    .await else { continue };
                let Some(dpi) = window.dpi().await else { continue };
                let window_size = window_size.to_logical(dpi as _);
                ctx.set_dpi(dpi as _);
                ctx.draw(&render_target, |cmd| {
                    cmd.clear((0.0, 0.0, 0.3, 0.0));
                    cmd.draw_image(
                        &image,
                        mltg::Rect::new(
                            (0.0, 0.0),
                            (window_size.width as f32, window_size.height as f32),
                        ),
                        None,
                        mltg::Interpolation::HighQualityCubic,
                    );
                })
                .unwrap();
            }
            witas::Event::Quit => break,
            _ => {}
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .on_thread_start(|| unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED | COINIT_DISABLE_OLE1DDE).unwrap();
        })
        .on_thread_stop(|| unsafe {
            CoUninitialize();
        })
        .build()?
        .block_on(main_loop())
}
