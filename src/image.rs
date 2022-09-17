use crate::*;
use std::path::{Path, PathBuf};
use windows::core::{Interface, HSTRING};
use windows::Win32::{Graphics::Imaging::D2D::*, Graphics::Imaging::*, System::SystemServices::*};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum Interpolation {
    NearestNeighbor = D2D1_INTERPOLATION_MODE_NEAREST_NEIGHBOR.0,
    Linear = D2D1_INTERPOLATION_MODE_LINEAR.0,
    Cubic = D2D1_INTERPOLATION_MODE_CUBIC.0,
    MultiSampleLinear = D2D1_INTERPOLATION_MODE_MULTI_SAMPLE_LINEAR.0,
    Anisotropic = D2D1_INTERPOLATION_MODE_ANISOTROPIC.0,
    HighQualityCubic = D2D1_INTERPOLATION_MODE_HIGH_QUALITY_CUBIC.0,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Image(ID2D1Bitmap1);

impl Image {
    #[inline]
    pub(crate) fn new(
        dc: &ID2D1DeviceContext,
        factory: &IWICImagingFactory2,
        loader: impl ImageLoader,
    ) -> Result<Self> {
        loader.load(dc, factory)
    }

    #[inline]
    pub(crate) fn draw(
        &self,
        dc: &ID2D1DeviceContext,
        dest_rect: Rect,
        src_rect: Option<Rect>,
        interpolation: Interpolation,
    ) {
        let dest = Inner(dest_rect).into();
        let src: Option<D2D_RECT_F> = src_rect.map(|src| Inner(src).into());
        unsafe {
            dc.DrawBitmap2(
                &self.0,
                Some(&dest),
                1.0,
                D2D1_INTERPOLATION_MODE(interpolation as _),
                src.as_ref(),
                None,
            );
        }
    }

    #[inline]
    pub fn size(&self) -> gecl::Size<u32> {
        unsafe {
            let size = self.0.GetPixelSize();
            gecl::Size::new(size.width, size.height)
        }
    }
}

unsafe impl Send for Image {}
unsafe impl Sync for Image {}

pub trait ImageLoader {
    fn load(&self, dc: &ID2D1DeviceContext, factory: &IWICImagingFactory2) -> Result<Image>;
}

impl<'a> ImageLoader for &'a Path {
    fn load(&self, dc: &ID2D1DeviceContext, factory: &IWICImagingFactory2) -> Result<Image> {
        unsafe {
            let decoder = factory.CreateDecoderFromFilename(
                &HSTRING::from(self.to_str().unwrap()),
                &windows::core::GUID::zeroed(),
                GENERIC_READ,
                WICDecodeMetadataCacheOnDemand,
            )?;
            let frame = decoder.GetFrame(0)?;
            let converter = {
                let converter = factory.CreateFormatConverter()?;
                let guid = GUID_WICPixelFormat32bppPBGRA;
                converter.Initialize(
                    &frame,
                    &guid,
                    WICBitmapDitherTypeNone,
                    None,
                    1.0,
                    WICBitmapPaletteTypeMedianCut,
                )?;
                converter
            };
            let bitmap = { dc.CreateBitmapFromWicBitmap(&converter, None)?.cast()? };
            Ok(Image(bitmap))
        }
    }
}

impl ImageLoader for PathBuf {
    fn load(&self, dc: &ID2D1DeviceContext, factory: &IWICImagingFactory2) -> Result<Image> {
        self.as_path().load(dc, factory)
    }
}

impl<'a> ImageLoader for &'a str {
    fn load(&self, dc: &ID2D1DeviceContext, factory: &IWICImagingFactory2) -> Result<Image> {
        Path::new(self).load(dc, factory)
    }
}

impl<'a> ImageLoader for &'a String {
    fn load(&self, dc: &ID2D1DeviceContext, factory: &IWICImagingFactory2) -> Result<Image> {
        Path::new(self).load(dc, factory)
    }
}
