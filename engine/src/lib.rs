// Based on the Vulkano triangle example.

// Triangle example Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json;
use std::cmp::{max, min};
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess};
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
pub type FbCoords = (usize, usize);
const WIDTH: usize = 320;
const HEIGHT: usize = 240;

#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position, uv);

#[allow(non_snake_case)]
#[derive(Clone, Serialize, Deserialize)]
pub struct Card {
    name: String,
    playCost: usize,
    health: usize,
    defense: usize,
    passiveCost: usize,
    specialCost: usize,
    specialTag: String,
    special: String, //should be a function somehow
    attack: usize,
    attackTag: String,
    specialAttribute: String,
}

impl Drop for Card {
    fn drop(&mut self) {
        {}
    }
}

impl Card {
    pub fn take_damage(&mut self, dmg: usize) -> bool {
        //returns isAlive - false if health is 0
        let rem_dmg = dmg - self.defense;

        self.health -= rem_dmg;

        self.health != 0
    }

    pub fn attack(&self, other_card: &mut Card) -> bool {
        //returns isOtherCardAlive

        other_card.take_damage(self.attack)
    }

    pub fn get_description(&self) -> String {
        //for rendering the card itself
        let name = &self.name;

        let stats = format!(
            "HP:{} | AC:{} | Upkeep: {} \n {}",
            self.health, self.defense, self.passiveCost, self.specialAttribute
        );

        let attack_block = format!("ATK: {} \n {}", self.attack, self.attackTag);

        let special_block = format!(
            "Special | Cost: {} \n {}",
            self.specialCost, self.specialTag
        );

        format!(
            "{} \n \n {} \n \n {} \n\n {}",
            name, stats, attack_block, special_block
        )
    }
}

#[derive(Clone, Deserialize)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new(cards: Vec<Card>) -> Deck {
        Deck { cards }
    }

    pub fn new_empty() -> Deck {
        Deck { cards: vec![] }
    }

    pub fn add_card(self: &mut Deck, card: Card) {
        self.cards.push(card);
    }

    pub fn add_cards(self: &mut Deck, cards: &mut Vec<Card>) {
        self.cards.append(cards);
    }

    pub fn set_cards(self: &mut Deck, cards: Vec<Card>) {
        self.cards = cards;
    }

    pub fn remove_card(self: &mut Deck, index: usize) {
        self.cards.remove(index);
    }

    // Removes card from the deck
    pub fn draw_and_remove(self: &mut Deck) -> Card {
        self.cards.remove(0)
    }

    // Places card back in bottom of deck
    pub fn draw_and_cycle(self: &mut Deck) -> Card {
        let next_card = self.cards.remove(0);
        self.cards.push(next_card.clone());
        next_card
    }

    pub fn shuffle(self: &mut Deck) {
        let deck_size = self.cards.len();
        let mut random_generator = rand::thread_rng();

        (0..(deck_size - 1)).for_each(|range_min_index| {
            let next_index = random_generator.gen_range(range_min_index..deck_size);
            let next_card = self.cards.remove(next_index);
            self.cards.insert(0, next_card);
        });
    }
}

