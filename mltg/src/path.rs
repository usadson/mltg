use crate::*;

#[derive(Clone, PartialEq, Eq)]
pub struct Path(pub(crate) ID2D1PathGeometry);

impl Fill for Path {
    #[inline]
    fn fill(&self, dc: &ID2D1DeviceContext, brush: &ID2D1Brush) {
        unsafe {
            dc.FillGeometry(&self.0, brush, None);
        }
    }
}

impl Stroke for Path {
    #[inline]
    fn stroke(
        &self,
        dc: &ID2D1DeviceContext,
        brush: &ID2D1Brush,
        width: f32,
        style: Option<ID2D1StrokeStyle>,
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

pub struct PathBuilder {
    geometry: ID2D1PathGeometry,
    sink: ID2D1GeometrySink,
}

pub struct Figure {
    geometry: ID2D1PathGeometry,
    sink: ID2D1GeometrySink,
}

impl PathBuilder {
    #[inline]
    pub(crate) fn new(geometry: ID2D1PathGeometry) -> Self {
        unsafe {
            let sink = {
                let mut p = None;
                geometry.Open(&mut p).and_some(p).unwrap()
            };
            Self { geometry, sink }
        }
    }

    #[inline]
    pub fn begin(self, point: impl Into<Point>) -> Figure {
        unsafe {
            let point: D2D_POINT_2F = Inner(point.into()).into();
            self.sink.BeginFigure(&point, D2D1_FIGURE_BEGIN_FILLED);
            Figure {
                geometry: self.geometry,
                sink: self.sink,
            }
        }
    }

    #[inline]
    pub fn build(self) -> Path {
        unsafe {
            self.sink.Close().unwrap();
            Path(self.geometry)
        }
    }
}

impl Figure {
    #[inline]
    pub fn line_to(self, point: impl Into<Point>) -> Self {
        unsafe {
            let point: D2D_POINT_2F = Inner(point.into()).into();
            self.sink.AddLine(point);
            self
        }
    }

    #[inline]
    pub fn quadratic_bezier_to(self, ctrl: impl Into<Point>, to: impl Into<Point>) -> Self {
        unsafe {
            let ctrl: D2D_POINT_2F = Inner(ctrl.into()).into();
            let to: D2D_POINT_2F = Inner(to.into()).into();
            self.sink
                .AddQuadraticBezier(&D2D1_QUADRATIC_BEZIER_SEGMENT {
                    point1: ctrl,
                    point2: to,
                });
            self
        }
    }

    #[inline]
    pub fn cubic_bezier_to(
        self,
        c0: impl Into<Point>,
        c1: impl Into<Point>,
        to: impl Into<Point>,
    ) -> Self {
        unsafe {
            let c0: D2D_POINT_2F = Inner(c0.into()).into();
            let c1: D2D_POINT_2F = Inner(c1.into()).into();
            let to: D2D_POINT_2F = Inner(to.into()).into();
            self.sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: c0,
                point2: c1,
                point3: to,
            });
            self
        }
    }

    #[inline]
    pub fn end(self, t: FigureEnd) -> PathBuilder {
        unsafe {
            self.sink.EndFigure(D2D1_FIGURE_END(t as u32));
            PathBuilder {
                geometry: self.geometry,
                sink: self.sink,
            }
        }
    }
}
