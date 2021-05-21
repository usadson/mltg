use crate::*;
use bindings::Windows::Win32::{Graphics::Imaging::*, System::SystemServices::*};
use std::path::{Path, PathBuf};
use windows::Interface;

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
        factory: &IWICImagingFactory,
        loader: impl ImageLoader,
    ) -> windows::Result<Self> {
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
        let dest = dest_rect.into();
        let src = src_rect.map(|src| src.into());
        unsafe {
            dc.DrawBitmap2(
                &self.0,
                &dest,
                1.0,
                D2D1_INTERPOLATION_MODE(interpolation as _),
                if let Some(src) = src.as_ref() {
                    src as *const _
                } else {
                    std::ptr::null()
                },
                std::ptr::null(),
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

pub trait ImageLoader {
    fn load(&self, dc: &ID2D1DeviceContext, factory: &IWICImagingFactory)
        -> windows::Result<Image>;
}

impl<'a> ImageLoader for &'a Path {
    fn load(
        &self,
        dc: &ID2D1DeviceContext,
        factory: &IWICImagingFactory,
    ) -> windows::Result<Image> {
        unsafe {
            let decoder = {
                let mut p = None;
                factory
                    .CreateDecoderFromFilename(
                        self.to_str().unwrap(),
                        std::ptr::null(),
                        GENERIC_READ,
                        WICDecodeMetadataCacheOnDemand,
                        &mut p,
                    )
                    .and_some(p)?
            };
            let frame = {
                let mut p = None;
                decoder.GetFrame(0, &mut p).and_some(p)?
            };
            let converter = {
                let mut p = None;
                let converter = factory.CreateFormatConverter(&mut p).and_some(p)?;
                let mut guid = GUID_WICPixelFormat32bppPBGRA.clone();
                converter
                    .Initialize(
                        &frame,
                        &mut guid,
                        WICBitmapDitherTypeNone,
                        None,
                        1.0,
                        WICBitmapPaletteTypeMedianCut,
                    )
                    .ok()?;
                converter
            };
            let bitmap = {
                let mut p = None;
                dc.CreateBitmapFromWicBitmap(&converter, std::ptr::null(), &mut p)
                    .and_some(p)?
                    .cast()?
            };
            Ok(Image(bitmap))
        }
    }
}

impl ImageLoader for PathBuf {
    fn load(
        &self,
        dc: &ID2D1DeviceContext,
        factory: &IWICImagingFactory,
    ) -> windows::Result<Image> {
        self.as_path().load(dc, factory)
    }
}

impl<'a> ImageLoader for &'a str {
    fn load(
        &self,
        dc: &ID2D1DeviceContext,
        factory: &IWICImagingFactory,
    ) -> windows::Result<Image> {
        Path::new(self).load(dc, factory)
    }
}

impl<'a> ImageLoader for &'a String {
    fn load(
        &self,
        dc: &ID2D1DeviceContext,
        factory: &IWICImagingFactory,
    ) -> windows::Result<Image> {
        Path::new(self).load(dc, factory)
    }
}
