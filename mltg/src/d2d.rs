use crate::bindings::Windows::Win32::{
    Graphics::{Direct3D11::*, Dxgi::*},
    System::SystemServices::*,
    UI::WindowsAndMessaging::*,
};
use crate::d3d11;
use crate::*;

pub type RenderTarget = d3d11::RenderTarget;

pub struct Direct2D {
    _d3d11_device: ID3D11Device,
    swap_chain: IDXGISwapChain1,
    object: d3d11::Direct3D11,
}

impl Direct2D {
    pub fn new(
        hwnd: *mut std::ffi::c_void,
        size: impl Into<gecl::Size<u32>>,
    ) -> windows::Result<Self> {
        unsafe {
            let d3d11_device = {
                const FEATURE_LEVELS: [D3D_FEATURE_LEVEL; 1] = [D3D_FEATURE_LEVEL_11_0];
                let mut p = None;
                D3D11CreateDevice(
                    None,
                    D3D_DRIVER_TYPE_HARDWARE,
                    HINSTANCE::NULL,
                    D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                    FEATURE_LEVELS.as_ptr() as _,
                    FEATURE_LEVELS.len() as _,
                    D3D11_SDK_VERSION,
                    &mut p,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
                .and_some(p)?
            };
            let dxgi_factory: IDXGIFactory2 = { CreateDXGIFactory1()? };
            let swap_chain = {
                let hwnd = HWND(hwnd as _);
                let size = size.into();
                let mut p = None;
                dxgi_factory
                    .CreateSwapChainForHwnd(
                        &d3d11_device,
                        hwnd,
                        &DXGI_SWAP_CHAIN_DESC1 {
                            Width: size.width,
                            Height: size.height,
                            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                            BufferCount: 2,
                            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                            SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                            SampleDesc: DXGI_SAMPLE_DESC {
                                Count: 1,
                                Quality: 0,
                            },
                            ..Default::default()
                        },
                        std::ptr::null_mut(),
                        None,
                        &mut p,
                    )
                    .and_some(p)?
            };
            let object = d3d11::Direct3D11::new(&d3d11_device)?;
            Ok(Self {
                _d3d11_device: d3d11_device,
                swap_chain,
                object,
            })
        }
    }

    pub fn swap_chain(&self) -> &IDXGISwapChain1 {
        &self.swap_chain
    }

    pub fn resize(&self, size: impl Into<gecl::Size<u32>>) {
        unsafe {
            let size = size.into();
            self.swap_chain
                .ResizeBuffers(0, size.width, size.height, DXGI_FORMAT_UNKNOWN, 0)
                .ok()
                .ok();
        }
    }
}

impl Backend for Direct2D {
    type RenderTarget = RenderTarget;

    #[inline]
    fn device_context(&self) -> &ID2D1DeviceContext {
        self.object.device_context()
    }

    #[inline]
    fn d2d1_factory(&self) -> &ID2D1Factory1 {
        self.object.d2d1_factory()
    }

    #[inline]
    fn back_buffers(
        &self,
        swap_chain: &IDXGISwapChain1,
    ) -> windows::Result<Vec<Self::RenderTarget>> {
        self.object.back_buffers(swap_chain)
    }

    #[inline]
    fn render_target(
        &self,
        target: &impl windows::Interface,
    ) -> windows::Result<Self::RenderTarget> {
        Ok(d3d11::RenderTarget(
            target
                .cast::<ID2D1Bitmap1>()
                .expect("cannot cast to ID2D1Bitmap1"),
        ))
    }

    #[inline]
    fn begin_draw(&self, target: &Self::RenderTarget) {
        self.object.begin_draw(target);
    }

    #[inline]
    fn end_draw(&self, target: &Self::RenderTarget) {
        self.object.end_draw(target);
        unsafe {
            self.swap_chain.Present(1, 0).ok().ok();
        }
    }
}
