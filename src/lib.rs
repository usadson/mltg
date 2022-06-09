mod brush;
mod context;
pub mod d2d;
pub mod d3d11;
pub mod d3d12;
pub mod error;
mod image;
mod path;
mod shape;
mod stroke_style;
mod text;
mod utility;

pub use brush::*;
pub use context::*;
pub use d2d::Direct2D;
pub use d3d11::Direct3D11;
pub use d3d12::Direct3D12;
pub use error::{Error, ErrorKind};
pub use gecl;
pub use gecl::{circle, point, rect, rgba, size, vector};
pub use image::*;
pub use path::*;
pub use shape::*;
pub use stroke_style::*;
pub use text::*;
pub use utility::*;
use windows::Win32::Graphics::{Direct2D::Common::*, Direct2D::*, Direct3D::*};

pub type Result<T> = core::result::Result<T, Error>;
pub type RenderTarget<T> = <T as Backend>::RenderTarget;

pub trait Target {
    fn bitmap(&self) -> &ID2D1Bitmap1;
    fn size(&self) -> Size;
    fn physical_size(&self) -> gecl::Size<u32>;
}

pub trait Fill {
    fn fill(&self, dc: &ID2D1DeviceContext, brush: &ID2D1Brush);
}

pub trait Stroke {
    fn stroke(
        &self,
        dc: &ID2D1DeviceContext,
        brush: &ID2D1Brush,
        width: f32,
        style: Option<ID2D1StrokeStyle>,
    );
}
