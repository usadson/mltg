use crate::*;
use windows::core::{IUnknown, Interface};
use windows::Win32::Graphics::{Direct3D11::*, Direct3D11on12::*, Direct3D12::*, Dxgi::*};

#[derive(Clone, PartialEq, Eq)]
pub struct RenderTarget {
    wrapper: ID3D11Resource,
    bitmap: ID2D1Bitmap1,
}

unsafe impl Send for RenderTarget {}
unsafe impl Sync for RenderTarget {}

impl Target for RenderTarget {
    fn bitmap(&self) -> &ID2D1Bitmap1 {
        &self.bitmap
    }

    fn size(&self) -> Size {
        unsafe {
            let size = self.bitmap.GetSize();
            Size::new(size.width, size.height)
        }
    }

    fn physical_size(&self) -> gecl::Size<u32> {
        unsafe {
            let size = self.bitmap.GetPixelSize();
            gecl::Size::new(size.width, size.height)
        }
    }
}

#[derive(Clone)]
pub struct Direct3D12 {
    d3d11on12_device: ID3D11On12Device,
    d2d1_factory: ID2D1Factory1,
    d2d1_device_context: ID2D1DeviceContext,
    d3d11_device_context: ID3D11DeviceContext,
}

impl Direct3D12 {
    /// # Safety
    ///
    /// `d3d12_device` must be an `ID3D12Device` and `command_queue` must be an `ID3D12CommandQueue`.
    pub unsafe fn new<T, U>(d3d12_device: &T, command_queue: &U) -> Result<Self> {
        let d3d12_device: ID3D12Device = (*(d3d12_device as *const _ as *const IUnknown)).cast()?;
        let command_queue: ID3D12CommandQueue =
            (*(command_queue as *const _ as *const IUnknown)).cast()?;
        let (d3d11on12_device, d3d11_device_context) = {
            let queues = [Some(command_queue.cast::<IUnknown>().unwrap())];
            let mut p = None;
            let mut dc = None;
            D3D11On12CreateDevice(
                &d3d12_device,
                D3D11_CREATE_DEVICE_BGRA_SUPPORT.0,
                &[],
                &queues,
                0,
                &mut p,
                &mut dc,
                std::ptr::null_mut(),
            )
            .map(|_| (p.unwrap().cast::<ID3D11On12Device>().unwrap(), dc.unwrap()))?
        };
        let d2d1_factory =
            D2D1CreateFactory::<ID2D1Factory1>(D2D1_FACTORY_TYPE_MULTI_THREADED, std::ptr::null())?;
        let dxgi_device = d3d11on12_device.cast::<IDXGIDevice>()?;
        let d2d1_device = d2d1_factory.CreateDevice(&dxgi_device)?;
        let d2d1_device_context =
            d2d1_device.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;
        Ok(Self {
            d3d11on12_device,
            d2d1_factory,
            d2d1_device_context,
            d3d11_device_context,
        })
    }

    fn back_buffers(&self, swap_chain: &IDXGISwapChain1) -> Result<Vec<RenderTarget>> {
        unsafe {
            let desc = swap_chain.GetDesc1()?;
            let bmp_props = D2D1_BITMAP_PROPERTIES1 {
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: desc.Format,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                ..Default::default()
            };
            let mut targets = vec![];
            for i in 0..desc.BufferCount {
                let buffer: ID3D12Resource = swap_chain.GetBuffer(i)?;
                let flags = D3D11_RESOURCE_FLAGS {
                    BindFlags: D3D11_BIND_RENDER_TARGET.0,
                    ..Default::default()
                };
                let wrapper = {
                    let mut wrapper: Option<ID3D11Resource> = None;
                    self.d3d11on12_device
                        .CreateWrappedResource(
                            &buffer,
                            &flags,
                            D3D12_RESOURCE_STATE_RENDER_TARGET,
                            D3D12_RESOURCE_STATE_PRESENT,
                            &mut wrapper,
                        )
                        .map(|_| wrapper.unwrap())?
                };
                let surface = wrapper.cast::<IDXGISurface>()?;
                let bitmap = {
                    self.d2d1_device_context
                        .CreateBitmapFromDxgiSurface(&surface, &bmp_props)?
                };
                targets.push(RenderTarget { wrapper, bitmap });
            }
            Ok(targets)
        }
    }
}

impl Context<Direct3D12> {
    /// # Safety
    ///
    /// `swap_chain` must be an `IDXGISwapChain1`.
    #[inline]
    pub unsafe fn create_back_buffers<T>(&self, swap_chain: &T) -> Result<Vec<RenderTarget>> {
        let p = swap_chain as *const _ as *const IUnknown;
        let swap_chain: IDXGISwapChain1 = p.as_ref().unwrap().cast()?;
        self.backend.back_buffers(&swap_chain)
    }

    /// # Safety
    ///
    /// `target` must be an `ID3D12Resource`.
    pub unsafe fn create_render_target<T>(&self, target: &T) -> Result<RenderTarget> {
        let target = target as *const _ as *const IUnknown;
        let resource: ID3D12Resource = target.as_ref().unwrap().cast()?;
        let desc = resource.GetDesc();
        if cfg!(debug_assertions) {
            assert!(
                (desc.Flags & D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET) != D3D12_RESOURCE_FLAG_NONE
            );
        }
        let wrapper = {
            let mut wrapper: Option<ID3D11Resource> = None;
            self.backend
                .d3d11on12_device
                .CreateWrappedResource(
                    &resource,
                    &D3D11_RESOURCE_FLAGS {
                        BindFlags: D3D11_BIND_RENDER_TARGET.0,
                        ..Default::default()
                    },
                    D3D12_RESOURCE_STATE_RENDER_TARGET,
                    D3D12_RESOURCE_STATE_COMMON,
                    &mut wrapper,
                )
                .map(|_| wrapper.unwrap())?
        };
        let surface: IDXGISurface = wrapper.cast()?;
        let bitmap = {
            self.backend
                .d2d1_device_context
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
                )?
        };
        Ok(RenderTarget { wrapper, bitmap })
    }

    #[inline]
    pub fn flush(&self) {
        unsafe {
            self.backend.d3d11_device_context.Flush();
        }
    }
}

unsafe impl Send for Direct3D12 {}
unsafe impl Sync for Direct3D12 {}

impl Backend for Direct3D12 {
    type RenderTarget = RenderTarget;

    #[inline]
    fn device_context(&self) -> &ID2D1DeviceContext {
        &self.d2d1_device_context
    }

    #[inline]
    fn d2d1_factory(&self) -> &ID2D1Factory1 {
        &self.d2d1_factory
    }

    #[inline]
    fn begin_draw(&self, target: &Self::RenderTarget) {
        unsafe {
            self.d3d11on12_device
                .AcquireWrappedResources(&[Some(target.wrapper.clone())]);
        }
    }

    #[inline]
    fn end_draw(&self, target: &Self::RenderTarget) {
        unsafe {
            self.d3d11on12_device
                .ReleaseWrappedResources(&[Some(target.wrapper.clone())]);
            self.d3d11_device_context.Flush();
            self.d3d11_device_context.ClearState();
        }
    }
}
