use crate::d3d11;
use crate::*;
use windows::Win32::{
    Foundation::*,
    Graphics::{Direct3D11::*, Dxgi::Common::*, Dxgi::*},
};

pub struct RenderTarget {
    backend: Direct2D,
    render_target: Option<d3d11::RenderTarget>,
    swap_chain: IDXGISwapChain1,
}

impl Target for RenderTarget {
    #[inline]
    fn bitmap(&self) -> &ID2D1Bitmap1 {
        self.render_target.as_ref().unwrap().bitmap()
    }

    #[inline]
    fn size(&self) -> Size {
        self.render_target.as_ref().unwrap().size()
    }

    #[inline]
    fn physical_size(&self) -> gecl::Size<u32> {
        self.render_target.as_ref().unwrap().physical_size()
    }
}

impl RenderTarget {
    pub fn resize(&mut self, size: impl Into<gecl::Size<u32>>) -> Result<()> {
        let size = size.into();
        self.render_target = None;
        unsafe {
            self.swap_chain
                .ResizeBuffers(0, size.width, size.height, DXGI_FORMAT_UNKNOWN, 0)?;
        }
        self.render_target = Some(self.backend.object.back_buffers(&self.swap_chain)?[0].clone());
        Ok(())
    }

    pub fn present(
        &self,
        sync_interval: u32,
        dirty_rects: Option<&[ScreenRect]>,
        scroll: Option<Scroll>,
    ) {
        assert!(sync_interval <= 4);
        let scroll_rect = scroll.as_ref().map(|s| {
            let ep = s.rect.endpoint();
            RECT {
                left: s.rect.origin.x,
                top: s.rect.origin.y,
                right: ep.x,
                bottom: ep.y,
            }
        });
        let scroll_offset = scroll.as_ref().map(|s| POINT {
            x: s.offset.x,
            y: s.offset.y,
        });
        let params = DXGI_PRESENT_PARAMETERS {
            DirtyRectsCount: dirty_rects.as_ref().map_or(0, |dr| dr.len() as u32),
            pDirtyRects: dirty_rects.map_or(std::ptr::null_mut(), |dr| {
                dr.as_ptr() as *mut ScreenRect as *mut RECT
            }),
            pScrollRect: scroll_rect
                .as_ref()
                .map_or(std::ptr::null_mut(), |s| s as *const _ as *mut RECT),
            pScrollOffset: scroll_offset
                .as_ref()
                .map_or(std::ptr::null_mut(), |s| s as *const _ as *mut POINT),
        };
        unsafe {
            self.swap_chain.Present1(sync_interval, 0, &params).ok();
        }
    }
}

pub struct Scroll {
    pub rect: gecl::Rect<i32>,
    pub offset: gecl::Point<i32>,
}

impl Scroll {
    #[inline]
    pub fn new(rect: impl Into<gecl::Rect<i32>>, offset: impl Into<gecl::Point<i32>>) -> Self {
        Self {
            rect: rect.into(),
            offset: offset.into(),
        }
    }
}

#[derive(Clone)]
pub struct Direct2D {
    d3d11_device: ID3D11Device,
    dxgi_factory: IDXGIFactory2,
    object: d3d11::Direct3D11,
}

impl Direct2D {
    pub fn new() -> Result<Self> {
        unsafe {
            let d3d11_device: ID3D11Device = {
                const FEATURE_LEVELS: [D3D_FEATURE_LEVEL; 1] = [D3D_FEATURE_LEVEL_11_0];
                let mut p = None;
                D3D11CreateDevice(
                    None,
                    D3D_DRIVER_TYPE_HARDWARE,
                    HINSTANCE::default(),
                    D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                    &FEATURE_LEVELS,
                    D3D11_SDK_VERSION,
                    &mut p,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
                .map(|_| p.unwrap())?
            };
            let dxgi_factory: IDXGIFactory2 = CreateDXGIFactory1()?;
            let object = d3d11::Direct3D11::new(&d3d11_device)?;
            Ok(Self {
                d3d11_device,
                dxgi_factory,
                object,
            })
        }
    }
}

impl Context<Direct2D> {
    pub fn create_render_target(
        &self,
        hwnd: *const std::ffi::c_void,
        size: impl Into<gecl::Size<u32>>,
    ) -> Result<RenderTarget> {
        let swap_chain = unsafe {
            let hwnd = HWND(hwnd as _);
            let size = size.into();
            self.backend.dxgi_factory.CreateSwapChainForHwnd(
                &self.backend.d3d11_device,
                hwnd,
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
                std::ptr::null_mut(),
                None,
            )?
        };
        let render_targets = self.backend.object.back_buffers(&swap_chain)?;
        Ok(RenderTarget {
            backend: self.backend.clone(),
            swap_chain,
            render_target: Some(render_targets[0].clone()),
        })
    }
}

unsafe impl Send for Direct2D {}
unsafe impl Sync for Direct2D {}

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
    fn begin_draw(&self, target: &Self::RenderTarget) {
        self.object
            .begin_draw(target.render_target.as_ref().unwrap());
    }

    #[inline]
    fn end_draw(&self, target: &Self::RenderTarget) {
        self.object.end_draw(target.render_target.as_ref().unwrap());
    }
}
