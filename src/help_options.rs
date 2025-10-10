pub struct HelpOption {
    pub short: Option<String>,
    pub long: Option<String>,
    pub desc: Option<String>,
    pub datatype: Option<String>,
}

pub struct OptionGroup {
    pub name: Option<String>,
    pub options: Vec<HelpOption>,
}

pub fn init_options() -> Vec<OptionGroup> {
    let mut options = Vec::new();

    // information options
    let mut information_options = Vec::new();
    // help
    let help = HelpOption {
        short: Some("h".into()),
        long: Some("help".into()),
        desc: Some("Display this help message".into()),
        datatype: None,
    };
    information_options.push(help);

    // version
    let version = HelpOption {
        short: Some("v".into()),
        long: Some("version".into()),
        desc: Some("Show mofetch version".into()),
        datatype: None,
    };
    information_options.push(version);

    // verbose
    let verbose = HelpOption {
        short: Some("V".into()),
        long: Some("verbose".into()),
        desc: Some("Show process information".into()),
        datatype: None,
    };
    information_options.push(verbose);

    let sysinfo = HelpOption {
        short: Some("I".into()),
        long: Some("hide-info".into()),
        desc: Some("Do not show system info, and only render the thumbnail".into()),
        datatype: None,
    };
    information_options.push(sysinfo);

    // adapters
    let gpus = HelpOption {
        short: None,
        long: Some("gpus".into()),
        desc: Some("Get all available GPU adapters by index".into()),
        datatype: None,
    };
    information_options.push(gpus);

    let information_vec = OptionGroup {
        name: Some("Information options".into()),
        options: information_options,
    };
    options.push(information_vec);


    // pre-processing options
    let mut pre_processing_options: Vec<HelpOption> = Vec::new();
    // overwrite cache
    let overwrite_cache = HelpOption {
        short: Some("o".into()),
        long: Some("overwrite-cache".into()),
        desc: Some("Ignore and overwrite existing cache. Useful if the cache is corrupt".into()),
        datatype: None,
    };
    pre_processing_options.push(overwrite_cache);

    // max width
    let max_width = HelpOption {
        short: Some("W".into()),
        long: Some("max-width".into()),
        desc: Some("Set max width of source image to (0..n) * 100 % of terminal width".into()),
        datatype: Some("float".into()),
    };
    pre_processing_options.push(max_width);

    // max height
    let max_height = HelpOption {
        short: Some("H".into()),
        long: Some("max-height".into()),
        desc: Some("Set max height of source image to (0..n) * 100 % of terminal height".into()),
        datatype: Some("float".into()),
    };
    pre_processing_options.push(max_height);

    // fps
    let fps = HelpOption {
        short: Some("f".into()),
        long: Some("fps".into()),
        desc: Some("Set the frames per second".into()),
        datatype: Some("int".into()),
    };
    pre_processing_options.push(fps);

    // brightness
    let brightness = HelpOption {
        short: Some("b".into()),
        long: Some("brightness".into()),
        desc: Some("Set the brightness of the input".into()),
        datatype: Some("float".into()),
    };
    pre_processing_options.push(brightness);

    // contrast
    let contrast = HelpOption {
        short: Some("c".into()),
        long: Some("contrast".into()),
        desc: Some("Set the contrast of the input".into()),
        datatype: Some("float".into()),
    };
    pre_processing_options.push(contrast);

    let pre_processing_vec = OptionGroup {
        name: Some("Pre-processing options".into()),
        options: pre_processing_options,
    };
    options.push(pre_processing_vec);


    // shader options
    let mut shader_options: Vec<HelpOption> = Vec::new();
    // adapter index
    let adapter_index = HelpOption {
        short: Some("a".into()),
        long: Some("adapter-index".into()),
        desc: Some("Use GPU adapter with the provided index to specify GPU device. Use --gpus to get available adapters".into()),
        datatype: Some("int".into()),
    };
    shader_options.push(adapter_index);

    // no edges
    let no_edges = HelpOption {
        short: Some("n".into()),
        long: Some("no-edges".into()),
        desc: Some("Don't draw ASCII edge lines (/ \\ _ |)".into()),
        datatype: None,
    };
    shader_options.push(no_edges);

    // edge threshold
    let edge_threshold = HelpOption {
        short: Some("t".into()),
        long: Some("edge-threshold".into()),
        desc: Some("Set the required percentage of edge strength in each tile for a tile to be rendered as an edge (0..1). Increasing this value can be useful if stronger edgelines are desired, while decreasing it can reduce the noise created by too many edges from the input.".into()),
        datatype: Some("float".into()),
    };
    shader_options.push(edge_threshold);

    let shader_vec = OptionGroup {
        name: Some("Shader options".into()),
        options: shader_options,
    };
    options.push(shader_vec);

    return options;
}