//! Scripts responsible for serializing and deserializing the mofetfch config file

/// default toml configuration
const PROGRAM_DEFAULTS: &str = r#"
include_order = ["host_name","os","kernel","cpu","gpu","linebreak","cpu_usage","uptime"]

[key_names]
os = "OS"
kernel = "Kernel"
cpu = "CPU"
gpu = "GPU"
cpu_usage = "CPU usage"
uptime = "Uptime"

[key_values]
host_name = "system_default"
os = "system_default"
kernel = "system_default"
cpu = "system_default"
gpu = "system_default"

[options_defaults]
input = "None"
fps = 24
brightness = 1.1
contrast = 1.1
draw_edges = true
edge_threshold = 0.3
overwrite_cache = false
max_width = 0.7
max_height = 1.0
adapter_index = 0
hide_info = false
verbose = false
"#;

#[derive(serde::Deserialize)]
pub struct KeyNames {
    pub os: String,
    pub kernel: String,
    pub cpu: String,
    pub gpu: String,
    pub cpu_usage: String,
    pub uptime: String,
}

#[derive(serde::Deserialize)]
pub struct KeyValues {
    pub host_name: String,
    pub os: String,
    pub kernel: String,
    pub cpu: String,
    pub gpu: String,
}

#[derive(serde::Deserialize)]
pub struct OptionsDefaults {
    pub input: Option<String>,
    pub fps: u16,
    pub brightness: f32,
    pub contrast: f32,
    pub draw_edges: bool,
    pub edge_threshold: f32,
    pub overwrite_cache: bool,
    pub max_width: f32,
    pub max_height: f32,
    pub adapter_index: usize,
    pub hide_info: bool,
    pub verbose: bool,
}

#[derive(serde::Deserialize)]
pub struct Config {
    pub include_order: Vec<String>,
    pub key_names: KeyNames,
    pub key_values: KeyValues,
    pub options_defaults: OptionsDefaults,
}

pub fn retrieve_config() -> Config {
    let config_dir = format!("{}/mofetch",dirs::config_dir().unwrap().to_str().unwrap());
    let config_path = format!("{}/config.toml",&config_dir);
    let config_exists = std::path::Path::new(&config_path).exists();

    let config_str = if !config_exists {
        std::fs::create_dir(&config_dir).unwrap();
        std::fs::write(&config_path, PROGRAM_DEFAULTS).unwrap();
        String::from(PROGRAM_DEFAULTS)
    }
    else {
        std::fs::read_to_string(&config_path).unwrap()
    };
    let config: Config = toml::from_str(&config_str.as_str()).unwrap();
    return config;
}