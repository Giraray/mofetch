//! This file is responsible for every step in the processing pipeline, from source processing, 
//! post-processing and then to frame buffer caching. It also includes the function to render 
//! frame buffers to a terminal.

#[path ="./shaders/dog_shader.rs"]
mod dog_shader;

#[path ="./shaders/sobel_shader.rs"]
mod sobel_shader;

#[path ="./shaders/downscale_shader.rs"]
mod downscale_shader;

#[path = "./utils.rs"]
pub mod utils;

use std::process::Command;
use std::fs;
use wgpu::{ComputePipeline, RenderPipeline};
use std::fs::{File, read_to_string};
use std::io::prelude::*;
use image::GenericImageView;
use downscale_shader::WorkgroupSize;
use std::time::{Instant,Duration};

use crate::TERM_FONT_DIMS;

pub struct FfmpegConfig <'a> {
    pub input_path: &'a str,
    pub fps: &'a u16,
}

// Resolutions for ascii tiles;
// Each tile will always have a fixed size determined by the font size of the terminal. Mine is (10,22).
// The tile resolution denotes how many pixels are used in each tile. All of the different resolutions 
// maintain roughly the same aspect ratio as (10,22). Smaller tile resolutions result in larger renders 
// since each tile uses fewer pixels.
//
// TODO: runtime variable for shader wg_size
const LARGEST_TILE: downscale_shader::WorkgroupSize = downscale_shader::WorkgroupSize{
    x: 10,
    y: 22,
    z: 1,
};
const LARGE_TILE: downscale_shader::WorkgroupSize = downscale_shader::WorkgroupSize{
    x: 8,
    y: 18,
    z: 1,
};
const MEDIUM_TILE: downscale_shader::WorkgroupSize = downscale_shader::WorkgroupSize{
    x: 6,
    y: 13,
    z: 1,
};
const SMALL_TILE: downscale_shader::WorkgroupSize = downscale_shader::WorkgroupSize{
    x: 4,
    y: 9,
    z: 1,
};
const TILE_RESOLUTIONS: [WorkgroupSize;4] = [SMALL_TILE,MEDIUM_TILE,LARGE_TILE,LARGEST_TILE];

pub struct FFmpegReturn {
    pub frame_count: i32,
    pub width: u16,
    pub height: u16,
}

fn get_frames_path() -> String {
    format!("{}/mofetch/frames",dirs::data_dir().unwrap().to_str().unwrap())
}

