use crate::bindings::Windows::Foundation::Numerics::*;
use crate::bindings::Windows::Win32::Graphics::{DirectWrite::*, Dxgi::*, Imaging::*};
use crate::bindings::Windows::Win32::System::Com::*;
use crate::utility::*;
use crate::*;
use windows::{Abi, Interface};

pub struct DrawCommand<'a> {
    device_context: &'a ID2D1DeviceContext,
    dwrite_factory: &'a IDWriteFactory5,
}

impl<'a> DrawCommand<'a> {
    #[inline]
    pub fn clear(&self, color: impl Into<Rgba>) {
        unsafe {
            let color: D2D1_COLOR_F = Inner(color.into()).into();
            self.device_context.Clear(&color);
        }
    }

    #[inline]
    pub fn fill(&self, object: &impl Fill, brush: &Brush) {
        object.fill(self.device_context, &brush.handle());
    }

    #[inline]
    pub fn stroke(
        &self,
        object: &impl Stroke,
        brush: &Brush,
        width: f32,
        style: Option<&StrokeStyle>,
    ) {
        object.stroke(self.device_context, &brush.handle(), width, style.map(|s| s.0.clone()));
    }

    #[inline]
    pub fn draw_text(&self, text: &str, format: &TextFormat, brush: &Brush, origin: impl Into<Point>) {
        let layout = TextLayout::new(
            &self.dwrite_factory.cast().unwrap(),
            text,
            format,
            TextAlignment::Leading,
            None,
        );
        if let Ok(layout) = layout {
            self.draw_text_layout(&layout, brush, origin);
        }
    }

    #[inline]
    pub fn draw_text_layout(&self, layout: &TextLayout, brush: &Brush, origin: impl Into<Point>) {
        layout.draw(&self.device_context, brush, origin.into());
    }

    #[inline]
    pub fn draw_image(
        &self,
        image: &Image,
        dest_rect: impl Into<Rect>,
        src_rect: Option<Rect>,
        interpolation: Interpolation,
    ) {
        image.draw(&self.device_context, dest_rect.into(), src_rect, interpolation);
    }

    #[inline]
    pub fn clip(&self, rect: impl Into<Rect>, f: impl FnOnce(&DrawCommand)) {
        let rect: D2D_RECT_F = Inner(rect.into()).into();
        unsafe {
            self.device_context
                .PushAxisAlignedClip(&rect, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
            f(self);
            self.device_context.PopAxisAlignedClip();
        }
    }

    #[inline]
    pub fn set_offset(&self, point: impl Into<Point>) {
        let point = point.into();
        let m = Matrix3x2::translation(point.x, point.y);
        unsafe {
            self.device_context.SetTransform(&m);
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
    fn render_target(&self, target: *mut std::ffi::c_void) -> windows::Result<Self::RenderTarget>;
    fn begin_draw(&self, target: &Self::RenderTarget);
    fn end_draw(&self, target: &Self::RenderTarget);
}

#[derive(Clone)]
pub struct Factory {
    d2d1_factory: ID2D1Factory1,
    device_context: ID2D1DeviceContext,
    dwrite_factory: IDWriteFactory5,
    wic_imaging_factory: IWICImagingFactory,
}

impl Factory {
    #[inline]
    pub fn create_gradient_stop_collection<U>(
        &self,
        stops: &[U],
    ) -> windows::Result<GradientStopCollection>
    where
        U: Into<GradientStop> + Clone,
    {
        GradientStopCollection::new(&self.device_context, stops)
    }

    #[inline]
    pub fn create_solid_color_brush(&self, color: impl Into<Rgba>) -> windows::Result<Brush> {
        Brush::solid_color(&self.device_context, color)
    }

    #[inline]
    pub fn create_linear_gradient_brush(
        &self,
        start: impl Into<Point>,
        end: impl Into<Point>,
        stop_collection: &GradientStopCollection,
    ) -> windows::Result<Brush> {
        Brush::linear_gradient(&self.device_context, start, end, stop_collection)
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
            &self.device_context,
            center,
            offset,
            radius,
            stop_collection,
        )
    }

    #[inline]
    pub fn create_path(&self) -> PathBuilder {
        let geometry = unsafe { self.d2d1_factory.CreatePathGeometry().unwrap() };
        PathBuilder::new(geometry)
    }

    #[inline]
    pub fn create_stroke_style(
        &self,
        props: &StrokeStyleProperties,
    ) -> windows::Result<StrokeStyle> {
        StrokeStyle::new(&self.d2d1_factory, props)
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
        Image::new(&self.device_context, &self.wic_imaging_factory, loader)
    }
}

unsafe impl Send for Factory {}
unsafe impl Sync for Factory {}

#[derive(Clone)]
pub struct Context<T> {
    backend: T,
    dwrite_factory: IDWriteFactory5,
    wic_imaging_factory: IWICImagingFactory,
}

impl<T> Context<T>
where
    T: Backend + Clone,
{
    #[inline]
    pub fn new(backend: T) -> windows::Result<Self> {
        unsafe {
            let dwrite_factory =
                DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED, &IDWriteFactory5::IID)?.cast()?;
            let wic_imaging_factory =
                CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
            Ok(Self {
                backend,
                dwrite_factory,
                wic_imaging_factory,
            })
        }
    }

    #[inline]
    pub fn create_factory(&self) -> Factory {
        Factory {
            d2d1_factory: self.backend.d2d1_factory().clone(),
            device_context: self.backend.device_context().clone(),
            dwrite_factory: self.dwrite_factory.clone(),
            wic_imaging_factory: self.wic_imaging_factory.clone(),
        }
    }

    #[inline]
    pub fn backend(&self) -> &T {
        &self.backend
    }

    #[inline]
    pub fn create_back_buffers(
        &self,
        swap_chain: *mut std::ffi::c_void,
    ) -> windows::Result<Vec<T::RenderTarget>> {
        let swap_chain = unsafe { IDXGISwapChain1::from_abi(swap_chain)? };
        let ret = self.backend.back_buffers(&swap_chain);
        std::mem::forget(swap_chain);
        ret
    }

    #[inline]
    pub fn create_render_target(
        &self,
        target: *mut std::ffi::c_void,
    ) -> windows::Result<T::RenderTarget> {
        self.backend.render_target(target)
    }

    #[inline]
    pub fn set_dpi(&self, dpi: f32) {
        unsafe {
            self.backend.device_context().SetDpi(dpi, dpi);
        }
    }

    pub fn draw<R>(&self, target: &T::RenderTarget, f: impl FnOnce(&DrawCommand) -> R) -> R {
        let device_context = self.backend.device_context();
        unsafe {
            self.backend.begin_draw(target);
            device_context.SetTarget(target.bitmap());
            device_context.BeginDraw();
            let ret = f(&DrawCommand {
                device_context: self.backend.device_context(),
                dwrite_factory: &self.dwrite_factory,
            });
            device_context
                .EndDraw(std::ptr::null_mut(), std::ptr::null_mut())
                .unwrap();
            device_context.SetTarget(None);
            self.backend.end_draw(target);
            ret
        }
    }
}
