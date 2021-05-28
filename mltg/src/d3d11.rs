use crate::bindings::Windows::Win32::Graphics::{Direct3D11::*, Dxgi::*};
use crate::*;
use windows::{Abi, Interface};

#[derive(Clone, PartialEq, Eq)]
pub struct RenderTarget(pub(crate) ID2D1Bitmap1);

impl Target for RenderTarget {
    fn bitmap(&self) -> &ID2D1Bitmap1 {
        &self.0
    }
}

pub struct Direct3D11 {
    d2d1_factory: ID2D1Factory1,
    device_context: ID2D1DeviceContext,
}

impl Direct3D11 {
    pub fn new(d3d11_device: &impl windows::Interface) -> windows::Result<Self> {
        unsafe {
            let d3d11_device: ID3D11Device =
                d3d11_device.cast().expect("cannot cast to ID3D11Device");
            let d2d1_factory: ID2D1Factory1 = {
                let mut p = None;
                D2D1CreateFactory(
                    D2D1_FACTORY_TYPE_SINGLE_THREADED,
                    &ID2D1Factory1::IID,
                    &D2D1_FACTORY_OPTIONS {
                        debugLevel: D2D1_DEBUG_LEVEL_ERROR,
                    },
                    p.set_abi(),
                )
                .and_some(p)?
            };
            let dxgi_device: IDXGIDevice = d3d11_device.cast()?;
            let d2d1_device = {
                let mut p = None;
                d2d1_factory
                    .CreateDevice(&dxgi_device, &mut p)
                    .and_some(p)?
            };
            let device_context = {
                let mut p = None;
                d2d1_device
                    .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE, &mut p)
                    .and_some(p)?
            };
            Ok(Self {
                d2d1_factory,
                device_context,
            })
        }
    }
}

impl Backend for Direct3D11 {
    type RenderTarget = RenderTarget;

    #[inline]
    fn device_context(&self) -> &ID2D1DeviceContext {
        &self.device_context
    }

    #[inline]
    fn d2d1_factory(&self) -> &ID2D1Factory1 {
        &self.d2d1_factory
    }

    fn back_buffers(
        &self,
        swap_chain: &IDXGISwapChain1,
    ) -> windows::Result<Vec<Self::RenderTarget>> {
        unsafe {
            let desc = {
                let mut desc = Default::default();
                swap_chain.GetDesc1(&mut desc).ok()?;
                desc
            };
            let surface: IDXGISurface = swap_chain.GetBuffer(0)?;
            let bitmap = {
                let mut p = None;
                self.device_context
                    .CreateBitmapFromDxgiSurface(
                        &surface,
                        &D2D1_BITMAP_PROPERTIES1 {
                            pixelFormat: D2D1_PIXEL_FORMAT {
                                format: desc.Format,
                                alphaMode: D2D1_ALPHA_MODE_IGNORE,
                            },
                            bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET
                                | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                            dpiX: 96.0,
                            dpiY: 96.0,
                            ..Default::default()
                        },
                        &mut p,
                    )
                    .and_some(p)?
            };
            Ok(vec![RenderTarget(bitmap)])
        }
    }

    fn render_target(
        &self,
        target: &impl windows::Interface,
    ) -> windows::Result<Self::RenderTarget> {
        let texture: ID3D11Texture2D = target.cast().expect("cannot cast to ID3D11Texture2D");
        let desc = unsafe {
            let mut desc = D3D11_TEXTURE2D_DESC::default();
            texture.GetDesc(&mut desc);
            desc
        };
        if cfg!(debug_assertions) {
            assert!((desc.BindFlags & D3D11_BIND_RENDER_TARGET.0 as u32) != 0);
        }
        let surface: IDXGISurface = target.cast()?;
        let bitmap = unsafe {
            let mut p = None;
            self.device_context
                .CreateBitmapFromDxgiSurface(
                    &surface,
                    &D2D1_BITMAP_PROPERTIES1 {
                        pixelFormat: D2D1_PIXEL_FORMAT {
                            format: desc.Format,
                            alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                        },
                        bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                        ..Default::default()
                    },
                    &mut p,
                )
                .and_some(p)?
        };
        Ok(RenderTarget(bitmap))
    }

    fn begin_draw(&self, _target: &RenderTarget) {}
    fn end_draw(&self, _target: &RenderTarget) {}
}
