struct Application {
    back_buffer: mltg::RenderTarget<mltg::Direct2D>,
    context: mltg::Context<mltg::Direct2D>,
    factory: mltg::Factory,
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
        let backend = mltg::Direct2D::new()?;
        let context = mltg::Context::new(backend)?;
        let factory = context.create_factory();
        let back_buffer = context
            .create_render_target(window.raw_handle(), (window_size.width, window_size.height))?;
        let white_brush = factory.create_solid_color_brush([1.0, 1.0, 1.0, 1.0])?;
        let grad = factory.create_gradient_stop_collection(&[
            (0.0, [1.0, 0.0, 0.0, 1.0]),
            (1.0, [0.0, 1.0, 0.0, 1.0]),
        ])?;
        let text_format = factory.create_text_format(
            mltg::Font::System("Meiryo"),
            mltg::font_point(14.0),
            None,
        )?;
        let text = factory.create_text_layout(
            "abcdefghijklmnopqrstuvwxyz",
            &text_format,
            mltg::TextAlignment::Leading,
            None,
        )?;
        let text2 = factory.create_text_layout(
            "abcdefghijklmnopqrstuvwxyz",
            &text_format,
            mltg::TextAlignment::Center,
            Some(text.size() + (30.0, 30.0)),
        )?;
        let stroke_style = factory.create_stroke_style(&mltg::StrokeStyleProperties {
            start_cap: mltg::CapStyle::Triangle,
            end_cap: mltg::CapStyle::Round,
            line_join: mltg::LineJoin::Round,
            dash: Some(mltg::Dash {
                style: mltg::DashStyle::Dash,
                ..Default::default()
            }),
        })?;
        let image = factory.create_image("examples/ferris.png")?;
        context.set_dpi(window.dpi() as _);
        Ok(Self {
            back_buffer,
            context,
            factory,
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
    fn draw(&mut self, ev: wita::event::Draw) {
        let window_size = ev.window.inner_size().to_logical(ev.window.dpi());
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
            .factory
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
            .factory
            .create_linear_gradient_brush((30.0, 30.0), (hw - 30.0, hh - 30.0), &self.grad)
            .unwrap();
        let ret = self.context.draw(&self.back_buffer, |cmd| {
            cmd.clear([0.0, 0.0, 0.3, 0.0]);
            cmd.fill(&rect, &linear_grad_brush);
            cmd.stroke(&text_box, &self.white_brush, 2.0, None);
            cmd.draw_text_layout(&self.text, &self.white_brush, (hw, hh));
            cmd.stroke(&text_box2, &self.white_brush, 2.0, None);
            cmd.draw_text_layout(&self.text2, &self.white_brush, (hw, hh + 50.0));
            cmd.draw_image(
                &self.image,
                mltg::Rect::new((hw, 10.0), image_size),
                None,
                mltg::Interpolation::HighQualityCubic,
            );
            cmd.stroke(&path, &self.white_brush, 5.0, Some(&self.stroke_style));
        });
        match ret {
            Ok(_) => {
                self.back_buffer.present(0, None, None);
            }
            Err(e) if e == mltg::ErrorKind::RecreateTarget => {
                let size = ev.window.inner_size();
                self.back_buffer = self
                    .context
                    .create_render_target(ev.window.raw_handle(), (size.width, size.height))
                    .unwrap();
                ev.window.redraw();
            }
            Err(e) => panic!("{:?}", e),
        }
    }

    fn dpi_changed(&mut self, ev: wita::event::DpiChanged) {
        self.context.set_dpi(ev.new_dpi as _);
        self.resizing(wita::event::Resizing {
            window: ev.window,
            size: &mut ev.window.inner_size(),
            edge: wita::ResizingEdge::Right,
        });
    }

    fn resizing(&mut self, ev: wita::event::Resizing) {
        self.back_buffer
            .resize((ev.size.width, ev.size.height))
            .ok();
        ev.window.redraw();
    }
}

fn main() {
    let _coinit = coinit::init(coinit::APARTMENTTHREADED | coinit::DISABLE_OLE1DDE).unwrap();
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
