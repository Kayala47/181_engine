// Based on the Vulkano triangle example.

// Triangle example Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess};
use vulkano::command_buffer::pool::standard::StandardCommandPoolAlloc;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents};
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{Device, DeviceExtensions, Features};
use vulkano::format::Format;
use vulkano::image::ImageCreateFlags;
use vulkano::image::{
    view::ImageView, ImageAccess, ImageDimensions, ImageUsage, StorageImage, SwapchainImage,
};
use vulkano::instance::Instance;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};
use vulkano::render_pass::{Framebuffer, RenderPass, Subpass};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use vulkano::shader::ShaderModule;
use vulkano::swapchain::{self, AcquireError, Swapchain, SwapchainCreationError};
use vulkano::sync::{self, FlushError, GpuFuture};
use vulkano::Version;
use vulkano_win::VkSurfaceBuild;
pub use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

// We'll make our Color type an RGBA8888 pixel.
pub type Color = (u8, u8, u8, u8);

const WIDTH: usize = 800;
const HEIGHT: usize = 400;

#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position, uv);

pub struct State {
    pub fb2d: [(u8, u8, u8, u8); WIDTH * HEIGHT],
    pub drawables: Vec<Drawable>,
    pub bg_color: Color,
    previous_frame_end: std::option::Option<std::boxed::Box<dyn vulkano::sync::GpuFuture>>,
    recreate_swapchain: bool,
    pub event_loop: EventLoop<()>,
    fb2d_buffer: Arc<vulkano::buffer::CpuAccessibleBuffer<[(u8, u8, u8, u8)]>>,
    now_keys: [bool; 255],
    prev_keys: [bool; 255],
    fb2d_image: std::sync::Arc<vulkano::image::StorageImage>,
    queue: std::sync::Arc<vulkano::device::Queue>,
    swapchain: std::sync::Arc<vulkano::swapchain::Swapchain<winit::window::Window>>,
    viewport: Viewport,
    framebuffers: Vec<Arc<Framebuffer>>,
    pipeline: Arc<GraphicsPipeline>,
    render_pass: Arc<RenderPass>,
    dimensions: ImageDimensions,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,
    surface: std::sync::Arc<vulkano::swapchain::Surface<winit::window::Window>>,
    device: Arc<vulkano::device::Device>,
    set: std::sync::Arc<vulkano::descriptor_set::PersistentDescriptorSet>,
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
}

// impl Clone for State {
//     fn clone(&self) -> State {
//         let prv_frame = self.previous_frame_end.take();
//         State {
//             fb2d: self.fb2d,
//             drawables: vec![],
//             bg_color: (255, 255, 255, 255),
//             previous_frame_end: prv_frame,
//             recreate_swapchain: self.recreate_swapchain.clone(),
//             event_loop: EventLoop::new(),
//             fb2d_buffer: self.fb2d_buffer.clone(),
//             now_keys: [false; 255],
//             prev_keys: [false; 255],
//             fb2d_image: self.fb2d_image.clone(),
//             queue: self.queue.clone(),
//             swapchain: self.swapchain.clone(),
//             viewport: self.viewport.clone(),
//             framebuffers: self.framebuffers.clone(),
//             pipeline: self.pipeline.clone(),
//             render_pass: self.render_pass.clone(),
//             dimensions: self.dimensions.clone(),
//             vs: self.vs.clone(),
//             fs: self.fs.clone(),
//             surface: self.surface.clone(),
//             device: self.device.clone(),
//             set: self.set.clone(),
//             vertex_buffer: self.vertex_buffer.clone(),
//         }
//     }
// }

#[derive(Copy, Clone)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

#[derive(Clone)]
pub enum Drawable {
    Rectangle(Rect, Color),
    RectOutlined(Rect, Color),
}

impl Rect {
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Rect {
        Rect { x, y, w, h }
    }
}

fn draw_objects(fb: &mut [Color], drawables: Vec<Drawable>) {
    for obj in drawables {
        match obj {
            Drawable::Rectangle(r, c) => {
                rectangle(fb, r, c);
            }
            Drawable::RectOutlined(r, c) => {
                rect_outlined(fb, r, c);
            }
        }
    }
}

