use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use windows::Win32::{
    Foundation::{HINSTANCE, HWND},
    Graphics::{Direct3D::Fxc::*, Direct3D::*, Direct3D11::*, Dxgi::Common::*, Dxgi::*},
};
use winit::{dpi::*, event::*, event_loop::*, window::*};

#[repr(C)]
struct Vertex {
    position: [f32; 3],
    tex: [f32; 2],
}

impl Vertex {
    const fn new(position: [f32; 3], tex: [f32; 2]) -> Self {
        Self { position, tex }
    }
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("mltg to_d3d11_texture")
        .with_inner_size(LogicalSize::new(640, 480))
        .build(&event_loop)?;
    let (device, device_context) = unsafe {
        const FEATURE_LEVELS: [D3D_FEATURE_LEVEL; 1] = [D3D_FEATURE_LEVEL_11_0];
        let mut device = None;
        let mut device_context = None;
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            HINSTANCE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            Some(&FEATURE_LEVELS),
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            Some(&mut device_context),
        )
        .map(|_| (device.unwrap(), device_context.unwrap()))?
    };
    let dxgi_factory = unsafe { CreateDXGIFactory1::<IDXGIFactory2>()? };
    let window_size = window.inner_size();
    let swap_chain = unsafe {
        let RawWindowHandle::Win32(handle) = window.raw_window_handle() else { panic!() };
        dxgi_factory.CreateSwapChainForHwnd(
            &device,
            HWND(handle.hwnd as _),
            &DXGI_SWAP_CHAIN_DESC1 {
                Width: window_size.width,
                Height: window_size.height,
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                BufferCount: 2,
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                Scaling: DXGI_SCALING_NONE,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                ..Default::default()
            },
            None,
            None,
        )?
    };
    let rtv = unsafe {
        let buffer = swap_chain.GetBuffer::<ID3D11Texture2D>(0)?;
        device.CreateRenderTargetView(&buffer, None)?
    };
    let vertex_buffer = unsafe {
        const VERTICES: [Vertex; 4] = [
            Vertex::new([-0.5, 0.5, 0.0], [0.0, 0.0]),
            Vertex::new([0.5, 0.5, 0.0], [1.0, 0.0]),
            Vertex::new([-0.5, -0.5, 0.0], [0.0, 1.0]),
            Vertex::new([0.5, -0.5, 0.0], [1.0, 1.0]),
        ];
        device.CreateBuffer(
            &D3D11_BUFFER_DESC {
                ByteWidth: std::mem::size_of::<[Vertex; 4]>() as _,
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: D3D11_BIND_VERTEX_BUFFER,
                ..Default::default()
            },
            Some(&D3D11_SUBRESOURCE_DATA {
                pSysMem: VERTICES.as_ptr() as _,
                ..Default::default()
            }),
        )?
    };
    let index_buffer = unsafe {
        const INDICES: [u32; 6] = [0, 1, 2, 1, 3, 2];
        device.CreateBuffer(
            &D3D11_BUFFER_DESC {
                ByteWidth: std::mem::size_of::<[u32; 6]>() as _,
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: D3D11_BIND_INDEX_BUFFER,
                ..Default::default()
            },
            Some(&D3D11_SUBRESOURCE_DATA {
                pSysMem: INDICES.as_ptr() as _,
                ..Default::default()
            }),
        )?
    };
    let (vs, ps, input_layout) = unsafe {
        let hlsl_file = include_bytes!("../resources/tex.hlsl");
        let mut vs_blob = None;
        let mut ps_blob = None;
        let vs_blob = D3DCompile(
            hlsl_file.as_ptr() as _,
            hlsl_file.len(),
            windows::s!("tex.hlsl"),
            None,
            None,
            windows::s!("vs_main"),
            windows::s!("vs_5_0"),
            0,
            0,
            &mut vs_blob,
            None,
        )
        .map(|_| vs_blob.unwrap())?;
        let ps_blob = D3DCompile(
            hlsl_file.as_ptr() as _,
            hlsl_file.len(),
            windows::s!("tex.hlsl"),
            None,
            None,
            windows::s!("ps_main"),
            windows::s!("ps_5_0"),
            0,
            0,
            &mut ps_blob,
            None,
        )
        .map(|_| ps_blob.unwrap())?;
        let vs_blob = std::slice::from_raw_parts(
            vs_blob.GetBufferPointer() as *const u8,
            vs_blob.GetBufferSize(),
        );
        let ps_blob = std::slice::from_raw_parts(
            ps_blob.GetBufferPointer() as *const u8,
            ps_blob.GetBufferSize(),
        );
        let vs = device.CreateVertexShader(&vs_blob, None)?;
        let ps = device.CreatePixelShader(&ps_blob, None)?;
        let descs = [
            D3D11_INPUT_ELEMENT_DESC {
                SemanticName: windows::s!("POSITION"),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32B32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: 0,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
            D3D11_INPUT_ELEMENT_DESC {
                SemanticName: windows::s!("TEXCOORD"),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
        ];
        let input_layout = device.CreateInputLayout(&descs, vs_blob)?;
        (vs, ps, input_layout)
    };
    let tex = unsafe {
        device.CreateTexture2D(
            &D3D11_TEXTURE2D_DESC {
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: D3D11_BIND_RENDER_TARGET | D3D11_BIND_SHADER_RESOURCE,
                Width: window_size.width,
                Height: window_size.height,
                ArraySize: 1,
                MipLevels: 1,
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                ..Default::default()
            },
            None,
        )?
    };
    let tex_view = unsafe { device.CreateShaderResourceView(&tex, None)? };
    let sampler = unsafe {
        device.CreateSamplerState(&D3D11_SAMPLER_DESC {
            Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR,
            AddressU: D3D11_TEXTURE_ADDRESS_CLAMP,
            AddressV: D3D11_TEXTURE_ADDRESS_CLAMP,
            AddressW: D3D11_TEXTURE_ADDRESS_CLAMP,
            MinLOD: f32::MIN,
            MaxLOD: f32::MAX,
            MaxAnisotropy: 1,
            ..Default::default()
        })?
    };
    let blend = unsafe {
        let rt = D3D11_RENDER_TARGET_BLEND_DESC {
            BlendEnable: true.into(),
            SrcBlend: D3D11_BLEND_SRC_ALPHA,
            DestBlend: D3D11_BLEND_INV_SRC_ALPHA,
            BlendOp: D3D11_BLEND_OP_ADD,
            SrcBlendAlpha: D3D11_BLEND_ONE,
            DestBlendAlpha: D3D11_BLEND_ZERO,
            BlendOpAlpha: D3D11_BLEND_OP_ADD,
            RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL.0 as _,
        };
        device.CreateBlendState(&D3D11_BLEND_DESC {
            RenderTarget: [rt; 8],
            ..Default::default()
        })?
    };
    let context = mltg::Context::new(mltg::Direct3D11::new(&device)?)?;
    let factory = context.create_factory();
    let target = context.create_render_target(&tex)?;
    let image = factory.create_image_from_file("resources/ferris.png")?;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::RedrawRequested(_) => unsafe {
                let dc = &device_context;
                let window_size = window.inner_size();
                context
                    .draw(&target, |cmd| {
                        let desc = {
                            let mut desc = D3D11_TEXTURE2D_DESC::default();
                            tex.GetDesc(&mut desc);
                            desc
                        };
                        cmd.clear([0.0, 0.0, 0.0, 0.0]);
                        cmd.draw_image(
                            &image,
                            mltg::Rect::new((0.0, 0.0), (desc.Width as f32, desc.Height as f32)),
                            None,
                            mltg::Interpolation::HighQualityCubic,
                        );
                    })
                    .unwrap();
                dc.ClearRenderTargetView(&rtv, [0.0, 0.0, 0.3, 0.0].as_ptr());
                dc.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
                dc.IASetInputLayout(&input_layout);
                dc.IASetIndexBuffer(&index_buffer, DXGI_FORMAT_R32_UINT, 0);
                dc.IASetVertexBuffers(
                    0,
                    1,
                    Some([Some(vertex_buffer.clone())].as_mut_ptr()),
                    Some([std::mem::size_of::<Vertex>() as u32].as_ptr()),
                    Some([0].as_ptr()),
                );
                dc.OMSetRenderTargets(Some(&[Some(rtv.clone())]), None);
                dc.OMSetBlendState(&blend, None, u32::MAX);
                dc.VSSetShader(&vs, None);
                dc.PSSetShader(&ps, None);
                dc.PSSetShaderResources(0, Some(&[Some(tex_view.clone())]));
                dc.PSSetSamplers(0, Some(&[Some(sampler.clone())]));
                dc.RSSetViewports(Some(&[D3D11_VIEWPORT {
                    Width: window_size.width as f32,
                    Height: window_size.height as f32,
                    MaxDepth: 1.0,
                    ..Default::default()
                }]));
                dc.DrawIndexed(6, 0, 0);
                swap_chain.Present(1, 0).unwrap();
            },
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
