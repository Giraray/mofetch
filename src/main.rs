mod core;
mod help_options;
mod info;

use std::{panic, path::Path, io::Write};
use lexopt::Arg::{Long, Short};
use lexopt::ValueExt;

const TERM_FONT_DIMS: (u16,u16) = (10,22);

/// TODO:
/// * Cacheless direct rendering to terminal 
fn main() {
    let args = parse_args().unwrap();
    let fps = args.fps;
    let input = args.input_path.unwrap();
    let overwrite_cache = args.overwrite_cache;
    let max_width = args.max_width;
    let max_height: f32 = args.max_height;
    let adapter_index = args.adapter_index;

    let term_size_char = termion::terminal_size().unwrap();

    let term_width: u16 = term_size_char.0 * TERM_FONT_DIMS.0;
    let max_width = (term_width as f32 * max_width).floor() as u16;

    let term_height: u16 = (term_size_char.1 - 1) * TERM_FONT_DIMS.1;
    let max_height = (term_height as f32 * max_height).floor() as u16;

    let ffmpeg_config = core::FfmpegConfig {
        input_path: input.as_str(),
        fps: &fps,
    };

    // establish connection to GPU
    let mut process_desc = pollster::block_on(core::ProcessDescriptor::init(adapter_index));
    
    let input_name = input.split('/').last().unwrap();

    // afb: animated frame buffer
    // sfb: static frame buffer (for images)
    let cache_dir = format!("{}/mofetch",dirs::cache_dir().unwrap().to_str().unwrap());
    let cache_path_afb = format!("{}/{}.afb",&cache_dir,input_name);
    let cache_path_sfb = format!("{}/{}.sfb",&cache_dir,input_name);
    let mut cache_path: String = Default::default();
    let mut is_image = false;
    let afb_path_exists = Path::new(&cache_path_afb).exists() && !overwrite_cache;
    let sfb_path_exists = Path::new(&cache_path_sfb).exists() && !overwrite_cache;
    
    // TODO: there is nothing to prevent this from happening right now. will have to fix later 
    // or come up with a smarter solution.
    if afb_path_exists && sfb_path_exists {
        panic!("Cache conflict: Input has 2 existing caches with different file formats");
    }

    let mut cache_file: std::fs::File;

    // look for existing cache
    if afb_path_exists {
        cache_path = format!("{}/{}.afb",&cache_dir,input_name);
    }
    else if sfb_path_exists {
        cache_path = format!("{}/{}.sfb",&cache_dir,input_name);
        is_image = true;
    }

    let mut config = core::ProgramConfig {
        ffmpeg_config: &ffmpeg_config,
        cache_path: &cache_path,
    };

    // make cache file if it doesnt exist. make sfb or afb based on frame_count > 1
    if !afb_path_exists && !sfb_path_exists {
        let ffmpeg_return = core::get_frames(&ffmpeg_config, max_width, max_height);
        if ffmpeg_return.frame_count == 1 {
            is_image = true;
            cache_path = format!("{}/{}.sfb",&cache_dir,input_name);
        }
        else {
            cache_path = format!("{}/{}.afb",&cache_dir,input_name);
        }

        std::fs::create_dir(&cache_dir).ok();
        std::fs::File::create(&cache_path).unwrap();

        cache_file = std::fs::File::options().append(true).open(&cache_path).unwrap(); // w cache file
        cache_file.write(format!("[] fps={}\n",fps).as_bytes()).ok(); // write fps config in cache file

        process_desc = pollster::block_on(core::ProcessDescriptor::init(adapter_index));
        config = core::ProgramConfig {
            ffmpeg_config: &ffmpeg_config,
            cache_path: &cache_path,
        };
        core::process_frames(&ffmpeg_return.frame_count, &process_desc, cache_file,
            ffmpeg_return.width, ffmpeg_return.height, max_width, max_height);
    }
    let frame_dims = read_frame_size(&config.cache_path);
    std::thread::spawn(move || {
        info::sys_info_manager(process_desc.adapter.get_info(), frame_dims.0, frame_dims.1);
    });
    core::print_frame_loop(&config, is_image);
}

/// Reads the first frame to return the dimensions of the cache frames.
pub fn read_frame_size(path: &str) -> (u32,u32) {
    let file_string = std::fs::read_to_string(path).unwrap();
    let mut render_width = 0;
    let mut render_height = 0;
    let mut frame_state = false;
    for line in file_string.lines() {

        // if line is not a config line AND line is not a frame separator
        if !line.contains("[]") && line.len() > 1 {
            let new_render_width = std::cmp::max(line.len() as u32, render_width);
            render_width = new_render_width; // buffer alignment bug that causes the last line in a frame to sometimes be shorter than its supposed to be
            render_height += 1;
            frame_state = true;
        }

        // once reader reached the end of the first frame separator, end loop and register dims
        else if line.len() <= 1 && frame_state == true {
            break;
        }
    }
    return (render_width, render_height);
}

