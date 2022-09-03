use std::cell::Cell;
use windows::core::Interface;
use windows::Win32::{
    Foundation::*,
    Graphics::{Direct3D::*, Direct3D12::*, Dxgi::Common::*, Dxgi::*},
    System::{Threading::*, WindowsProgramming::*},
};

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

struct Application {
    d3d12_device: ID3D12Device,
    command_queue: ID3D12CommandQueue,
    command_allocator: ID3D12CommandAllocator,
    command_list: ID3D12GraphicsCommandList,
    swap_chain: IDXGISwapChain4,
    rtv_heap: ID3D12DescriptorHeap,
    rtv_descriptor_size: usize,
    render_targets: Vec<ID3D12Resource>,
    fence: ID3D12Fence,
    fence_value: Cell<u64>,
    context: mltg::Context<mltg::Direct3D12>,
    factory: mltg::Factory,
    bitmaps: Vec<mltg::RenderTarget<mltg::Direct3D12>>,
    text: mltg::TextLayout,
    white_brush: mltg::Brush,
    grad: mltg::GradientStopCollection,
    image: mltg::Image,
}

impl Application {
    fn new() -> anyhow::Result<Self> {
        unsafe {
            let window = wita::WindowBuilder::new().title("mltg d3d12").build()?;
            let window_size = window.inner_size();
            let d3d12_device = {
                let mut device: Option<ID3D12Device> = None;
                D3D12CreateDevice(None, D3D_FEATURE_LEVEL_12_0, &mut device)
                    .map(|_| device.unwrap())?
            };
            let command_queue: ID3D12CommandQueue =
                d3d12_device.CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC {
                    Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                    ..Default::default()
                })?;
            let command_allocator: ID3D12CommandAllocator =
                d3d12_device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)?;
            let command_list: ID3D12GraphicsCommandList = d3d12_device.CreateCommandList(
                0,
                D3D12_COMMAND_LIST_TYPE_DIRECT,
                &command_allocator,
                None,
            )?;
            command_list.Close().unwrap();
            let dxgi_factory: IDXGIFactory4 = CreateDXGIFactory1()?;
            let swap_chain: IDXGISwapChain4 = {
                dxgi_factory
                    .CreateSwapChainForHwnd(
                        &command_queue,
                        HWND(window.raw_handle() as _),
                        &DXGI_SWAP_CHAIN_DESC1 {
                            Width: window_size.width as _,
                            Height: window_size.height as _,
                            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                            BufferCount: 2,
                            SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                            Scaling: DXGI_SCALING_NONE,
                            SampleDesc: DXGI_SAMPLE_DESC {
                                Count: 1,
                                Quality: 0,
                            },
                            ..Default::default()
                        },
                        std::ptr::null(),
                        None,
                    )?
                    .cast()?
            };
            let rtv_heap: ID3D12DescriptorHeap =
                d3d12_device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                    Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                    NumDescriptors: 2,
                    ..Default::default()
                })?;
            let rtv_descriptor_size = d3d12_device
                .GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV)
                as usize;
            let render_targets = {
                let mut handle = rtv_heap.GetCPUDescriptorHandleForHeapStart();
                let mut buffers = vec![];
                for i in 0..2 {
                    let buffer: ID3D12Resource = swap_chain.GetBuffer(i as _)?;
                    d3d12_device.CreateRenderTargetView(&buffer, std::ptr::null(), handle);
                    buffers.push(buffer);
                    handle.ptr += rtv_descriptor_size;
                }
                buffers
            };
            let fence = d3d12_device.CreateFence(0, D3D12_FENCE_FLAG_NONE)?;
            let context =
                mltg::Context::new(mltg::Direct3D12::new(&d3d12_device, &command_queue)?)?;
            let factory = context.create_factory();
            let bitmaps = context.create_back_buffers(&swap_chain)?;
            let text_format = factory.create_text_format(
                mltg::Font::System("Meiryo"),
                mltg::font_point(14.0),
                None,
            )?;
            let text = factory.create_text_layout(
                "abcdefghijklmnopqrstuvwxyz",
                &text_format,
                mltg::TextAlignment::Leading,
                None,
            )?;
            let white_brush = factory.create_solid_color_brush([1.0, 1.0, 1.0, 1.0])?;
            let grad = factory.create_gradient_stop_collection(&[
                (0.0, [1.0, 0.0, 0.0, 1.0]),
                (1.0, [0.0, 1.0, 0.0, 1.0]),
            ])?;
            let image = factory.create_image("examples/ferris.png")?;
            context.set_dpi(window.dpi() as _);
            Ok(Self {
                d3d12_device,
                command_queue,
                command_allocator,
                command_list,
                swap_chain,
                rtv_heap,
                rtv_descriptor_size,
                render_targets,
                fence,
                fence_value: Cell::new(1),
                context,
                factory,
                bitmaps,
                text,
                white_brush,
                grad,
                image,
            })
        }
    }

    fn wait_gpu(&self) {
        unsafe {
            let fv = self.fence_value.get();
            self.command_queue.Signal(&self.fence, fv).unwrap();
            if self.fence.GetCompletedValue() < fv {
                let event = CreateEventW(std::ptr::null_mut(), false, false, None).unwrap();
                self.fence.SetEventOnCompletion(fv, event).unwrap();
                WaitForSingleObject(event, INFINITE);
                CloseHandle(event);
            }
            self.fence_value.set(fv + 1);
        }
    }
}

