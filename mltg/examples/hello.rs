struct Application {
    context: mltg::Context<mltg::Direct2D>,
    back_buffer: Vec<mltg::d2d::RenderTarget>,
    white: mltg::Brush,
    text_layout: mltg::TextLayout,
}

impl Application {
    fn new() -> anyhow::Result<Self> {
        let window = wita::WindowBuilder::new().title("mltg hello").build()?;
        let window_size = window.inner_size();
        let backend =
            mltg::Direct2D::new(window.raw_handle(), (window_size.width, window_size.height))?;
        let context = mltg::Context::new(backend)?;
        let back_buffer = context.create_back_buffers(context.backend().swap_chain())?;
        let white = context.create_solid_color_brush([1.0, 1.0, 1.0, 1.0])?;
        let text_format = context.create_text_format(
            &mltg::Font::system("Yu Gothic UI"),
            mltg::font_point(28.0),
            None,
        )?;
        let text_layout = context.create_text_layout(
            "Hello, world!",
            &text_format,
            mltg::TextAlignment::Leading,
            None,
        )?;
        Ok(Self {
            context,
            back_buffer,
            white,
            text_layout,
        })
    }
}

impl wita::EventHandler for Application {
    fn draw(&mut self, _window: &wita::Window) {
        self.context.draw(&self.back_buffer[0], |cmd| {
            cmd.clear([0.0, 0.0, 0.3, 0.0]);
            cmd.draw_text(&self.text_layout, &self.white, (0.0, 0.0));
        });
    }

    fn resizing(&mut self, window: &wita::Window, size: wita::PhysicalSize<u32>) {
        self.back_buffer.clear();
        self.context.backend().resize((size.width, size.height));
        self.back_buffer = self
            .context
            .create_back_buffers(self.context.backend().swap_chain())
            .unwrap();
        window.redraw();
    }
}

fn main() {
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
