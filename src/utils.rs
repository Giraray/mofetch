/// Round buffer size up to nearest multiple of 256 to 
/// align buffer offset (prevents UnalignedCopyOffset error).
pub fn align_buffer_size(size: u32, align: u64) -> u64 {
    return align * (size as f32 / align as f32).ceil() as u64;
}

/// align_buffer_size() that returns a f32
pub fn align_buffer_size_f(size: u32, align: u64) -> f32 {
    return align as f32 * (size as f32 / align as f32).ceil() as f32;
}

/// Used to create differing RenderPipelineDescriptors for multiple 
/// RenderPipelines. Reduces boilerplate (in `process_frames()`)
pub fn create_render_pipeline_desc(module: &wgpu::ShaderModule) -> wgpu::RenderPipelineDescriptor {
    let pipeline_desc = wgpu::RenderPipelineDescriptor {
        label: Some("shader pipeline"),
        layout: None,
        vertex: wgpu::VertexState {
            module: module,
            entry_point: Some("v_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: module,
            entry_point: Some("f_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions:: default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    };
    return pipeline_desc;
}