/// Runs the ffmpeg command to break down the input from a path into png frames, 
/// storing them in a `frames` directory.
pub fn get_frames(config: &FfmpegConfig, max_width: u16, max_height: u16, verbose: bool) -> FFmpegReturn {
    // make sure output dir is cleared
    let frames_path = get_frames_path();
    fs::remove_dir_all(&frames_path).ok();
    fs::create_dir_all(&frames_path).unwrap();

    // get media dimensions, used for resizing the source and choosing a tile resolution later
    let dims_stdout = Command::new("ffprobe")
        .args(["-hide_banner", "-select_streams", "v:0", "-show_entries",
            "stream=width,height","-of", "csv=s=x:p=0", config.input_path])
        .stdout(std::process::Stdio::piped())
        .output()
        .unwrap();

    let output = String::from_utf8(dims_stdout.stdout).unwrap();
    let dimensions: Vec<&str> = output.split('x').collect();

    let width = dimensions[0].parse::<u16>().unwrap();
    let height = dimensions[1].trim().parse::<u16>().unwrap();

    // run frame conversion ffmpeg with path and config
    let fps_string = config.fps.to_string();
    let mut ffmpeg_process = Command::new("ffmpeg");
    let ffmpeg_log_level = if verbose {"info"} else {"fatal"};
    ffmpeg_process.args(["-hide_banner", "-loglevel",ffmpeg_log_level, "-i", config.input_path,"-r", fps_string.as_str()]);
    
    // compress source and retain aspect ratio if width or height exceed the max (from user args -W and -H)
    if width > max_width || height > max_height {
        let preferred_aspect_ratio = max_width as f32 / max_height as f32;
        let source_aspect_ratio = width as f32 / height as f32;

        let adjustment;
        let scaled_resolution: (u16,u16) =
            if preferred_aspect_ratio > source_aspect_ratio {
                adjustment = String::from("width");
                ((width as f32 * (max_height as f32 / height as f32)) as u16, max_height)
            }
            else {
                adjustment = String::from("height");
                (max_width, (height as f32 * (max_width as f32 / width as f32)) as u16)
        };
        let scaled_res_string = format!("scale={}:{}",scaled_resolution.0, scaled_resolution.1);
        let compression_args = ("-vf",&scaled_res_string);
        ffmpeg_process.args([compression_args.0,compression_args.1]);

        if verbose {
            println!("source_width: {}, source_height: {} | max_width: {}, max_height: {}",width,height, max_width,max_height);
            if !adjustment.is_empty() {
                println!("Adjusted {} | width: {}, height: {}",adjustment,scaled_resolution.0,scaled_resolution.1);
            }
        }
    }

    if verbose {println!("");}

    println!("Processing source...");
    ffmpeg_process.arg(format!("{}/output_frame_%d.png", frames_path));
    ffmpeg_process.status().unwrap();

    if verbose {println!("");}

    // count amount of frames to determine if output is a video or image and should be looped or not
    let paths = fs::read_dir(&frames_path).unwrap();
    let mut total_frames = 0;
    for _ in paths {
        total_frames += 1;
    }

    return FFmpegReturn {
        frame_count: total_frames,
        width,
        height
    };
}

pub struct ProcessDescriptor {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter: wgpu::Adapter,
    pub adapters_vec: Vec<wgpu::Adapter>
}

impl ProcessDescriptor {
    /// Create device, queue and open the cache_file once and pass them to the shader_process() as references
    pub async fn init(adapter_index: usize) -> ProcessDescriptor {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        // ugly, but it works
        let mut adapters_vec = instance.enumerate_adapters(wgpu::Backends::PRIMARY);
        let adapter = adapters_vec.remove(adapter_index);
        adapters_vec = instance.enumerate_adapters(wgpu::Backends::PRIMARY);

        let (device, queue) = adapter
            .request_device(&Default::default())
            .await
            .unwrap();
        
        return Self {
            device,
            queue,
            adapter,
            adapters_vec,
        }
    }
}

fn get_tile_res(width: u16, height: u16, max_width: u16, max_height: u16) -> WorkgroupSize {
    let mut calc_width = TILE_RESOLUTIONS.last().unwrap().clone();
    for res in TILE_RESOLUTIONS.iter().rev() {
        if (width / res.x as u16) <= (max_width / TERM_FONT_DIMS.0) as u16 {
            calc_width = *res;
        }
    }
    let mut calc_height = TILE_RESOLUTIONS.last().unwrap().clone();
    for res in TILE_RESOLUTIONS.iter().rev() {
        if (height / res.y as u16) <= (max_height / TERM_FONT_DIMS.1) as u16 {
            calc_height = *res;
        }
    }
    return WorkgroupSize {
        x: std::cmp::max(calc_width.x, calc_height.x),
        y: std::cmp::max(calc_width.y, calc_height.y),
        z: 1,
    };
}

pub struct Benchmark {
    pub total_time: Duration,
    pub image_decode_time: Duration,
    pub render_time: Duration,
    pub cache_time: Duration,
}
impl Benchmark {
    fn init() -> Benchmark {
        return Benchmark {
            total_time: Duration::from_millis(0),
            image_decode_time: Duration::from_millis(0),
            render_time: Duration::from_millis(0),
            cache_time: Duration::from_millis(0),
        }
    }
    
    fn average(&mut self, total_frames: u32) {
        self.image_decode_time /= total_frames;
        self.render_time /= total_frames;
        self.cache_time /= total_frames;
    }
}

