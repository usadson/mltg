use crate::*;

pub type Point = gecl::Point<f32>;
pub type Size = gecl::Size<f32>;
pub type Rect = gecl::Rect<f32>;
pub type Vector = gecl::Vector<f32>;
pub type Circle = gecl::Circle<f32>;
pub type Rgba = gecl::Rgba<f32>;

impl From<Point> for D2D_POINT_2F {
    #[inline]
    fn from(src: Point) -> D2D_POINT_2F {
        D2D_POINT_2F { x: src.x, y: src.y }
    }
}

impl From<Size> for D2D_SIZE_F {
    #[inline]
    fn from(src: Size) -> D2D_SIZE_F {
        D2D_SIZE_F {
            width: src.width,
            height: src.height,
        }
    }
}

impl From<Rect> for D2D_RECT_F {
    #[inline]
    fn from(src: Rect) -> D2D_RECT_F {
        let ep = src.endpoint();
        D2D_RECT_F {
            left: src.origin.x,
            top: src.origin.y,
            right: ep.x,
            bottom: ep.y,
        }
    }
}

impl From<Vector> for D2D_POINT_2F {
    #[inline]
    fn from(src: Vector) -> D2D_POINT_2F {
        D2D_POINT_2F { x: src.x, y: src.y }
    }
}

impl From<Circle> for D2D1_ELLIPSE {
    #[inline]
    fn from(src: Circle) -> D2D1_ELLIPSE {
        D2D1_ELLIPSE {
            point: src.center.into(),
            radiusX: src.radius,
            radiusY: src.radius,
        }
    }
}

impl From<Rgba> for D2D1_COLOR_F {
    #[inline]
    fn from(src: Rgba) -> D2D1_COLOR_F {
        D2D1_COLOR_F {
            r: src.r,
            g: src.g,
            b: src.b,
            a: src.a,
        }
    }
}
