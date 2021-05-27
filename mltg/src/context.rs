use crate::bindings::Windows::Win32::Graphics::{DirectWrite::*, Dxgi::*, Imaging::*};
use crate::utility::*;
use crate::*;
use windows::{Abi, Interface};

pub struct Command<'a>(&'a ID2D1DeviceContext);

impl<'a> Command<'a> {
    #[inline]
    pub fn clear(&self, color: impl Into<Rgba>) {
        unsafe {
            let color: D2D1_COLOR_F = Inner(color.into()).into();
            self.0.Clear(&color);
        }
    }

    #[inline]
    pub fn fill(&self, object: &impl Fill, brush: &Brush) {
        object.fill(self.0, &brush.handle());
    }

    #[inline]
    pub fn stroke(
        &self,
        object: &impl Stroke,
        brush: &Brush,
        width: f32,
        style: Option<&StrokeStyle>,
    ) {
        object.stroke(self.0, &brush.handle(), width, style.map(|s| s.0.clone()));
    }

    #[inline]
    pub fn draw_text(&self, layout: &TextLayout, brush: &Brush, origin: impl Into<Point>) {
        layout.draw(&self.0, brush, origin.into());
    }

    #[inline]
    pub fn draw_image(
        &self,
        image: &Image,
        dest_rect: impl Into<Rect>,
        src_rect: Option<Rect>,
        interpolation: Interpolation,
    ) {
        image.draw(&self.0, dest_rect.into(), src_rect, interpolation);
    }

    #[inline]
    pub fn clip(&self, rect: impl Into<Rect>, f: impl Fn(&Command)) {
        let rect: D2D_RECT_F = Inner(rect.into()).into();
        unsafe {
            self.0
                .PushAxisAlignedClip(&rect, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
            f(self);
            self.0.PopAxisAlignedClip();
        }
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
    dwrite_factory: IDWriteFactory5,
    wic_imaging_factory: IWICImagingFactory,
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
                    &IDWriteFactory5::IID,
                    p.set_abi() as _,
                )
                .and_some(p)?
            };
            let wic_imaging_factory = windows::create_instance(&CLSID_WICImagingFactory)?;
            Ok(Self {
                backend,
                dwrite_factory,
                wic_imaging_factory,
            })
        }
    }

    #[inline]
    pub fn backend(&self) -> &T {
        &self.backend
    }

    #[inline]
    pub fn create_back_buffers(
        &self,
        swap_chain: &impl windows::Interface,
    ) -> windows::Result<Vec<T::RenderTarget>> {
        self.backend
            .back_buffers(&swap_chain.cast().expect("cannot cast to IDXGISwapChain1"))
    }

    #[inline]
    pub fn create_gradient_stop_collection<U>(
        &self,
        stops: &[U],
    ) -> windows::Result<GradientStopCollection>
    where
        U: Into<GradientStop> + Clone,
    {
        GradientStopCollection::new(self.backend.device_context(), stops)
    }

    #[inline]
    pub fn create_solid_color_brush(&self, color: impl Into<Rgba>) -> windows::Result<Brush> {
        Brush::solid_color(self.backend.device_context(), color)
    }

    #[inline]
    pub fn create_linear_gradient_brush(
        &self,
        start: impl Into<Point>,
        end: impl Into<Point>,
        stop_collection: &GradientStopCollection,
    ) -> windows::Result<Brush> {
        Brush::linear_gradient(self.backend.device_context(), start, end, stop_collection)
    }

    #[inline]
    pub fn create_radial_gradient_brush(
        &self,
        center: impl Into<Point>,
        offset: impl Into<Point>,
        radius: impl Into<Vector>,
        stop_collection: &GradientStopCollection,
    ) -> windows::Result<Brush> {
        Brush::radial_gradient(
            self.backend.device_context(),
            center,
            offset,
            radius,
            stop_collection,
        )
    }

    #[inline]
    pub fn create_path(&self) -> PathBuilder {
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
    pub fn create_stroke_style(
        &self,
        props: &StrokeStyleProperties,
    ) -> windows::Result<StrokeStyle> {
        StrokeStyle::new(self.backend.d2d1_factory(), props)
    }

    #[inline]
    pub fn create_text_format(
        &self,
        font: &Font,
        size: impl Into<f32>,
        style: Option<&TextStyle>,
    ) -> windows::Result<TextFormat> {
        TextFormat::new(&self.dwrite_factory, font, size.into(), style)
    }

    #[inline]
    pub fn create_text_layout(
        &self,
        text: impl AsRef<str>,
        format: &TextFormat,
        alignment: TextAlignment,
        size: Option<Size>,
    ) -> windows::Result<TextLayout> {
        TextLayout::new(
            &self.dwrite_factory.clone().into(),
            text.as_ref(),
            format,
            alignment,
            size,
        )
    }

    #[inline]
    pub fn create_image(&self, loader: impl ImageLoader) -> windows::Result<Image> {
        Image::new(
            self.backend.device_context(),
            &self.wic_imaging_factory,
            loader,
        )
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