// Here's what clear looks like, though we won't use it
#[allow(dead_code)]
pub fn clear(fb: &mut [Color], c: Color) {
    fb.fill(c);
}

#[allow(dead_code)]
fn line(fb: &mut [Color], x0: usize, x1: usize, y: usize, c: Color) {
    assert!(y < HEIGHT);
    assert!(x0 <= x1);
    assert!(x1 < WIDTH);
    fb[y * WIDTH + x0..(y * WIDTH + x1)].fill(c);
}

#[allow(dead_code)]
fn rectangle(fb: &mut [Color], r: Rect, c: Color) {
    assert!(r.w + r.x <= WIDTH);
    assert!(r.h + r.y <= HEIGHT);

    for i in (r.y)..(r.y + r.h) {
        line(fb, r.x, r.x + r.w, i, c);
    }
}

#[allow(dead_code)]
fn rect_outlined(fb: &mut [Color], r: Rect, c: Color) {
    let t = 1;
    let x = r.x;
    let y = r.y;
    let h = r.h;
    let w = r.w;
    (y..(y + (t))).for_each(|y1| line(fb, x, x + w, y1, c));
    ((y + h - t)..(y + h)).for_each(|y1| line(fb, x, x + w, y1, c));

    ((y + t)..(y + h - (t))).for_each(|y1| {
        line(fb, x, x + t, y1, c);
        line(fb, x + w - t, x + w, y1, c);
    });
}

#[allow(dead_code)]
fn point(fb: &mut [Color], x: usize, y: usize, c: Color) {
    assert!(y < HEIGHT);
    assert!(x < WIDTH);
    fb[y * WIDTH + x] = c;
}