/// Processes each frame with the shader process and caches the resulting ASCII 
/// text buffers.
pub fn process_frames(frame_count: &i32, process_desc: &ProcessDescriptor,
    cache_file: File, width: u16, height: u16, max_width: u16, max_height: u16, shader_config: utils::ShaderConfig,
    verbose: bool,
) {
    println!("Processing frames...");

    let device = &process_desc.device;

    // Init shader pipelines
    // DoG shader
    let dog_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader_code/dog_shader.wgsl").into()),
    });

    // sobel shader
    let sobel_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader_code/sobel_shader.wgsl").into()),
    });

    // downscale compute shader; Each alternative is the same except with different workgroup sizes, 
    // since those cannot be runtime variables. Each workgroup size determines the source resolution 
    // for each ascii tile 
    let target_res = get_tile_res(width, height, max_width, max_height);
    let shader_source =
        if target_res.x == SMALL_TILE.x {
            wgpu::ShaderSource::Wgsl(include_str!("shaders/shader_code/downscale_shader_small.wgsl").into())
        }
        else if target_res.x == MEDIUM_TILE.x {
            wgpu::ShaderSource::Wgsl(include_str!("shaders/shader_code/downscale_shader_medium.wgsl").into())
        }
        else if target_res.x == LARGE_TILE.x {
            wgpu::ShaderSource::Wgsl(include_str!("shaders/shader_code/downscale_shader_large.wgsl").into())
        }
        else {
            wgpu::ShaderSource::Wgsl(include_str!("shaders/shader_code/downscale_shader_largest.wgsl").into())
        };
        let ds_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: shader_source,
    });
    
    let dog_desc = utils::create_render_pipeline_desc(&dog_module);
    let sobel_desc = utils::create_render_pipeline_desc(&sobel_module);

    let dog_pipeline = device.create_render_pipeline(&dog_desc);
    let sobel_pipeline = device.create_render_pipeline(&sobel_desc);
    let ds_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        module: &ds_module,
        layout: None,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    // start image processing
    let is_image = if *frame_count == 1 {1} else {0};
    let mut benchmark = Benchmark::init();

    if verbose { println!("Using GPU adapter: {:?}", process_desc.adapter.get_info().name); }

    let frames_path = get_frames_path();
    for n in 1..frame_count + is_image {
        let frame_path = format!("{}/output_frame_{}.png",frames_path, n);
        let new_benchmark = pollster::block_on(shader_process(
            frame_path.as_str(), &process_desc, &cache_file,&dog_pipeline,&sobel_pipeline,&ds_pipeline,
            target_res, &shader_config, verbose,
        ));
        
        benchmark.total_time += new_benchmark.total_time;
        benchmark.image_decode_time += new_benchmark.image_decode_time;
        benchmark.render_time += new_benchmark.render_time;
        benchmark.cache_time += new_benchmark.cache_time;
    }

    // get average benchmark times
    if verbose {
        benchmark.average(*frame_count as u32);
        println!("Total processing time: {:.3?} | AVERAGE: image_decode: {:.3?} | render: {:.3?} | cache: {:.3?}",
            benchmark.total_time, benchmark.image_decode_time, benchmark.render_time, benchmark.cache_time);
    }

    fs::remove_dir_all(frames_path).ok();
}

