use crate::*;
use std::sync::Arc;
use windows::core::Interface;
use windows::Win32::Graphics::{
    Direct2D::Common::*, Direct2D::*, DirectWrite::*, Imaging::D2D::*, Imaging::*,
};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};

pub trait Fill {
    fn fill(&self, dc: &ID2D1DeviceContext5, brush: &ID2D1Brush);
}

pub trait Stroke {
    fn stroke(
        &self,
        dc: &ID2D1DeviceContext5,
        brush: &ID2D1Brush,
        width: f32,
        style: Option<&ID2D1StrokeStyle>,
    );
}

pub struct DrawCommand {
    dc: ID2D1DeviceContext5,
}

impl DrawCommand {
    #[inline]
    pub fn clear(&self, color: impl Into<Rgba<f32>>) {
        unsafe {
            let color = Wrapper(color.into()).into();
            self.dc.Clear(Some(&color));
        }
    }

    #[inline]
    pub fn fill(&self, object: &impl Fill, brush: &Brush) {
        object.fill(&self.dc, &brush.handle());
    }

    #[inline]
    pub fn stroke(
        &self,
        object: &impl Stroke,
        brush: &Brush,
        width: f32,
        stroke_style: Option<&StrokeStyle>,
    ) {
        object.stroke(
            &self.dc,
            &brush.handle(),
            width,
            stroke_style.map(|ss| ss.handle()),
        );
    }

    #[inline]
    pub fn draw_image(
        &self,
        image: &Image,
        dest_rect: impl Into<Rect<f32>>,
        src_rect: Option<Rect<f32>>,
        interpolation: Interpolation,
    ) {
        let dest: D2D_RECT_F = Wrapper(dest_rect.into()).into();
        let src: Option<D2D_RECT_F> = src_rect.map(|src| Wrapper(src).into());
        unsafe {
            self.dc.DrawBitmap2(
                image.handle(),
                Some(&dest),
                1.0,
                D2D1_INTERPOLATION_MODE(interpolation as _),
                src.as_ref().map(|src| src as _),
                None,
            );
        }
    }

    #[inline]
    pub fn push_clip(&self, rect: impl Into<Rect<f32>>) {
        unsafe {
            self.dc.PushAxisAlignedClip(
                &Wrapper(rect.into()).into(),
                D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
            );
        }
    }

    #[inline]
    pub fn pop_clip(&self) {
        unsafe {
            self.dc.PopAxisAlignedClip();
        }
    }

    #[inline]
    pub fn clip<F, R>(&self, rect: impl Into<Rect<f32>>, f: F) -> R
    where
        F: FnOnce(&Self) -> R,
    {
        self.push_clip(rect);
        let ret = f(self);
        self.pop_clip();
        ret
    }
}

pub trait Target {
    fn bitmap(&self) -> &ID2D1Bitmap1;
    fn size(&self) -> Size<f32>;
    fn physical_size(&self) -> Size<u32>;
}

pub trait Backend {
    type RenderTarget: Target;

    fn d2d1_factory(&self) -> &ID2D1Factory6;
    fn d2d1_device(&self) -> &ID2D1Device5;
    fn begin_draw(&self, target: &Self::RenderTarget);
    fn end_draw(&self, target: &Self::RenderTarget, ret: Result<()>) -> Result<()>;
}

#[derive(Clone, Debug)]
struct FontFileLoader {
    factory: IDWriteFactory6,
    loader: IDWriteInMemoryFontFileLoader,
}

impl Drop for FontFileLoader {
    fn drop(&mut self) {
        unsafe {
            self.factory
                .UnregisterFontFileLoader(&self.loader)
                .unwrap_or(());
        }
    }
}

pub struct LockGuard<'a> {
    multithread: &'a ID2D1Multithread,
}

impl<'a> Drop for LockGuard<'a> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.multithread.Leave();
        }
    }
}

#[derive(Clone, Debug)]
pub struct Factory {
    d2d1_factory: ID2D1Factory6,
    d2d1_device_context: ID2D1DeviceContext5,
    dwrite_factory: IDWriteFactory6,
    wic_imaging_factory: IWICImagingFactory2,
    font_loader: Arc<FontFileLoader>,
    multithread: ID2D1Multithread,
}

impl Factory {
    #[inline]
    pub fn create_solid_color_brush(&self, color: impl Into<Rgba<f32>>) -> Result<Brush> {
        Brush::solid_color(&self.d2d1_device_context, color)
    }

    #[inline]
    pub fn create_gradient_stop_collection<T>(
        &self,
        mode: GradientMode,
        stops: &[T],
    ) -> Result<GradientStopCollection>
    where
        T: Into<GradientStop> + Clone,
    {
        GradientStopCollection::new(&self.d2d1_device_context, mode, stops)
    }

    #[inline]
    pub fn create_linear_gradient_brush(
        &self,
        start: impl Into<Point<f32>>,
        end: impl Into<Point<f32>>,
        stops: &GradientStopCollection,
    ) -> Result<Brush> {
        Brush::linear_gradient(&self.d2d1_device_context, start, end, stops)
    }

