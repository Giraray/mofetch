use wgpu::{util::DeviceExt};

#[path = "../utils.rs"]
mod utils;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ResBinding {
    resolution: [f32; 2],
}

pub struct DogShader<'a> {
    pub pipeline: &'a wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub render_target: wgpu::Texture,
    pub output_buffer: wgpu::Buffer,
}

pub fn new<'a>(
    device: &wgpu::Device, texture: &wgpu::Texture, size: wgpu::Extent3d,
    pipeline: &'a wgpu::RenderPipeline, 
) -> DogShader<'a> {

    // bindgroup entries
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    // buffers
    let resolution = ResBinding {
        resolution: [size.width as f32, size.height as f32],
    };
    let res_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&[resolution]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let entries = [
        wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&texture_view),
        },
        wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::Sampler(&sampler),
        },
        wgpu::BindGroupEntry {
            binding: 2,
            resource: res_buffer.as_entire_binding(),
        }
    ];
    let bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            layout: &pipeline.get_bind_group_layout(0),
            entries: &entries,
            label: Some("DoG bind group"),
        }
    );

    // this is where the rendered output goes
    let render_target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("input texture"),
        size: size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    // output_buffer
    let padded_width = 64 * (size.width as f32 / 64.0).ceil() as u32;
    let read_buffer_size = utils::align_buffer_size(4 * padded_width * size.height, 256);
    let read_buffer_desc = wgpu::BufferDescriptor {
        size: read_buffer_size,
        usage: wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::MAP_READ,
        label: None,
        mapped_at_creation: false,
    };
    let read_buffer = device.create_buffer(&read_buffer_desc);

    return DogShader {
        pipeline,
        bind_group,
        render_target,
        output_buffer: read_buffer,
    }
}