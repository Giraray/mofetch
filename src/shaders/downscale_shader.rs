// downscale shader
use wgpu::{util::DeviceExt};

#[path = "../utils.rs"]
mod utils;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ResBinding {
    resolution: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WorkgroupSize {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

pub struct DownscaleShader<'a> {
    pub pipeline: &'a wgpu::ComputePipeline,
    pub bind_group: wgpu::BindGroup,
    pub storage_buffer: wgpu::Buffer, // the compute "render_target"
    pub output_buffer: wgpu::Buffer,
}

pub struct DownscaleShaderStruct<'a> {
    pub device: &'a wgpu::Device,
    pub texture: &'a wgpu::Texture,
    pub sobel_texture: &'a wgpu::Texture,
    pub size:  wgpu::Extent3d,
    pub buffer_size: &'a u64,
    pub ascii_style: &'a str,
}

pub fn new<'a>(desc: DownscaleShaderStruct, pipeline: &'a wgpu::ComputePipeline) -> DownscaleShader<'a> {

    let device = desc.device;
    let texture = desc.texture;
    let sobel_texture = desc.sobel_texture;
    let size = desc.size;
    let buffer_size = *desc.buffer_size;

    // bindgroup entries
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sobel_view = sobel_texture.create_view(&wgpu::TextureViewDescriptor::default());

    // buffers
    let resolution = ResBinding {
        resolution: [utils::align_buffer_size(size.width,64) as f32, size.height as f32],
    };
    let res_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("res buffer"),
        contents: bytemuck::cast_slice(&[resolution]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let quantize: f32 = desc.ascii_style.len() as f32;
    let quant_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("quant buffer"),
        contents: bytemuck::cast_slice(&[quantize]),
        usage: wgpu::BufferUsages::UNIFORM
    });

    // storage buffer, where data is placed
    let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        size: buffer_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        label: Some("compute source buffer"),
        mapped_at_creation: false,
    });

    let entries = [
        wgpu::BindGroupEntry {
            binding: 0,
            resource: storage_buffer.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::TextureView(&texture_view),
        },
        wgpu::BindGroupEntry {
            binding: 2,
            resource: wgpu::BindingResource::TextureView(&sobel_view),
        },
        wgpu::BindGroupEntry {
            binding: 3,
            resource: res_buffer.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
            binding: 4,
            resource: quant_buffer.as_entire_binding(),
        }
    ];
    let bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            layout: &pipeline.get_bind_group_layout(0),
            entries: &entries,
            label: Some("ds bind group"),
        }
    );

    let read_buffer_desc = wgpu::BufferDescriptor {
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::MAP_READ,
        label: Some("compute read buffer"),
        mapped_at_creation: false,
    };
    let read_buffer = device.create_buffer(&read_buffer_desc);

    return DownscaleShader {
        pipeline,
        bind_group,
        storage_buffer,
        output_buffer: read_buffer,
    }
}