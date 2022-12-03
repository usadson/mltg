use crate::*;
use windows::Win32::Graphics::{Direct2D::Common::*, Direct2D::*};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FilledPath(ID2D1PathGeometry);

impl Fill for FilledPath {
    #[inline]
    fn fill(&self, dc: &ID2D1DeviceContext5, brush: &ID2D1Brush) {
        unsafe {
            dc.FillGeometry(&self.0, brush, None);
        }
    }
}

impl Stroke for FilledPath {
    #[inline]
    fn stroke(
        &self,
        dc: &ID2D1DeviceContext5,
        brush: &ID2D1Brush,
        width: f32,
        style: Option<&ID2D1StrokeStyle>,
    ) {
        unsafe {
            dc.DrawGeometry(&self.0, brush, width, style);
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct HollowPath(ID2D1PathGeometry);

impl Stroke for HollowPath {
    #[inline]
    fn stroke(
        &self,
        dc: &ID2D1DeviceContext5,
        brush: &ID2D1Brush,
        width: f32,
        style: Option<&ID2D1StrokeStyle>,
    ) {
        unsafe {
            dc.DrawGeometry(&self.0, brush, width, style);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum FigureEnd {
    Open = D2D1_FIGURE_END_OPEN.0,
    Closed = D2D1_FIGURE_END_CLOSED.0,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct QuadraticBezierSegment {
    pub ctrl: Point<f32>,
    pub to: Point<f32>,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct CubicBezierSegment {
    pub c0: Point<f32>,
    pub c1: Point<f32>,
    pub to: Point<f32>,
}

pub struct Figure<T> {
    geometry: ID2D1PathGeometry,
    sink: ID2D1GeometrySink,
    _t: std::marker::PhantomData<T>,
}

impl<T> Figure<T> {
    #[inline]
    pub fn line_to(self, point: impl Into<Point<f32>>) -> Self {
        unsafe {
            self.sink.AddLine(Wrapper(point.into()).into());
        }
        self
    }

    #[inline]
    pub fn lines(self, points: &[Point<f32>]) -> Self {
        unsafe {
            let lines =
                std::slice::from_raw_parts(points.as_ptr() as *const D2D_POINT_2F, points.len());
            self.sink.AddLines(lines);
        }
        self
    }

    #[inline]
    pub fn quadratic_bezier_to(
        self,
        ctrl: impl Into<Point<f32>>,
        to: impl Into<Point<f32>>,
    ) -> Self {
        unsafe {
            self.sink
                .AddQuadraticBezier(&D2D1_QUADRATIC_BEZIER_SEGMENT {
                    point1: Wrapper(ctrl.into()).into(),
                    point2: Wrapper(to.into()).into(),
                });
        }
        self
    }

    #[inline]
    pub fn quadratic_beziers(self, segments: &[QuadraticBezierSegment]) -> Self {
        unsafe {
            let segments = std::slice::from_raw_parts(
                segments.as_ptr() as *const D2D1_QUADRATIC_BEZIER_SEGMENT,
                segments.len(),
            );
            self.sink.AddQuadraticBeziers(segments);
        }
        self
    }

    #[inline]
    pub fn cubic_bezier_to(
        self,
        c0: impl Into<Point<f32>>,
        c1: impl Into<Point<f32>>,
        to: impl Into<Point<f32>>,
    ) -> Self {
        unsafe {
            self.sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: Wrapper(c0.into()).into(),
                point2: Wrapper(c1.into()).into(),
                point3: Wrapper(to.into()).into(),
            });
        }
        self
    }

    #[inline]
    pub fn cubic_beziers(self, segments: &[CubicBezierSegment]) -> Self {
        unsafe {
            let segments = std::slice::from_raw_parts(
                segments.as_ptr() as *const D2D1_BEZIER_SEGMENT,
                segments.len(),
            );
            self.sink.AddBeziers(segments);
        }
        self
    }

    #[inline]
    pub fn end(self, end: FigureEnd) -> Result<PathBuilder<T>> {
        unsafe {
            self.sink.EndFigure(D2D1_FIGURE_END(end as _));
        }
        Ok(PathBuilder {
            geometry: self.geometry,
            sink: self.sink,
            _t: std::marker::PhantomData,
        })
    }
}

pub struct PathBuilder<T> {
    geometry: ID2D1PathGeometry,
    sink: ID2D1GeometrySink,
    _t: std::marker::PhantomData<T>,
}

impl<T> PathBuilder<T> {
    pub(crate) fn new(geometry: ID2D1PathGeometry) -> Result<Self> {
        let sink = unsafe { geometry.Open()? };
        Ok(Self {
            geometry,
            sink,
            _t: std::marker::PhantomData,
        })
    }
}

impl PathBuilder<FilledPath> {
    #[inline]
    pub fn begin(self, start: impl Into<Point<f32>>) -> Figure<FilledPath> {
        unsafe {
            self.sink
                .BeginFigure(Wrapper(start.into()).into(), D2D1_FIGURE_BEGIN_FILLED);
        }
        Figure {
            geometry: self.geometry,
            sink: self.sink,
            _t: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn close(self) -> Result<FilledPath> {
        unsafe {
            self.sink.Close()?;
        }
        Ok(FilledPath(self.geometry))
    }
}

impl PathBuilder<HollowPath> {
    #[inline]
    pub fn begin(self, start: impl Into<Point<f32>>) -> Figure<HollowPath> {
        unsafe {
            self.sink
                .BeginFigure(Wrapper(start.into()).into(), D2D1_FIGURE_BEGIN_HOLLOW);
        }
        Figure {
            geometry: self.geometry,
            sink: self.sink,
            _t: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn close(self) -> Result<HollowPath> {
        unsafe {
            self.sink.Close()?;
        }
        Ok(HollowPath(self.geometry))
    }
}
