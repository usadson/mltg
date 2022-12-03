use crate::*;
use windows::Win32::Graphics::{Direct2D::Common::*, Direct2D::*};

pub type Point<T> = gecl::Point<T>;
pub type Size<T> = gecl::Size<T>;
pub type Rect<T> = gecl::Rect<T>;
pub type Vector<T> = gecl::Vector<T>;
pub type Circle = gecl::Circle<f32>;
pub type Ellipse = gecl::Ellipse<f32>;

impl From<Wrapper<Point<f32>>> for D2D_POINT_2F {
    #[inline]
    fn from(src: Wrapper<Point<f32>>) -> Self {
        Self {
            x: src.0.x,
            y: src.0.y,
        }
    }
}

impl From<Wrapper<D2D_POINT_2F>> for Point<f32> {
    #[inline]
    fn from(src: Wrapper<D2D_POINT_2F>) -> Self {
        Self {
            x: src.0.x,
            y: src.0.y,
        }
    }
}

impl From<Wrapper<Size<f32>>> for D2D_SIZE_F {
    #[inline]
    fn from(src: Wrapper<Size<f32>>) -> Self {
        Self {
            width: src.0.width,
            height: src.0.height,
        }
    }
}

impl From<Wrapper<D2D_SIZE_F>> for Size<f32> {
    #[inline]
    fn from(src: Wrapper<D2D_SIZE_F>) -> Self {
        Self {
            width: src.0.width,
            height: src.0.height,
        }
    }
}

impl From<Wrapper<Size<u32>>> for D2D_SIZE_U {
    #[inline]
    fn from(src: Wrapper<Size<u32>>) -> Self {
        Self {
            width: src.0.width,
            height: src.0.height,
        }
    }
}

impl From<Wrapper<D2D_SIZE_U>> for Size<u32> {
    #[inline]
    fn from(src: Wrapper<D2D_SIZE_U>) -> Self {
        Self {
            width: src.0.width,
            height: src.0.height,
        }
    }
}

impl From<Wrapper<Vector<f32>>> for D2D_POINT_2F {
    #[inline]
    fn from(src: Wrapper<Vector<f32>>) -> Self {
        Self {
            x: src.0.x,
            y: src.0.y,
        }
    }
}

impl From<Wrapper<Rect<f32>>> for D2D_RECT_F {
    #[inline]
    fn from(src: Wrapper<Rect<f32>>) -> Self {
        let ep = src.0.endpoint();
        Self {
            left: src.0.origin.x,
            top: src.0.origin.y,
            right: ep.x,
            bottom: ep.y,
        }
    }
}

impl From<Wrapper<Circle>> for D2D1_ELLIPSE {
    #[inline]
    fn from(src: Wrapper<Circle>) -> Self {
        Self {
            point: Wrapper(src.0.center).into(),
            radiusX: src.0.radius,
            radiusY: src.0.radius,
        }
    }
}

impl From<Wrapper<Ellipse>> for D2D1_ELLIPSE {
    #[inline]
    fn from(src: Wrapper<Ellipse>) -> Self {
        Self {
            point: Wrapper(src.0.center).into(),
            radiusX: src.0.radius.x,
            radiusY: src.0.radius.y,
        }
    }
}

impl Fill for Rect<f32> {
    #[inline]
    fn fill(&self, dc: &ID2D1DeviceContext5, brush: &ID2D1Brush) {
        unsafe {
            dc.FillRectangle(&Wrapper(*self).into(), brush);
        }
    }
}

impl Stroke for Rect<f32> {
    #[inline]
    fn stroke(
        &self,
        dc: &ID2D1DeviceContext5,
        brush: &ID2D1Brush,
        width: f32,
        style: Option<&ID2D1StrokeStyle>,
    ) {
        unsafe {
            dc.DrawRectangle(&Wrapper(*self).into(), brush, width, style);
        }
    }
}

impl Fill for Circle {
    #[inline]
    fn fill(&self, dc: &ID2D1DeviceContext5, brush: &ID2D1Brush) {
        unsafe {
            dc.FillEllipse(&Wrapper(Ellipse::from(*self)).into(), brush);
        }
    }
}

impl Stroke for Circle {
    #[inline]
    fn stroke(
        &self,
        dc: &ID2D1DeviceContext5,
        brush: &ID2D1Brush,
        width: f32,
        style: Option<&ID2D1StrokeStyle>,
    ) {
        unsafe {
            dc.DrawEllipse(&Wrapper(Ellipse::from(*self)).into(), brush, width, style);
        }
    }
}

impl Fill for Ellipse {
    #[inline]
    fn fill(&self, dc: &ID2D1DeviceContext5, brush: &ID2D1Brush) {
        unsafe {
            dc.FillEllipse(&Wrapper(*self).into(), brush);
        }
    }
}

impl Stroke for Ellipse {
    #[inline]
    fn stroke(
        &self,
        dc: &ID2D1DeviceContext5,
        brush: &ID2D1Brush,
        width: f32,
        style: Option<&ID2D1StrokeStyle>,
    ) {
        unsafe {
            dc.DrawEllipse(&Wrapper(*self).into(), brush, width, style);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Line(pub Point<f32>, pub Point<f32>);

impl Line {
    #[inline]
    pub fn new(x0: impl Into<Point<f32>>, x1: impl Into<Point<f32>>) -> Self {
        Self(x0.into(), x1.into())
    }
}

#[inline]
pub fn line(x0: impl Into<Point<f32>>, x1: impl Into<Point<f32>>) -> Line {
    Line::new(x0, x1)
}

impl Stroke for Line {
    #[inline]
    fn stroke(
        &self,
        dc: &ID2D1DeviceContext5,
        brush: &ID2D1Brush,
        width: f32,
        style: Option<&ID2D1StrokeStyle>,
    ) {
        unsafe {
            dc.DrawLine(
                Wrapper(self.0).into(),
                Wrapper(self.1).into(),
                brush,
                width,
                style,
            );
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RoundedRect {
    pub rect: Rect<f32>,
    pub radius: Vector<f32>,
}

impl RoundedRect {
    #[inline]
    pub fn new(rect: impl Into<Rect<f32>>, radius: impl Into<Vector<f32>>) -> Self {
        Self {
            rect: rect.into(),
            radius: radius.into(),
        }
    }
}

impl From<RoundedRect> for D2D1_ROUNDED_RECT {
    #[inline]
    fn from(src: RoundedRect) -> Self {
        Self {
            rect: Wrapper(src.rect).into(),
            radiusX: src.radius.x,
            radiusY: src.radius.y,
        }
    }
}

#[inline]
pub fn rounded_rect(rect: impl Into<Rect<f32>>, radius: impl Into<Vector<f32>>) -> RoundedRect {
    RoundedRect::new(rect, radius)
}

impl Fill for RoundedRect {
    #[inline]
    fn fill(&self, dc: &ID2D1DeviceContext5, brush: &ID2D1Brush) {
        unsafe {
            dc.FillRoundedRectangle(&(*self).into(), brush);
        }
    }
}

impl Stroke for RoundedRect {
    #[inline]
    fn stroke(
        &self,
        dc: &ID2D1DeviceContext5,
        brush: &ID2D1Brush,
        width: f32,
        style: Option<&ID2D1StrokeStyle>,
    ) {
        unsafe {
            dc.DrawRoundedRectangle(&(*self).into(), brush, width, style);
        }
    }
}