struct CursorPos {
    x: u16,
    y: u16,
}
/// Renders and loops the ASCII txt frames through stdout.
pub fn print_frame_loop(cache_path: &str, is_image: bool) {
    let file_string = read_to_string(cache_path).unwrap();

    // read fps configuration from cache file
    let config_line = file_string.lines().nth(0).unwrap();
    let config_string = &String::from(config_line)[3..];
    let fps_iter: Vec<&str> = config_string.split('=').collect();
    let fps = fps_iter[1].parse::<i32>().unwrap();

    let frame_duration = (1000.0/(fps as f32)).ceil() as u64;

    let mut cursor_pos = CursorPos {
        x: 2,
        y: 2,
    };
    let mut frame_buffer: String = Default::default();
    print!("{}{}",termion::clear::All, termion::cursor::Goto(cursor_pos.x,cursor_pos.y));

    loop {
        for line in file_string.lines() {
            // if line is a frame separator
            if line.len() <= 1 {
                cursor_pos.x = 1;
                cursor_pos.y = 1;

                // if end of frame (if frame separator appears when buffer is not empty)
                if frame_buffer.len() > 1 {
                    print!("{}{}",termion::cursor::Goto(cursor_pos.x,cursor_pos.y),frame_buffer);
                    frame_buffer.clear();
                    std::thread::sleep(Duration::from_millis(frame_duration));
                }
            }
            // else add frame line to frame_buffer
            else if !line.contains("[]") {
                frame_buffer += format!("{}\n",line).as_str();
            }
        }
        if is_image {
            // TODO: breaking this loop also kills the fetch thread. Maybe add an arbitrary loop here?
            print!("{}{}",termion::cursor::Goto(cursor_pos.x,cursor_pos.y),frame_buffer);
            break;
        }
    }
}


// TODO: need a way to get terminal font and calculate the brightness of each letter.
// Will use this ascii string, which looks good on my terminal.
const ASCII_STYLE: &str = " .,:?c79WNB@";
// const ASCII_STYLE: &str = " .'`^,:;!i><~+-r?tfxjnuvczXQ0OZ#MW&8%B@$";
// const ASCII_STYLE: &str = " .-+:;aAbBcCdDeEfFghHikKnNrR*?@";

const ASCII_EDGES: &str = "|/_\\";

