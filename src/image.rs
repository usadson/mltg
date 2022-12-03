use crate::*;
use std::path::Path;
use windows::core::{Interface, GUID, HSTRING};
use windows::Win32::{
    Graphics::Direct2D::*, Graphics::Imaging::D2D::*, Graphics::Imaging::*,
    System::SystemServices::*,
};

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
    pub(crate) fn from_file(
        dc: &ID2D1DeviceContext5,
        factory: &IWICImagingFactory2,
        path: impl AsRef<Path>,
    ) -> Result<Image> {
        unsafe {
            let decoder = factory.CreateDecoderFromFilename(
                &HSTRING::from(path.as_ref().to_string_lossy().as_ref()),
                &GUID::zeroed(),
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
            let bitmap = dc.CreateBitmapFromWicBitmap(&converter, None)?.cast()?;
            Ok(Image(bitmap))
        }
    }

    pub fn size(&self) -> Size<u32> {
        unsafe { Wrapper(self.0.GetPixelSize()).into() }
    }

    pub(crate) fn handle(&self) -> &ID2D1Bitmap1 {
        &self.0
    }
}