pub fn load_cards_from_file(file_path: &str) -> Deck {
    let mut file = File::open(file_path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    // let deckf: DeckFile = serde_json::from_str(&data).unwrap();
    serde_json::from_str::<Deck>(&data).unwrap()

    // let str_rep = root.get("deck").unwrap().get(1).unwrap().as_str().unwrap();
    // let c: Card = serde_json::from_str(str_rep).unwrap();

    // println!("{}", deckf.deck.cards[0].name);
    // println!("{}", deck.cards[0].name);
    // cards.push(c);
}

pub struct State {
    pub fb2d: [(u8, u8, u8, u8); WIDTH * HEIGHT],
    pub drawables: Vec<Drawable>,
    pub bg_color: Color,
    previous_frame_end: std::option::Option<std::boxed::Box<dyn vulkano::sync::GpuFuture>>,
    recreate_swapchain: bool,
    pub event_loop: EventLoop<()>,
    fb2d_buffer: Arc<vulkano::buffer::CpuAccessibleBuffer<[(u8, u8, u8, u8)]>>,
    pub now_keys: [bool; 255],
    pub prev_keys: [bool; 255],
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
    window_width: f64,
    window_height: f64,
    pub left_mouse_down: bool,
    pub prev_left_mouse_down: bool,
    pub mouse_coords: FbCoords,
    pub prev_mouse_coords: FbCoords,
    pub initial_mouse_down_coords: Option<FbCoords>,
    pub drag_item_id: Option<usize>,
    pub drag_item_initial_coords: Option<FbCoords>,
}

fn coord_shift(initial: FbCoords, shifter: (i32, i32)) -> FbCoords {
    let (x_initial, y_initial) = initial;
    let (x_shift, y_shift) = shifter;
    (
        max(x_initial as i32 + x_shift, 0) as usize,
        max(y_initial as i32 + y_shift, 0) as usize,
    )
}

fn generate_deck_slots(
    card_size: (usize, usize),
    card_padding_bottom: usize,
    card_padding_top: usize,
    num_slots: usize,
) -> Vec<Drawable> {
    let (card_width, card_height) = card_size;
    assert!(card_width * (num_slots + 1) < WIDTH);

    let total_spacer_space = WIDTH - (num_slots * card_width);
    let spacer_width = total_spacer_space / (num_slots + 3); // 3 represents double space between last card and deck, plus space to right of deck
    let container = Drawable::Rectangle(
        Rect {
            x: 0,
            y: HEIGHT - card_height - (card_padding_bottom + card_padding_top),
            w: WIDTH,
            h: HEIGHT - card_height - card_padding_bottom,
        },
        (255, 255, 255, 100),
        None,
    );

    let mut slot_drawables: Vec<Drawable> = vec![container];

    (1..num_slots + 1).for_each(|slot| {
        let card_x = slot * spacer_width + (slot - 1) * card_width;
        let card_y = HEIGHT - card_height - (card_padding_bottom);
        let card_slot_background = Drawable::Rectangle(
            Rect {
                x: card_x,
                y: card_y,
                w: card_width,
                h: card_height,
            },
            (255, 0, 0, 255),
            None,
        );
        let card_slot_frame = Drawable::RectOutlined(
            Rect {
                x: card_x,
                y: card_y,
                w: card_width,
                h: card_height,
            },
            (255, 255, 255, 255),
            Some(DraggableSnapType::Card(false, true)),
        );
        slot_drawables.append(vec![card_slot_background, card_slot_frame])
    });

    vec![]
}

pub fn check_and_handle_drag(state: &mut State) {
    let temp_drawables = state.drawables.clone();
    if state.left_mouse_down {
        if !state.prev_left_mouse_down {
            let dragged_item = temp_drawables
                .iter()
                .rev()
                .enumerate()
                .find(|(_, item)| item.contains(state.mouse_coords) && item.is_draggable());

            if let Some((index, item)) = dragged_item {
                state.drag_item_id = Some((temp_drawables.len() - 1) - index);
                state.drag_item_initial_coords = Some(item.get_coords());
                state.initial_mouse_down_coords = Some(state.mouse_coords);
            } else {
                state.drag_item_id = None;
                state.drag_item_initial_coords = None;
            }
        } else if let (
            Some(index),
            Some((initial_mouse_x, initial_mouse_y)),
            Some(initial_item_coords),
        ) = (
            state.drag_item_id,
            state.initial_mouse_down_coords,
            state.drag_item_initial_coords,
        ) {
            let drawable = &mut state.drawables[index];
            let x_shift = (state.mouse_coords.0 as i32) - (initial_mouse_x as i32);
            let y_shift = (state.mouse_coords.1 as i32) - (initial_mouse_y as i32);

            let shifted_coords = coord_shift(initial_item_coords, (x_shift, y_shift));
            drawable.move_to(shifted_coords);
        }
    } else if let (
        Some(index),
        Some((initial_mouse_x, initial_mouse_y)),
        Some(initial_item_coords),
    ) = (
        state.drag_item_id,
        state.initial_mouse_down_coords,
        state.drag_item_initial_coords,
    ) {
        let drawable = &mut state.drawables[index];
        let x_shift = (state.mouse_coords.0 as i32) - (initial_mouse_x as i32);
        let y_shift = (state.mouse_coords.1 as i32) - (initial_mouse_y as i32);

        let shifted_coords = coord_shift(initial_item_coords, (x_shift, y_shift));
        drawable.move_to(shifted_coords);

        state.drag_item_id = None;
        state.drag_item_initial_coords = None;
        state.initial_mouse_down_coords = None;
    }
}

#[derive(Copy, Clone)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

// Two fields, one for whether it can snap to one like it, and whether one like it can snap to it
#[derive(Copy, Clone)]
pub enum DraggableSnapType {
    Card(bool, bool),
}

#[derive(Copy, Clone)]
pub enum Drawable {
    Rectangle(Rect, Color, Option<DraggableSnapType>),
    RectOutlined(Rect, Color, Option<DraggableSnapType>),
}

impl Drawable {
    pub fn contains(self: &Drawable, coord: FbCoords) -> bool {
        let (x, y) = coord;
        match self {
            Drawable::Rectangle(rect, _, _) => {
                (x >= rect.x && x <= rect.x + rect.w) && (y >= rect.y && y <= rect.y + rect.h)
            }
            Drawable::RectOutlined(rect, _, _) => {
                (x >= rect.x && x <= rect.x + rect.w) && (y >= rect.y && y <= rect.y + rect.h)
            }
        }
    }

    pub fn get_coords(self: &Drawable) -> FbCoords {
        match self {
            Drawable::Rectangle(rect, _, _) => (rect.x, rect.y),
            Drawable::RectOutlined(rect, _, _) => (rect.x, rect.y),
        }
    }

    pub fn shift(self: &mut Drawable, amount: (i32, i32)) {
        let (x, y) = amount;
        // println!{"amount shifted: {:?}", amount};
        match self {
            Drawable::Rectangle(rect, _, _) => {
                rect.x = max(rect.x as i32 + x, 0) as usize;
                rect.y = max(rect.y as i32 + y, 0) as usize;
            }
            Drawable::RectOutlined(rect, _, _) => {
                rect.x = max(rect.x as i32 + x, 0) as usize;
                rect.y = max(rect.y as i32 + y, 0) as usize;
            }
        }
    }

    pub fn move_to(self: &mut Drawable, coords: FbCoords) {
        let (x, y) = coords;
        match self {
            Drawable::Rectangle(rect, _, _) => {
                rect.x = x;
                rect.y = y;
            }
            Drawable::RectOutlined(rect, _, _) => {
                rect.x = x;
                rect.y = y;
            }
        }
    }

    pub fn is_draggable(self: &Drawable) -> bool {
        match self {
            Drawable::Rectangle(_, _, draggable) => draggable.is_some(),
            Drawable::RectOutlined(_, _, draggable) => draggable.is_some(),
        }
    }

    pub fn debug_x(self: &Drawable) -> usize {
        match self {
            Drawable::Rectangle(rect, _, _) => rect.x,
            Drawable::RectOutlined(rect, _, _) => rect.x,
        }
    }

    fn debug_coords(self: &Drawable) -> (usize, usize) {
        match self {
            Drawable::Rectangle(rect, _, _) => (rect.x, rect.y),
            Drawable::RectOutlined(rect, _, _) => (rect.x, rect.y),
        }
    }
}

impl Rect {
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Rect {
        Rect { x, y, w, h }
    }
}

fn draw_objects(fb: &mut [Color], drawables: Vec<Drawable>) {
    drawables.into_iter().enumerate().for_each(|(_, obj)| {
        match obj {
            Drawable::Rectangle(r, c, _) => {
                // println!("rectangle x: {:?}", r.x);
                rectangle(fb, r, c);
            }
            Drawable::RectOutlined(r, c, _) => {
                rect_outlined(fb, r, c);
            }
        }
    });
}

// Here's what clear looks like, though we won't use it
#[allow(dead_code)]
pub fn clear(fb: &mut [Color], c: Color) {
    fb.fill(c);
}

// #[allow(dead_code)]
// fn line(fb: &mut [Color], x0: usize, x1: usize, y: usize, c: Color) {
//     assert!(y < HEIGHT);
//     assert!(x0 <= x1);
//     assert!(x1 < WIDTH);
//     fb[y * WIDTH + x0..(y * WIDTH + x1)].fill(c);
// }

#[allow(dead_code)]
fn line(fb: &mut [Color], x0: usize, x1: usize, y: usize, c: Color) {
    let min_x = min(max(0, x0), WIDTH);
    let max_x = max(min(WIDTH, x1), 0);
    if y >= HEIGHT {
        return;
    }
    fb[y * WIDTH + min_x..(y * WIDTH + max_x)].fill(c);
}

#[allow(dead_code)]
fn rectangle(fb: &mut [Color], r: Rect, c: Color) {
    // assert!(r.w + r.x < WIDTH);
    // assert!(r.h + r.y < HEIGHT);

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

// fn generate_card_spaces(num_cards: usize, card_width: usize, card_height: usize) -> Vec<Drawable> {
//     assert!(card_width * (num_cards + 1) < WIDTH);
//     assert!(card_height < HEIGHT / 4);

//     let even_spacing =
//     return vec![]
// }

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
) -> (Vec<Arc<Framebuffer>>, (f64, f64)) {
    let dimensions = images[0].dimensions().width_height();
    viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];
    let mut window_width = dimensions[0].into();
    let mut window_height = dimensions[1].into();
    (
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
            .collect::<Vec<_>>(),
        (window_width, window_height),
    )
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
    let (swapchain, images) = {
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

    let (framebuffers, (window_width, window_height)) =
        window_size_dependent_setup(&images, render_pass.clone(), &mut viewport);
    let recreate_swapchain = false;
    let previous_frame_end = Some(sync::now(device.clone()).boxed());

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
        window_width,
        window_height,
        left_mouse_down: false,
        prev_left_mouse_down: false,
        mouse_coords: (WIDTH, HEIGHT),
        prev_mouse_coords: (WIDTH, HEIGHT),
        initial_mouse_down_coords: None,
        drag_item_id: None,
        drag_item_initial_coords: None,
    }
}