#[allow(dead_code)]
fn line_bresenham(
    fb: &mut [Color],
    (x0, y0): (usize, usize),
    (x1, y1): (usize, usize),
    col: Color,
) {
    let mut x = x0 as i64;
    let mut y = y0 as i64;
    let x0 = x0 as i64;
    let y0 = y0 as i64;
    let x1 = x1 as i64;
    let y1 = y1 as i64;
    let dx = (x1 - x0).abs();
    let sx: i64 = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy: i64 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    while x != x1 || y != y1 {
        fb[(y as usize * WIDTH + x as usize)..(y as usize * WIDTH + (x as usize + 1))].fill(col);
        let e2 = 2 * err;
        if dy <= e2 {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
) -> Vec<Arc<Framebuffer>> {
    let dimensions = images[0].dimensions().width_height();
    viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];

    images
        .iter()
        .map(|image| {
            let view = ImageView::new(image.clone()).unwrap();
            Framebuffer::start(render_pass.clone())
                .add(view)
                .unwrap()
                .build()
                .unwrap()
        })
        .collect::<Vec<_>>()
}

pub fn setup() -> State {
    let required_extensions = vulkano_win::required_extensions();
    let instance = Instance::new(None, Version::V1_1, &required_extensions, None).unwrap();
    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())
        .unwrap();
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    };
    let (physical_device, queue_family) = PhysicalDevice::enumerate(&instance)
        .filter(|&p| p.supported_extensions().is_superset_of(&device_extensions))
        .filter_map(|p| {
            p.queue_families()
                .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
                .map(|q| (p, q))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            PhysicalDeviceType::Other => 4,
        })
        .unwrap();
    let (device, mut queues) = Device::new(
        physical_device,
        &Features::none(),
        &physical_device
            .required_extensions()
            .union(&device_extensions),
        [(queue_family, 0.5)].iter().cloned(),
    )
    .unwrap();
    let queue = queues.next().unwrap();
    let (mut swapchain, images) = {
        let caps = surface.capabilities(physical_device).unwrap();
        let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;
        let dimensions: [u32; 2] = surface.window().inner_size().into();
        Swapchain::start(device.clone(), surface.clone())
            .num_images(caps.min_image_count)
            .format(format)
            .dimensions(dimensions)
            .usage(ImageUsage::color_attachment())
            .sharing_mode(&queue)
            .composite_alpha(composite_alpha)
            .build()
            .unwrap()
    };

    // We now create a buffer that will store the shape of our triangl

    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        false,
        [
            Vertex {
                position: [-1.0, -1.0],
                uv: [0.0, 0.0],
            },
            Vertex {
                position: [3.0, -1.0],
                uv: [2.0, 0.0],
            },
            Vertex {
                position: [-1.0, 3.0],
                uv: [0.0, 2.0],
            },
        ]
        .iter()
        .cloned(),
    )
    .unwrap();
    mod vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            src: "
                #version 450

                layout(location = 0) in vec2 position;
                layout(location = 1) in vec2 uv;
                layout(location = 0) out vec2 out_uv;
                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                    out_uv = uv;
                }
            "
        }
    }

    mod fs {
        vulkano_shaders::shader! {
            ty: "fragment",
            src: "
                #version 450

                layout(set = 0, binding = 0) uniform sampler2D tex;
                layout(location = 0) in vec2 uv;
                layout(location = 0) out vec4 f_color;

                void main() {
                    f_color = texture(tex, uv);
                }
            "
        }
    }

    let vs = vs::load(device.clone()).unwrap();
    let fs = fs::load(device.clone()).unwrap();

    // Here's our (2D drawing) framebuffer.
    dbg!(WIDTH * HEIGHT);
    // let fb2d_l = [(128 as u8, 64 as u8, 64 as u8, 255 as u8); WIDTH * HEIGHT];
    // let mut fb2d = vec![fb2d_l];
    let fb2d = [(128 as u8, 64 as u8, 64 as u8, 255 as u8); WIDTH * HEIGHT];
    // We'll work on it locally, and copy it to a GPU buffer every frame.
    // Then on the GPU, we'll copy it into an Image.
    let fb2d_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::transfer_source(),
        false,
        (0..WIDTH * HEIGHT).map(|_| (255_u8, 0_u8, 0_u8, 0_u8)),
    )
    .unwrap();
    // Let's set up the Image we'll copy into:
    let dimensions = ImageDimensions::Dim2d {
        width: WIDTH as u32,
        height: HEIGHT as u32,
        array_layers: 1,
    };
    //image n GPU
    let fb2d_image = StorageImage::with_usage(
        device.clone(),
        dimensions,
        Format::R8G8B8A8_UNORM,
        ImageUsage {
            // This part is key!
            transfer_destination: true,
            sampled: true,
            storage: true,
            transfer_source: false,
            color_attachment: false,
            depth_stencil_attachment: false,
            transient_attachment: false,
            input_attachment: false,
        },
        ImageCreateFlags::default(),
        std::iter::once(queue_family),
    )
    .unwrap();
    // Get a view on it to use as a texture:
    let fb2d_texture = ImageView::new(fb2d_image.clone()).unwrap();

    let fb2d_sampler = Sampler::new(
        device.clone(),
        Filter::Linear,
        Filter::Linear,
        MipmapMode::Nearest,
        SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat,
        0.0,
        1.0,
        0.0,
        0.0,
    )
    .unwrap();

    let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                // Pro move: We're going to cover the screen completely. Trust us!
                load: DontCare,
                store: Store,
                format: swapchain.format(),
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    )
    .unwrap();
    let pipeline = GraphicsPipeline::start()
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap();

    let layout = pipeline.layout().descriptor_set_layouts().get(0).unwrap();
    let mut set_builder = PersistentDescriptorSet::start(layout.clone());

    set_builder
        .add_sampled_image(fb2d_texture, fb2d_sampler)
        .unwrap();

    let set = set_builder.build().unwrap();

    let mut viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [0.0, 0.0],
        depth_range: 0.0..1.0,
    };

    let framebuffers = window_size_dependent_setup(&images, render_pass.clone(), &mut viewport);
    let recreate_swapchain = false;
    let mut previous_frame_end = Some(sync::now(device.clone()).boxed());

    State {
        fb2d,
        drawables: vec![],
        bg_color: (255, 255, 255, 255),
        previous_frame_end,
        recreate_swapchain,
        event_loop,
        fb2d_buffer,
        now_keys: [false; 255],
        prev_keys: [false; 255],
        fb2d_image,
        queue,
        swapchain,
        viewport,
        framebuffers,
        pipeline,
        render_pass,
        dimensions,
        vs,
        fs,
        surface,
        device,
        set,
        vertex_buffer,
    }
}