pub struct UserArgs {
    pub input_path: Option<String>,
    pub fps: u16,
    pub brightness: f32,
    pub contrast: f32,
    pub draw_edges: bool,
    pub edge_threshold: f32,
    // pub characterSet: Option<String>,
    pub overwrite_cache: bool,
    pub max_width: f32,
    pub max_height: f32,
    pub adapter_index: usize,
}

fn parse_args() -> Result<UserArgs, lexopt::Error> {
    let mut parser = lexopt::Parser::from_env();

    let mut input_path: Option<String> = None;
    let mut fps: u16 = 24;
    let mut brightness: f32 = 1.0;
    let mut contrast: f32 = 1.0;
    let mut draw_edges: bool = true;
    let mut edge_threshold: f32 = 0.3;
    let mut overwrite_cache: bool = false;
    let mut max_width: f32 = 0.7;
    let mut max_height: f32 = 1.0;
    let mut adapter_index: usize = 0;

    while let Some(arg) = parser.next()? {
        match arg {
            Short('i') | Long("input") => {
                input_path = Some(parser.value()?.parse()?);
            }
            Short('o') | Long("overwrite-cache") => {
                overwrite_cache = true;
            }
            Short('f') | Long("fps") => {
                fps = parser.value()?.parse()?;
                overwrite_cache = true;
            }
            Short('b') | Long("brightness") => {
                brightness = parser.value()?.parse()?;
                overwrite_cache = true;
            }
            Short('c') | Long("contrast") => {
                contrast = parser.value()?.parse()?;
                overwrite_cache = true;
            }
            Short('n') | Long("no-edges") => {
                draw_edges = false;
                overwrite_cache = true;
            }
            Short('t') | Long("edge-threshold") => {
                edge_threshold = parser.value()?.parse()?;
                overwrite_cache = true;
            }
            Short('W') | Long("max-width") => {
                max_width = parser.value()?.parse()?;
                overwrite_cache = true;
            }
            Short('H') | Long("max-height") => {
                max_height = parser.value()?.parse()?;
                overwrite_cache = true;
            }
            Long("gpus") => {
                let process_desc = pollster::block_on(core::ProcessDescriptor::init(0));
                for gpu in process_desc.adapters_vec.iter().enumerate() {
                    println!("{}: {:?}",gpu.0, gpu.1.get_info());
                }
                std::process::exit(0);
            }
            Short('a') | Long("adapter-index") => {
                adapter_index = parser.value()?.parse()?;
                overwrite_cache = true;
            }

            Short('h') | Short('?') | Long("help") => {
                let help_intro = String::from("Mofetch is a system information fetching tool with fancy user-generated ASCII art");
                let help_usage = String::from("Usage: mofetch [-i input-file-path] [options]");
                println!("{}",help_intro);
                println!("{}",help_usage);
                println!("\nNOTE: Specifying pre-processing options or Shader options will force --overwrite-cache.");

                let options = help_options::init_options();
                for group in options {
                    println!("\n{}",group.name.unwrap());
                    for option in group.options {
                        let short = if option.short.is_some() {
                            format!("-{}",option.short.unwrap())
                        } else {
                            String::from("  ")
                        };
                        let datatype = if option.datatype.is_some() {
                            format!(" <{}>",option.datatype.unwrap())
                        } else {
                            String::from("")
                        };
                        let pre_desc_text = format!("{0} --{2}{1}",short,datatype,option.long.unwrap());
                        let space_count = 29 - pre_desc_text.len();
                        let mut spaces: String = Default::default();
                        for _ in 1..space_count {
                            spaces += " ";
                        }
                        println!("{0}{1}{2}", pre_desc_text, spaces, option.desc.unwrap());
                    }
                }
                std::process::exit(0);
            }
            _ => return Err(arg.unexpected()),
        }
    }
    if input_path.is_none() {
        println!("Error: Expected file input. Use \"mofetch --help\" for usage help");
        std::process::exit(0);
    }

    Ok(UserArgs {
        input_path,
        fps,
        brightness,
        contrast,
        draw_edges,
        edge_threshold,
        overwrite_cache,
        max_width,
        max_height,
        adapter_index,
    })
}