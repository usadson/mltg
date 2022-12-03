use crate::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::{Direct2D::*, Direct3D::*, Direct3D11::*, Dxgi::Common::*, Dxgi::*};

#[derive(Clone, PartialEq, Eq)]
pub struct RenderTarget {
    swap_chain: IDXGISwapChain1,
    render_target: Option<d3d11::RenderTarget>,
    interval: u32,
}

impl RenderTarget {
    #[inline]
    pub fn set_interval(&mut self, interval: u32) {
        self.interval = interval;
    }
}

impl Target for RenderTarget {
    fn bitmap(&self) -> &ID2D1Bitmap1 {
        self.render_target.as_ref().unwrap().bitmap()
    }

    fn size(&self) -> Size<f32> {
        self.render_target.as_ref().unwrap().size()
    }

    fn physical_size(&self) -> Size<u32> {
        self.render_target.as_ref().unwrap().physical_size()
    }
}

#[derive(Clone)]
pub struct Direct2D {
    d3d11_device: ID3D11Device,
    d3d11: Direct3D11,
    dxgi_factory: IDXGIFactory2,
}

impl Direct2D {
    #[inline]
    pub fn new() -> Result<Self> {
        unsafe {
            let d3d11_device: ID3D11Device = {
                let mut p = None;
                D3D11CreateDevice(
                    None,
                    D3D_DRIVER_TYPE_HARDWARE,
                    HINSTANCE::default(),
                    D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                    Some(&[D3D_FEATURE_LEVEL_11_0]),
                    D3D11_SDK_VERSION,
                    Some(&mut p),
                    None,
                    None,
                )
                .map(|_| p.unwrap())?
            };
            let dxgi_factory: IDXGIFactory2 = CreateDXGIFactory1()?;
            let d3d11 = Direct3D11::new(&d3d11_device)?;
            Ok(Self {
                d3d11_device,
                dxgi_factory,
                d3d11,
            })
        }
    }
}

impl Context<Direct2D> {
    pub fn create_render_target(
        &self,
        hwnd: impl WindowHandle,
        size: impl Into<Size<u32>>,
    ) -> Result<RenderTarget> {
        let size = size.into();
        unsafe {
            let swap_chain = self.backend.dxgi_factory.CreateSwapChainForHwnd(
                &self.backend.d3d11_device,
                hwnd.handle(),
                &DXGI_SWAP_CHAIN_DESC1 {
                    Width: size.width,
                    Height: size.height,
                    Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    BufferCount: 2,
                    BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                    SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
                    Scaling: DXGI_SCALING_NONE,
                    SampleDesc: DXGI_SAMPLE_DESC {
                        Count: 1,
                        Quality: 0,
                    },
                    ..Default::default()
                },
                None,
                None,
            )?;
            let render_target = self
                .backend
                .d3d11
                .create_render_target_from_swap_chain(&self.d2d1_device_context, &swap_chain)?;
            Ok(RenderTarget {
                swap_chain,
                render_target: Some(render_target),
                interval: 1,
            })
        }
    }

    pub fn resize_target(
        &self,
        target: &mut RenderTarget,
        size: impl Into<Size<u32>>,
    ) -> Result<()> {
        let size = size.into();
        target.render_target = None;
        unsafe {
            target
                .swap_chain
                .ResizeBuffers(0, size.width, size.height, DXGI_FORMAT_UNKNOWN, 0)?;
        }
        target.render_target =
            Some(self.backend.d3d11.create_render_target_from_swap_chain(
                &self.d2d1_device_context,
                &target.swap_chain,
            )?);
        Ok(())
    }
}

impl Backend for Direct2D {
    type RenderTarget = RenderTarget;

    fn d2d1_device(&self) -> &ID2D1Device5 {
        self.d3d11.d2d1_device()
    }

    fn d2d1_factory(&self) -> &ID2D1Factory6 {
        self.d3d11.d2d1_factory()
    }

    fn begin_draw(&self, target: &Self::RenderTarget) {
        self.d3d11
            .begin_draw(target.render_target.as_ref().unwrap())
    }

    fn end_draw(&self, target: &Self::RenderTarget, ret: Result<()>) -> Result<()> {
        let ret = self
            .d3d11
            .end_draw(target.render_target.as_ref().unwrap(), ret);
        match ret {
            Ok(_) => unsafe {
                let params = DXGI_PRESENT_PARAMETERS::default();
                target
                    .swap_chain
                    .Present1(target.interval, 0, &params)
                    .ok()?;
                Ok(())
            },
            Err(e) => Err(e),
        }
    }
}
