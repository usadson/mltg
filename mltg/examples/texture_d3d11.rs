use mltg_bindings::Windows::Win32::{
    Graphics::{Direct3D11::*, Dxgi::*},
    System::SystemServices::*,
    UI::WindowsAndMessaging::*,
};

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

struct Application {
    device: ID3D11Device,
    device_context: ID3D11DeviceContext,
    swap_chain: IDXGISwapChain1,
    rtv: Option<ID3D11RenderTargetView>,
    vertex_buffer: ID3D11Buffer,
    index_buffer: ID3D11Buffer,
    vs: ID3D11VertexShader,
    ps: ID3D11PixelShader,
    input_layout: ID3D11InputLayout,
    tex: ID3D11Texture2D,
    tex_view: ID3D11ShaderResourceView,
    sampler: ID3D11SamplerState,
    blend: ID3D11BlendState,
    context: mltg::Context<mltg::Direct3D11>,
    target: mltg::d3d11::RenderTarget,
    image: mltg::Image,
}

impl Application {
    fn new() -> anyhow::Result<Self> {
        let window = wita::WindowBuilder::new()
            .title("mltg offscreen d3d11")
            .build()?;
        let window_size = window.inner_size();
        let device = unsafe {
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
        let device_context = unsafe {
            let mut p = None;
            device.GetImmediateContext(&mut p);
            p.unwrap()
        };
        let dxgi_factory: IDXGIFactory2 = unsafe { CreateDXGIFactory1()? };
        let swap_chain = unsafe {
            let hwnd = HWND(window.raw_handle() as _);
            let mut p = None;
            dxgi_factory
                .CreateSwapChainForHwnd(
                    &device,
                    hwnd,
                    &DXGI_SWAP_CHAIN_DESC1 {
                        Width: window_size.width,
                        Height: window_size.height,
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
        let rtv = unsafe {
            let buffer: ID3D11Texture2D = swap_chain.GetBuffer(0)?;
            let mut p = None;
            device
                .CreateRenderTargetView(buffer, std::ptr::null(), &mut p)
                .and_some(p)?
        };
        let vertex_buffer = unsafe {
            const VERTICES: [Vertex; 4] = [
                Vertex::new([-0.5, 0.5, 0.0], [0.0, 0.0]),
                Vertex::new([0.5, 0.5, 0.0], [1.0, 0.0]),
                Vertex::new([-0.5, -0.5, 0.0], [0.0, 1.0]),
                Vertex::new([0.5, -0.5, 0.0], [1.0, 1.0]),
            ];
            let mut p = None;
            device
                .CreateBuffer(
                    &D3D11_BUFFER_DESC {
                        ByteWidth: std::mem::size_of::<[Vertex; 4]>() as _,
                        Usage: D3D11_USAGE_DEFAULT,
                        BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as _,
                        ..Default::default()
                    },
                    &D3D11_SUBRESOURCE_DATA {
                        pSysMem: VERTICES.as_ptr() as _,
                        ..Default::default()
                    },
                    &mut p,
                )
                .and_some(p)?
        };
        let index_buffer = unsafe {
            const INDICES: [u32; 6] = [0, 1, 2, 1, 3, 2];
            let mut p = None;
            device
                .CreateBuffer(
                    &D3D11_BUFFER_DESC {
                        ByteWidth: std::mem::size_of::<[u32; 6]>() as _,
                        Usage: D3D11_USAGE_DEFAULT,
                        BindFlags: D3D11_BIND_INDEX_BUFFER.0 as _,
                        ..Default::default()
                    },
                    &D3D11_SUBRESOURCE_DATA {
                        pSysMem: INDICES.as_ptr() as _,
                        ..Default::default()
                    },
                    &mut p,
                )
                .and_some(p)?
        };
        let (vs, ps, input_layout) = unsafe {
            let vs_blob = include_bytes!("d3d11_hlsl/tex.vs");
            let ps_blob = include_bytes!("d3d11_hlsl/tex.ps");
            let vs = {
                let mut p = None;
                device
                    .CreateVertexShader(vs_blob.as_ptr() as _, vs_blob.len() as _, None, &mut p)
                    .and_some(p)?
            };
            let ps = {
                let mut p = None;
                device
                    .CreatePixelShader(ps_blob.as_ptr() as _, ps_blob.len() as _, None, &mut p)
                    .and_some(p)?
            };
            let descs = [
                D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: PSTR(b"POSITION\0".as_ptr() as _),
                    SemanticIndex: 0,
                    Format: DXGI_FORMAT_R32G32B32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: 0,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                },
                D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: PSTR(b"TEXCOORD\0".as_ptr() as _),
                    SemanticIndex: 0,
                    Format: DXGI_FORMAT_R32G32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                },
            ];
            let input_layout = {
                let mut p = None;
                device
                    .CreateInputLayout(
                        descs.as_ptr(),
                        descs.len() as _,
                        vs_blob.as_ptr() as _,
                        vs_blob.len() as _,
                        &mut p,
                    )
                    .and_some(p)?
            };
            (vs, ps, input_layout)
        };
        let tex = unsafe {
            let mut p = None;
            device
                .CreateTexture2D(
                    &D3D11_TEXTURE2D_DESC {
                        Usage: D3D11_USAGE_DEFAULT,
                        BindFlags: (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as _,
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
                    std::ptr::null(),
                    &mut p,
                )
                .and_some(p)?
        };
        let tex_view = unsafe {
            let mut p = None;
            device
                .CreateShaderResourceView(&tex, std::ptr::null(), &mut p)
                .and_some(p)?
        };
        let sampler = unsafe {
            let mut p = None;
            device
                .CreateSamplerState(
                    &D3D11_SAMPLER_DESC {
                        Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR,
                        AddressU: D3D11_TEXTURE_ADDRESS_CLAMP,
                        AddressV: D3D11_TEXTURE_ADDRESS_CLAMP,
                        AddressW: D3D11_TEXTURE_ADDRESS_CLAMP,
                        MinLOD: f32::MIN,
                        MaxLOD: f32::MAX,
                        MaxAnisotropy: 1,
                        ..Default::default()
                    },
                    &mut p,
                )
                .and_some(p)?
        };
        let blend = unsafe {
            let rt = D3D11_RENDER_TARGET_BLEND_DESC {
                BlendEnable: TRUE,
                SrcBlend: D3D11_BLEND_SRC_ALPHA,
                DestBlend: D3D11_BLEND_INV_SRC_ALPHA,
                BlendOp: D3D11_BLEND_OP_ADD,
                SrcBlendAlpha: D3D11_BLEND_ONE,
                DestBlendAlpha: D3D11_BLEND_ZERO,
                BlendOpAlpha: D3D11_BLEND_OP_ADD,
                RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL.0 as _,
            };
            let mut p = None;
            device
                .CreateBlendState(
                    &D3D11_BLEND_DESC {
                        RenderTarget: [rt; 8],
                        ..Default::default()
                    },
                    &mut p,
                )
                .and_some(p)?
        };
        let context = mltg::Context::new(mltg::Direct3D11::new(&device)?)?;
        let target = context.create_render_target(&tex)?;
        let image = context.create_image("ferris.png")?;
        Ok(Self {
            device,
            device_context,
            swap_chain,
            rtv: Some(rtv),
            vertex_buffer,
            index_buffer,
            vs,
            ps,
            input_layout,
            tex,
            tex_view,
            sampler,
            blend,
            context,
            target,
            image,
        })
    }
}

impl wita::EventHandler for Application {
    fn draw(&mut self, window: &wita::Window) {
        let dc = &self.device_context;
        let window_size = window.inner_size();
        unsafe {
            self.context.draw(&self.target, |cmd| {
                let desc = {
                    let mut desc = D3D11_TEXTURE2D_DESC::default();
                    self.tex.GetDesc(&mut desc);
                    desc
                };
                cmd.clear([0.0, 0.0, 0.0, 0.0]);
                cmd.draw_image(
                    &self.image,
                    mltg::Rect::new((0.0, 0.0), (desc.Width as f32, desc.Height as f32)),
                    None,
                    mltg::Interpolation::HighQualityCubic,
                );
            });
            dc.ClearRenderTargetView(&self.rtv, [0.0, 0.0, 0.3, 0.0].as_ptr());
            dc.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            dc.IASetInputLayout(&self.input_layout);
            dc.IASetIndexBuffer(&self.index_buffer, DXGI_FORMAT_R32_UINT, 0);
            dc.IASetVertexBuffers(
                0,
                1,
                [Some(self.vertex_buffer.clone())].as_mut_ptr(),
                [std::mem::size_of::<Vertex>() as u32].as_ptr(),
                [0].as_ptr(),
            );
            dc.OMSetRenderTargets(1, [Some(self.rtv.clone().unwrap())].as_mut_ptr(), None);
            dc.OMSetBlendState(&self.blend, std::ptr::null(), u32::MAX);
            dc.VSSetShader(&self.vs, std::ptr::null_mut(), 0);
            dc.PSSetShader(&self.ps, std::ptr::null_mut(), 0);
            dc.PSSetShaderResources(0, 1, [Some(self.tex_view.clone())].as_mut_ptr());
            dc.PSSetSamplers(0, 1, [Some(self.sampler.clone())].as_mut_ptr());
            dc.RSSetViewports(
                1,
                [D3D11_VIEWPORT {
                    Width: window_size.width as f32,
                    Height: window_size.height as f32,
                    MaxDepth: 1.0,
                    ..Default::default()
                }]
                .as_ptr(),
            );
            dc.DrawIndexed(6, 0, 0);
            self.swap_chain.Present(1, 0).unwrap();
        }
    }

    fn resizing(&mut self, window: &wita::Window, size: wita::PhysicalSize<u32>) {
        unsafe {
            self.device_context.ClearState();
            self.rtv = None;
            self.swap_chain
                .ResizeBuffers(0, size.width, size.height, DXGI_FORMAT_UNKNOWN, 0)
                .unwrap();
            self.rtv = {
                let buffer: ID3D11Texture2D = self.swap_chain.GetBuffer(0).unwrap();
                let mut p = None;
                self.device
                    .CreateRenderTargetView(buffer, std::ptr::null(), &mut p)
                    .and_some(p)
                    .ok()
            };
            window.redraw();
        }
    }
}

fn main() {
    windows::initialize_sta().unwrap();
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
