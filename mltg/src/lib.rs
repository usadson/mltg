mod brush;
mod context;
pub mod d2d;
pub mod d3d11;
pub mod d3d12;
mod image;
mod path;
mod shape;
mod stroke_style;
mod text;
mod utility;

use mltg_bindings as bindings;

use bindings::Windows::Win32::Graphics::Direct2D::*;
pub use brush::*;
pub use context::*;
pub use d2d::Direct2D;
pub use d3d11::Direct3D11;
pub use d3d12::Direct3D12;
pub use gecl;
pub use gecl::{circle, point, rect, rgba, size, vector};
pub use image::*;
pub use path::*;
pub use shape::*;
pub use stroke_style::*;
pub use text::*;
pub use utility::*;

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

pub mod api {
    pub use mltg_bindings::Windows::Win32::System::Com::{
        CoInitializeEx,
        CoUninitialize,
        COINIT_APARTMENTTHREADED,
        COINIT_MULTITHREADED,
        COINIT_DISABLE_OLE1DDE,
        COINIT_SPEED_OVER_MEMORY,
    };
}