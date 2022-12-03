use crate::utility::*;
use crate::*;
use windows::Win32::Graphics::Direct2D::*;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SolidColorBrush(ID2D1SolidColorBrush);

#[derive(Clone, Copy, Debug)]
pub struct GradientStop {
    pub position: f32,
    pub color: Rgba<f32>,
}

impl GradientStop {
    #[inline]
    pub fn new(position: f32, color: impl Into<Rgba<f32>>) -> Self {
        Self {
            position,
            color: color.into(),
        }
    }
}

impl From<GradientStop> for D2D1_GRADIENT_STOP {
    #[inline]
    fn from(src: GradientStop) -> Self {
        Self {
            position: src.position,
            color: Wrapper(src.color).into(),
        }
    }
}

impl<T> From<(f32, T)> for GradientStop
where
    T: Into<Rgba<f32>>,
{
    #[inline]
    fn from(src: (f32, T)) -> Self {
        Self {
            position: src.0,
            color: src.1.into(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum GradientMode {
    Clamp = D2D1_EXTEND_MODE_CLAMP.0,
    Mirror = D2D1_EXTEND_MODE_MIRROR.0,
    Wrap = D2D1_EXTEND_MODE_WRAP.0,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GradientStopCollection(ID2D1GradientStopCollection);

impl GradientStopCollection {
    pub(crate) fn new<T>(dc: &ID2D1DeviceContext5, mode: GradientMode, stops: &[T]) -> Result<Self>
    where
        T: Into<GradientStop> + Clone,
    {
        let stops = stops
            .iter()
            .cloned()
            .map(|stop| stop.into().into())
            .collect::<Vec<_>>();
        let collection = unsafe {
            dc.CreateGradientStopCollection(&stops, D2D1_GAMMA_2_2, D2D1_EXTEND_MODE(mode as u32))?
        };
        Ok(Self(collection))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LinearGradientBrush(ID2D1LinearGradientBrush);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RadialGradientBrush(ID2D1RadialGradientBrush);

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Brush {
    SolidColor(SolidColorBrush),
    LinearGradient(LinearGradientBrush),
    RadialGradient(RadialGradientBrush),
}

impl Brush {
    pub(crate) fn solid_color(
        dc: &ID2D1DeviceContext5,
        color: impl Into<Rgba<f32>>,
    ) -> Result<Self> {
        let color = Wrapper(color.into()).into();
        let brush = unsafe { dc.CreateSolidColorBrush(&color, None)? };
        Ok(Self::SolidColor(SolidColorBrush(brush)))
    }

    pub(crate) fn linear_gradient(
        dc: &ID2D1DeviceContext5,
        start: impl Into<Point<f32>>,
        end: impl Into<Point<f32>>,
        stops: &GradientStopCollection,
    ) -> Result<Self> {
        let start = start.into();
        let end = end.into();
        let brush = unsafe {
            dc.CreateLinearGradientBrush(
                &D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                    startPoint: Wrapper(start).into(),
                    endPoint: Wrapper(end).into(),
                },
                None,
                &stops.0,
            )?
        };
        Ok(Self::LinearGradient(LinearGradientBrush(brush)))
    }

    pub(crate) fn radial_gradient(
        dc: &ID2D1DeviceContext5,
        ellipse: impl Into<Ellipse>,
        offset: impl Into<Point<f32>>,
        stops: &GradientStopCollection,
    ) -> Result<Self> {
        let ellipse = ellipse.into();
        let offset = offset.into();
        let brush = unsafe {
            dc.CreateRadialGradientBrush(
                &D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES {
                    center: Wrapper(ellipse.center).into(),
                    radiusX: ellipse.radius.x,
                    radiusY: ellipse.radius.y,
                    gradientOriginOffset: Wrapper(offset).into(),
                },
                None,
                &stops.0,
            )?
        };
        Ok(Self::RadialGradient(RadialGradientBrush(brush)))
    }

    #[inline]
    pub(crate) fn handle(&self) -> ID2D1Brush {
        match self {
            Self::SolidColor(b) => b.0.clone().into(),
            Self::LinearGradient(b) => b.0.clone().into(),
            Self::RadialGradient(b) => b.0.clone().into(),
        }
    }
}