pub fn draw(state: &mut State) {
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
    draw_objects(&mut state.fb2d, state.drawables.clone());
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
        let setup_result = window_size_dependent_setup(
            &new_images,
            state.render_pass.clone(),
            &mut state.viewport,
        );

        state.framebuffers = setup_result.0;
        state.window_width = setup_result.1 .0;
        state.window_height = setup_result.1 .1;
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

pub fn handle_winit_event(
    event: winit::event::Event<()>,
    control_flow: &mut winit::event_loop::ControlFlow,
    state: &mut State,
) {
    match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(_),
            ..
        } => {
            state.recreate_swapchain = true;
        }
        // NewEvents: Let's start processing events.
        Event::NewEvents(_) => {
            // Leave now_keys alone, but copy over all changed keys
            state.prev_keys.copy_from_slice(&state.now_keys);
            state.prev_left_mouse_down = state.left_mouse_down;
            state.prev_mouse_coords = state.mouse_coords;
        }
        // WindowEvent->KeyboardInput: Keyboard input!
        Event::WindowEvent {
            // Note this deeply nested pattern match
            event:
                WindowEvent::KeyboardInput {
                    input:
                        winit::event::KeyboardInput {
                            // Which serves to filter out only events we actually want
                            virtual_keycode: Some(keycode),
                            state: key_state,
                            ..
                        },
                    ..
                },
            ..
        } => {
            // It also binds these handy variable names!
            match key_state {
                winit::event::ElementState::Pressed => {
                    // VirtualKeycode is an enum with a defined representation
                    state.now_keys[keycode as usize] = true;
                }
                winit::event::ElementState::Released => {
                    state.now_keys[keycode as usize] = false;
                }
            }
        }
        Event::WindowEvent {
            event:
                WindowEvent::CursorMoved {
                    device_id: _,
                    position,
                    ..
                },
            window_id: _,
        } => {
            let cursor_x = position.x / state.window_width;
            let cursor_y = position.y / state.window_height;
            state.mouse_coords = (
                (cursor_x * WIDTH as f64) as usize,
                (cursor_y * HEIGHT as f64) as usize,
            );
        }
        Event::WindowEvent {
            event:
                WindowEvent::MouseInput {
                    device_id: _,
                    state: button_state,
                    button,
                    ..
                },
            window_id: _,
        } => {
            if button == winit::event::MouseButton::Left {
                state.left_mouse_down = button_state == winit::event::ElementState::Pressed;
            }
        }
        _ => {}
    }
}