pub fn draw(state: &mut State, drawables: Vec<Drawable>) {
    //instead, take a state struct
    {
        // We need to synchronize here to send new data to the GPU.
        // We can't send the new framebuffer until the previous frame is done being drawn.
        // Dropping the future will block until it's done.
        if let Some(mut fut) = state.previous_frame_end.take() {
            fut.cleanup_finished();
        }
    }

    // First clear the framebuffer...
    clear(&mut state.fb2d, state.bg_color);
    // clear(&mut state.fb2d.as_slice()[0], state.bg_color);

    // here is where we draw!!!
    draw_objects(&mut state.fb2d, drawables);
    // draw_objects(&mut state.fb2d.as_slice()[0], drawables);

    // Now we can copy into our buffer.
    {
        let writable_fb = &mut *state.fb2d_buffer.write().unwrap();
        writable_fb.copy_from_slice(&state.fb2d); //copy frame buffer into GPU
    }

    if state.recreate_swapchain {
        let dimensions: [u32; 2] = state.surface.window().inner_size().into();
        let (new_swapchain, new_images) =
            match state.swapchain.recreate().dimensions(dimensions).build() {
                Ok(r) => r,
                Err(SwapchainCreationError::UnsupportedDimensions) => return,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
            };

        state.swapchain = new_swapchain;
        state.framebuffers = window_size_dependent_setup(
            &new_images,
            state.render_pass.clone(),
            &mut state.viewport,
        );
        state.recreate_swapchain = false;
    }
    let (image_num, suboptimal, acquire_future) =
        match swapchain::acquire_next_image(state.swapchain.clone(), None) {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                state.recreate_swapchain = true;
                return;
            }
            Err(e) => panic!("Failed to acquire next image: {:?}", e),
        };
    if suboptimal {
        state.recreate_swapchain = true;
    }

    let mut builder = AutoCommandBufferBuilder::primary(
        state.device.clone(),
        state.queue.family(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    builder
        // Now copy that framebuffer buffer into the framebuffer image
        .copy_buffer_to_image(state.fb2d_buffer.clone(), state.fb2d_image.clone())
        .unwrap()
        // And resume our regularly scheduled programming
        .begin_render_pass(
            state.framebuffers[image_num].clone(),
            SubpassContents::Inline,
            std::iter::once(vulkano::format::ClearValue::None),
        )
        .unwrap()
        .set_viewport(0, [state.viewport.clone()])
        .bind_pipeline_graphics(state.pipeline.clone())
        .bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            state.pipeline.layout().clone(),
            0,
            state.set.clone(),
        )
        .bind_vertex_buffers(0, state.vertex_buffer.clone())
        .draw(state.vertex_buffer.len() as u32, 1, 0, 0)
        .unwrap()
        .end_render_pass()
        .unwrap();

    let command_buffer = builder.build().unwrap();

    let future = acquire_future
        .then_execute(state.queue.clone(), command_buffer)
        .unwrap()
        .then_swapchain_present(state.queue.clone(), state.swapchain.clone(), image_num)
        .then_signal_fence_and_flush();

    match future {
        Ok(future) => {
            state.previous_frame_end = Some(future.boxed());
        }
        Err(FlushError::OutOfDate) => {
            state.recreate_swapchain = true;
            state.previous_frame_end = Some(sync::now(state.device.clone()).boxed());
        }
        Err(e) => {
            println!("Failed to flush future: {:?}", e);
            state.previous_frame_end = Some(sync::now(state.device.clone()).boxed());
        }
    }
}

pub fn synchronize_prev_frame_end(mut state: State) {
    {
        // We need to synchronize here to send new data to the GPU.
        // We can't send the new framebuffer until the previous frame is done being drawn.
        // Dropping the future will block until it's done.
        if let Some(mut fut) = state.previous_frame_end.take() {
            fut.cleanup_finished();
        }
    }
}
