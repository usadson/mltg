mod brush;
mod context;
pub mod d2d;
pub mod d3d11;
pub mod d3d12;
mod error;
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
pub use error::*;
pub use image::*;
pub use path::*;
pub use shape::*;
pub use stroke_style::*;
pub use text::*;
pub use utility::*;

pub type RenderTarget<T> = <T as Backend>::RenderTarget;
