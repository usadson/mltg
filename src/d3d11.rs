use crate::*;
use windows::core::{IUnknown, Interface};
use windows::Win32::Graphics::{Direct3D11::*, Dxgi::*};

#[derive(Clone, PartialEq, Eq)]
pub struct RenderTarget(pub(crate) ID2D1Bitmap1);

impl Target for RenderTarget {
    fn bitmap(&self) -> &ID2D1Bitmap1 {
        &self.0
    }

    fn size(&self) -> Size {
        unsafe {
            let size = self.0.GetSize();
            Size::new(size.width, size.height)
        }
    }

    fn physical_size(&self) -> gecl::Size<u32> {
        unsafe {
            let size = self.0.GetPixelSize();
            gecl::Size::new(size.width, size.height)
        }
    }
}

unsafe impl Send for RenderTarget {}
unsafe impl Sync for RenderTarget {}

#[derive(Clone)]
pub struct Direct3D11 {
    d2d1_factory: ID2D1Factory1,
    device_context: ID2D1DeviceContext,
}

impl Direct3D11 {
    /// # Safety
    /// 
    /// `d3d12_device` must be a `ID3D11Device`.
    pub unsafe fn new<T>(d3d11_device: &T) -> Result<Self> {
        let d3d11_device: ID3D11Device = (*(d3d11_device as *const _ as *const IUnknown)).cast()?;
        let d2d1_factory = {
            let mut p: Option<ID2D1Factory1> = None;
            D2D1CreateFactory(
                D2D1_FACTORY_TYPE_MULTI_THREADED,
                &ID2D1Factory1::IID,
                &D2D1_FACTORY_OPTIONS {
                    debugLevel: D2D1_DEBUG_LEVEL_ERROR,
                },
                &mut p as *mut _ as _,
            )
            .map(|_| p.unwrap())?
        };
        let dxgi_device: IDXGIDevice = d3d11_device.cast()?;
        let d2d1_device = d2d1_factory.CreateDevice(&dxgi_device)?;
        let device_context = d2d1_device.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;
        Ok(Self {
            d2d1_factory,
            device_context,
        })
    }

    pub(crate) fn back_buffers(&self, swap_chain: &IDXGISwapChain1) -> Result<Vec<RenderTarget>> {
        unsafe {
            let desc = swap_chain.GetDesc1()?;
            let surface: IDXGISurface = swap_chain.GetBuffer(0)?;
            let bitmap = {
                self.device_context.CreateBitmapFromDxgiSurface(
                    &surface,
                    &D2D1_BITMAP_PROPERTIES1 {
                        pixelFormat: D2D1_PIXEL_FORMAT {
                            format: desc.Format,
                            alphaMode: D2D1_ALPHA_MODE_IGNORE,
                        },
                        bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                        dpiX: 96.0,
                        dpiY: 96.0,
                        ..Default::default()
                    },
                )?
            };
            Ok(vec![RenderTarget(bitmap)])
        }
    }
}

impl Context<Direct3D11> {
    /// # Safety
    /// 
    /// `target` must be an `ID3D11SwapChain1`.
    #[inline]
    pub unsafe fn create_back_buffers<T>(&self, swap_chain: &T) -> Result<Vec<RenderTarget>> {
        let p = swap_chain as *const _ as *const IUnknown;
        let swap_chain: IDXGISwapChain1 = (*p).cast()?;
        self.backend.back_buffers(&swap_chain)
    }

    /// # Safety
    /// 
    /// `target` must be an `ID3D11Texture2D`.
    pub unsafe fn create_render_target<T>(&self, target: &T) -> Result<RenderTarget> {
        let target = target as *const _ as *const IUnknown;
        let texture: ID3D11Texture2D = (*target).cast()?;
        let desc = {
            let mut desc = D3D11_TEXTURE2D_DESC::default();
            texture.GetDesc(&mut desc);
            desc
        };
        if cfg!(debug_assertions) {
            assert!((desc.BindFlags & D3D11_BIND_RENDER_TARGET) == D3D11_BIND_RENDER_TARGET);
        }
        let surface: IDXGISurface = texture.cast()?;
        let bitmap = {
            self.backend.device_context.CreateBitmapFromDxgiSurface(
                &surface,
                &D2D1_BITMAP_PROPERTIES1 {
                    pixelFormat: D2D1_PIXEL_FORMAT {
                        format: desc.Format,
                        alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                    },
                    bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                    ..Default::default()
                },
            )?
        };
        Ok(RenderTarget(bitmap))
    }
}

unsafe impl Send for Direct3D11 {}
unsafe impl Sync for Direct3D11 {}

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

    fn begin_draw(&self, _target: &RenderTarget) {}
    fn end_draw(&self, _target: &RenderTarget) {}
}