    #[inline]
    pub fn create_radial_gradient_brush(
        &self,
        ellipse: impl Into<Ellipse>,
        offset: impl Into<Point<f32>>,
        stops: &GradientStopCollection,
    ) -> Result<Brush> {
        Brush::radial_gradient(&self.d2d1_device_context, ellipse, offset, stops)
    }

    #[inline]
    pub fn create_text_format(
        &self,
        font: Font,
        size: impl Into<f32>,
        style: Option<&TextStyle>,
        locale: Option<&str>,
    ) -> Result<TextFormat> {
        TextFormat::new(
            &self.dwrite_factory,
            &self.font_loader.loader,
            font,
            size.into(),
            style,
            locale.unwrap_or(""),
        )
    }

    #[inline]
    pub fn create_text_layout(
        &self,
        text: impl AsRef<str>,
        format: &TextFormat,
        alignment: TextAlignment,
        size: Option<Size<f32>>,
    ) -> Result<TextLayout> {
        TextLayout::new(&self.dwrite_factory, text.as_ref(), format, alignment, size)
    }

    #[inline]
    pub fn create_stroke_style(&self, props: &StrokeStyleProperties) -> Result<StrokeStyle> {
        StrokeStyle::new(&self.d2d1_factory, props)
    }

    #[inline]
    pub fn create_image_from_file(&self, path: impl AsRef<std::path::Path>) -> Result<Image> {
        Image::from_file(&self.d2d1_device_context, &self.wic_imaging_factory, path)
    }

    #[inline]
    pub fn create_filled_path(&self) -> Result<PathBuilder<FilledPath>> {
        let geometry = unsafe { self.d2d1_factory.CreatePathGeometry()? };
        PathBuilder::new(geometry)
    }

    #[inline]
    pub fn create_hollow_path(&self) -> Result<PathBuilder<HollowPath>> {
        let geometry = unsafe { self.d2d1_factory.CreatePathGeometry()? };
        PathBuilder::new(geometry)
    }

    #[inline]
    pub fn lock(&self) -> LockGuard {
        unsafe {
            self.multithread.Enter();
        }
        LockGuard {
            multithread: &self.multithread,
        }
    }
}

#[derive(Debug)]
pub struct Context<T> {
    pub(crate) backend: T,
    pub(crate) d2d1_device_context: ID2D1DeviceContext5,
    dwrite_factory: IDWriteFactory6,
    wic_imaging_factory: IWICImagingFactory2,
    font_loader: Arc<FontFileLoader>,
}

impl<T> Context<T>
where
    T: Backend,
{
    pub fn new(backend: T) -> Result<Self> {
        unsafe {
            let d2d1_device_context = backend
                .d2d1_device()
                .CreateDeviceContext6(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;
            let dwrite_factory: IDWriteFactory6 = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
            let wic_imaging_factory =
                CoCreateInstance(&CLSID_WICImagingFactory2, None, CLSCTX_INPROC_SERVER)?;
            let font_loader = {
                let loader = dwrite_factory.CreateInMemoryFontFileLoader()?;
                dwrite_factory.RegisterFontFileLoader(&loader)?;
                Arc::new(FontFileLoader {
                    factory: dwrite_factory.clone(),
                    loader,
                })
            };
            Ok(Self {
                backend,
                d2d1_device_context,
                dwrite_factory,
                wic_imaging_factory,
                font_loader,
            })
        }
    }

    #[inline]
    pub fn create_factory(&self) -> Factory {
        Factory {
            d2d1_factory: self.backend.d2d1_factory().clone(),
            d2d1_device_context: self.d2d1_device_context.clone(),
            dwrite_factory: self.dwrite_factory.clone(),
            wic_imaging_factory: self.wic_imaging_factory.clone(),
            font_loader: self.font_loader.clone(),
            multithread: self.backend.d2d1_factory().cast().unwrap(),
        }
    }

    #[inline]
    pub fn set_dpi(&self, dpi: f32) {
        unsafe {
            self.d2d1_device_context.SetDpi(dpi, dpi);
        }
    }

    #[inline]
    pub fn set_scale_factor(&self, scale: f32) {
        self.set_dpi(scale * 96.0)
    }

    #[inline]
    pub fn draw<R>(
        &self,
        target: &T::RenderTarget,
        f: impl FnOnce(&DrawCommand) -> R,
    ) -> Result<R> {
        let ctx = &self.d2d1_device_context;
        unsafe {
            self.backend.begin_draw(target);
            ctx.SetTarget(target.bitmap());
            ctx.BeginDraw();
            let ret = f(&DrawCommand { dc: ctx.clone() });
            let e = ctx.EndDraw(None, None);
            ctx.SetTarget(None);
            self.backend.end_draw(target, e.map_err(|e| e.into()))?;
            Ok(ret)
        }
    }
}
