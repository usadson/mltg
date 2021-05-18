use crate::bindings::Windows::Win32::Graphics::{Direct2D::*, DirectWrite::*, Dxgi::*};
use crate::utility::*;
use crate::*;
use windows::{Abi, Interface};

pub struct Command<'a>(&'a ID2D1DeviceContext);

impl<'a> Command<'a> {
    #[inline]
    pub fn clear(&self, color: impl Into<Rgba>) {
        unsafe {
            let color: D2D1_COLOR_F = color.into().into();
            self.0.Clear(&color);
        }
    }

    #[inline]
    pub fn fill(&self, object: &impl Fill, brush: &Brush) {
        object.fill(self.0, &brush.0);
    }

    #[inline]
    pub fn stroke(&self, object: &impl Stroke, brush: &Brush, width: f32, style: Option<&StrokeStyle>) {
        object.stroke(self.0, &brush.0, width, style.map(|s| s.0.clone()));
    }

    #[inline]
    pub fn draw_text(&self, layout: &TextLayout, brush: &Brush, origin: impl Into<Point>) {
        layout.draw(&self.0, brush, origin.into());
    }
}

pub trait Backend {
    type RenderTarget: Target;

    fn device_context(&self) -> &ID2D1DeviceContext;
    fn d2d1_factory(&self) -> &ID2D1Factory1;
    fn back_buffers(
        &self,
        swap_chain: &IDXGISwapChain1,
    ) -> windows::Result<Vec<Self::RenderTarget>>;
    fn begin_draw(&self, target: &Self::RenderTarget);
    fn end_draw(&self, target: &Self::RenderTarget);
}

pub struct Context<T> {
    backend: T,
    dwrite_factory: IDWriteFactory,
}

impl<T> Context<T>
where
    T: Backend,
{
    #[inline]
    pub fn new(backend: T) -> windows::Result<Self> {
        unsafe {
            let dwrite_factory = {
                let mut p = None;
                DWriteCreateFactory(
                    DWRITE_FACTORY_TYPE_SHARED,
                    &IDWriteFactory::IID,
                    p.set_abi() as _,
                )
                .and_some(p)?
            };
            Ok(Self {
                backend,
                dwrite_factory,
            })
        }
    }

    #[inline]
    pub fn backend(&self) -> &T {
        &self.backend
    }

    #[inline]
    pub fn back_buffers(
        &self,
        swap_chain: &impl windows::Interface,
    ) -> windows::Result<Vec<T::RenderTarget>> {
        self.backend
            .back_buffers(&swap_chain.cast().expect("cannot cast to IDXGISwapChain1"))
    }

    #[inline]
    pub fn solid_color_brush(&self, color: impl Into<Rgba>) -> windows::Result<Brush> {
        let color: D2D1_COLOR_F = color.into().into();
        let brush = unsafe {
            let mut p = None;
            self.backend
                .device_context()
                .CreateSolidColorBrush(&color, std::ptr::null(), &mut p)
                .and_some(p)?
        };
        Ok(Brush(brush.into()))
    }

    #[inline]
    pub fn path(&self) -> PathBuilder {
        let geometry = unsafe {
            let mut p = None;
            self.backend
                .d2d1_factory()
                .CreatePathGeometry(&mut p)
                .and_some(p)
                .unwrap()
        };
        PathBuilder::new(geometry)
    }

    #[inline]
    pub fn stroke_style(&self, props: &StrokeStyleProperties) -> windows::Result<StrokeStyle> {
        StrokeStyle::new(self.backend.d2d1_factory(), props)
    }

    #[inline]
    pub fn text_format(
        &self,
        font_name: impl AsRef<str>,
        size: impl Into<f32>,
        style: &TextStyle,
    ) -> windows::Result<TextFormat> {
        TextFormat::new(&self.dwrite_factory, font_name.as_ref(), size.into(), style)
    }

    #[inline]
    pub fn text_layout(
        &self,
        text: impl AsRef<str>,
        format: &TextFormat,
    ) -> windows::Result<TextLayout> {
        TextLayout::new(&self.dwrite_factory, text.as_ref(), format)
    }

    #[inline]
    pub fn set_dpi(&self, dpi: f32) {
        unsafe {
            self.backend.device_context().SetDpi(dpi, dpi);
        }
    }

    pub fn draw(&self, target: &T::RenderTarget, f: impl FnOnce(&Command)) {
        let device_context = self.backend.device_context();
        unsafe {
            self.backend.begin_draw(target);
            device_context.SetTarget(target.bitmap());
            device_context.BeginDraw();
            f(&Command(self.backend.device_context()));
            device_context
                .EndDraw(std::ptr::null_mut(), std::ptr::null_mut())
                .unwrap();
            device_context.SetTarget(None);
            self.backend.end_draw(target);
        }
    }
}
