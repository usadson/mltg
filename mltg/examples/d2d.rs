struct Application {
    back_buffer: Vec<mltg::d2d::RenderTarget>,
    context: mltg::Context<mltg::Direct2D>,
    white_brush: mltg::Brush,
    grad: mltg::GradientStopCollection,
    text: mltg::TextLayout,
    text2: mltg::TextLayout,
    stroke_style: mltg::StrokeStyle,
    image: mltg::Image,
}

impl Application {
    pub fn new() -> anyhow::Result<Self> {
        let window = wita::WindowBuilder::new().title("mltg d2d").build()?;
        let window_size = window.inner_size();
        let backend =
            mltg::Direct2D::new(window.raw_handle(), (window_size.width, window_size.height))?;
        let context = mltg::Context::new(backend)?;
        let back_buffer = context.create_back_buffers(context.backend().swap_chain())?;
        let white_brush = context.create_solid_color_brush([1.0, 1.0, 1.0, 1.0])?;
        let grad = context.create_gradient_stop_collection(&[
            (0.0, [1.0, 0.0, 0.0, 1.0]),
            (1.0, [0.0, 1.0, 0.0, 1.0]),
        ])?;
        let text_format = context.create_text_format(
            &mltg::Font::system("Meiryo"),
            mltg::font_point(14.0),
            None,
        )?;
        let text = context.create_text_layout(
            "abcdefghijklmnopqrstuvwxyz",
            &text_format,
            mltg::TextAlignment::Leading,
            None,
        )?;
        let text2 = context.create_text_layout(
            "abcdefghijklmnopqrstuvwxyz",
            &text_format,
            mltg::TextAlignment::Center,
            Some(text.size() + (30.0, 30.0)),
        )?;
        let stroke_style = context.create_stroke_style(&mltg::StrokeStyleProperties {
            start_cap: mltg::CapStyle::Triangle,
            end_cap: mltg::CapStyle::Round,
            line_join: mltg::LineJoin::Round,
            dash: Some(mltg::Dash {
                style: mltg::DashStyle::Dash,
                ..Default::default()
            }),
        })?;
        let image = context.create_image("ferris.png")?;
        context.set_dpi(window.dpi() as _);
        Ok(Self {
            back_buffer,
            context,
            white_brush,
            grad,
            text,
            text2,
            stroke_style,
            image,
        })
    }
}

impl wita::EventHandler for Application {
    fn draw(&mut self, window: &wita::Window) {
        let window_size = window.inner_size().to_logical(window.dpi());
        let hw = window_size.width as f32 / 2.0;
        let hh = window_size.height as f32 / 2.0;
        let rect = {
            let margin = 30.0;
            let pos = mltg::point(margin, margin);
            let pos2 = pos.x * 2.0;
            mltg::rect(pos, (hw - pos2, hh - pos2))
        };
        let text_box = mltg::rect((hw, hh), self.text.size());
        let text_box2 = mltg::rect((hw, hh + 50.0), self.text2.size());
        let path = self
            .context
            .create_path()
            .begin((30.0, hh + 30.0))
            .cubic_bezier_to(
                (hw / 2.0, hh + 30.0),
                (hw / 2.0, window_size.height as f32 - 30.0),
                (hw - 30.0, window_size.height as f32 - 30.0),
            )
            .end(mltg::FigureEnd::Open)
            .build();
        let image_size = {
            let size = self.image.size();
            (size.width as f32 / 4.0, size.height as f32 / 4.0)
        };
        let linear_grad_brush = self
            .context
            .create_linear_gradient_brush((30.0, 30.0), (hw - 30.0, hh - 30.0), &self.grad)
            .unwrap();
        self.context.draw(&self.back_buffer[0], |cmd| {
            cmd.clear([0.0, 0.0, 0.3, 0.0]);
            cmd.fill(&rect, &linear_grad_brush);
            cmd.stroke(&text_box, &self.white_brush, 2.0, None);
            cmd.draw_text(&self.text, &self.white_brush, (hw, hh));
            cmd.stroke(&text_box2, &self.white_brush, 2.0, None);
            cmd.draw_text(&self.text2, &self.white_brush, (hw, hh + 50.0));
            cmd.draw_image(
                &self.image,
                mltg::Rect::new((hw, 10.0), image_size),
                None,
                mltg::Interpolation::HighQualityCubic,
            );
            cmd.stroke(&path, &self.white_brush, 5.0, Some(&self.stroke_style));
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
            .create_back_buffers(self.context.backend().swap_chain())
            .unwrap();
        window.redraw();
    }
}

fn main() {
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