/// Processes an image with the ASCII shader algorithm and generates and caches a 
/// text buffer to `cache_path` with the processed result.
pub async fn shader_process(
    frame_path: &str, desc: &ProcessDescriptor, file: &File, dog_pipeline: &RenderPipeline,
    sobel_pipeline: &RenderPipeline, ds_pipeline: &ComputePipeline, wg_size: WorkgroupSize,
    shader_config: &utils::ShaderConfig, verbose: bool,
) -> Benchmark {

    let device = &desc.device;
    let queue = &desc.queue;

    // create image
    let benchmark_image_decode = Instant::now();
    let diffuse_image = image::ImageReader::open(frame_path)
        .unwrap().decode().unwrap();
    let diffuse_rgba = diffuse_image.to_rgba8();
    let image_decode_time = benchmark_image_decode.elapsed();

    let benchmark_write_texture = Instant::now();
    let dimensions = diffuse_image.dimensions();

    let texture_size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };
    let image_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("input texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &image_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &diffuse_rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(texture_size.width * 4),
            rows_per_image: Some(texture_size.height),
        },
        texture_size,
    );
    let write_texture_time = benchmark_write_texture.elapsed();

    let benchmark_render = Instant::now();

    // Calculate buffer size for ascii storage/read buffer and round up to 
    // nearest multiple of 256 to align buffer offset (prevents UnalignedCopyOffset 
    // error).
    let temp_buff_size = (4.0 * texture_size.width as f32 * texture_size.height as f32 / wg_size.x as f32 / wg_size.y as f32).floor();
    let ascii_buffer_size = (256.0 * (temp_buff_size/256.0).ceil()) as wgpu::BufferAddress;

    // create and compile shaders
    let dog_shader = dog_shader::new(
        &device, &image_texture, texture_size, dog_pipeline
    );
    let sobel_shader = sobel_shader::new(
        &device, &dog_shader.render_target, texture_size, sobel_pipeline
    );
    let ds_shader = downscale_shader::new(downscale_shader::DownscaleShaderStruct {
        device: &device,
        texture: &image_texture,
        sobel_texture: &sobel_shader.render_target,
        size: texture_size,
        buffer_size: &ascii_buffer_size,
        ascii_style: ASCII_STYLE,

        brightness: shader_config.brightness,
        contrast: shader_config.contrast,
        draw_edges: shader_config.draw_edges,
        edge_threshold: shader_config.edge_threshold,
    }, ds_pipeline);
    
    // start processing the image. Each subsequent render uses the 
    // result (rendertarget) from the last render.
    render(RenderDescriptor {
        pipeline: &dog_shader.pipeline,
        bind_group: &dog_shader.bind_group,
        render_target: &dog_shader.render_target,
        output_buffer: &dog_shader.output_buffer,
        texture_size: &texture_size,
    }, &queue, &device);

    render(RenderDescriptor {
        pipeline: &sobel_shader.pipeline,
        bind_group: &sobel_shader.bind_group,
        render_target: &sobel_shader.render_target,
        output_buffer: &sobel_shader.output_buffer,
        texture_size: &texture_size,
    }, &queue, &device);

    // render and store output into output_buffer
    render_compute(ComputeDescriptor {
        pipeline: &ds_shader.pipeline,
        bind_group: &ds_shader.bind_group,
        storage_buffer: &ds_shader.storage_buffer,
        output_buffer: &ds_shader.output_buffer,
        size: &ascii_buffer_size,
        texture_size: &texture_size,
    }, &device, &queue, wg_size);

    //// !!! Only uncomment when input is an image !!!
    //// these  two functions can be used to generate images from each step in the shader program
    // copy_to_img(&CopyDescriptor {
    //     output_buffer: &dog_shader.output_buffer,
    //     texture_size: &texture_size
    // }, &device).await;

    // copy_to_img(&CopyDescriptor {
    //     output_buffer: &sobel_shader.output_buffer,
    //     texture_size: &texture_size
    // }, &device).await;

    let render_time = benchmark_render.elapsed();

    let benchmark_cache = Instant::now();

    // copy data from output_buffer into a CPU mappable buffer
    let data = copy_data(&ds_shader.output_buffer, &device).await;

    // append the processed text buffer to the last frame in cache file
    cache_result(&data, &texture_size, &file, wg_size);

    let cache_time = benchmark_cache.elapsed();
    let total_elapsed_time = image_decode_time + write_texture_time + render_time + cache_time;

    if verbose {
        println!("Frame processed  wg_size: ({},{}) | Total: {:.5?} | image_decode: {:.5?}, render: {:.5?}, cache: {:.5?}",
            wg_size.x, wg_size.y, total_elapsed_time, image_decode_time, render_time, cache_time
        );
    }

    return Benchmark {
        total_time: total_elapsed_time,
        image_decode_time,
        render_time,
        cache_time,
    };
}

struct RenderDescriptor<'a> {
    pipeline: &'a wgpu::RenderPipeline,
    bind_group: &'a wgpu::BindGroup,
    render_target: &'a wgpu::Texture,
    output_buffer: &'a wgpu::Buffer,
    texture_size: &'a wgpu::Extent3d,
}

/// Run a fragment shader to process texture
fn render(desc: RenderDescriptor, queue: &wgpu::Queue, device: &wgpu::Device) {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { 
        label: None,
    });

    {
        let render_target_view = desc.render_target.create_view(&Default::default());
        let render_pass_desc = wgpu::RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &render_target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu:: LoadOp::Clear(wgpu::Color {
                            r:0.0,
                            g:0.0,
                            b:0.0,
                            a:1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        let mut pass = encoder.begin_render_pass(&render_pass_desc);
        pass.set_pipeline(desc.pipeline);
        pass.set_bind_group(0, desc.bind_group, &[]);
        pass.draw(0..6,0..1);
    }

    // only used for last render step before compute shader
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            aspect: wgpu::TextureAspect::All,
                    texture: desc.render_target,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: desc.output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(256 * (4.0 * desc.texture_size.width as f32 / 256.0).ceil() as u32),
                rows_per_image: Some(desc.texture_size.height),
            },
        },
        *desc.texture_size
    );
    queue.submit(Some(encoder.finish()));
}

