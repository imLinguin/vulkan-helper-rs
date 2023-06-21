use ash::{vk, Entry};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

const APP_NAME: &str = "Heroic\0";
#[derive(Serialize, Deserialize)]
struct Device {
    pub name: String,
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}
fn get_instance_version() -> [u32; 3] {
    let entry = unsafe { Entry::load() }.expect("Failed to load vulkan");
    if let Ok(Some(version)) = entry.try_enumerate_instance_version() {
        let major = vk::api_version_major(version);
        let minor = vk::api_version_minor(version);
        let patch = vk::api_version_patch(version);

        [major, minor, patch]
    } else {
        [0, 0, 0]
    }
}

fn get_physical_versions() -> Vec<Device> {
    let entry = unsafe { Entry::load() }.expect("Failed to load vulkan");

    let app_info = vk::ApplicationInfo {
        p_application_name: APP_NAME.as_ptr() as *const i8,
        application_version: vk::make_api_version(0, 1, 0, 0),
        api_version: vk::make_api_version(0, 1, 3, 0),
        ..Default::default()
    };

    let instance_info = vk::InstanceCreateInfo {
        p_application_info: &app_info,
        ..Default::default()
    };

    let instance = unsafe { entry.create_instance(&instance_info, None) }
        .expect("Failed to create Vulkan instance");

    let devices =
        unsafe { instance.enumerate_physical_devices() }.expect("Failed to enumerate devices");

    let mut array: Vec<Device> = Vec::new();
    for device in devices {
        let properties = unsafe { instance.get_physical_device_properties(device) };

        if properties.device_type == vk::PhysicalDeviceType::CPU {
            continue;
        }

        let slice: &[u8; 256] = unsafe { std::mem::transmute(&properties.device_name) };
        let name = String::from(std::str::from_utf8(slice).unwrap().trim_end_matches('\0'));

        let major = vk::api_version_major(properties.api_version);
        let minor = vk::api_version_minor(properties.api_version);
        let patch = vk::api_version_patch(properties.api_version);
        let device_struct = Device {
            name,
            major,
            minor,
            patch,
        };
        array.push(device_struct);
    }

    unsafe { instance.destroy_instance(None) };
    array
}

#[derive(Subcommand)]
enum Commands {
    InstanceVersion,
    PhysicalVersions,
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();
    let data = match &cli.command {
        Commands::InstanceVersion => {
            let version = get_instance_version();
            serde_json::to_string(&version).expect("Failed to create json output")
        }
        Commands::PhysicalVersions => {
            let versions = get_physical_versions();
            serde_json::to_string(&versions).expect("Failed to create json output")
        }
    };
    print!("{}", data);
}
