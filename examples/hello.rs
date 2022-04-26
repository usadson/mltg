use windows::Win32::System::Com::*;

struct Application {
    context: mltg::Context<mltg::Direct2D>,
    back_buffer: mltg::d2d::RenderTarget,
    white: mltg::Brush,
    text_format: mltg::TextFormat,
}

impl Application {
    fn new() -> anyhow::Result<Self> {
        unsafe {
            CoInitialize(std::ptr::null_mut())?;
        }
        let window = wita::WindowBuilder::new().title("mltg hello").build()?;
        let window_size = window.inner_size();
        let backend = mltg::Direct2D::new()?;
        let context = mltg::Context::new(backend)?;
        let factory = context.create_factory();
        let back_buffer = context
            .create_render_target(window.raw_handle(), (window_size.width, window_size.height))?;
        let white = factory.create_solid_color_brush([1.0, 1.0, 1.0, 1.0])?;
        let text_format = factory.create_text_format(
            mltg::Font::System("Yu Gothic UI"),
            mltg::font_point(28.0),
            None,
        )?;
        context.set_dpi(window.dpi() as _);
        Ok(Self {
            context,
            back_buffer,
            white,
            text_format,
        })
    }
}

impl wita::EventHandler for Application {
    fn draw(&mut self, _: wita::event::Draw) {
        self.context
            .draw(&self.back_buffer, |cmd| {
                cmd.clear([0.0, 0.0, 0.3, 0.0]);
                cmd.draw_text("Hello, world!", &self.text_format, &self.white, (0.0, 0.0));
            })
            .unwrap();
        self.back_buffer.present(None, None);
    }

    fn dpi_changed(&mut self, ev: wita::event::DpiChanged) {
        self.context.set_dpi(ev.new_dpi as _);
        self.resizing(wita::event::Resizing {
            window: ev.window,
            size: &mut ev.window.inner_size(),
        });
    }

    fn resizing(&mut self, ev: wita::event::Resizing) {
        self.back_buffer
            .resize((ev.size.width, ev.size.height))
            .unwrap();
        ev.window.redraw();
    }
}

fn main() {
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
