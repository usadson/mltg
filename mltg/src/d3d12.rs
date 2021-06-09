use crate::bindings::Windows::Win32::Graphics::{Direct3D11::*, Direct3D12::*, Dxgi::*};
use crate::*;
use windows::{Abi, IUnknown, Interface};

#[derive(Clone, PartialEq, Eq)]
pub struct RenderTarget {
    wrapper: ID3D11Resource,
    bitmap: ID2D1Bitmap1,
}

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

pub struct Direct3D12 {
    d3d11on12_device: ID3D11On12Device,
    d2d1_factory: ID2D1Factory1,
    d2d1_device_context: ID2D1DeviceContext,
    d3d11_device_context: ID3D11DeviceContext,
}

impl Direct3D12 {
    pub fn new(
        d3d12_device: &impl windows::Interface,
        command_queue: &impl windows::Interface,
    ) -> windows::Result<Self> {
        unsafe {
            let d3d12_device = d3d12_device
                .cast::<ID3D12Device>()
                .expect("cannot cast to ID3D12Device");
            let command_queue = command_queue
                .cast::<ID3D12CommandQueue>()
                .expect("cannot cast to ID3D12CommandQueue");
            let (d3d11on12_device, d3d11_device_context) = {
                let mut queues = [command_queue.cast::<IUnknown>().unwrap()];
                let mut p = None;
                let mut dc = None;
                D3D11On12CreateDevice(
                    &d3d12_device,
                    D3D11_CREATE_DEVICE_BGRA_SUPPORT.0,
                    std::ptr::null(),
                    0,
                    queues.as_mut_ptr() as _,
                    queues.len() as _,
                    0,
                    &mut p,
                    &mut dc,
                    std::ptr::null_mut(),
                )
                .and_then(|| (p.unwrap().cast::<ID3D11On12Device>().unwrap(), dc.unwrap()))?
            };
            let d2d1_factory = {
                let mut p: Option<ID2D1Factory1> = None;
                D2D1CreateFactory(
                    D2D1_FACTORY_TYPE_SINGLE_THREADED,
                    &ID2D1Factory1::IID,
                    std::ptr::null(),
                    p.set_abi(),
                )
                .and_some(p)?
            };
            let dxgi_device = d3d11on12_device.cast::<IDXGIDevice>()?;
            let d2d1_device = {
                let mut p = None;
                d2d1_factory
                    .CreateDevice(&dxgi_device, &mut p)
                    .and_some(p)?
            };
            let d2d1_device_context = {
                let mut p = None;
                d2d1_device
                    .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE, &mut p)
                    .and_some(p)?
            };
            Ok(Self {
                d3d11on12_device,
                d2d1_factory,
                d2d1_device_context,
                d3d11_device_context,
            })
        }
    }

    #[inline]
    pub fn flush(&self) {
        unsafe {
            self.d3d11_device_context.Flush();
        }
    }
}

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
                    BindFlags: D3D11_BIND_RENDER_TARGET.0 as _,
                    ..Default::default()
                };
                let wrapper: ID3D11Resource = self.d3d11on12_device.CreateWrappedResource(
                    &buffer,
                    &flags,
                    D3D12_RESOURCE_STATE_RENDER_TARGET,
                    D3D12_RESOURCE_STATE_PRESENT,
                )?;
                let surface = wrapper.cast::<IDXGISurface>()?;
                let bitmap = {
                    let mut p = None;
                    self.d2d1_device_context
                        .CreateBitmapFromDxgiSurface(&surface, &bmp_props, &mut p)
                        .and_some(p)?
                };
                targets.push(RenderTarget { wrapper, bitmap });
            }
            Ok(targets)
        }
    }

    fn render_target(
        &self,
        target: &impl windows::Interface,
    ) -> windows::Result<Self::RenderTarget> {
        unsafe {
            let resource: ID3D12Resource = target.cast().expect("cannot cast to ID3D12Resource");
            let desc = resource.GetDesc();
            if cfg!(debug_assertions) {
                assert!((desc.Flags.0 & D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET.0) != 0);
            }
            let wrapper: ID3D11Resource = {
                self.d3d11on12_device.CreateWrappedResource(
                    &resource,
                    &D3D11_RESOURCE_FLAGS {
                        BindFlags: D3D11_BIND_RENDER_TARGET.0 as _,
                        ..Default::default()
                    },
                    D3D12_RESOURCE_STATE_RENDER_TARGET,
                    D3D12_RESOURCE_STATE_COMMON,
                )?
            };
            let surface: IDXGISurface = wrapper.cast()?;
            let bitmap = {
                let mut p = None;
                self.d2d1_device_context
                    .CreateBitmapFromDxgiSurface(
                        &surface,
                        &D2D1_BITMAP_PROPERTIES1 {
                            pixelFormat: D2D1_PIXEL_FORMAT {
                                format: desc.Format,
                                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                            },
                            bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET
                                | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                            ..Default::default()
                        },
                        &mut p,
                    )
                    .and_some(p)?
            };
            Ok(RenderTarget { wrapper, bitmap })
        }
    }

    #[inline]
    fn begin_draw(&self, target: &Self::RenderTarget) {
        unsafe {
            let mut wrappers = [target.wrapper.clone()];
            self.d3d11on12_device
                .AcquireWrappedResources(wrappers.as_mut_ptr() as _, wrappers.len() as _);
        }
    }

    #[inline]
    fn end_draw(&self, target: &Self::RenderTarget) {
        unsafe {
            let mut wrappers = [target.wrapper.clone()];
            self.d3d11on12_device
                .ReleaseWrappedResources(wrappers.as_mut_ptr() as _, wrappers.len() as _);
            self.d3d11_device_context.Flush();
            self.d3d11_device_context.ClearState();
        }
    }
}
