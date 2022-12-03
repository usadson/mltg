use raw_window_handle::RawWindowHandle;
use windows::Win32::{Foundation::HWND, Graphics::Direct2D::Common::*};

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub(crate) struct Wrapper<T>(pub(crate) T);

pub type Rgba<T> = gecl::Rgba<T>;

impl From<Wrapper<Rgba<f32>>> for D2D1_COLOR_F {
    #[inline]
    fn from(src: Wrapper<Rgba<f32>>) -> Self {
        Self {
            r: src.0.r,
            g: src.0.g,
            b: src.0.b,
            a: src.0.a,
        }
    }
}

pub trait WindowHandle {
    fn handle(&self) -> HWND;
}

impl WindowHandle for HWND {
    #[inline]
    fn handle(&self) -> HWND {
        *self
    }
}

impl WindowHandle for RawWindowHandle {
    #[inline]
    fn handle(&self) -> HWND {
        let RawWindowHandle::Win32(handle) = self else { panic!() };
        HWND(handle.hwnd as _)
    }
}

impl WindowHandle for *const std::ffi::c_void {
    #[inline]
    fn handle(&self) -> HWND {
        HWND(*self as _)
    }
}

impl WindowHandle for *mut std::ffi::c_void {
    #[inline]
    fn handle(&self) -> HWND {
        HWND(*self as _)
    }
}

impl WindowHandle for isize {
    #[inline]
    fn handle(&self) -> HWND {
        HWND(*self)
    }
}
