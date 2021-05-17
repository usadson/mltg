struct Application {
    back_buffer: Vec<mltg::d2d::Bitmap>,
    context: mltg::Context<mltg::Direct2D>,
    white_brush: mltg::Brush,
    text: mltg::TextLayout,
}

impl Application {
    pub fn new() -> anyhow::Result<Self> {
        let window = wita::WindowBuilder::new().title("mltg d2d").build()?;
        let window_size = window.inner_size();
        let backend =
            mltg::Direct2D::new(window.raw_handle(), (window_size.width, window_size.height))?;
        let context = mltg::Context::new(backend)?;
        let back_buffer = context.back_buffers(context.backend().swap_chain())?;
        let white_brush = context.solid_color_brush((1.0, 1.0, 1.0, 1.0))?;
        let text_format =
            context.text_format("Meiryo", mltg::font_point(14.0), &Default::default())?;
        let text = context.text_layout("abcdefghijklmnopqrstuvwxyz", &text_format)?;
        context.set_dpi(window.dpi() as _);
        Ok(Self {
            back_buffer,
            context,
            white_brush,
            text,
        })
    }
}

impl wita::EventHandler for Application {
    fn draw(&mut self, window: &wita::Window) {
        let window_size = window.inner_size().to_logical(window.dpi());
        let hw = window_size.width as f32 / 2.0;
        let hh = window_size.height as f32 / 2.0;
        let white_rect = {
            let margin = 30.0;
            let pos = mltg::point(margin, margin);
            let pos2 = pos.x * 2.0;
            mltg::rect(pos, (hw - pos2, hh - pos2))
        };
        let text_box = mltg::rect((hw, hh), self.text.size());
        let path = self.context.path()
            .begin((30.0, hh + 30.0))
            .cubic_bezier_to(
                (hw / 2.0, hh + 30.0),
                (hw / 2.0, window_size.height as f32 - 30.0), 
                (hw - 30.0, window_size.height as f32 - 30.0)
            )
            .end(mltg::FigureEnd::Open)
            .build();
        self.context.draw(&self.back_buffer[0], |cmd| {
            cmd.clear((0.0, 0.0, 0.3, 0.0));
            cmd.fill(&white_rect, &self.white_brush);
            cmd.stroke(&text_box, &self.white_brush, 2.0);
            cmd.draw_text(&self.text, &self.white_brush, (hw, hh));
            cmd.stroke(&path, &self.white_brush, 5.0);
        });
    }

    fn dpi_changed(&mut self, window: &wita::Window) {
        self.context.set_dpi(window.dpi() as _);
        self.resizing(window, window.inner_size());
    }

    fn resizing(&mut self, window: &wita::Window, size: wita::PhysicalSize<u32>) {
        self.back_buffer.clear();
        self.context.backend().resize((size.width, size.height));
        self.back_buffer = self
            .context
            .back_buffers(self.context.backend().swap_chain())
            .unwrap();
        window.redraw();
    }
}

fn main() {
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
