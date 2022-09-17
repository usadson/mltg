use windows::core::PCSTR;
use windows::Win32::{
    Foundation::*,
    Graphics::{Direct3D::*, Direct3D11::*, Dxgi::Common::*, Dxgi::*},
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
    target: mltg::RenderTarget<mltg::Direct3D11>,
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
                HINSTANCE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                Some(&FEATURE_LEVELS),
                D3D11_SDK_VERSION,
                Some(&mut p),
                None,
                None,
            )
            .map(|_| p.unwrap())?
        };
        let device_context = unsafe {
            let mut p = None;
            device.GetImmediateContext(&mut p);
            p.unwrap()
        };
        let dxgi_factory: IDXGIFactory2 = unsafe { CreateDXGIFactory1()? };
        let swap_chain = unsafe {
            let hwnd = HWND(window.raw_handle() as _);
            dxgi_factory.CreateSwapChainForHwnd(
                &device,
                hwnd,
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
            let buffer: ID3D11Texture2D = swap_chain.GetBuffer(0)?;
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
            let vs_blob = include_bytes!("d3d11_hlsl/tex.vs");
            let ps_blob = include_bytes!("d3d11_hlsl/tex.ps");
            let vs = device.CreateVertexShader(vs_blob, None)?;
            let ps = device.CreatePixelShader(ps_blob, None)?;
            let descs = [
                D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: PCSTR(b"POSITION\0".as_ptr() as _),
                    SemanticIndex: 0,
                    Format: DXGI_FORMAT_R32G32B32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: 0,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                },
                D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: PCSTR(b"TEXCOORD\0".as_ptr() as _),
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
        let context = mltg::Context::new(unsafe { mltg::Direct3D11::new(&device)? })?;
        let factory = context.create_factory();
        let target = unsafe { context.create_render_target(&tex)? };
        let image = factory.create_image("examples/ferris.png")?;
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
    fn draw(&mut self, ev: wita::event::Draw) {
        let dc = &self.device_context;
        let window_size = ev.window.inner_size();
        unsafe {
            let ret = self.context.draw(&self.target, |cmd| {
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
            match ret {
                Ok(_) => {}
                Err(e) if e == mltg::ErrorKind::RecreateTarget => {
                    self.target = self.context.create_render_target(&self.tex).unwrap();
                    ev.window.redraw();
                    return;
                }
                Err(e) => panic!("{:?}", e),
            }
            dc.ClearRenderTargetView(self.rtv.as_ref(), &*[0.0, 0.0, 0.3, 0.0].as_ptr());
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
            dc.OMSetRenderTargets(Some(&[Some(self.rtv.clone().unwrap())]), None);
            dc.OMSetBlendState(&self.blend, None, u32::MAX);
            dc.VSSetShader(&self.vs, None);
            dc.PSSetShader(&self.ps, None);
            dc.PSSetShaderResources(0, Some(&[Some(self.tex_view.clone())]));
            dc.PSSetSamplers(0, Some(&[Some(self.sampler.clone())]));
            dc.RSSetViewports(Some(&[D3D11_VIEWPORT {
                Width: window_size.width as f32,
                Height: window_size.height as f32,
                MaxDepth: 1.0,
                ..Default::default()
            }]));
            dc.DrawIndexed(6, 0, 0);
            self.swap_chain.Present(1, 0).unwrap();
        }
    }

    fn resizing(&mut self, ev: wita::event::Resizing) {
        unsafe {
            self.device_context.ClearState();
            self.rtv = None;
            self.swap_chain
                .ResizeBuffers(0, ev.size.width, ev.size.height, DXGI_FORMAT_UNKNOWN, 0)
                .unwrap();
            self.rtv = {
                let buffer: ID3D11Texture2D = self.swap_chain.GetBuffer(0).unwrap();
                self.device.CreateRenderTargetView(&buffer, None).ok()
            };
            ev.window.redraw();
        }
    }
}

fn main() {
    let _coinit = coinit::init(coinit::APARTMENTTHREADED | coinit::DISABLE_OLE1DDE).unwrap();
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
