use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use windows::core::Interface;
use windows::Win32::{
    Foundation::*,
    Graphics::{Direct3D::Dxc::*, Direct3D::*, Direct3D12::*, Dxgi::Common::*, Dxgi::*},
    System::{Threading::*, WindowsProgramming::*},
};
use winit::{dpi::*, event::*, event_loop::*, window::*};

#[repr(C)]
struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
}

impl Vertex {
    const fn new(position: [f32; 3], uv: [f32; 2]) -> Self {
        Self { position, uv }
    }
}

fn resource_barrier(
    command_list: &ID3D12GraphicsCommandList,
    resource: &ID3D12Resource,
    before: D3D12_RESOURCE_STATES,
    after: D3D12_RESOURCE_STATES,
) {
    unsafe {
        let mut barrier = [D3D12_RESOURCE_BARRIER {
            Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
            Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
            Anonymous: D3D12_RESOURCE_BARRIER_0 {
                Transition: std::mem::ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                    pResource: Some(resource.clone()),
                    Subresource: 0,
                    StateBefore: before,
                    StateAfter: after,
                }),
            },
        }];
        command_list.ResourceBarrier(&barrier);
        std::mem::ManuallyDrop::drop(&mut barrier[0].Anonymous.Transition);
    }
}

fn wait_gpu(command_queue: &ID3D12CommandQueue, fence: &ID3D12Fence, fence_value: &mut u64) {
    unsafe {
        let fv = *fence_value;
        command_queue.Signal(fence, fv).unwrap();
        if fence.GetCompletedValue() < fv {
            let event = CreateEventW(None, false, false, None).unwrap();
            fence.SetEventOnCompletion(fv, event).unwrap();
            WaitForSingleObject(event, INFINITE);
            CloseHandle(event);
        }
        *fence_value = fv + 1;
    }
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("mltg to_d3d12_texture")
        .with_inner_size(LogicalSize::new(640, 480))
        .build(&event_loop)?;
    let device = unsafe {
        let mut device: Option<ID3D12Device> = None;
        D3D12CreateDevice(None, D3D_FEATURE_LEVEL_12_0, &mut device).map(|_| device.unwrap())?
    };
    let command_queue: ID3D12CommandQueue = unsafe {
        device.CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC {
            Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
            ..Default::default()
        })?
    };
    let command_allocator: ID3D12CommandAllocator =
        unsafe { device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)? };
    let command_list = unsafe {
        let command_list: ID3D12GraphicsCommandList = device.CreateCommandList(
            0,
            D3D12_COMMAND_LIST_TYPE_DIRECT,
            &command_allocator,
            None,
        )?;
        command_list.SetName(windows::w!("command_list")).unwrap();
        command_list.Close().unwrap();
        command_list
    };
    let window_size = window.inner_size();
    let swap_chain: IDXGISwapChain4 = unsafe {
        let RawWindowHandle::Win32(window_handle) = window.raw_window_handle() else { panic!() };
        let dxgi_factory: IDXGIFactory4 = CreateDXGIFactory1()?;
        dxgi_factory
            .CreateSwapChainForHwnd(
                &command_queue,
                HWND(window_handle.hwnd as _),
                &DXGI_SWAP_CHAIN_DESC1 {
                    Width: window_size.width as _,
                    Height: window_size.height as _,
                    Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                    BufferCount: 2,
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
            .cast()?
    };
    let rtv_heap: ID3D12DescriptorHeap = unsafe {
        device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            NumDescriptors: 2,
            ..Default::default()
        })?
    };
    let rtv_descriptor_size =
        unsafe { device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV) as usize };
    let render_targets = unsafe {
        let mut handle = rtv_heap.GetCPUDescriptorHandleForHeapStart();
        let mut buffers = Vec::with_capacity(2);
        for i in 0..2 {
            let buffer: ID3D12Resource = swap_chain.GetBuffer(i as _)?;
            device.CreateRenderTargetView(&buffer, None, handle);
            buffers.push(buffer);
            handle.ptr += rtv_descriptor_size;
        }
        buffers
    };
    const VERTICES: [Vertex; 4] = [
        Vertex::new([-0.5, 0.5, 0.0], [0.0, 0.0]),
        Vertex::new([0.5, 0.5, 0.0], [1.0, 0.0]),
        Vertex::new([-0.5, -0.5, 0.0], [0.0, 1.0]),
        Vertex::new([0.5, -0.5, 0.0], [1.0, 1.0]),
    ];
    const INDICES: [u32; 6] = [0, 1, 2, 1, 3, 2];
    let vertex_buffer = unsafe {
        let mut vb: Option<ID3D12Resource> = None;
        let vb = device
            .CreateCommittedResource(
                &D3D12_HEAP_PROPERTIES {
                    Type: D3D12_HEAP_TYPE_UPLOAD,
                    CreationNodeMask: 1,
                    VisibleNodeMask: 1,
                    ..Default::default()
                },
                D3D12_HEAP_FLAG_NONE,
                &D3D12_RESOURCE_DESC {
                    Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                    Width: std::mem::size_of_val(&VERTICES) as _,
                    Height: 1,
                    DepthOrArraySize: 1,
                    MipLevels: 1,
                    Format: DXGI_FORMAT_UNKNOWN,
                    Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
                    SampleDesc: DXGI_SAMPLE_DESC {
                        Count: 1,
                        Quality: 0,
                    },
                    ..Default::default()
                },
                D3D12_RESOURCE_STATE_GENERIC_READ,
                None,
                &mut vb,
            )
            .map(|_| vb.unwrap())?;
        let mut p = std::ptr::null_mut();
        vb.Map(0, None, Some(&mut p)).unwrap();
        std::ptr::copy_nonoverlapping(&VERTICES, p as _, VERTICES.len());
        vb.Unmap(0, None);
        vb
    };
    let vbv = D3D12_VERTEX_BUFFER_VIEW {
        BufferLocation: unsafe { vertex_buffer.GetGPUVirtualAddress() },
        StrideInBytes: std::mem::size_of::<Vertex>() as _,
        SizeInBytes: std::mem::size_of_val(&VERTICES) as _,
    };
    let index_buffer = unsafe {
        let mut ib: Option<ID3D12Resource> = None;
        let ib = device
            .CreateCommittedResource(
                &D3D12_HEAP_PROPERTIES {
                    Type: D3D12_HEAP_TYPE_UPLOAD,
                    CreationNodeMask: 1,
                    VisibleNodeMask: 1,
                    ..Default::default()
                },
                D3D12_HEAP_FLAG_NONE,
                &D3D12_RESOURCE_DESC {
                    Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                    Width: std::mem::size_of_val(&INDICES) as _,
                    Height: 1,
                    DepthOrArraySize: 1,
                    MipLevels: 1,
                    Format: DXGI_FORMAT_UNKNOWN,
                    Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
                    SampleDesc: DXGI_SAMPLE_DESC {
                        Count: 1,
                        Quality: 0,
                    },
                    ..Default::default()
                },
                D3D12_RESOURCE_STATE_GENERIC_READ,
                None,
                &mut ib,
            )
            .map(|_| ib.unwrap())?;
        let mut p = std::ptr::null_mut();
        ib.Map(0, None, Some(&mut p)).unwrap();
        std::ptr::copy_nonoverlapping(&INDICES, p as _, INDICES.len());
        ib.Unmap(0, None);
        ib
    };
    let ibv = D3D12_INDEX_BUFFER_VIEW {
        BufferLocation: unsafe { index_buffer.GetGPUVirtualAddress() },
        SizeInBytes: std::mem::size_of_val(&INDICES) as _,
        Format: DXGI_FORMAT_R32_UINT,
    };
    let tex = unsafe {
        let mut tex: Option<ID3D12Resource> = None;
        let tex = device
            .CreateCommittedResource(
                &D3D12_HEAP_PROPERTIES {
                    Type: D3D12_HEAP_TYPE_DEFAULT,
                    CreationNodeMask: 1,
                    VisibleNodeMask: 1,
                    ..Default::default()
                },
                D3D12_HEAP_FLAG_NONE,
                &D3D12_RESOURCE_DESC {
                    Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
                    Width: window_size.width as _,
                    Height: window_size.height,
                    DepthOrArraySize: 1,
                    MipLevels: 1,
                    Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                    Flags: D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET
                        | D3D12_RESOURCE_FLAG_ALLOW_SIMULTANEOUS_ACCESS,
                    SampleDesc: DXGI_SAMPLE_DESC {
                        Count: 1,
                        Quality: 0,
                    },
                    ..Default::default()
                },
                D3D12_RESOURCE_STATE_COMMON,
                Some(&D3D12_CLEAR_VALUE {
                    Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                    Anonymous: D3D12_CLEAR_VALUE_0 {
                        Color: [0.0, 0.5, 0.0, 0.5],
                    },
                }),
                &mut tex,
            )
            .map(|_| tex.unwrap())?;
        tex.SetName(windows::w!("tex")).unwrap();
        tex
    };
    let srv_heap = unsafe {
        let srv_heap: ID3D12DescriptorHeap =
            device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
                Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
                NumDescriptors: 1,
                ..Default::default()
            })?;
        device.CreateShaderResourceView(
            &tex,
            Some(&D3D12_SHADER_RESOURCE_VIEW_DESC {
                ViewDimension: D3D12_SRV_DIMENSION_TEXTURE2D,
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
                Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                    Texture2D: D3D12_TEX2D_SRV {
                        MipLevels: 1,
                        ..Default::default()
                    },
                },
            }),
            srv_heap.GetCPUDescriptorHandleForHeapStart(),
        );
        srv_heap
    };
    let root_signature: ID3D12RootSignature = unsafe {
        let ranges = [D3D12_DESCRIPTOR_RANGE {
            RangeType: D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
            NumDescriptors: 1,
            OffsetInDescriptorsFromTableStart: D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND,
            ..Default::default()
        }];
        let params = [D3D12_ROOT_PARAMETER {
            ParameterType: D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
            ShaderVisibility: D3D12_SHADER_VISIBILITY_PIXEL,
            Anonymous: D3D12_ROOT_PARAMETER_0 {
                DescriptorTable: D3D12_ROOT_DESCRIPTOR_TABLE {
                    NumDescriptorRanges: ranges.len() as _,
                    pDescriptorRanges: ranges.as_ptr() as _,
                },
            },
        }];
        let sampler = [D3D12_STATIC_SAMPLER_DESC {
            Filter: D3D12_FILTER_MIN_MAG_MIP_LINEAR,
            AddressU: D3D12_TEXTURE_ADDRESS_MODE_CLAMP,
            AddressV: D3D12_TEXTURE_ADDRESS_MODE_CLAMP,
            AddressW: D3D12_TEXTURE_ADDRESS_MODE_CLAMP,
            MinLOD: 0.0,
            MaxLOD: f32::MAX,
            ShaderVisibility: D3D12_SHADER_VISIBILITY_PIXEL,
            ..Default::default()
        }];
        let root = D3D12_ROOT_SIGNATURE_DESC {
            NumParameters: params.len() as _,
            pParameters: params.as_ptr() as _,
            NumStaticSamplers: sampler.len() as _,
            pStaticSamplers: sampler.as_ptr() as _,
            Flags: D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
        };
        let blob = {
            let mut p = None;
            D3D12SerializeRootSignature(&root, D3D_ROOT_SIGNATURE_VERSION_1, &mut p, None)
                .map(|_| p.unwrap())?
        };
        device.CreateRootSignature(
            0,
            std::slice::from_raw_parts(blob.GetBufferPointer() as *const u8, blob.GetBufferSize()),
        )?
    };
    let pipeline: ID3D12PipelineState = unsafe {
        // need dxcompiler.dll and dxil.dll
        let lib: IDxcLibrary = DxcCreateInstance(&CLSID_DxcLibrary)?;
        let compiler: IDxcCompiler = DxcCreateInstance(&CLSID_DxcCompiler)?;
        let blob =
            lib.CreateBlobFromFile(windows::w!("./resources/tex.hlsl"), Some(&DXC_CP_UTF8))?;
        let vs_blob = {
            let ret = compiler.Compile(
                &blob,
                windows::w!("tex.hlsl"),
                windows::w!("vs_main"),
                windows::w!("vs_6_0"),
                None,
                &[],
                None,
            )?;
            ret.GetResult()?
        };
        let ps_blob = {
            let ret = compiler.Compile(
                &blob,
                windows::w!("tex.hlsl"),
                windows::w!("ps_main"),
                windows::w!("ps_6_0"),
                None,
                &[],
                None,
            )?;
            ret.GetResult()?
        };
        let input_layout = [
            D3D12_INPUT_ELEMENT_DESC {
                SemanticName: windows::s!("POSITION"),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32B32_FLOAT,
                AlignedByteOffset: 0,
                InputSlot: 0,
                InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
            D3D12_INPUT_ELEMENT_DESC {
                SemanticName: windows::s!("TEXCOORD"),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32_FLOAT,
                AlignedByteOffset: D3D12_APPEND_ALIGNED_ELEMENT,
                InputSlot: 0,
                InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
        ];
        let render_target_blend = D3D12_RENDER_TARGET_BLEND_DESC {
            BlendEnable: true.into(),
            LogicOpEnable: false.into(),
            SrcBlend: D3D12_BLEND_SRC_ALPHA,
            DestBlend: D3D12_BLEND_INV_SRC_ALPHA,
            BlendOp: D3D12_BLEND_OP_ADD,
            SrcBlendAlpha: D3D12_BLEND_ONE,
            DestBlendAlpha: D3D12_BLEND_ZERO,
            BlendOpAlpha: D3D12_BLEND_OP_ADD,
            LogicOp: D3D12_LOGIC_OP_NOOP,
            RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL.0 as _,
        };
        let mut rtv_formats = [DXGI_FORMAT_UNKNOWN; 8];
        rtv_formats[0] = DXGI_FORMAT_R8G8B8A8_UNORM;
        let desc = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
            pRootSignature: Some(root_signature.clone()),
            VS: D3D12_SHADER_BYTECODE {
                pShaderBytecode: vs_blob.GetBufferPointer(),
                BytecodeLength: vs_blob.GetBufferSize() as _,
            },
            PS: D3D12_SHADER_BYTECODE {
                pShaderBytecode: ps_blob.GetBufferPointer() as _,
                BytecodeLength: ps_blob.GetBufferSize() as _,
            },
            InputLayout: D3D12_INPUT_LAYOUT_DESC {
                pInputElementDescs: input_layout.as_ptr() as _,
                NumElements: input_layout.len() as _,
            },
            BlendState: D3D12_BLEND_DESC {
                AlphaToCoverageEnable: false.into(),
                IndependentBlendEnable: false.into(),
                RenderTarget: [render_target_blend; 8],
            },
            RasterizerState: D3D12_RASTERIZER_DESC {
                FillMode: D3D12_FILL_MODE_SOLID,
                CullMode: D3D12_CULL_MODE_NONE,
                ..Default::default()
            },
            SampleMask: u32::MAX,
            PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
            NumRenderTargets: 1,
            RTVFormats: rtv_formats,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            ..Default::default()
        };
        device.CreateGraphicsPipelineState(&desc)?
    };
    let fence: ID3D12Fence = unsafe { device.CreateFence(0, D3D12_FENCE_FLAG_NONE)? };
    let mut fence_value = 0u64;
    let context = mltg::Context::new(mltg::Direct3D12::new(&device, &command_queue, 0)?)?;
    let factory = context.create_factory();
    let target = context.create_render_target(&tex)?;
    let image = factory.create_image_from_file("./resources/ferris.png")?;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        let window_size = window.inner_size();
        match event {
            Event::RedrawRequested(_) => unsafe {
                context
                    .draw(&target, |cmd| {
                        let desc = tex.GetDesc();
                        cmd.clear([0.0, 0.0, 0.0, 0.0]);
                        cmd.draw_image(
                            &image,
                            mltg::Rect::new((0.0, 0.0), (desc.Width as f32, desc.Height as f32)),
                            None,
                            mltg::Interpolation::HighQualityCubic,
                        );
                    })
                    .unwrap();
                let index = swap_chain.GetCurrentBackBufferIndex() as usize;
                let rtv_handle = {
                    let mut handle = rtv_heap.GetCPUDescriptorHandleForHeapStart();
                    handle.ptr += rtv_descriptor_size * index;
                    handle
                };
                command_allocator.Reset().unwrap();
                command_list.Reset(&command_allocator, &pipeline).unwrap();
                command_list.SetDescriptorHeaps(&[Some(srv_heap.clone())]);
                command_list.SetGraphicsRootSignature(&root_signature);
                command_list.SetGraphicsRootDescriptorTable(
                    0,
                    srv_heap.GetGPUDescriptorHandleForHeapStart(),
                );
                command_list.RSSetViewports(&[D3D12_VIEWPORT {
                    Width: window_size.width as _,
                    Height: window_size.height as _,
                    MaxDepth: 1.0,
                    ..Default::default()
                }]);
                command_list.RSSetScissorRects(&[RECT {
                    right: window_size.width as _,
                    bottom: window_size.height as _,
                    ..Default::default()
                }]);
                resource_barrier(
                    &command_list,
                    &render_targets[index],
                    D3D12_RESOURCE_STATE_PRESENT,
                    D3D12_RESOURCE_STATE_RENDER_TARGET,
                );
                resource_barrier(
                    &command_list,
                    &tex,
                    D3D12_RESOURCE_STATE_COMMON,
                    D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
                );
                command_list.ClearRenderTargetView(
                    rtv_handle,
                    &*[0.0, 0.0, 0.3, 0.0].as_ptr(),
                    &[],
                );
                command_list.OMSetRenderTargets(
                    1,
                    Some(&*[rtv_handle.clone()].as_ptr()),
                    false,
                    None,
                );
                command_list.IASetVertexBuffers(0, Some(&[vbv.clone()]));
                command_list.IASetIndexBuffer(Some(&ibv));
                command_list.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
                command_list.DrawIndexedInstanced(6, 1, 0, 0, 0);
                resource_barrier(
                    &command_list,
                    &tex,
                    D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
                    D3D12_RESOURCE_STATE_COMMON,
                );
                resource_barrier(
                    &command_list,
                    &render_targets[index],
                    D3D12_RESOURCE_STATE_RENDER_TARGET,
                    D3D12_RESOURCE_STATE_PRESENT,
                );
                command_list.Close().unwrap();
                let command_lists = [Some(command_list.clone().cast().unwrap())];
                command_queue.ExecuteCommandLists(&command_lists);
                swap_chain.Present(1, 0).unwrap();
                wait_gpu(&command_queue, &fence, &mut fence_value);
            },
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                wait_gpu(&command_queue, &fence, &mut fence_value);
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
