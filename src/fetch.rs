//! script responsible for fetching sysinfo

use sysinfo::{System};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn get_uptime() -> String {
    let raw_boot_time = System::boot_time();
    let in_seconds = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - raw_boot_time;
    let total_hours = ((in_seconds as f32)/3600.0).floor() as u64;
    let total_minutes = ((in_seconds as f32)/60.0).floor() as u64 - total_hours * 60;

    // format based on singular or multiple minutes/hours
    let hrs_string =
        if total_hours == 1 {format!("{} hr ", total_hours)}
        else if total_hours > 1 {format!("{} hrs ", total_hours)}
        else {format!("")};

    let mins_string =
        if total_minutes == 1 {format!("{} min", total_minutes)}
        else {format!("{} mins", total_minutes)};

    return String::from(format!("{}{}",hrs_string, mins_string));
}

enum SpaceType {
    None,
    Single,
    Double,
    Line,
    DoubleLine,
}

impl SpaceType {
    fn to_str(&self) -> &'static str {
        match self {
            SpaceType::None => "",
            SpaceType::Single => "\n",
            SpaceType::Double => "\n\n",
            SpaceType::Line => "\n---------",
            SpaceType::DoubleLine => "\n-----------\n"
        }
    }
}

struct InfoValuePair {
    name: String,
    value: String,
    omit_name: bool,
    space: SpaceType,
}

struct StaticInfo {
    host_name: InfoValuePair,
    os: InfoValuePair,
    kernel: InfoValuePair,
    cpu: InfoValuePair,
    gpu: InfoValuePair,
    uptime: InfoValuePair,
}

impl StaticInfo {
    pub fn as_array(&self) -> [&InfoValuePair; 6] {
        [&self.host_name, &self.os, &self.kernel, &self.cpu, &self.gpu, &self.uptime]
    }
}

fn get_static_info(sys: &System, gpu: &wgpu::AdapterInfo) -> StaticInfo {
    let host_name = InfoValuePair {
        name: String::from("Host Name"),
        value: System::host_name().unwrap(),
        omit_name: true,
        space: SpaceType::DoubleLine,
    };
    
    let os = InfoValuePair {
        name: String::from("OS"),
        value: format!("{} ({})",System::name().unwrap(),System::os_version().unwrap()),
        omit_name: false,
        space: SpaceType::None,
    };
    let kernel = InfoValuePair {
        name: String::from("Kernel"),
        value: System::kernel_long_version(),
        omit_name: false,
        space: SpaceType::None,
    };
    let cpu = InfoValuePair {
        name: String::from("CPU"),
        value: String::from(sys.cpus()[0].brand()),
        omit_name: false,
        space: SpaceType::None,
    };
    let gpu = InfoValuePair {
        name: String::from("GPU"),
        value: format!("{} [{:?}]",gpu.name, gpu.device_type),
        omit_name: false,
        space: SpaceType::None,
    };
    let uptime = InfoValuePair {
        name: String::from("Uptime"),
        value: get_uptime(),
        omit_name: false,
        space: SpaceType::None,
    };
    // let shell
    // let font
    // let cursor
    // let terminal
    // let terminal_font

    let info = StaticInfo {
        host_name,
        os,
        kernel,
        cpu,
        gpu,
        uptime,
    };
    return info;
}

fn get_dyn_info(sys: &System) -> f32 {
    let cpu_usage = sys.global_cpu_usage();
    // let uptime = get_uptime();
    return cpu_usage;
}

// WIP
pub fn sys_info_manager(gpu: wgpu::AdapterInfo, ascii_w: u32, ascii_h: u32) {
    let mut sys = System::new_all();
    let static_info = get_static_info(&sys, &gpu);

    let info_array = static_info.as_array();

    for (i,data) in info_array.iter().enumerate() {
        let name_string: String =
            if data.omit_name {String::from("")}
            else {format!("{}: ", data.name)};
        
        println!("{}{}{}{}",termion::cursor::Goto((ascii_w + 2) as u16, (i + 2) as u16),
            name_string, data.value, data.space.to_str());
    }

    loop {
        std::thread::sleep(Duration::from_millis(1000));
        sys.refresh_cpu_usage();
        let cpu_usage = get_dyn_info(&sys);
        println!("{}CPU usage: {:.3}%  ",termion::cursor::Goto((ascii_w + 2) as u16, 9), cpu_usage);
    }
}