impl wita::EventHandler for Application {
    fn draw(&mut self, ev: wita::event::Draw) {
        const CLEAR_COLOR: [f32; 4] = [0.0, 0.0, 0.3, 0.0];
        let window_size = ev.window.inner_size().to_logical(ev.window.dpi());
        let hw = window_size.width as f32 / 2.0;
        let hh = window_size.height as f32 / 2.0;
        let rect = {
            let margin = 30.0;
            let pos = mltg::point(margin, margin);
            let pos2 = pos.x * 2.0;
            mltg::rect(pos, (hw - pos2, hh - pos2))
        };
        let text_box = mltg::rect((hw, hh), self.text.size());
        let path = self
            .factory
            .create_path()
            .begin((30.0, hh + 30.0))
            .cubic_bezier_to(
                (hw / 2.0, hh + 30.0),
                (hw / 2.0, window_size.height as f32 - 30.0),
                (hw - 30.0, window_size.height as f32 - 30.0),
            )
            .end(mltg::FigureEnd::Open)
            .build();
        let linear_grad_brush = self
            .factory
            .create_linear_gradient_brush((30.0, 30.0), (hw - 30.0, hh - 30.0), &self.grad)
            .unwrap();
        let image_size = {
            let size = self.image.size();
            (size.width as f32 / 4.0, size.height as f32 / 4.0)
        };
        unsafe {
            let index = self.swap_chain.GetCurrentBackBufferIndex() as usize;
            let mut rtv_handle = self.rtv_heap.GetCPUDescriptorHandleForHeapStart();
            rtv_handle.ptr += self.rtv_descriptor_size * index;
            self.command_allocator.Reset().unwrap();
            self.command_list
                .Reset(&self.command_allocator, None)
                .unwrap();
            resource_barrier(
                &self.command_list,
                &self.render_targets[index],
                D3D12_RESOURCE_STATE_PRESENT,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
            );
            let viewports = [D3D12_VIEWPORT {
                Width: window_size.width as _,
                Height: window_size.height as _,
                MaxDepth: 1.0,
                ..Default::default()
            }];
            self.command_list.RSSetViewports(&viewports);
            let scissor_rect = [RECT {
                right: window_size.width as _,
                bottom: window_size.height as _,
                ..Default::default()
            }];
            self.command_list.RSSetScissorRects(&scissor_rect);
            self.command_list
                .ClearRenderTargetView(rtv_handle, CLEAR_COLOR.as_ptr(), &[]);
            self.command_list.Close().unwrap();
            let command_lists = [Some(self.command_list.cast::<ID3D12CommandList>().unwrap())];
            self.command_queue.ExecuteCommandLists(&command_lists);
            let ret = self.context.draw(&self.bitmaps[index], |cmd| {
                cmd.fill(&rect, &linear_grad_brush);
                cmd.stroke(&text_box, &self.white_brush, 2.0, None);
                cmd.draw_text_layout(&self.text, &self.white_brush, (hw, hh));
                cmd.draw_image(
                    &self.image,
                    mltg::Rect::new((hw, 10.0), image_size),
                    None,
                    mltg::Interpolation::HighQualityCubic,
                );
                cmd.stroke(&path, &self.white_brush, 5.0, None);
            });
            match ret {
                Ok(_) => {
                    self.swap_chain.Present(1, 0).unwrap();
                }
                Err(e) if e == mltg::ErrorKind::RecreateTarget => {
                    self.bitmaps.clear();
                    self.context.flush();
                    self.bitmaps = self.context.create_back_buffers(&self.swap_chain).unwrap();
                    ev.window.redraw();
                }
                Err(e) => panic!("{:?}", e),
            }
        }
        self.wait_gpu();
    }

    fn dpi_changed(&mut self, ev: wita::event::DpiChanged) {
        self.context.set_dpi(ev.new_dpi as _);
        self.resizing(wita::event::Resizing {
            window: ev.window,
            size: &mut ev.window.inner_size(),
            edge: wita::ResizingEdge::Right,
        });
    }

    fn resizing(&mut self, ev: wita::event::Resizing) {
        self.bitmaps.clear();
        self.render_targets.clear();
        self.context.flush();
        unsafe {
            self.swap_chain
                .ResizeBuffers(0, ev.size.width, ev.size.height, DXGI_FORMAT_UNKNOWN, 0)
                .unwrap();
        }
        self.render_targets = unsafe {
            let mut handle = self.rtv_heap.GetCPUDescriptorHandleForHeapStart();
            let mut buffers = vec![];
            for i in 0..2 {
                let buffer: ID3D12Resource = self.swap_chain.GetBuffer(i as _).unwrap();
                self.d3d12_device
                    .CreateRenderTargetView(&buffer, std::ptr::null(), handle);
                buffers.push(buffer);
                handle.ptr += self.rtv_descriptor_size;
            }
            buffers
        };
        self.bitmaps = unsafe { self.context.create_back_buffers(&self.swap_chain).unwrap() };
        ev.window.redraw();
    }
}

fn main() {
    let _coinit = coinit::init(coinit::APARTMENTTHREADED | coinit::DISABLE_OLE1DDE).unwrap();
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
