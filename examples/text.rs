const TEXT: &str = "abcdefghijklmnopqrstuvwxyzffffff";

struct Application {
    context: mltg::Context<mltg::Direct2D>,
    factory: mltg::Factory,
    render_target: mltg::RenderTarget<mltg::Direct2D>,
    text_format: mltg::TextFormat,
    text: mltg::TextLayout,
    point: mltg::TextLayout,
    hit: mltg::TextLayout,
    text_position: usize,
    white: mltg::Brush,
}

impl Application {
    fn new() -> anyhow::Result<Self> {
        let window = wita::Window::builder().title("mltg text").build()?;
        let window_size = window.inner_size();
        let context = mltg::Context::new(mltg::Direct2D::new()?)?;
        context.set_dpi(window.dpi() as _);
        let factory = context.create_factory();
        let render_target = context
            .create_render_target(window.raw_handle(), (window_size.width, window_size.height))?;
        let text_format = factory.create_text_format(
            mltg::Font::System("Meiryo"),
            mltg::font_point(14.0),
            None,
        )?;
        let text =
            factory.create_text_layout(TEXT, &text_format, mltg::TextAlignment::Leading, None)?;
        let point = factory.create_text_layout(
            "a: (0.0, 0.0)",
            &text_format,
            mltg::TextAlignment::Leading,
            None,
        )?;
        let hit = factory.create_text_layout(
            "(0.0, 0.0): [0] = a, false",
            &text_format,
            mltg::TextAlignment::Leading,
            None,
        )?;
        let white = factory.create_solid_color_brush([1.0, 1.0, 1.0, 1.0])?;
        Ok(Self {
            context,
            factory,
            render_target,
            text_format,
            text,
            point,
            hit,
            text_position: 0,
            white,
        })
    }
}

impl wita::EventHandler for Application {
    fn draw(&mut self, ev: wita::event::Draw) {
        let ret = self.context.draw(&self.render_target, |cmd| {
            cmd.clear([0.0, 0.0, 0.3, 0.0]);
            cmd.draw_text_layout(&self.text, &self.white, (0.0, 0.0));
            cmd.draw_text_layout(
                &self.point,
                &self.white,
                (0.0, self.text.size().height + 10.0),
            );
            cmd.draw_text_layout(
                &self.hit,
                &self.white,
                (
                    0.0,
                    self.text.size().height + self.point.size().height + 20.0,
                ),
            )
        });
        match ret {
            Ok(_) => {
                self.render_target.present(None, None);
            }
            Err(e) if e == mltg::ErrorKind::RecreateTarget => {
                let size = ev.window.inner_size();
                self.render_target = self
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
        ev.window.redraw();
    }

    fn resizing(&mut self, ev: wita::event::Resizing) {
        self.render_target
            .resize((ev.size.width, ev.size.height))
            .ok();
        ev.window.redraw();
    }

    fn mouse_input(&mut self, ev: wita::event::MouseInput) {
        if ev.button == wita::MouseButton::Left && ev.button_state == wita::KeyState::Pressed {
            let position = ev
                .mouse_state
                .position
                .to_logical(ev.window.dpi() as _)
                .cast::<f32>();
            let ret = self.text.hit_test((position.x, position.y));
            self.hit = self
                .factory
                .create_text_layout(
                    &format!(
                        "({}, {}): [{}] = {}, {}",
                        position.x,
                        position.y,
                        ret.text_position,
                        TEXT.chars().nth(ret.text_position).unwrap_or('-'),
                        ret.trailing_hit
                    ),
                    &self.text_format,
                    mltg::TextAlignment::Leading,
                    None,
                )
                .unwrap();
            ev.window.redraw();
        }
    }

    fn key_input(&mut self, ev: wita::event::KeyInput) {
        let create_layout = |text_position| {
            let pos = self.text.text_position_to_point(text_position, false);
            self.factory
                .create_text_layout(
                    &format!(
                        "{}: ({}, {})",
                        TEXT.chars().nth(text_position).unwrap(),
                        pos.x,
                        pos.y
                    ),
                    &self.text_format,
                    mltg::TextAlignment::Leading,
                    None,
                )
                .unwrap()
        };
        match ev.key_code.vkey {
            wita::VirtualKey::Left if ev.state == wita::KeyState::Pressed => {
                if self.text_position > 0 {
                    self.text_position -= 1;
                    self.point = create_layout(self.text_position);
                }
                ev.window.redraw();
            }
            wita::VirtualKey::Right if ev.state == wita::KeyState::Pressed => {
                if self.text_position < TEXT.len() - 1 {
                    self.text_position += 1;
                    self.point = create_layout(self.text_position);
                }
                ev.window.redraw();
            }
            _ => {}
        }
    }
}

fn main() {
    let _coinit = coinit::init(coinit::APARTMENTTHREADED | coinit::DISABLE_OLE1DDE).unwrap();
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
