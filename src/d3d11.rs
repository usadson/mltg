use crate::*;
use windows::core::Interface;
use windows::Win32::Graphics::{Direct2D::Common::*, Direct2D::*, Direct3D11::*, Dxgi::*};

#[derive(Clone, PartialEq, Eq)]
pub struct RenderTarget(pub(crate) ID2D1Bitmap1);

impl Target for RenderTarget {
    fn bitmap(&self) -> &ID2D1Bitmap1 {
        &self.0
    }

    fn size(&self) -> Size<f32> {
        unsafe { Wrapper(self.0.GetSize()).into() }
    }

    fn physical_size(&self) -> Size<u32> {
        unsafe { Wrapper(self.0.GetPixelSize()).into() }
    }
}

#[derive(Clone, Debug)]
pub struct Direct3D11 {
    d2d1_factory: ID2D1Factory6,
    d2d1_device: ID2D1Device5,
}

impl Direct3D11 {
    pub fn new<T>(d3d11_device: &T) -> Result<Self>
    where
        T: Interface,
    {
        unsafe {
            let d3d11_device: ID3D11Device = d3d11_device.cast()?;
            let d2d1_factory =
                D2D1CreateFactory::<ID2D1Factory6>(D2D1_FACTORY_TYPE_MULTI_THREADED, None)?;
            let dxgi_device: IDXGIDevice = d3d11_device.cast()?;
            let d2d1_device = d2d1_factory.CreateDevice(&dxgi_device)?.cast()?;
            Ok(Self {
                d2d1_factory,
                d2d1_device,
            })
        }
    }

    pub(crate) fn create_render_target_from_swap_chain(
        &self,
        ctx: &ID2D1DeviceContext5,
        swap_chain: &IDXGISwapChain1,
    ) -> Result<RenderTarget> {
        unsafe {
            let swap_chain: IDXGISwapChain1 =
                swap_chain.cast().expect("cannot cast to IDXGISwapChain1");
            let desc = swap_chain.GetDesc1()?;
            let surface: IDXGISurface = swap_chain.GetBuffer(0)?;
            let bitmap = ctx.CreateBitmapFromDxgiSurface(
                &surface,
                Some(&D2D1_BITMAP_PROPERTIES1 {
                    pixelFormat: D2D1_PIXEL_FORMAT {
                        format: desc.Format,
                        alphaMode: D2D1_ALPHA_MODE_IGNORE,
                    },
                    bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                    dpiX: 96.0,
                    dpiY: 96.0,
                    ..Default::default()
                }),
            )?;
            Ok(RenderTarget(bitmap))
        }
    }
}

impl Context<Direct3D11> {
    #[inline]
    pub fn create_render_target_from_swap_chain<T>(&self, swap_chain: &T) -> Result<RenderTarget>
    where
        T: Interface,
    {
        let swap_chain = swap_chain.cast().expect("cannot cast to IDXGISwapChain1");
        self.backend
            .create_render_target_from_swap_chain(&self.d2d1_device_context, &swap_chain)
    }

    #[inline]
    pub fn create_render_target<T>(&self, target: &T) -> Result<RenderTarget>
    where
        T: Interface,
    {
        unsafe {
            let target: ID3D11Texture2D = target.cast().expect("cannot cast to ID3D11Texture2D");
            let mut desc = D3D11_TEXTURE2D_DESC::default();
            target.GetDesc(&mut desc);
            let surface: IDXGISurface = target.cast().unwrap();
            let bitmap = self.d2d1_device_context.CreateBitmapFromDxgiSurface(
                &surface,
                Some(&D2D1_BITMAP_PROPERTIES1 {
                    pixelFormat: D2D1_PIXEL_FORMAT {
                        format: desc.Format,
                        alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                    },
                    bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                    ..Default::default()
                }),
            )?;
            Ok(RenderTarget(bitmap))
        }
    }
}

impl Backend for Direct3D11 {
    type RenderTarget = RenderTarget;

    fn d2d1_factory(&self) -> &ID2D1Factory6 {
        &self.d2d1_factory
    }

    fn d2d1_device(&self) -> &ID2D1Device5 {
        &self.d2d1_device
    }

    fn begin_draw(&self, _target: &Self::RenderTarget) {}

    fn end_draw(&self, _target: &Self::RenderTarget, ret: Result<()>) -> Result<()> {
        ret
    }
}