struct ComputeDescriptor<'a> {
    pipeline: &'a wgpu::ComputePipeline,
    bind_group: &'a wgpu::BindGroup,
    storage_buffer: &'a wgpu::Buffer,
    output_buffer: &'a wgpu::Buffer,
    size: &'a wgpu::BufferAddress,
    texture_size: &'a wgpu::Extent3d,
}

/// Run a compute shader to process texture and store result in a buffer
fn render_compute(desc: ComputeDescriptor, device: &wgpu::Device, queue: &wgpu::Queue, wg_size: WorkgroupSize) {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { 
        label: None,
    });

    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });

        let tex_x = desc.texture_size.width as f32;
        let tex_y = desc.texture_size.height as f32;

        // determine dispatch size
        let x = (tex_x/wg_size.x as f32).floor() as u32;
        let y = (tex_y/wg_size.y as f32).floor() as u32;

        pass.set_pipeline(desc.pipeline);
        pass.set_bind_group(0, desc.bind_group, &[]);
        pass.dispatch_workgroups(x, y, 1);
    }

    encoder.copy_buffer_to_buffer(
        desc.storage_buffer,
         0,
          desc.output_buffer,
           0,
            *desc.size
    );

    queue.submit(Some(encoder.finish()));
}

// struct CopyDescriptor<'a> {
//     output_buffer: &'a wgpu::Buffer,
//     texture_size: &'a wgpu::Extent3d
// }

/// Optional; Copy texture to generate an image.
/// Used if you want to see what each step in the shader algorithm does
// async fn copy_to_img(desc: &CopyDescriptor<'_>, device: &wgpu::Device) {
//     let buffer_slice = desc.output_buffer.slice(..);
//     let size = desc.texture_size;

//     let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
//     buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
//         tx.send(result).unwrap();
//     });
//     device.poll(wgpu::PollType::Wait).unwrap();
//     rx.receive().await.unwrap().unwrap();

//     let data = buffer_slice.get_mapped_range();

//     // absolutely psychotic way to avoid name collisions
//     let mut rng = rand::rng();
//     let rand_num: u32 = rng.random();

//     let buffer =
//         ImageBuffer::<Rgba<u8>, _>::from_raw(size.width, size.height, data).unwrap();
//     buffer.save(format!("example/image_{}.png", rand_num)).unwrap();

//     drop(buffer);
//     desc.output_buffer.unmap();
// }

/// Copy data from a compute shader output buffer and return it as a `Vec<u32>`
async fn copy_data(output_buffer: &wgpu::Buffer, device: &wgpu::Device) -> Vec<u32> {
    let buffer_slice = output_buffer.slice(..);

    let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
    });
    device.poll(wgpu::PollType::Wait).unwrap();
    rx.receive().await.unwrap().unwrap();

    let data = buffer_slice.get_mapped_range();
    let result: Vec<u32> = bytemuck::cast_slice(&data).to_vec();

    return result;
}

fn index_string(s: &str, n: u32) -> char {
    return s.chars().nth((n).try_into().unwrap()).unwrap();
}

/// Read a copy of a compute shader output buffer and cache them to `file`
fn cache_result(vec: &Vec<u32>, tex_size: &wgpu::Extent3d, mut file: &File, wg_size: WorkgroupSize) {
    let mut count = 0;
    let padded_row_size = (utils::align_buffer_size_f(tex_size.width,64)/wg_size.x as f32).ceil();
    let mut cache_string: String = Default::default();
    for num in vec.iter() {
        let char: char;
        let index: u32 = *num;

        // go to next row
        if count == padded_row_size as u32 {
         cache_string += "\n";
            count = 0;
        }

        // if edge
        if index > 999 {
            char = index_string(ASCII_EDGES, (index / 1000) - 1);
        }
        else {
            char = ASCII_STYLE.chars().nth((*num).try_into().unwrap()).unwrap();
        }

        cache_string += &char.to_string();
        count += 1;
    }

    file.write(format!("\n{}\n", cache_string).as_bytes()).ok();
}