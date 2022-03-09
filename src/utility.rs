use crate::*;

pub type Point = gecl::Point<f32>;
pub type Size = gecl::Size<f32>;
pub type Rect = gecl::Rect<f32>;
pub type Vector = gecl::Vector<f32>;
pub type Circle = gecl::Circle<f32>;
pub type Rgba = gecl::Rgba<f32>;

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub(crate) struct Inner<T>(pub(crate) T);

impl From<Inner<Point>> for D2D_POINT_2F {
    #[inline]
    fn from(src: Inner<Point>) -> D2D_POINT_2F {
        D2D_POINT_2F {
            x: src.0.x,
            y: src.0.y,
        }
    }
}

impl From<Inner<Size>> for D2D_SIZE_F {
    #[inline]
    fn from(src: Inner<Size>) -> D2D_SIZE_F {
        D2D_SIZE_F {
            width: src.0.width,
            height: src.0.height,
        }
    }
}

impl From<Inner<Rect>> for D2D_RECT_F {
    #[inline]
    fn from(src: Inner<Rect>) -> D2D_RECT_F {
        let ep = src.0.endpoint();
        D2D_RECT_F {
            left: src.0.origin.x,
            top: src.0.origin.y,
            right: ep.x,
            bottom: ep.y,
        }
    }
}

impl From<Inner<Vector>> for D2D_POINT_2F {
    #[inline]
    fn from(src: Inner<Vector>) -> D2D_POINT_2F {
        D2D_POINT_2F {
            x: src.0.x,
            y: src.0.y,
        }
    }
}

impl From<Inner<Circle>> for D2D1_ELLIPSE {
    #[inline]
    fn from(src: Inner<Circle>) -> D2D1_ELLIPSE {
        D2D1_ELLIPSE {
            point: Inner(src.0.center).into(),
            radiusX: src.0.radius,
            radiusY: src.0.radius,
        }
    }
}

impl From<Inner<Rgba>> for D2D1_COLOR_F {
    #[inline]
    fn from(src: Inner<Rgba>) -> D2D1_COLOR_F {
        D2D1_COLOR_F {
            r: src.0.r,
            g: src.0.g,
            b: src.0.b,
            a: src.0.a,
        }
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ScreenRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl ScreenRect {
    #[inline]
    pub const fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }
}

impl From<[i32; 4]> for ScreenRect {
    #[inline]
    fn from(src: [i32; 4]) -> Self {
        Self::new(src[0], src[1], src[2], src[3])
    }
}

impl From<(i32, i32, i32, i32)> for ScreenRect {
    #[inline]
    fn from(src: (i32, i32, i32, i32)) -> Self {
        Self::new(src.0, src.1, src.2, src.3)
    }
}
