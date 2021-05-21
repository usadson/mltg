use crate::*;

#[derive(Clone, Copy, Debug)]
pub struct Line(Point, Point);

impl Line {
    #[inline]
    pub fn new(x0: impl Into<Point>, x1: impl Into<Point>) -> Self {
        Self(x0.into(), x1.into())
    }
}

#[inline]
pub fn line(x0: impl Into<Point>, x1: impl Into<Point>) -> Line {
    Line::new(x0, x1)
}

impl Stroke for Line {
    #[inline]
    fn stroke(
        &self,
        dc: &ID2D1DeviceContext,
        brush: &ID2D1Brush,
        width: f32,
        style: Option<ID2D1StrokeStyle>,
    ) {
        let x0: D2D_POINT_2F = self.0.into();
        let x1: D2D_POINT_2F = self.1.into();
        unsafe {
            dc.DrawLine(x0, x1, brush, width, style);
        }
    }
}

impl Fill for Rect {
    #[inline]
    fn fill(&self, dc: &ID2D1DeviceContext, brush: &ID2D1Brush) {
        unsafe {
            dc.FillRectangle(&D2D_RECT_F::from(*self), brush);
        }
    }
}

impl Stroke for Rect {
    #[inline]
    fn stroke(
        &self,
        dc: &ID2D1DeviceContext,
        brush: &ID2D1Brush,
        width: f32,
        style: Option<ID2D1StrokeStyle>,
    ) {
        unsafe {
            dc.DrawRectangle(&D2D_RECT_F::from(*self), brush, width, style);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ellipse {
    pub center: Point,
    pub radius: Vector,
}

impl From<Ellipse> for D2D1_ELLIPSE {
    #[inline]
    fn from(src: Ellipse) -> D2D1_ELLIPSE {
        D2D1_ELLIPSE {
            point: src.center.into(),
            radiusX: src.radius.x,
            radiusY: src.radius.y,
        }
    }
}

impl Fill for Ellipse {
    #[inline]
    fn fill(&self, dc: &ID2D1DeviceContext, brush: &ID2D1Brush) {
        unsafe {
            dc.FillEllipse(&D2D1_ELLIPSE::from(*self), brush);
        }
    }
}

impl Stroke for Ellipse {
    #[inline]
    fn stroke(
        &self,
        dc: &ID2D1DeviceContext,
        brush: &ID2D1Brush,
        width: f32,
        style: Option<ID2D1StrokeStyle>,
    ) {
        unsafe {
            dc.DrawEllipse(&D2D1_ELLIPSE::from(*self), brush, width, style);
        }
    }
}

impl Fill for Circle {
    #[inline]
    fn fill(&self, dc: &ID2D1DeviceContext, brush: &ID2D1Brush) {
        unsafe {
            dc.FillEllipse(&D2D1_ELLIPSE::from(*self), brush);
        }
    }
}

impl Stroke for Circle {
    #[inline]
    fn stroke(
        &self,
        dc: &ID2D1DeviceContext,
        brush: &ID2D1Brush,
        width: f32,
        style: Option<ID2D1StrokeStyle>,
    ) {
        unsafe {
            dc.DrawEllipse(&D2D1_ELLIPSE::from(*self), brush, width, style);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RoundedRect {
    pub rect: Rect,
    pub radius: Vector,
}

impl RoundedRect {
    #[inline]
    pub fn new(rect: impl Into<Rect>, radius: impl Into<Vector>) -> Self {
        Self {
            rect: rect.into(),
            radius: radius.into(),
        }
    }
}

impl From<RoundedRect> for D2D1_ROUNDED_RECT {
    #[inline]
    fn from(src: RoundedRect) -> D2D1_ROUNDED_RECT {
        D2D1_ROUNDED_RECT {
            rect: src.rect.into(),
            radiusX: src.radius.x,
            radiusY: src.radius.y,
        }
    }
}

impl Fill for RoundedRect {
    #[inline]
    fn fill(&self, dc: &ID2D1DeviceContext, brush: &ID2D1Brush) {
        unsafe {
            dc.FillRoundedRectangle(&D2D1_ROUNDED_RECT::from(*self), brush);
        }
    }
}

impl Stroke for RoundedRect {
    #[inline]
    fn stroke(
        &self,
        dc: &ID2D1DeviceContext,
        brush: &ID2D1Brush,
        width: f32,
        style: Option<ID2D1StrokeStyle>,
    ) {
        unsafe {
            dc.DrawRoundedRectangle(&D2D1_ROUNDED_RECT::from(*self), brush, width, style);
        }
    }
}
